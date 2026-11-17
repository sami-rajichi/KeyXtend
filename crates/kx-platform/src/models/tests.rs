//! The models host over a fake disk and scripted fetch workers on a clock that moves only when a test moves it;
//! folder tests use a real folder under the system's temporary folder on Windows, where paths are drive paths.

use super::*;
use crate::worker::fake::{Call, FetchAct, Hands, Script};
use kx_module_api::Redacted;
use kx_module_api::ports::{JobLimits, Link};
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

mod fetch;
#[cfg(windows)]
mod folder;

/// How long a test waits for a note; a wait, never a sleep.
const LONG: Duration = Duration::from_secs(10);
/// A test file's bytes, the room the drive keeps, and the test fingerprint's bytes.
const BYTES: u64 = 4_000;
const MIN_FREE_MB: u32 = 256;
const PRINT: [u8; SHA256_BYTES] = [7; SHA256_BYTES];
/// A test download's address and host, and a file on a drive no test machine has.
const URL: &str = "https://models.example/a.gguf";
const HOST: &str = "models.example";
const FILE: &str = r"Q:\kx-test\a.gguf";

/// A disk with `free` bytes on every drive, `None` when the folder is gone, that hides the names in `hidden`.
#[derive(Default)]
struct FakeDisk {
    free: Option<u64>,
    hidden: Vec<&'static str>,
    /// A guarded folder is refused as the system would refuse it.
    refuse: bool,
    guarded: Mutex<Vec<PathBuf>>,
}

impl Disk for FakeDisk {
    fn drives(&self) -> Vec<Drive> {
        let (root, name) = (
            Redacted::new(PathBuf::from(r"Q:\")),
            Redacted::new("Q:".into()),
        );
        let free = self.free.unwrap_or_default();
        vec![Drive { root, name, free }]
    }

    fn free(&self, _folder: &Path) -> io::Result<u64> {
        self.free.ok_or_else(|| io::ErrorKind::NotFound.into())
    }

    fn sha256(&self, _file: &Path) -> io::Result<[u8; SHA256_BYTES]> {
        self.free
            .map(|_| PRINT)
            .ok_or_else(|| io::ErrorKind::NotFound.into())
    }

    fn hidden(&self, path: &Path) -> bool {
        let name = path.file_name().and_then(|n| n.to_str());
        name.is_some_and(|n| self.hidden.contains(&n))
    }

    fn make_guarded(&self, folder: &Path) -> io::Result<()> {
        if self.refuse {
            return Err(io::ErrorKind::PermissionDenied.into());
        }
        std::fs::create_dir(folder)?;
        self.guarded.lock().unwrap().push(folder.to_path_buf());
        Ok(())
    }
}

/// A disk whose drive has `free` bytes.
fn roomy(free: u64) -> FakeDisk {
    FakeDisk {
        free: Some(free),
        ..FakeDisk::default()
    }
}

/// The room a test download needs with no part yet.
fn need() -> u64 {
    u64::from(MIN_FREE_MB) * (1 << 20) + BYTES
}

/// A models host over `disk` whose fetch workers `script` starts.
fn host(script: &Script<FetchFrames>, disk: FakeDisk) -> WorkerModels<Script<FetchFrames>> {
    WorkerModels::new(script.clone(), Arc::new(Hands::default()), Arc::new(disk))
}

/// Test job limits, each inside its range.
fn limits() -> JobLimits {
    JobLimits {
        job_mb: 128,
        idle_min: 5,
        ask_ms: 3_000,
        load_s: 60,
        warm_s: 180,
        piece_wait_s: 120,
        restart_ms: 1_000,
        restart_max_ms: 30_000,
    }
}

/// A download of the test file into `file`.
fn ask(file: &str) -> FetchAsk {
    FetchAsk {
        job: limits(),
        links: vec![Link::new(URL.to_owned()).unwrap()],
        hosts: vec![HOST.to_owned()],
        file: Redacted::new(PathBuf::from(file)),
        bytes: BYTES,
        print: Fingerprint::new(PRINT),
        min_free_mb: MIN_FREE_MB,
        stall_s: 30,
    }
}

/// A download's callback and what it hears.
fn ear() -> (Box<dyn Fn(FetchNote) + Send>, Receiver<FetchNote>) {
    let (said, heard) = mpsc::channel();
    let tell = move |note| {
        let _ = said.send(note);
    };
    (Box::new(tell), heard)
}

#[test]
fn each_system_error_is_the_ports_and_a_read_knows_only_missing_or_io() {
    let cases = [
        (
            io::ErrorKind::NotFound,
            ModelError::Missing,
            ModelError::Missing,
        ),
        (
            io::ErrorKind::PermissionDenied,
            ModelError::ReadOnly,
            ModelError::Io,
        ),
        (
            io::ErrorKind::ReadOnlyFilesystem,
            ModelError::ReadOnly,
            ModelError::Io,
        ),
        (io::ErrorKind::StorageFull, ModelError::Full, ModelError::Io),
        (io::ErrorKind::Other, ModelError::Io, ModelError::Io),
    ];
    for (kind, write, read) in cases {
        let e = io::Error::from(kind);
        assert_eq!((fault(&e), read_fault(&e)), (write, read), "{kind:?}");
    }
}

#[test]
fn a_path_that_is_no_plain_drive_path_is_refused_before_the_disk() {
    let models = host(&Script::default(), roomy(need()));
    for bad in [
        r"relative\models",
        r"Q:\models\..\up",
        r"Q:\models\a.gguf:stream",
    ] {
        let path = Path::new(bad);
        assert_eq!(models.make(path), Err(ModelError::BadPath), "{bad}");
        assert_eq!(models.scan(path, &[]), Err(ModelError::BadPath));
        assert_eq!(models.fingerprint(path), Err(ModelError::BadPath));
        assert_eq!(models.remove(path), Err(ModelError::BadPath));
    }
}

#[test]
fn the_drives_and_a_files_fingerprint_come_from_the_disk() {
    let models = host(&Script::default(), roomy(need()));
    assert_eq!(models.drives()[0].free, need());
    assert_eq!(
        models.fingerprint(Path::new(FILE)),
        Ok(Fingerprint::new(PRINT))
    );
    let gone = host(&Script::default(), FakeDisk::default());
    assert_eq!(gone.fingerprint(Path::new(FILE)), Err(ModelError::Missing));
}

/// The fetch asks a script's workers took, in order.
fn gets(script: &Script<FetchFrames>) -> Vec<kx_ipc::FetchAsk<'static>> {
    let fetch = |call| match call {
        Call::Fetch(ask) => Some(ask),
        _ => None,
    };
    script.calls().into_iter().filter_map(fetch).collect()
}

/// What each act says, for the scripted job.
fn says(notes: Vec<fn(u32) -> kx_ipc::FetchNote>) -> FetchAct {
    FetchAct::Say(notes)
}
