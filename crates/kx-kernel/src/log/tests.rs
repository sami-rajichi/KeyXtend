use super::*;
use crate::KernelSettings;
use crate::kernel::{MAX_LOG_MB, MIN_LOG_MB};
use kx_module_api::{Redacted, Settings};
use std::sync::Arc;
use tracing::callsite::{DefaultCallsite, Identifier};
use tracing::field::{FieldSet, Value};
use tracing::metadata::Kind;
use tracing::{Event, Level, Metadata, info, info_span, warn};

/// A value no log line may ever show.
pub(super) const SECRET: &str = "hunter2";

/// A probe event with one field per sensitive name, so a test can loop over the list.
static PROBE: Metadata<'static> = Metadata::new(
    "probe",
    "kx_kernel::log::tests",
    Level::INFO,
    None,
    None,
    None,
    FieldSet::new(SENSITIVE, Identifier(&SITE)),
    Kind::EVENT,
);
static SITE: DefaultCallsite = DefaultCallsite::new(&PROBE);

/// Field names holding a sensitive name as a whole part, in other cases and spellings.
const VARIANTS: &[&str] = &[
    "typed_text",
    "clipboard_text",
    "api.key",
    "api-key",
    "Password",
    "user_password",
    "password_hash",
];

/// Ordinary field names, one of which holds a sensitive name only inside a longer part.
const ORDINARY: &[&str] = &["module", "context"];

/// A probe event with one field per variant name.
static VARIANT_PROBE: Metadata<'static> = Metadata::new(
    "variants",
    "kx_kernel::log::tests",
    Level::INFO,
    None,
    None,
    None,
    FieldSet::new(VARIANTS, Identifier(&VARIANT_SITE)),
    Kind::EVENT,
);
static VARIANT_SITE: DefaultCallsite = DefaultCallsite::new(&VARIANT_PROBE);

/// One `probe` event per name in `names`, each holding `SECRET`.
fn probe_each(probe: &'static Metadata<'static>, names: &[&str]) {
    let fields = probe.fields();
    for name in names {
        let field = fields.field(name).unwrap();
        let value = SECRET;
        let values = [(&field, Some(&value as &dyn Value))];
        Event::dispatch(probe, &fields.value_set(&values));
    }
}

/// One event per sensitive name, each holding `SECRET`.
fn probe_each_name() {
    probe_each(&PROBE, SENSITIVE);
}

/// A shared byte buffer that acts as the log writer of a test.
#[derive(Clone, Default)]
struct Buf(Arc<Mutex<Vec<u8>>>);

impl Write for Buf {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl MakeWriter<'_> for Buf {
    type Writer = Self;

    fn make_writer(&self) -> Self {
        self.clone()
    }
}

/// Runs `run` under a scoped subscriber and returns what it wrote.
fn capture(level: LevelFilter, run: impl FnOnce()) -> String {
    let buf = Buf::default();
    tracing::subscriber::with_default(subscriber(level, buf.clone()), run);
    let bytes = buf.0.lock().unwrap().clone();
    String::from_utf8(bytes).unwrap()
}

fn info_log(run: impl FnOnce()) -> String {
    capture(LevelFilter::INFO, run)
}

/// Checks that field `name` printed as the marker.
fn assert_marked(out: &str, name: &str) {
    let want = format!("{name}={REDACTED_MARKER}");
    assert!(out.contains(&want), "{out}");
}

#[test]
fn a_password_field_prints_the_marker_and_never_its_value() {
    let out = info_log(|| info!(password = SECRET, "x"));
    assert_marked(&out, "password");
    assert!(!out.contains(SECRET), "{out}");
}

#[test]
fn every_sensitive_name_prints_the_marker_and_never_its_value() {
    let out = info_log(probe_each_name);
    for name in SENSITIVE {
        assert_marked(&out, name);
    }
    assert!(!out.contains(SECRET), "{out}");
}

#[test]
fn a_sensitive_name_inside_a_longer_name_or_another_spelling_is_redacted() {
    let out = info_log(|| probe_each(&VARIANT_PROBE, VARIANTS));
    for name in VARIANTS {
        assert_marked(&out, name);
    }
    assert!(!out.contains(SECRET), "{out}");
}

#[test]
fn only_whole_parts_of_a_name_count() {
    assert!(VARIANTS.iter().all(|name| sensitive(name)), "{VARIANTS:?}");
    assert!(ORDINARY.iter().all(|name| !sensitive(name)), "{ORDINARY:?}");
    let out = info_log(|| info!(module = "keyboard", context = 3, "x"));
    assert!(out.contains("module=\"keyboard\" context=3"), "{out}");
}

#[test]
fn a_redacted_value_in_an_ordinary_field_prints_the_marker() {
    let out = info_log(|| {
        info!(note = ?Redacted::new(SECRET), "x");
        info!(other = %Redacted::new(SECRET), "y");
    });
    assert_marked(&out, "note");
    assert_marked(&out, "other");
    assert!(!out.contains(SECRET), "{out}");
}

#[test]
fn an_ordinary_field_prints_its_value() {
    let out = info_log(|| info!(count = 3, name = "abc", "x"));
    assert!(out.contains("count=3"), "{out}");
    assert!(out.contains("name=\"abc\""), "{out}");
}

#[test]
fn a_span_field_is_redacted_too() {
    let out = info_log(|| info_span!("scope", password = SECRET).in_scope(|| info!("x")));
    assert_marked(&out, "password");
    assert!(!out.contains(SECRET), "{out}");
}

#[test]
fn the_message_prints_as_it_is() {
    let out = info_log(|| info!("hello there"));
    assert!(out.contains(": hello there\n"), "{out}");
}

#[test]
fn a_newline_in_a_string_field_stays_on_one_line() {
    let out = info_log(|| info!(name = "a\nb\r\u{1b}[0m", "x"));
    assert_eq!(out.lines().count(), 1, "{out}");
    assert!(out.contains(r#"name="a\nb\r\u{1b}[0m""#), "{out}");
}

#[test]
fn a_newline_in_a_displayed_value_stays_on_one_line() {
    let said = "a\nb\r\u{1b}";
    let out = info_log(|| info!(note = %said, "x"));
    assert_eq!(out.lines().count(), 1, "{out}");
    assert!(out.contains(r"note=a\nb\r\u{1b}"), "{out}");
}

#[test]
fn a_newline_in_the_message_stays_on_one_line() {
    let said = "a\nb\r\u{1b}";
    let out = info_log(|| info!("said {said}"));
    assert_eq!(out.lines().count(), 1, "{out}");
    assert!(out.contains(r"said a\nb\r\u{1b}"), "{out}");
}

/// Line and paragraph separators and the bidi controls, listed apart from `MARKS` so the test checks it.
const WANT_ESCAPED: [char; 13] = [
    '\u{2028}', '\u{2029}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}',
    '\u{202e}', '\u{2066}', '\u{2067}', '\u{2068}', '\u{2069}',
];

#[test]
fn every_separator_and_bidi_control_is_escaped() {
    for mark in WANT_ESCAPED {
        let said = format!("a{mark}b");
        let out = info_log(|| info!(note = %said, "said {said}"));
        let escaped = mark.escape_unicode().to_string();
        assert!(!out.contains(mark), "{out:?}");
        assert_eq!(out.matches(&escaped).count(), 2, "{mark:?}: {out:?}");
    }
}

#[test]
fn a_line_has_no_colour_codes() {
    let out = info_log(|| warn!(count = 1, "x"));
    assert!(!out.contains('\u{1b}'), "{out}");
}

#[test]
fn the_warn_level_drops_info_and_keeps_warn() {
    let out = capture(LevelFilter::WARN, || {
        info!("quiet");
        warn!("loud");
    });
    assert!(out.contains("loud"), "{out}");
    assert!(!out.contains("quiet"), "{out}");
}

#[test]
fn every_level_name_maps_to_a_filter() {
    for name in LOG_LEVELS {
        assert!(level(name).is_some(), "{name}");
    }
    assert_eq!(level("error"), Some(LevelFilter::ERROR));
    assert_eq!(level("warn"), Some(LevelFilter::WARN));
    assert_eq!(level("info"), Some(LevelFilter::INFO));
    assert_eq!(level("debug"), Some(LevelFilter::DEBUG));
    assert_eq!(level("trace"), Some(LevelFilter::TRACE));
}

#[test]
fn an_unknown_level_name_is_refused() {
    for name in ["", "loud", "INFO", "off", "3"] {
        assert_eq!(level(name), None, "{name:?}");
    }
}

#[test]
fn the_kernel_settings_take_their_level_names_from_the_same_list() {
    let with = |name: &str| KernelSettings {
        disabled: Vec::new(),
        log_level: name.to_owned(),
        log_max_mb: 1,
    };
    for name in LOG_LEVELS {
        assert!(with(name).check().is_ok(), "{name}");
    }
    assert!(with("loud").check().is_err());
}

#[test]
fn the_kernel_settings_cap_the_log_size() {
    let with = |mb: u32| KernelSettings {
        disabled: Vec::new(),
        log_level: LOG_LEVELS[0].to_owned(),
        log_max_mb: mb,
    };
    assert!(with(MIN_LOG_MB).check().is_ok());
    assert!(with(MAX_LOG_MB).check().is_ok());
    assert!(with(MIN_LOG_MB - 1).check().is_err());
    assert!(with(MAX_LOG_MB + 1).check().is_err());
}

#[test]
fn a_mebibyte_count_multiplies_without_overflow() {
    assert_eq!(max_bytes(0), 0);
    assert_eq!(max_bytes(1), 1_048_576);
    assert_eq!(max_bytes(10), 10 * 1_048_576);
    assert_eq!(max_bytes(u32::MAX), u64::from(u32::MAX) * 1_048_576);
}
