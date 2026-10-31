//! The app over the fake platform: temp folders with a portable data folder, the fake injector and layouts on the
//! captured EN, FR and AR tables, the fake mouse hook, scroll target, shortcuts, power dialog, box watch, word packs,
//! sealed store, clipboard, shelf, pictures and calendar, and the shared fake UI ports with a reveal the rig keeps; a
//! waker keeps every wake.

use crate::fakes::{ports, tray};
use keyxtend::boot::{Boot, Services};
use kx_mod_keyboard::BoardSpec;
use kx_mod_keyboard::spec::RowKey;
use kx_module_api::ports::{Date, Scan, TypingMode, TypingModeAsk};
use kx_module_api::{Bus, Event, Subscription};
use kx_platform::{AppDirs, PORTABLE_DIR};
use kx_platform_fake::{
    FakeCalendar, FakeClipboard, FakeClipboardWriter, FakeFieldWatch, FakeInjector,
    FakeLayoutSource, FakePictures, FakePlatform, FakePointerHook, FakePowerDialog, FakeReveal,
    FakeScrollTarget, FakeSealedStore, FakeShelf, FakeShortcuts, FakeTray, FakeWordPacks,
};
use kx_settings::Files;
use kx_test_support::tempdir::TempDir;
use kx_ui::Wake;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// The captured layout tables: English, French AZERTY and Arabic (102) AZERTY.
const GOLDEN: [&str; 3] = ["en-us", "fr-fr", "ar-tn"];
/// The longest a test waits for a module's thread; it never sleeps.
pub const WAIT: Duration = Duration::from_secs(10);
/// The calendar's day, hour, minute and offset east of UTC: Paris on 31 October 2026 at 16:32.
const TODAY: (u16, u8, u8) = (2026, 10, 31);
const NOW: (u8, u8, i16) = (16, 32, 60);

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
    /// Behind the mouse hook.
    pub pointer: Arc<FakePointerHook>,
    /// Behind the scroll target.
    pub scroll: Arc<FakeScrollTarget>,
    /// Behind the shortcuts, knowing none until a test sets them.
    pub shortcuts: Arc<FakeShortcuts>,
    /// Behind the power dialog, which counts its opens.
    pub power: Arc<FakePowerDialog>,
    /// Behind the box watch, with focus on no box.
    pub field: Arc<FakeFieldWatch>,
    /// Behind the word packs, shipping no language until a test lists one.
    pub packs: Arc<FakeWordPacks>,
    /// Behind the sealed store, which counts its loads.
    pub sealed: Arc<FakeSealedStore>,
    /// Behind the tray icon.
    pub tray: Arc<FakeTray>,
    /// Behind the clipboard watch, which hears no copy until a test pushes one.
    pub clip: FakeClipboard,
    /// Behind the clipboard writer, which records each write.
    pub writer: FakeClipboardWriter,
    /// Behind the sealed shelf, empty.
    pub shelf: FakeShelf,
    /// Behind the pictures, which draw nothing until a test queues one.
    pub pics: FakePictures,
    /// Behind the calendar, on a fixed day.
    pub calendar: FakeCalendar,
    /// Behind the reveal, which records each folder it shows.
    pub reveal: Arc<FakeReveal>,
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
            pointer: Arc::new(FakePointerHook::new()),
            scroll: Arc::new(FakeScrollTarget::new()),
            shortcuts: Arc::new(FakeShortcuts::new()),
            power: Arc::new(FakePowerDialog::new()),
            field: Arc::new(FakeFieldWatch::new()),
            packs: Arc::new(FakeWordPacks::new()),
            sealed: Arc::new(FakeSealedStore::new()),
            tray: tray(),
            clip: FakeClipboard::new(),
            writer: FakeClipboardWriter::new(),
            shelf: FakeShelf::new(),
            pics: FakePictures::new(),
            calendar: calendar(),
            reveal: Arc::new(FakeReveal::new()),
        }
    }

    /// The data folder's files.
    pub fn files(&self) -> Files {
        Files::new(&self.exe.path().join(PORTABLE_DIR))
    }

    /// A fake platform on the rig's folders.
    pub fn platform(&self) -> Arc<FakePlatform> {
        let dirs = AppDirs {
            exe_dir: self.exe.path().to_path_buf(),
            user_dir: self.user.path().to_path_buf(),
        };
        Arc::new(FakePlatform::new(dirs))
    }

    /// The app loaded over the fakes, as `main` loads it over the Windows adapter.
    pub fn load(&self) -> Boot {
        let services = Services {
            injector: self.injector.clone(),
            layouts: self.layouts.clone(),
            pointer: self.pointer.clone(),
            scroll: self.scroll.clone(),
            shortcuts: self.shortcuts.clone(),
            power: self.power.clone(),
            field: self.field.clone(),
            packs: self.packs.clone(),
            sealed: self.sealed.clone(),
            clipboard: Arc::new(self.clip.clone()),
            writer: Arc::new(self.writer.clone()),
            shelf: Arc::new(self.shelf.clone()),
            pictures: Arc::new(self.pics.clone()),
            calendar: Arc::new(self.calendar.clone()),
        };
        let mut ports = ports(self.tray.clone());
        ports.reveal = self.reveal.clone();
        Boot::load(self.platform(), services, ports).unwrap()
    }

    /// True once the started predict module settled in language `iso`: it opened that pack and heard the typing mode,
    /// so keys typed next follow both, as its thread asks for them only after the start.
    pub fn settled(&self, bus: &Bus, iso: &str) -> bool {
        // Handlers run in subscription order, so a mode this later ear hears reached predict first.
        let (_ear, modes) = hear::<TypingMode>(bus);
        if !self.packs.wait_opens(iso, 1, WAIT) {
            return false;
        }
        // Asked after predict's own ask, which went before its open; the keyboard answers both in order.
        bus.publish(&TypingModeAsk).unwrap();
        modes.recv_timeout(WAIT).is_ok()
    }
}

/// The fake calendar at `TODAY` and `NOW`.
fn calendar() -> FakeCalendar {
    let ((year, month, day), (hour, minute, offset)) = (TODAY, NOW);
    FakeCalendar::new(Date::new(year, month, day).unwrap(), hour, minute, offset)
}

/// Sends every `E` published on `bus` to the returned channel.
pub fn hear<E: Event + Clone>(bus: &Bus) -> (Subscription, Receiver<E>) {
    let (tx, rx) = mpsc::channel();
    let ear = bus.subscribe(move |e: &E| {
        let _ = tx.send(e.clone());
    });
    (ear.unwrap(), rx)
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

/// The folder `cargo xtask words` writes the packs to: `xtask/words.toml`'s `[text] out`, under the workspace root.
pub fn words_out() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let manifest = std::fs::read_to_string(root.join("xtask/words.toml")).unwrap();
    let manifest: toml::Table = manifest.parse().unwrap();
    root.join(manifest["text"]["out"].as_str().unwrap())
}
