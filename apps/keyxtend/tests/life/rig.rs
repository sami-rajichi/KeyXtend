//! The app over the fake platform: temp folders with a portable data folder, the fake injector and layouts on the
//! captured EN, FR and AR tables, and the shared fake UI ports; a waker keeps every wake.

use crate::fakes::{ports, tray};
use keyxtend::boot::{Boot, Services};
use kx_mod_keyboard::BoardSpec;
use kx_mod_keyboard::spec::RowKey;
use kx_module_api::ports::Scan;
use kx_platform::{AppDirs, PORTABLE_DIR};
use kx_platform_fake::{FakeInjector, FakeLayoutSource, FakePlatform, FakeTray};
use kx_settings::Files;
use kx_test_support::tempdir::TempDir;
use kx_ui::Wake;
use std::sync::{Arc, Mutex};

/// The captured layout tables: English, French AZERTY and Arabic (102) AZERTY.
const GOLDEN: [&str; 3] = ["en-us", "fr-fr", "ar-tn"];

/// The app's folders and fakes; the folders go when it drops.
pub struct Rig {
    /// The program's folder, holding the portable data folder.
    pub exe: TempDir,
    /// The per-user folder, which the portable edition leaves alone.
    pub user: TempDir,
    /// Behind the input injector.
    pub injector: Arc<FakeInjector>,
    /// Behind the layout source, with the app in front on the first layout.
    pub layouts: Arc<FakeLayoutSource>,
    /// Behind the tray icon.
    pub tray: Arc<FakeTray>,
}

impl Rig {
    /// Fresh folders and fakes, with an empty portable data folder.
    pub fn new() -> Self {
        let exe = TempDir::new("app-exe").unwrap();
        std::fs::create_dir(exe.path().join(PORTABLE_DIR)).unwrap();
        Self {
            exe,
            user: TempDir::new("app-user").unwrap(),
            injector: Arc::new(FakeInjector::new()),
            layouts: Arc::new(FakeLayoutSource::golden(&GOLDEN).unwrap()),
            tray: tray(),
        }
    }

    /// The data folder's files.
    pub fn files(&self) -> Files {
        Files::new(&self.exe.path().join(PORTABLE_DIR))
    }

    /// The app loaded over the fakes, as `main` loads it over the Windows adapter.
    pub fn load(&self) -> Boot {
        let dirs = AppDirs {
            exe_dir: self.exe.path().to_path_buf(),
            user_dir: self.user.path().to_path_buf(),
        };
        let services = Services {
            injector: self.injector.clone(),
            layouts: self.layouts.clone(),
        };
        let ports = ports(self.tray.clone());
        Boot::load(Arc::new(FakePlatform::new(dirs)), services, ports).unwrap()
    }
}

/// Every wake the hub hands the UI, oldest first.
pub type Wakes = Arc<Mutex<Vec<Wake>>>;

/// Sets `hub`'s waker to one that keeps each wake.
pub fn watch(hub: &kx_ui::Hub) -> Wakes {
    let wakes = Wakes::default();
    let seen = Arc::clone(&wakes);
    hub.set_waker(Box::new(move |w| seen.lock().unwrap().push(w)));
    wakes
}

/// The board id of the character key on scan code `sc`.
pub fn char_key(sc: u16) -> String {
    let spec = BoardSpec::shipped().unwrap();
    let mut keys = (0..spec.rows.len()).flat_map(|r| spec.row_keys(r));
    keys.find_map(|k| match k {
        RowKey::Char { id, sc: s } if s == Scan::new(sc) => Some(id),
        _ => None,
    })
    .unwrap()
}
