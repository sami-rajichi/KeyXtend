//! Boots two sample modules over the real settings store, in a folder given on the command line.
//! Run: `cargo run -p kx-kernel --example settings_demo -- <folder> [set [ms]]`.
//! A repair from the previous copy needs two saves with different values first.

#![forbid(unsafe_code)]

use kx_kernel::Kernel;
use kx_kernel::grants::Policy;
use kx_module_api::{
    Manifest, Module, ModuleCx, ModuleError, ModuleId, Notice, Settings, SettingsError,
    SettingsSpec, Subscription, keys, validate_as,
};
use kx_platform::{AppDirs, PORTABLE_DIR};
use kx_platform_fake::FakePlatform;
use kx_settings::Files;
use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsString;
use std::fmt::Debug;
use std::fs;
use std::marker::PhantomData;
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// The word that asks for one saved change.
const SET_WORD: &str = "set";
/// A hint for the manual check, printed with the usage.
const NOTE: &str =
    "note: a repair from the previous copy needs two saves with different values first";
/// The folder of the user's files, inside the given folder.
const USER_DIR: &str = "user";
/// The sample module that holds the change.
const MOUSE: ModuleId = ModuleId::new("mouse");
/// The sample module with the keyboard values.
const KEYBOARD: ModuleId = ModuleId::new("keyboard");
/// The value `set` changes.
const CHANGED_KEY: &str = "hold_ms";
/// The hold time `set` writes when no number follows it.
const DEFAULT_MS: i64 = 1200;
/// The sizes a step may take.
const STEP: RangeInclusive<u32> = 1..=10;
/// The gaps between keys, in pixels.
const GAP_PX: RangeInclusive<u32> = 0..=32;
/// The hold times, in milliseconds.
const HOLD_MS: RangeInclusive<u32> = 100..=5000;
/// Why a size step is refused.
const BAD_STEP: &str = "size_step is out of range";
/// Why a key gap is refused.
const BAD_GAP: &str = "key_gap_px is out of range";
/// Why a hold time is refused.
const BAD_HOLD: &str = "hold_ms is out of range";

/// The keyboard values.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Keyboard {
    size_step: u32,
    key_gap_px: u32,
}

/// The mouse values.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Mouse {
    hold_ms: u32,
}

/// Refuses `value` outside `range`, with `why`.
fn within(value: u32, range: &RangeInclusive<u32>, why: &str) -> Result<(), SettingsError> {
    if range.contains(&value) {
        Ok(())
    } else {
        Err(SettingsError::Invalid(why.into()))
    }
}

impl Settings for Keyboard {
    fn check(&self) -> Result<(), SettingsError> {
        within(self.size_step, &STEP, BAD_STEP)?;
        within(self.key_gap_px, &GAP_PX, BAD_GAP)
    }
}

impl Settings for Mouse {
    fn check(&self) -> Result<(), SettingsError> {
        within(self.hold_ms, &HOLD_MS, BAD_HOLD)
    }
}

static KEYBOARD_SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: "size_step = 3\nkey_gap_px = 4\n",
    migrations: &[],
    validate: validate_as::<Keyboard>,
};

static MOUSE_SPEC: SettingsSpec = SettingsSpec {
    version: 1,
    defaults: "hold_ms = 1500\n",
    migrations: &[],
    validate: validate_as::<Mouse>,
};

/// A manifest for a module that needs nothing and provides nothing.
const fn manifest(id: ModuleId, spec: &'static SettingsSpec) -> Manifest {
    Manifest {
        id,
        version: env!("CARGO_PKG_VERSION"),
        requires: &[],
        provides: &[],
        capabilities: &[],
        settings: Some(spec),
    }
}

static KEYBOARD_MANIFEST: Manifest = manifest(KEYBOARD, &KEYBOARD_SPEC);
static MOUSE_MANIFEST: Manifest = manifest(MOUSE, &MOUSE_SPEC);

/// A sample module: it reads its settings `T` at start and says what it got.
struct Demo<T> {
    manifest: &'static Manifest,
    values: PhantomData<fn() -> T>,
}

impl<T> Demo<T> {
    const fn new(manifest: &'static Manifest) -> Self {
        Self {
            manifest,
            values: PhantomData,
        }
    }
}

impl<T: Settings + Debug + 'static> Module for Demo<T> {
    fn manifest(&self) -> &'static Manifest {
        self.manifest
    }

    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError> {
        let values: T = cx.settings()?;
        say(&format!("{} uses: {values:?}", self.manifest.id));
        Ok(())
    }
}

/// Prints one line; this is the example's only output.
#[allow(
    clippy::print_stdout,
    reason = "An example shows the owner what happened, on the screen."
)]
fn say(text: &str) {
    println!("{text}");
}

/// One notice as one plain line: its key, then its module and args.
fn describe(notice: &Notice) -> String {
    let module = notice.module.map(|id| format!("module {id}"));
    let args = notice
        .args
        .iter()
        .map(|(name, value)| format!("{name}={value}"));
    let parts: Vec<String> = module.into_iter().chain(args).collect();
    if parts.is_empty() {
        format!("notice: {}", notice.key)
    } else {
        format!("notice: {} ({})", notice.key, parts.join(", "))
    }
}

/// Says each notice, then each module's state.
fn report(kernel: &Kernel, notices: &[Notice]) {
    if notices.is_empty() {
        say("no notices");
    }
    for notice in notices {
        say(&describe(notice));
    }
    for id in [KEYBOARD, MOUSE] {
        if let Some(state) = kernel.state(id) {
            say(&format!("{id}: {state:?}"));
        }
    }
}

/// Remembers whether the kernel said a save failed.
struct Unsaved {
    failed: Arc<AtomicBool>,
    _guard: Subscription,
}

impl Unsaved {
    /// Watches the kernel's bus from now on.
    fn watch(kernel: &Kernel) -> Self {
        let failed = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&failed);
        let guard = kernel.bus().subscribe(move |notice: &Notice| {
            if notice.key == keys::SETTINGS_UNSAVED {
                flag.store(true, Ordering::Relaxed);
            }
        });
        Self {
            failed,
            _guard: guard,
        }
    }

    fn happened(&self) -> bool {
        self.failed.load(Ordering::Relaxed)
    }
}

/// The settings file text, or `None` when it is missing or unreadable.
fn text(files: &Files) -> Option<String> {
    fs::read_to_string(&files.settings).ok()
}

/// Sets the mouse hold time to `ms`, then says what happened to the file.
fn save(kernel: &mut Kernel, files: &Files, ms: i64, unsaved: &Unsaved) -> Result<(), String> {
    let before = text(files);
    let value = toml::Value::Integer(ms);
    let set = kernel.set_setting(MOUSE, CHANGED_KEY, value);
    set.map_err(|e| e.to_string())?;
    if text(files) != before {
        say(&format!("saved: {MOUSE} {CHANGED_KEY} = {ms}"));
        say(&format!("settings file: {}", files.settings.display()));
    } else if unsaved.happened() {
        say(&format!(
            "could not save {CHANGED_KEY} = {ms}: it lives in memory only"
        ));
    } else {
        say(&format!("no change: {CHANGED_KEY} is already {ms}"));
    }
    Ok(())
}

/// Boots both modules on the settings in `folder`; with `change`, saves one new hold time.
fn run(folder: &Path, change: Option<i64>) -> Result<(), String> {
    let dirs = AppDirs {
        exe_dir: folder.to_path_buf(),
        user_dir: folder.join(USER_DIR),
    };
    let data = folder.join(PORTABLE_DIR);
    fs::create_dir_all(&data).map_err(|e| format!("cannot make {}: {e}", data.display()))?;
    let files = Files::new(dirs.data_dir().path());
    say(&format!("data folder: {}", files.dir.display()));

    let mut kernel = Kernel::new(Arc::new(FakePlatform::new(dirs)), Policy::new());
    let keyboard = Demo::<Keyboard>::new(&KEYBOARD_MANIFEST);
    let mouse = Demo::<Mouse>::new(&MOUSE_MANIFEST);
    kernel.add(Box::new(keyboard)).map_err(|e| e.to_string())?;
    kernel.add(Box::new(mouse)).map_err(|e| e.to_string())?;

    let unsaved = Unsaved::watch(&kernel);
    let notices = kernel.boot();
    report(&kernel, &notices);
    change.map_or(Ok(()), |ms| save(&mut kernel, &files, ms, &unsaved))
}

/// Reads `<folder> [set [ms]]` from the arguments; `None` when they do not fit.
/// The result holds the folder and, for `set`, the hold time to save.
fn parse(mut args: impl Iterator<Item = OsString>) -> Option<(PathBuf, Option<i64>)> {
    let folder = PathBuf::from(args.next()?);
    let change = match args.next() {
        None => None,
        Some(word) if word == SET_WORD => match args.next() {
            None => Some(DEFAULT_MS),
            Some(ms) => Some(ms.to_str()?.parse().ok()?),
        },
        Some(_) => return None,
    };
    args.next().is_none().then_some((folder, change))
}

fn main() -> ExitCode {
    let Some((folder, change)) = parse(env::args_os().skip(1)) else {
        say(&format!("usage: settings_demo <folder> [{SET_WORD} [ms]]"));
        say(NOTE);
        return ExitCode::FAILURE;
    };
    match run(&folder, change) {
        Ok(()) => ExitCode::SUCCESS,
        Err(why) => {
            say(&format!("stopped: {why}"));
            ExitCode::FAILURE
        }
    }
}
