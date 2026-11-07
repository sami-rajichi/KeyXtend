//! What the hidden-run tests share: a data folder, a kernel over it with no module, a hub over the shared fakes, and a
//! port whose deletes wait for a signal.

use crate::fakes;
use kx_kernel::{Kernel, Policy};
use kx_platform::AppDirs;
use kx_platform::ui::{Hidden, HiddenCopy, HiddenError, Probe};
use kx_platform_fake::{FakeHidden, FakePlatform};
use kx_settings::Files;
use kx_test_support::tempdir::TempDir;
use kx_ui::{Hub, Request};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// An app's short name, as the probe and the copies give it.
pub const APP: &str = "Claude";
/// The longest a held delete waits for its signal.
const WAIT: Duration = Duration::from_secs(5);

/// A program folder and an installed data folder, both gone when it drops.
pub struct Bed {
    exe: TempDir,
    data: TempDir,
}

impl Bed {
    /// Two fresh, empty folders.
    pub fn new() -> Self {
        Self {
            exe: TempDir::new("hidden-exe").unwrap(),
            data: TempDir::new("hidden-data").unwrap(),
        }
    }

    /// The data folder.
    pub fn data(&self) -> &Path {
        self.data.path()
    }

    /// The data folder's files.
    pub fn files(&self) -> Files {
        Files::new(self.data())
    }

    /// A kernel with no module over the data folder, as each run makes one.
    pub fn kernel(&self) -> Kernel {
        let dirs = AppDirs {
            exe_dir: self.exe.path().to_path_buf(),
            user_dir: self.data().to_path_buf(),
        };
        Kernel::new(Arc::new(FakePlatform::new(dirs)), Policy::new())
    }

    /// Everything right inside the data folder, A to Z.
    pub fn listing(&self) -> Vec<PathBuf> {
        let mut all: Vec<PathBuf> = std::fs::read_dir(self.data())
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        all.sort();
        all
    }
}

/// A hub over the shared fakes, and the queue of its requests.
pub fn hub() -> (Arc<Hub>, Receiver<Request>) {
    Hub::new(fakes::ports(fakes::tray())).unwrap()
}

/// A port listing through a fake whose deletes wait for a signal, and fail as undeleted when none comes in `WAIT`.
pub struct Held {
    /// The fake behind it, which lists the copies and records each delete.
    pub fake: FakeHidden,
    gate: Mutex<Receiver<()>>,
}

impl Held {
    /// A port listing `copies`, and the sender that lets one delete go.
    pub fn new(copies: Vec<HiddenCopy>) -> (Arc<Self>, Sender<()>) {
        let (open, gate) = mpsc::channel();
        let fake = FakeHidden::new();
        fake.set_copies(Ok(copies));
        let gate = Mutex::new(gate);
        (Arc::new(Self { fake, gate }), open)
    }
}

impl Hidden for Held {
    fn probe(&self, dir: &Path) -> Result<Probe, HiddenError> {
        self.fake.probe(dir)
    }

    fn copies(&self) -> Result<Vec<HiddenCopy>, HiddenError> {
        self.fake.copies()
    }

    fn remove(&self, copy: &HiddenCopy) -> Result<(), HiddenError> {
        let gate = self.gate.lock().unwrap();
        gate.recv_timeout(WAIT)
            .map_err(|_| HiddenError::Undeleted)?;
        self.fake.remove(copy)
    }
}
