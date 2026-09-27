//! P1 spike: the test keyboard in a Tauri 2 window (G2 no focus, G3 top band, G8 memory).
#![windows_subsystem = "windows"]

mod sizes;

use std::error::Error;
use std::ffi::c_void;
use std::sync::Mutex;
use std::time::Duration;

use spike_core::config::{self, KeyboardConfig};
use spike_core::layout::{self, Follow};
use spike_core::status::{self, Guard};
use spike_core::{inject, uiaccess, window};
use tauri::{App, AppHandle, Manager, State, WebviewWindowBuilder};
use windows::Win32::Foundation::HWND;

use sizes::Sizes;

/// Face name that picks the window title in `spike.toml`.
const FACE: &str = "tauri";
/// Number of the first guard try; `status::guard` counts from 1.
const FIRST_TRY: u32 = 1;

/// State shared by the commands.
struct Kb {
    cfg: KeyboardConfig,
    sizes: Sizes,
    /// Fixed part of the status line.
    base: String,
    /// Layout the keys show, so they change only when the app in front changes it.
    follow: Mutex<Follow>,
    /// Last part of the status line: the guard result.
    note: Mutex<String>,
}

/// Guard timing from spike.toml and the window to hand focus back to.
#[derive(Clone, Copy)]
struct Tries {
    /// The window in front at start-up, as a raw handle value.
    prev: isize,
    delay: Duration,
    max: u32,
}

/// Window and status-line sizes for the page.
#[tauri::command]
fn sizes(kb: State<'_, Kb>) -> Sizes {
    kb.sizes
}

/// How often the page asks for new labels, in ms.
#[tauri::command]
fn relabel_ms(kb: State<'_, Kb>) -> u64 {
    kb.cfg.relabel_ms
}

/// Rows labelled for the layout of the app in front, as JSON; each key carries its box.
#[tauri::command]
fn rows(kb: State<'_, Kb>) -> String {
    let hkl = layout::foreground_layout();
    if let Ok(mut follow) = kb.follow.lock() {
        *follow = Follow::new(hkl);
    }
    layout::rows_json(&kb.cfg, hkl)
}

/// Sends one key to the app in front.
#[tauri::command]
fn tap(code: u32) -> Result<(), String> {
    inject::tap(code)
}

/// The status line: uiAccess, key count and the guard result.
#[tauri::command]
fn status(kb: State<'_, Kb>) -> String {
    let note = kb.note.lock().map(|n| n.clone()).unwrap_or_default();
    status::with(&kb.base, &note)
}

/// New rows only when another app is in front and its layout changed.
#[tauri::command]
fn relabel(kb: State<'_, Kb>) -> Option<String> {
    let hkl = kb.follow.lock().ok()?.changed()?;
    Some(layout::rows_json(&kb.cfg, hkl))
}

/// Guards our windows on the UI thread after the delay, retrying while none is visible; then hands focus back once.
fn guard_later(app: AppHandle, tries: Tries, attempt: u32) {
    std::thread::spawn(move || {
        std::thread::sleep(tries.delay);
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            match status::guard(window::guard_own_windows(), attempt, tries.max) {
                Guard::Retry => guard_later(handle, tries, attempt + 1),
                Guard::Done(note) | Guard::GiveUp(note) => {
                    window::give_back(HWND(tries.prev as *mut c_void));
                    if let Ok(mut shown) = handle.state::<Kb>().note.lock() {
                        *shown = note;
                    }
                }
            }
        });
    });
}

/// Opens the keyboard window from `tauri.conf.json` with the title and size from `spike.toml`.
fn open(app: &mut App, title: &str, size: Sizes) -> Result<(), Box<dyn Error>> {
    let conf = app
        .config()
        .app
        .windows
        .first()
        .ok_or("no window in tauri.conf.json")?;
    // WebView2 data goes to TEMP, not beside the exe (Program Files is read-only).
    let data = std::env::temp_dir().join(&app.config().identifier);
    WebviewWindowBuilder::from_config(app.handle(), conf)?
        .title(title)
        .inner_size(size.width, size.height)
        .data_directory(data)
        .build()?;
    Ok(())
}

fn main() {
    // Release builds have no console, so panics and start-up errors go to a TEMP file.
    std::panic::set_hook(Box::new(|info| {
        config::report_error(FACE, &info.to_string())
    }));
    let cfg = config::load().unwrap_or_else(|err| {
        config::report_error(FACE, &err);
        std::process::exit(1);
    });
    let size = Sizes::new(&cfg.keyboard);
    let title = cfg.title(FACE);
    let keys = cfg.keyboard.rows.iter().map(Vec::len).sum();
    let (delay, max) = (cfg.keyboard.guard_delay_ms, cfg.keyboard.guard_tries);
    let kb = Kb {
        cfg: cfg.keyboard,
        sizes: size,
        base: status::base(uiaccess::active(), keys),
        follow: Mutex::new(Follow::default()),
        note: Mutex::new(status::GUARD_PENDING.to_string()),
    };
    tauri::Builder::default()
        .manage(kb)
        .invoke_handler(tauri::generate_handler![
            sizes, relabel_ms, rows, tap, status, relabel
        ])
        .setup(move |app| {
            // Captured before our window exists, so it is the app the user was in.
            let prev = window::foreground().0 as isize;
            let delay = Duration::from_millis(delay);
            open(app, &title, size)?;
            guard_later(app.handle().clone(), Tries { prev, delay, max }, FIRST_TRY);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("the Tauri app runs");
}
