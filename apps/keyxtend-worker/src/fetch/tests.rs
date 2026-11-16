//! The fetch job over a scripted server: whole, cut, resumed, paused, cancelled and stalled downloads, the final check,
//! the address rules and the loop. None touches the network or sleeps; files live in a folder under the target folder.

mod check;
mod hosts;
mod pipe;
mod props;
mod resume;
mod stop;

use super::Notes;
use super::hash::Print;
use super::job::Job;
use kx_ipc::{FetchAsk, FetchFault, FetchNote, IpcError, SHA256_BYTES};
use kx_platform_net::fake::{Act, FakeHttp, Seen, Step};
use kx_platform_net::{Head, Hop, Http, NetError, Reply};
use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// The file most tests download: 40 bytes, read in `STEP`-byte reads.
const FILE: &[u8] = b"0123456789abcdefghijABCDEFGHIJklmnopqrst";
/// The bytes one scripted read gives.
const STEP: usize = 10;
/// The table's first address.
const URL: &str = "https://models.example/m.gguf";
/// A mirror on the other listed host.
const MIRROR: &str = "https://cdn.example/m.gguf";
/// The hosts every test download lists.
const HOSTS: [&str; 2] = ["models.example", "cdn.example"];
/// The stall time every test asks with.
const STALL_S: u16 = 30;
/// The most steps `drive` takes, so a stuck job fails its test instead of hanging it.
const MAX_STEPS: usize = 10_000;
/// The folder, in the test program's profile folder, that holds every test's files.
const ROOT: &str = "kx-fetch-tests";

/// Makes each test's folder name unique within this process.
static UNIQUE: AtomicU32 = AtomicU32::new(0);

/// A fresh folder under the target folder, deleted when dropped.
struct Dir(PathBuf);

impl Dir {
    fn new() -> Self {
        let exe = std::env::current_exe().unwrap();
        let profile = exe.parent().and_then(Path::parent).unwrap();
        let n = UNIQUE.fetch_add(1, Ordering::Relaxed);
        let path = profile
            .join(ROOT)
            .join(format!("{}-{n}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    /// The file the downloads name.
    fn file(&self) -> PathBuf {
        self.0.join("m.gguf")
    }

    /// Its part.
    fn part(&self) -> PathBuf {
        self.0.join("m.gguf.part")
    }

    /// The named file's bytes and the part's, each `None` when absent.
    fn files(&self) -> (Option<Vec<u8>>, Option<Vec<u8>>) {
        (fs::read(self.file()).ok(), fs::read(self.part()).ok())
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
        if let Some(root) = self.0.parent() {
            // Fails while another test's folder is still there; the last one removes it.
            let _ = fs::remove_dir(root);
        }
    }
}

/// A scripted server the job and its test share.
#[derive(Clone)]
struct Server(Arc<FakeHttp>);

impl Server {
    fn new(acts: impl IntoIterator<Item = Act>) -> Self {
        Self(Arc::new(FakeHttp::new(acts)))
    }

    fn seen(&self) -> Vec<Seen> {
        self.0.seen()
    }
}

impl Http for Server {
    fn send(&self, hop: &Hop<'_>) -> Result<Reply, NetError> {
        self.0.send(hop)
    }
}

impl Notes for Vec<FetchNote> {
    fn tell(&mut self, note: FetchNote) -> Result<(), IpcError> {
        self.push(note);
        Ok(())
    }
}

/// `bytes`' SHA-256.
fn print(bytes: &[u8]) -> [u8; SHA256_BYTES] {
    let mut print = Print::default();
    print.add(bytes);
    print.done()
}

/// A download of `bytes` from `urls` into `dir`'s file, with their size and fingerprint.
fn get_of(id: u32, dir: &Dir, urls: &[&str], bytes: &[u8]) -> FetchAsk<'static> {
    FetchAsk::Get {
        id,
        urls: urls.iter().map(|u| Cow::Owned((*u).to_owned())).collect(),
        hosts: HOSTS.iter().map(|h| Cow::Borrowed(*h)).collect(),
        path: Cow::Owned(dir.file().to_string_lossy().into_owned()),
        size: bytes.len() as u64,
        sha256: print(bytes),
        stall_s: STALL_S,
    }
}

/// A download of `FILE` from `URL`.
fn get(id: u32, dir: &Dir) -> FetchAsk<'static> {
    get_of(id, dir, &[URL], FILE)
}

/// `bytes` as one scripted read per `STEP` bytes.
fn reads(bytes: &[u8]) -> Vec<Step> {
    bytes
        .chunks(STEP)
        .map(|c| Step::Bytes(c.to_vec()))
        .collect()
}

/// A whole-file answer holding `bytes`, read in steps.
fn whole(bytes: &[u8]) -> Act {
    Act::Answer(Head::whole(bytes.len()), reads(bytes))
}

/// A job over `server`, with no download yet.
fn job(server: &Server) -> Job {
    Job::new(Box::new(server.clone()))
}

/// Gives `ask` to `job`.
fn take(job: &mut Job, ask: FetchAsk<'static>, out: &mut Vec<FetchNote>) {
    job.take(ask, out).unwrap();
}

/// Steps `job` `n` times.
fn work(job: &mut Job, out: &mut Vec<FetchNote>, n: usize) {
    for _ in 0..n {
        job.work(out).unwrap();
    }
}

/// Steps `job` until its download stops reading, ended or paused.
fn drive(job: &mut Job, out: &mut Vec<FetchNote>) {
    for _ in 0..MAX_STEPS {
        if !job.busy() {
            return;
        }
        job.work(out).unwrap();
    }
    panic!("the download never stopped");
}

/// What the fake should have been asked: `url` from byte `from`.
fn seen(url: &str, from: u64) -> Seen {
    Seen {
        url: url.to_owned(),
        from,
        stall_s: STALL_S,
    }
}

fn progress(id: u32, got: u64) -> FetchNote {
    let total = FILE.len() as u64;
    FetchNote::Progress { id, got, total }
}

fn fault(id: u32, fault: FetchFault) -> FetchNote {
    FetchNote::Fault { id, fault }
}

fn done(id: u32) -> FetchNote {
    FetchNote::Done { id }
}

#[test]
fn the_fingerprint_is_sha_256() {
    use std::fmt::Write as _;
    let hex = |p: [u8; SHA256_BYTES]| {
        p.iter().fold(String::new(), |mut text, b| {
            let _ = write!(text, "{b:02x}");
            text
        })
    };
    let empty = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    assert_eq!(hex(print(b"")), empty);
    assert_eq!(hex(print(b"abc")), abc);
    let mut split = Print::default();
    split.add(b"a");
    split.add(b"bc");
    assert_eq!(hex(split.done()), abc, "fed in pieces");
}
