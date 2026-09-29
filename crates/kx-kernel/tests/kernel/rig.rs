//! A kernel on a fake platform with a temp data folder, watching its bus from before boot.

#![allow(
    clippy::unwrap_used,
    reason = "these fixture helpers run only in tests, where a failed setup should fail the test"
)]

use crate::record::Log;
use crate::samples::Shape;
use kx_kernel::Kernel;
use kx_kernel::grants::Policy;
use kx_module_api::{
    Bus, Event, ModuleId, ModuleState, ModuleStateChanged, Notice, Subscription, keys,
};
use kx_platform::AppDirs;
use kx_platform_fake::FakePlatform;
use kx_settings::{ARG_MODULE, Files};
use kx_test_support::tempdir::TempDir;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// The data folder, inside the rig's temp folder.
const DATA_DIR: &str = "user";

/// The `module-failed` notice about `id`.
pub fn failed(id: ModuleId) -> Notice {
    Notice {
        key: keys::MODULE_FAILED,
        module: Some(id),
        args: vec![(ARG_MODULE, id.to_string())],
    }
}

/// Every `E` a bus carried, in order.
#[derive(Clone)]
pub struct Seen<E>(Arc<Mutex<Vec<E>>>);

impl<E: Event + Clone> Seen<E> {
    /// Starts collecting `E` from `bus` until the guard drops.
    pub fn watch(bus: &Bus) -> (Self, Subscription) {
        let seen = Self(Arc::new(Mutex::new(Vec::new())));
        let keep = seen.clone();
        let guard = bus.subscribe(move |e: &E| keep.0.lock().unwrap().push(e.clone()));
        (seen, guard)
    }

    pub fn all(&self) -> Vec<E> {
        self.0.lock().unwrap().clone()
    }
}

/// The kernel under test, the samples' log and what its bus carried.
/// Fields drop in order, so the kernel stops its modules before the folder goes.
pub struct Rig {
    pub kernel: Kernel,
    pub log: Log,
    notices: Seen<Notice>,
    moves: Seen<ModuleStateChanged>,
    _watch: [Subscription; 2],
    dir: TempDir,
}

impl Rig {
    /// A rig whose policy allows nothing.
    pub fn new() -> Self {
        Self::with(Policy::new())
    }

    /// A rig with `policy`; its watchers subscribe before anything boots.
    pub fn with(policy: Policy) -> Self {
        let dir = TempDir::new("kernel").unwrap();
        let dirs = AppDirs {
            exe_dir: dir.path().to_path_buf(),
            user_dir: dir.path().join(DATA_DIR),
        };
        let kernel = Kernel::new(Arc::new(FakePlatform::new(dirs)), policy);
        let (notices, notice_guard) = Seen::watch(&kernel.bus());
        let (moves, move_guard) = Seen::watch(&kernel.bus());
        Self {
            kernel,
            log: Log::default(),
            notices,
            moves,
            _watch: [notice_guard, move_guard],
            dir,
        }
    }

    /// Adds `shape`, built on the rig's log.
    pub fn add(&mut self, shape: Shape) {
        self.kernel.add(shape.build(&self.log)).unwrap();
    }

    /// Every notice the bus carried so far.
    pub fn notices(&self) -> Vec<Notice> {
        self.notices.all()
    }

    /// The keys of every notice so far.
    pub fn notice_keys(&self) -> Vec<&'static str> {
        self.notices().iter().map(|n| n.key).collect()
    }

    /// Every state change the bus carried so far, as module and new state.
    pub fn moves(&self) -> Vec<(ModuleId, ModuleState)> {
        let pair = |m: ModuleStateChanged| (m.module, m.state);
        self.moves.all().into_iter().map(pair).collect()
    }

    pub fn state(&self, id: ModuleId) -> Option<ModuleState> {
        self.kernel.state(id)
    }

    /// The data folder the platform names; nothing creates it before a save.
    pub fn data(&self) -> PathBuf {
        self.dir.path().join(DATA_DIR)
    }

    /// The settings files in the data folder.
    pub fn files(&self) -> Files {
        Files::new(&self.data())
    }
}
