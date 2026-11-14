//! One download: its `.part` file opened and any bytes in it read back into the fingerprint, then the table's addresses
//! asked in order from the part's length, one read per step; an address that fails hands over to the next one.

mod part;

pub use part::{End, remove};

use super::hash::Print;
use kx_ipc::{FetchAsk, FetchFault, FetchNote, SHA256_BYTES, part_path};
use kx_platform_net::{Answer, Body, Get, Http};
use part::{disk, plain, taken};
use std::borrow::Cow;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::mem;
use std::path::PathBuf;

/// The most bytes one read, or one read of the part back, takes.
pub const BUF: usize = 64 * 1024;
/// The most progress notes a download tells between its first and its last.
pub const STEPS: u64 = 200;
/// The fewest bytes between two progress notes, so a small file is not told read by read.
pub const MIN_STEP: u64 = 1024 * 1024;

/// What one step of a download gives.
pub enum Step {
    /// It goes on, with a progress note when one is due.
    Go(Option<FetchNote>),
    /// It stopped on this fault.
    Fail(FetchFault),
    /// Every byte is in: check it and name it.
    Full,
}

/// Where a download stands.
enum Stage {
    /// Reading the part's bytes back into the fingerprint; this many are in.
    Back(u64),
    /// Asking the table's address at this place.
    Ask(usize),
    /// Reading the body the address at this place answered.
    Read(Box<dyn Body>, usize),
}

/// The file a download fetches, as its ask names it.
struct Row {
    id: u32,
    urls: Vec<Cow<'static, str>>,
    hosts: Vec<Cow<'static, str>>,
    path: PathBuf,
    part: PathBuf,
    size: u64,
    want: [u8; SHA256_BYTES],
    stall_s: u16,
}

/// One download into its part file.
pub struct Load {
    row: Row,
    file: File,
    /// The fingerprint of the part's first `got` bytes, once they are read back.
    print: Print,
    /// Bytes in the part.
    got: u64,
    /// The count the last progress note told.
    told: u64,
    /// The count at which the next progress note is due.
    due: u64,
    /// The first fault since an address last answered, told when no address is left.
    first: Option<FetchFault>,
    stage: Stage,
    paused: bool,
    buf: Box<[u8]>,
}

impl Load {
    /// Opens `ask`'s part, made when missing; a part longer than the file starts over.
    ///
    /// # Errors
    /// A `Disk` or `DiskFull` fault note when the path is not `plain`, its name is taken, or the part cannot be opened.
    pub fn open(ask: FetchAsk<'static>) -> Result<Self, FetchNote> {
        let row = Row::of(ask)?;
        let fail = |fault| FetchNote::Fault { id: row.id, fault };
        let (file, got) = part::open(&row.part, row.size).map_err(|e| fail(disk(&e)))?;
        let stage = if got > 0 {
            Stage::Back(0)
        } else {
            Stage::Ask(0)
        };
        Ok(Self {
            row,
            file,
            print: Print::default(),
            got,
            told: got,
            due: got,
            first: None,
            stage,
            paused: false,
            buf: vec![0; BUF].into_boxed_slice(),
        })
    }

    /// The download's id.
    pub fn id(&self) -> u32 {
        self.row.id
    }

    /// True between `pause` and `resume`.
    pub fn paused(&self) -> bool {
        self.paused
    }

    /// Stops reading: the request closes, the part stays open, and `resume` asks the table again from its start.
    pub fn pause(&mut self) {
        self.paused = true;
        if !matches!(self.stage, Stage::Back(_)) {
            self.stage = Stage::Ask(0);
            self.first = None;
        }
    }

    /// Goes on from where `pause` stopped; a closed request is asked again from the part's length.
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// One step: a chunk of the part read back, one address asked, or one read.
    pub fn step(&mut self, http: &dyn Http) -> Step {
        match mem::replace(&mut self.stage, Stage::Ask(0)) {
            Stage::Back(done) => self.back(done),
            _ if self.got == self.row.size => Step::Full,
            Stage::Ask(at) => self.ask(http, at),
            Stage::Read(body, at) => self.read(body, at),
        }
    }

    /// A progress note for the part's bytes; the next is due one `STEPS`th of the size later, at least `MIN_STEP`.
    pub fn progress(&mut self) -> FetchNote {
        let gap = (self.row.size / STEPS).max(MIN_STEP);
        (self.told, self.due) = (self.got, self.got.saturating_add(gap));
        let (id, got, total) = (self.row.id, self.got, self.row.size);
        FetchNote::Progress { id, got, total }
    }

    /// Reads the next chunk of the part back into the fingerprint, `done` bytes being in.
    fn back(&mut self, done: u64) -> Step {
        let want = cap(self.got - done, self.buf.len());
        match self.file.read(&mut self.buf[..want]) {
            Ok(0) => Step::Fail(FetchFault::Disk),
            Ok(n) => {
                self.print.add(&self.buf[..n]);
                let done = done + n as u64;
                self.stage = if done < self.got {
                    Stage::Back(done)
                } else {
                    Stage::Ask(0)
                };
                Step::Go(None)
            }
            Err(e) => Step::Fail(disk(&e)),
        }
    }

    /// Asks the address at `at` from the part's length.
    fn ask(&mut self, http: &dyn Http, at: usize) -> Step {
        let Some(url) = self.row.urls.get(at) else {
            return Step::Fail(FetchFault::Internal);
        };
        let get = Get {
            url,
            hosts: &self.row.hosts,
            from: self.got,
            stall_s: self.row.stall_s,
        };
        match http.get(&get) {
            Ok(answer) => self.answered(answer, at),
            Err(e) => self.pass(at, e.into()),
        }
    }

    /// Starts reading `answer` once its size is the row's; a server that sent the whole file restarts the part.
    fn answered(&mut self, answer: Answer, at: usize) -> Step {
        if answer.total.is_some_and(|total| total != self.row.size) {
            return self.pass(at, FetchFault::BadSize);
        }
        if answer.start != self.got {
            if answer.start != 0 {
                return Step::Fail(FetchFault::Internal);
            }
            if let Err(e) = self.restart() {
                return Step::Fail(disk(&e));
            }
        }
        self.first = None;
        self.stage = Stage::Read(answer.body, at);
        Step::Go((self.got != self.told).then(|| self.progress()))
    }

    /// Reads the body's next bytes into the part, never past the row's size; a body that ends short is a cut.
    fn read(&mut self, mut body: Box<dyn Body>, at: usize) -> Step {
        let want = cap(self.row.size - self.got, self.buf.len());
        let n = match body.read(&mut self.buf[..want]) {
            Ok(0) => return self.pass(at, FetchFault::Net),
            Ok(n) => n.min(want),
            Err(e) => return self.pass(at, e.into()),
        };
        if let Err(e) = self.file.write_all(&self.buf[..n]) {
            return Step::Fail(disk(&e));
        }
        self.print.add(&self.buf[..n]);
        self.got += n as u64;
        if self.got < self.row.size {
            self.stage = Stage::Read(body, at);
        }
        let due = self.got >= self.due || self.got == self.row.size;
        Step::Go(due.then(|| self.progress()))
    }

    /// Hands the download to the address after `at`, keeping the first fault; past the last one, fails on it.
    fn pass(&mut self, at: usize, fault: FetchFault) -> Step {
        let first = *self.first.get_or_insert(fault);
        if at + 1 < self.row.urls.len() {
            self.stage = Stage::Ask(at + 1);
            return Step::Go(None);
        }
        Step::Fail(first)
    }

    /// Empties the part and the fingerprint, for a server that sent the whole file.
    fn restart(&mut self) -> io::Result<()> {
        self.file.set_len(0)?;
        self.file.seek(SeekFrom::Start(0))?;
        self.print = Print::default();
        self.got = 0;
        Ok(())
    }
}

impl Row {
    /// The row `ask` names.
    ///
    /// # Errors
    /// A `Disk` fault note when its path is not `plain` or its name is taken, as a file is never replaced.
    fn of(ask: FetchAsk<'static>) -> Result<Self, FetchNote> {
        let FetchAsk::Get {
            id,
            urls,
            hosts,
            path,
            size,
            sha256,
            stall_s,
        } = ask
        else {
            let (id, fault) = (ask.id(), FetchFault::Internal);
            return Err(FetchNote::Fault { id, fault });
        };
        if !plain(&path) || taken(&*path) {
            let fault = FetchFault::Disk;
            return Err(FetchNote::Fault { id, fault });
        }
        let part = PathBuf::from(part_path(&path));
        let path = PathBuf::from(path.into_owned());
        let want = sha256;
        Ok(Self {
            id,
            urls,
            hosts,
            path,
            part,
            size,
            want,
            stall_s,
        })
    }
}

/// The bytes to take of `left`, at most `buf`.
fn cap(left: u64, buf: usize) -> usize {
    usize::try_from(left).map_or(buf, |left| left.min(buf))
}
