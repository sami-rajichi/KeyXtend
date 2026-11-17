//! Any run of downloads, pauses, resumes, cancels and steps over a server that answers whole, corrupt, too long, the
//! wrong size, cut, stalled, partly or not at all: the part never holds more than the row's size, a named file exists
//! only once a Done was told and holds the right bytes, a bad size or fingerprint or a cancel leaves no part, progress
//! stays within the size, and no note comes after a download's end.

use super::*;
use proptest::collection::vec;
use proptest::prelude::*;
use std::collections::BTreeSet;

/// Random cases; each writes files under the target folder.
const CASES: u32 = 64;
/// The most answers the server has, and steps in one case.
const MAX_ACTS: usize = 8;
const MAX_STEPS: usize = 30;
/// The most job steps one step of the case takes.
const MAX_WORK: usize = 12;
/// Bytes a too-long body sends past the file.
const EXTRA: &[u8] = b"past the end";
/// A quiet longer than any stall time.
const FOREVER_MS: u32 = u32::MAX;
/// An id no download takes.
const STRAY: u32 = u32::MAX;
/// The faults after which the part is gone.
const DELETES: [FetchFault; 3] = [
    FetchFault::BadSize,
    FetchFault::BadHash,
    FetchFault::Cancelled,
];

/// How the server answers one request; each place is a byte of `FILE`.
#[derive(Clone, Debug)]
enum Serve {
    Whole,
    /// The whole file with the byte at this place changed.
    Corrupt(usize),
    /// The file, then more bytes than its size, the read that crosses its end starting at this place.
    Long(usize),
    /// A head that names another size.
    Size,
    /// The bytes up to this place, then a cut.
    Cut(usize),
    /// The bytes up to this place, then no byte for longer than the stall time.
    Stall(usize),
    /// A partial answer from this place.
    Part(usize),
    /// No answer.
    Refuse,
}

/// The server's act for `serve`.
fn act(serve: &Serve) -> Act {
    let head = Head::whole(FILE.len());
    let ended = |at: usize, last: Step| {
        let mut steps = reads(&FILE[..at]);
        steps.push(last);
        Act::Answer(head.clone(), steps)
    };
    match *serve {
        Serve::Whole => whole(FILE),
        Serve::Corrupt(at) => {
            let mut body = FILE.to_vec();
            body[at] ^= u8::MAX;
            whole(&body)
        }
        Serve::Long(at) => {
            let cross = Step::Bytes([&FILE[at..], EXTRA].concat());
            Act::Answer(head.clone(), vec![Step::Bytes(FILE[..at].to_vec()), cross])
        }
        Serve::Size => Act::Answer(Head::whole(FILE.len() + 1), reads(FILE)),
        Serve::Cut(at) => ended(at, Step::Fail(NetError::Net)),
        Serve::Stall(at) => ended(at, Step::Quiet(FOREVER_MS)),
        Serve::Part(at) => Act::part(&FILE[at..], at as u64, FILE.len() as u64),
        Serve::Refuse => Act::Fail(NetError::Net),
    }
}

fn serve() -> impl Strategy<Value = Serve> {
    let at = || 0..FILE.len();
    prop_oneof![
        Just(Serve::Whole),
        at().prop_map(Serve::Corrupt),
        at().prop_map(Serve::Long),
        Just(Serve::Size),
        at().prop_map(Serve::Cut),
        at().prop_map(Serve::Stall),
        at().prop_map(Serve::Part),
        Just(Serve::Refuse),
    ]
}

/// One thing the job meets.
#[derive(Clone, Debug)]
enum Ask {
    /// A fresh download, with a mirror when true.
    Get(bool),
    /// Pause, resume or cancel the live download, or a stray id when false.
    Pause(bool),
    Resume(bool),
    Cancel(bool),
    /// The loop steps the download this many times while it reads.
    Work(usize),
}

fn ask() -> impl Strategy<Value = Ask> {
    prop_oneof![
        2 => any::<bool>().prop_map(Ask::Get),
        1 => any::<bool>().prop_map(Ask::Pause),
        1 => any::<bool>().prop_map(Ask::Resume),
        1 => any::<bool>().prop_map(Ask::Cancel),
        4 => (1..=MAX_WORK).prop_map(Ask::Work),
    ]
}

/// Gives `ask` to `job`, or steps it as the loop does: only while it reads.
fn run(job: &mut Job, dir: &Dir, ask: &Ask, next: u32, out: &mut Vec<FetchNote>) {
    let target = |live: bool| {
        if live {
            job.live().unwrap_or(STRAY)
        } else {
            STRAY
        }
    };
    let asked = match *ask {
        Ask::Get(mirror) => {
            let urls: &[&str] = if mirror { &[URL, MIRROR] } else { &[URL] };
            get_of(next, dir, urls, FILE)
        }
        Ask::Pause(live) => FetchAsk::Pause { id: target(live) },
        Ask::Resume(live) => FetchAsk::Resume { id: target(live) },
        Ask::Cancel(live) => FetchAsk::Cancel { id: target(live) },
        Ask::Work(n) => {
            for _ in 0..n {
                if job.busy() {
                    work(job, out, 1);
                }
            }
            return;
        }
    };
    take(job, asked, out);
}

/// What the case saw so far.
#[derive(Default)]
struct Told {
    /// The downloads that told their end.
    ended: BTreeSet<u32>,
    /// A Done was told.
    done: bool,
}

impl Told {
    /// Checks one step's notes and the files after it.
    fn check(&mut self, dir: &Dir, out: &[FetchNote]) -> Result<(), TestCaseError> {
        let mut deleted = false;
        for note in out {
            let id = note.id();
            prop_assert!(!self.ended.contains(&id), "{note:?} after its end");
            match *note {
                FetchNote::Progress { got, total, .. } => {
                    prop_assert!(got <= total && total == FILE.len() as u64, "{note:?}");
                }
                FetchNote::Done { .. } => {
                    self.ended.insert(id);
                    self.done = true;
                }
                FetchNote::Fault { fault, .. } => {
                    self.ended.insert(id);
                    deleted |= DELETES.contains(&fault);
                }
            }
        }
        let (file, part) = dir.files();
        let size = FILE.len();
        prop_assert!(
            part.as_ref().is_none_or(|p| p.len() <= size),
            "the part grew past the size"
        );
        prop_assert!(
            file.as_ref().is_none_or(|f| f == FILE),
            "a named file with the wrong bytes"
        );
        prop_assert_eq!(file.is_some(), self.done, "a named file without its Done");
        prop_assert!(
            !deleted || part.is_none(),
            "a bad size, fingerprint or cancel left the part"
        );
        Ok(())
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn the_part_stays_within_its_size_and_only_a_checked_file_is_named(
        acts in vec(serve(), 0..MAX_ACTS),
        asks in vec(ask(), 1..MAX_STEPS),
    ) {
        let dir = Dir::new();
        let server = Server::new(acts.iter().map(act));
        let mut job = job(&server);
        let mut told = Told::default();
        for (n, ask) in (1..).zip(&asks) {
            let mut out = Vec::new();
            run(&mut job, &dir, ask, n, &mut out);
            told.check(&dir, &out)?;
        }
        job.end();
    }
}
