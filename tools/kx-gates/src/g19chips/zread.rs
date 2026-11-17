//! The reads behind G19's hung-step notes: the z-order rows, and the keyboard lifted back into the topmost band from
//! outside, as G9 moves our windows. Classes and numbers only.
#![cfg(windows)]

use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::{Map, Value, json};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::UI::WindowsAndMessaging::{
    GW_OWNER, GetWindow, IsWindowVisible, WS_EX_TOPMOST,
};
use windows::core::{BOOL, PCSTR, PCWSTR, s, w};

use crate::g9win::{ex_of, set_topmost};
use crate::g19chips::ontop;
use crate::g19chips::plan::NOTES;
use crate::g19chips::zlist::{self, Pids, Row};
use crate::g25::part;
use crate::p10cfg::G19Product;
use crate::product::Keyboard;
use crate::win;

/// The library that holds `GetWindowBand`, loaded in every program with windows.
const USER32: PCWSTR = w!("user32.dll");
/// The undocumented export that tells a window's z-order band.
const BAND: PCSTR = s!("GetWindowBand");
/// Whether the key was on top within one follow period after the keyboard was lifted from outside.
const RELIFT: &str = "relift_uncovered";
/// The ms the key stayed covered after the lift.
const RELIFT_MS: &str = "relift_ms";
/// Why the key stayed covered after the lift, or why the lift failed.
const RELIFT_WHY: &str = "relift_error";

/// `GetWindowBand`'s shape: a window and where its band goes.
type BandFn = unsafe extern "system" fn(HWND, *mut u32) -> BOOL;

/// `GetWindowBand`, looked up once; none where this Windows lacks it.
#[allow(
    unsafe_code,
    reason = "Looks up an export of a loaded system library and gives it its shape."
)]
fn band_fn() -> Option<BandFn> {
    static FOUND: OnceLock<Option<BandFn>> = OnceLock::new();
    *FOUND.get_or_init(|| {
        // SAFETY: a constant name; user32 stays loaded for the life of a program with windows.
        let user32 = unsafe { GetModuleHandleW(USER32) }.ok()?;
        // SAFETY: a constant export name in that loaded library.
        let found = unsafe { GetProcAddress(user32, BAND) }?;
        // SAFETY: the export takes a window and a pointer to a 32-bit band and returns a BOOL, as `BandFn` says.
        Some(unsafe { std::mem::transmute::<unsafe extern "system" fn() -> isize, BandFn>(found) })
    })
}

/// Window `w`'s z-order band; none when unknown.
#[allow(unsafe_code, reason = "Calls the looked-up export with a local.")]
fn band(w: HWND) -> Option<u32> {
    let f = band_fn()?;
    let mut b = 0u32;
    // SAFETY: the export's own shape and a live local for its answer; a gone window gives FALSE.
    unsafe { f(w, &raw mut b) }.as_bool().then_some(b)
}

/// The window that owns `w`, when it has one.
#[allow(unsafe_code, reason = "A plain owner query.")]
fn owner(w: HWND) -> Option<HWND> {
    // SAFETY: a plain query; a window with no owner gives an error.
    unsafe { GetWindow(w, GW_OWNER) }.ok()
}

/// Window `w` as a row; `at` gives each window's place, and `pids` whose it is.
#[allow(unsafe_code, reason = "A plain visibility query.")]
fn row(w: HWND, at: &HashMap<isize, usize>, pids: &Pids) -> Row {
    let (pid, thread) = win::owner(w);
    // SAFETY: a plain query; any handle value is allowed.
    let shown = unsafe { IsWindowVisible(w) }.as_bool();
    Row {
        class: win::class(w),
        who: pids.who(pid),
        thread,
        owner: owner(w).and_then(|o| at.get(&win::key(o)).copied()),
        top: ex_of(w) & WS_EX_TOPMOST.0 != 0,
        shown,
        cloaked: win::cloaked(w),
        band: band(w),
    }
}

/// The z-order now, top first, around keyboard `kb` and Notepad's window `notepad`, noted as `g` sets its length.
fn snap(kb: HWND, notepad: HWND, pids: &Pids, g: &G19Product) -> Value {
    let all = win::all_windows();
    let at: HashMap<isize, usize> = (all.iter().enumerate())
        .map(|(i, &w)| (win::key(w), i))
        .collect();
    let Some(&k) = at.get(&win::key(kb)) else {
        return json!({ "error": "the keyboard is not among the top-level windows" });
    };
    let rows: Vec<Row> = all.iter().map(|&w| row(w, &at, pids)).collect();
    let pad = at.get(&win::key(notepad)).copied();
    zlist::note(&rows, k, g.z_below, &zlist::verdict(&rows, k, pad))
}

/// The hung step's z-order notes: the keyboard, whose each window is, and what was read so far.
pub struct Notes<'a> {
    kb: Result<HWND, String>,
    notepad: HWND,
    pids: Pids,
    g: &'a G19Product,
    got: Map<String, Value>,
}

impl<'a> Notes<'a> {
    /// Notes on the keyboard found now, beside the gate's Notepad `notepad` and target-window `target`.
    pub fn new(notepad: HWND, target: HWND, g: &'a G19Product) -> Self {
        let kb = Keyboard::find().map(|k| k.hwnd);
        let pid = |w: HWND| win::owner(w).0;
        let ours = kb.as_ref().map_or(0, |&k| pid(k));
        let pids = Pids {
            ours,
            target: pid(target),
            notepad: pid(notepad),
        };
        let got = Map::new();
        Self {
            kb,
            notepad,
            pids,
            g,
            got,
        }
    }

    /// Reads the z-order now and notes it as `moment`.
    pub fn snap(&mut self, moment: &str) {
        let read = |&kb: &HWND| snap(kb, self.notepad, &self.pids, self.g);
        let v = self
            .kb
            .as_ref()
            .map_or_else(|e| json!({ "error": e }), read);
        self.got.insert(moment.into(), v);
    }

    /// Lifts the keyboard into the topmost band from outside, then notes whether key `id` is on top within one follow
    /// period, asked every `poll_ms`.
    pub fn relift(&mut self, poll_ms: u64, id: &str) {
        let lifted = self.kb.clone().and_then(|kb| set_topmost(kb, true));
        let got = lifted.and_then(|()| ontop::covered(poll_ms, id));
        self.got.insert(RELIFT.into(), json!(got.is_ok()));
        let (key, v) = match got {
            Ok(ms) => (RELIFT_MS, json!(ms)),
            Err(e) => (RELIFT_WHY, json!(e)),
        };
        self.got.insert(key.into(), v);
    }

    /// Step result `got` with the notes beside it, never judged; an error becomes a failed result, as `part` makes it.
    pub fn attach(self, got: Result<Value, String>) -> Value {
        let mut v = part(got);
        v[NOTES] = Value::Object(self.got);
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_notes_sit_beside_the_step_result_and_an_error_still_fails_it() {
        let g = crate::config::load().expect("loads").g19_product;
        let mut n = Notes {
            kb: Err("no keyboard".into()),
            notepad: HWND::default(),
            pids: Pids {
                ours: 0,
                target: 0,
                notepad: 0,
            },
            g: &g,
            got: Map::new(),
        };
        n.snap("z");
        let v = n.attach(Err("covered".into()));
        assert_eq!((&v["ok"], &v["error"]), (&json!(false), &json!("covered")));
        assert_eq!(v[NOTES]["z"]["error"], "no keyboard");
    }
}
