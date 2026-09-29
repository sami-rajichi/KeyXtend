//! The local, redacted log: this run and the one before, never with typed text.
//! The app installs `subscriber` over a `LogFile`; nothing here sets a global default.

use kx_module_api::REDACTED_MARKER;
use kx_settings::Files;
use std::fmt::{self, Write as _};
use std::fs::{self, File};
use std::io::{self, Write};
use std::sync::{Mutex, MutexGuard, PoisonError};
use tracing::Subscriber;
use tracing::field::Field;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::field::MakeExt;
use tracing_subscriber::field::delimited::Delimited;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::fmt::format::{FieldFn, Writer, debug_fn};

/// The log level names, most important first; `level` maps each to a filter.
pub const LOG_LEVELS: &[&str] = &["error", "warn", "info", "debug", "trace"];

/// Names whose fields never print their value, also inside a longer name such as `user_password`.
/// Add names here, never remove them.
pub const SENSITIVE: &[&str] = &[
    "text",
    "typed",
    "chars",
    "clipboard",
    "password",
    "secret",
    "api_key",
    "token",
    "transcript",
];

/// The field that holds an event's message.
const MESSAGE: &str = "message";

/// What separates the parts of a field name, such as `api` and `key` in `api_key`.
const PART_SEP: &str = "_";
/// Other characters that separate name parts; they count as `PART_SEP`.
const PART_JOINERS: [char; 2] = ['.', '-'];

/// Line separators and bidi controls, escaped like control characters so no value can fake or
/// reorder a line.
const MARKS: [char; 14] = [
    '\u{2028}', '\u{2029}', '\u{061c}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}',
    '\u{202d}', '\u{202e}', '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}',
];

/// What separates the fields of one line.
const FIELD_SEP: &str = " ";

/// The bytes in one MiB.
const BYTES_PER_MIB: u64 = 1024 * 1024;

/// The last line written when the file reaches its size limit.
pub const LIMIT_LINE: &str = "log size limit reached; later lines are dropped\n";

/// The filter that keeps events of level `name` and above; `None` when `name` is not in `LOG_LEVELS`.
#[must_use]
pub fn level(name: &str) -> Option<LevelFilter> {
    if LOG_LEVELS.contains(&name) {
        name.parse().ok()
    } else {
        None
    }
}

/// The largest log file, in bytes, for a cap of `mb` MiB.
#[must_use]
pub fn max_bytes(mb: u32) -> u64 {
    u64::from(mb).saturating_mul(BYTES_PER_MIB)
}

/// The function that prints one field.
type WriteField = fn(&mut Writer<'_>, &Field, &dyn fmt::Debug) -> fmt::Result;

/// The field formatter of the log: see `fields`.
pub type Fields = Delimited<&'static str, FieldFn<WriteField>>;

/// A field whose name holds a `SENSITIVE` name prints `REDACTED_MARKER`. Other values print with
/// `Debug`, with control characters and `MARKS` escaped, so no value can start or reorder a line.
#[must_use]
pub fn fields() -> Fields {
    debug_fn(write_field as WriteField).delimited(FIELD_SEP)
}

/// True when field `name`, lower-cased, holds a `SENSITIVE` name as a whole run of its parts.
fn sensitive(name: &str) -> bool {
    let name = name.to_lowercase().replace(PART_JOINERS, PART_SEP);
    let parts: Vec<&str> = name.split(PART_SEP).collect();
    SENSITIVE.iter().any(|listed| {
        let want: Vec<&str> = listed.split(PART_SEP).collect();
        parts.windows(want.len()).any(|run| run == want.as_slice())
    })
}

fn write_field(out: &mut Writer<'_>, field: &Field, value: &dyn fmt::Debug) -> fmt::Result {
    let name = field.name();
    if name != MESSAGE {
        if sensitive(name) {
            return write!(out, "{name}={REDACTED_MARKER}");
        }
        write!(out, "{name}=")?;
    }
    write!(OneLine(out), "{value:?}")
}

/// Writes text with each control character and each of `MARKS` escaped, so a value stays on one line.
struct OneLine<'a, 'w>(&'a mut Writer<'w>);

impl fmt::Write for OneLine<'_, '_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        text.chars().try_for_each(|c| {
            if c.is_control() {
                write!(self.0, "{}", c.escape_debug())
            } else if MARKS.contains(&c) {
                write!(self.0, "{}", c.escape_unicode())
            } else {
                self.0.write_char(c)
            }
        })
    }
}

/// Where the sink is in its life.
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Open,
    /// The size limit was reached and its line written; later writes are dropped.
    Full,
    /// A write failed once; later writes are dropped.
    Failed,
}

/// The output, its byte count and its size limit.
/// A line is kept only while `LIMIT_LINE` still fits, so the file stays within the limit;
/// a limit under `LIMIT_LINE` itself holds only that line.
struct Sink<W> {
    out: W,
    written: u64,
    max: u64,
    state: State,
}

impl<W: Write> Sink<W> {
    fn new(out: W, max: u64) -> Self {
        Self {
            out,
            written: 0,
            max,
            state: State::Open,
        }
    }

    /// Writes `buf`, or the limit line when `buf` would leave no room for it. Only the first I/O error shows.
    fn put(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.state != State::Open {
            return Ok(buf.len());
        }
        let after = self.written.saturating_add(count(buf));
        let bytes = if after.saturating_add(count(LIMIT_LINE.as_bytes())) > self.max {
            self.state = State::Full;
            LIMIT_LINE.as_bytes()
        } else {
            buf
        };
        self.out
            .write_all(bytes)
            .inspect_err(|_| self.state = State::Failed)?;
        self.written = self.written.saturating_add(count(bytes));
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.state == State::Failed {
            return Ok(());
        }
        self.out.flush().inspect_err(|_| self.state = State::Failed)
    }
}

/// The length of `bytes` as a file size.
fn count(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

/// The log file of this run, behind a lock so any thread can write a whole line.
pub struct LogFile {
    sink: Mutex<Sink<File>>,
}

impl LogFile {
    /// Makes the logs folder, moves the last log to the previous name and starts a new log.
    ///
    /// # Errors
    /// When the folder or a file cannot be made or moved.
    pub fn open(files: &Files, max_bytes: u64) -> io::Result<Self> {
        fs::create_dir_all(&files.logs)?;
        match fs::rename(&files.log, &files.log_previous) {
            Err(why) if why.kind() != io::ErrorKind::NotFound => return Err(why),
            _ => {}
        }
        let file = File::create(&files.log)?;
        let sink = Mutex::new(Sink::new(file, max_bytes));
        Ok(Self { sink })
    }
}

/// A writer of `LogFile` for one event; it holds the lock only during each write.
pub struct LogWriter<'a>(&'a Mutex<Sink<File>>);

impl LogWriter<'_> {
    fn sink(&self) -> MutexGuard<'_, Sink<File>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Write for LogWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sink().put(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.sink().flush()
    }
}

impl<'a> MakeWriter<'a> for LogFile {
    type Writer = LogWriter<'a>;

    fn make_writer(&'a self) -> LogWriter<'a> {
        LogWriter(&self.sink)
    }
}

/// The subscriber the app installs: events at `level` and above, redacted, without colours.
/// It only builds the subscriber; installing it is the app's job, and a failed write never reaches stderr.
#[must_use]
pub fn subscriber<W>(level: LevelFilter, writer: W) -> impl Subscriber + Send + Sync + 'static
where
    W: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_ansi(false)
        .fmt_fields(fields())
        .log_internal_errors(false)
        .with_writer(writer)
        .finish()
}

#[cfg(test)]
mod file_tests;
#[cfg(test)]
mod tests;
