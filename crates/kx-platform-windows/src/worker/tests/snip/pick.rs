//! The `pick` job's dialogs: a worker under its job and restricted token shows the Save dialog in front of a normal
//! topmost window of ours that stands for the editor, at the folder asked with the proposed Arabic name, and answers
//! the PNG once the dialog's own Save button gets a click message: no mouse, no keys. A second Save, at a folder that
//! is gone, still opens and is cancelled by its own Cancel button, and the folder dialog answers its folder.
#![allow(
    unsafe_code,
    reason = "The test finds, reads and clicks its own worker's dialogs."
)]

use std::borrow::Cow;
use std::io::PipeWriter;
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use kx_ipc::{CLEAN_EXIT, FileKind, PickAsk, PickReply};
use kx_module_api::ports::PickLimits;
use serde::Deserialize;
use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    BM_CLICK, EnumWindows, GA_ROOT, GW_HWNDNEXT, GetAncestor, GetClassNameW, GetDlgItem,
    GetForegroundWindow, GetWindow, GetWindowRect, GetWindowThreadProcessId, IDCANCEL, IDOK,
    IsWindowVisible, MESSAGEBOX_RESULT, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WindowFromPoint,
};
use windows::core::BOOL;

use super::screen::{Plain, Rgb, physical, until, work_area};
use super::{Launch, Probing, Ran, Temp, WAIT, WinSpawner, hold, spawner};

/// The proposed name, a snip's date and time after "snip" in Arabic, with no extension: the dialog adds it.
const NAME: &str = "لقطة 2026-11-24 10-30-05";
/// A folder never made, at which a Save dialog still opens.
const GONE: &str = "gone";
/// The editor's colour.
const EDITOR: Rgb = Rgb(40, 40, 160);
/// How long a dialog may take to show, and its answer to come once closed.
const SHOW_WAIT: Duration = Duration::from_secs(20);
/// The class Windows gives every dialog box.
const DIALOG_CLASS: &str = "#32770";
/// The longest the click message waits for the dialog, in ms.
const CLICK_MS: u32 = 5_000;
/// Room for a class name and its end.
const CLASS_LEN: usize = 64;
/// Snip's shipped settings, read for the dialogs' job cap alone.
const SHIPPED: &str = include_str!("../../../../../kx-mod-snip/defaults.toml");

/// Snip's shipped settings, as far as the probe needs them.
#[derive(Deserialize)]
struct Shipped {
    pick_mb: u32,
}

/// The dialogs' job cap Snip ships with, in MB.
fn shipped_cap() -> u32 {
    toml::from_str::<Shipped>(SHIPPED).unwrap().pick_mb
}

/// A worker on the pick job: asks written on its stdin, answers read on a thread of their own.
struct Picker {
    probing: Probing,
    input: PipeWriter,
    answers: Receiver<PickReply<'static>>,
}

impl Picker {
    /// Starts a worker on the pick job, as the host would, under the shipped `pick_mb` cap, so its peak is told.
    fn start(spawner: &WinSpawner) -> Self {
        let cap = shipped_cap();
        let mut probing = Probing::launch(spawner, &Launch::pick(cap));
        let (input, mut output) = probing.pipes();
        let (tx, answers) = mpsc::channel();
        thread::spawn(move || {
            while let Ok(answer) = PickReply::read(&mut output) {
                if tx.send(answer).is_err() {
                    return;
                }
            }
        });
        Self {
            probing,
            input,
            answers,
        }
    }

    /// Writes `ask` and hands back its dialog once it shows; fails with the job's answer when none does.
    fn show(&mut self, ask: &PickAsk<'_>) -> HWND {
        ask.write(&mut self.input).unwrap();
        let pid = self.probing.pid();
        let mut dialog = None;
        let seen = until(SHOW_WAIT, || {
            dialog = dialog_of(pid);
            dialog.is_some()
        });
        match dialog.filter(|_| seen) {
            Some(dialog) => dialog,
            None => panic!("no dialog showed: {:?}", self.answers.try_recv()),
        }
    }

    /// The job's next answer.
    fn answer(&self) -> PickReply<'static> {
        self.answers.recv_timeout(SHOW_WAIT).expect("an answer")
    }

    /// Says goodbye and waits for the worker's end.
    fn bye(mut self) -> Ran {
        PickAsk::Bye.write(&mut self.input).unwrap();
        self.probing.finish(WAIT)
    }
}

/// A PNG Save ask `id` at `folder`, proposing `NAME`.
fn save(id: u32, folder: &Path) -> PickAsk<'static> {
    PickAsk::Save {
        id,
        folder: Cow::Owned(folder.to_string_lossy().into_owned()),
        name: Cow::Borrowed(NAME),
        kind: FileKind::Png,
    }
}

#[test]
fn the_probe_runs_under_the_shipped_pick_cap_which_its_range_holds() {
    let cap = shipped_cap();
    assert!(PickLimits::JOB_MB.contains(&cap), "{cap}");
}

#[test]
#[cfg_attr(
    not(kx_live_snip),
    ignore = "opens the Save and folder dialogs on the screen; opt-in"
)]
fn the_pick_jobs_dialogs_open_in_front_of_our_topmost_window_and_answer_checked_paths() {
    let (_live, _px, temp) = (hold(), physical(), Temp::new());
    let editor = Plain::new(work_area(), EDITOR);
    editor.show();
    let mut picker = Picker::start(&spawner());
    let first = save(1, temp.path());
    let dialog = picker.show(&first);
    let covered = inside(work_area(), centre(dialog));
    let (front, above, foreground) = standing(dialog, editor.hwnd());
    eprintln!("pick: Save in front {front}, above the editor {above}, foreground {foreground}");
    assert!(
        foreground,
        "a click message may not press an inactive dialog's button"
    );
    press(dialog, IDOK);
    let file = temp.path().join(format!("{NAME}.png"));
    let chosen = PickReply::Picked {
        id: 1,
        path: Cow::Owned(file.to_string_lossy().into_owned()),
    };
    assert_eq!(picker.answer(), chosen);
    assert!(!file.exists(), "a dialog writes nothing");
    press(picker.show(&save(2, &temp.path().join(GONE))), IDCANCEL);
    assert_eq!(picker.answer(), PickReply::Cancelled { id: 2 });
    folder_answers_its_folder(&mut picker, temp.path());
    let ran = picker.bye();
    eprintln!("pick: exit {:?}, peak {:?} MB", ran.code, ran.peak_mb);
    assert_eq!(ran.code, Some(u32::from(CLEAN_EXIT)));
    assert!(
        covered,
        "the editor covers the dialog's centre, so the check means something"
    );
    assert!(
        front && above,
        "the Save dialog opens in front of our topmost window"
    );
}

/// The folder dialog at `folder`, its own button pressed, answers that folder.
fn folder_answers_its_folder(picker: &mut Picker, folder: &Path) {
    let at = Cow::Owned(folder.to_string_lossy().into_owned());
    let ask = PickAsk::Folder { id: 3, at };
    press(picker.show(&ask), IDOK);
    let chosen = PickReply::Picked {
        id: 3,
        path: Cow::Owned(folder.to_string_lossy().into_owned()),
    };
    assert_eq!(picker.answer(), chosen);
}

/// Whether `dialog` is the window at its own centre, comes above `editor` in the z-order, and is the foreground window.
fn standing(dialog: HWND, editor: HWND) -> (bool, bool, bool) {
    // SAFETY: plain reads of the screen's windows.
    let (front, foreground) = unsafe {
        let hit = GetAncestor(WindowFromPoint(centre(dialog)), GA_ROOT);
        (hit == dialog, GetForegroundWindow() == dialog)
    };
    (front, above(dialog, editor), foreground)
}

/// The search `dialog_of` hands its callback: the process id sought and the dialog found.
type Search = (u32, Option<HWND>);

/// The visible dialog box of process `pid`, if one shows.
fn dialog_of(pid: u32) -> Option<HWND> {
    let mut find: Search = (pid, None);
    let lparam = LPARAM(
        std::ptr::from_mut(&mut find)
            .expose_provenance()
            .cast_signed(),
    );
    // SAFETY: the callback reads `find` only during this call, through the address passed.
    let _ = unsafe { EnumWindows(Some(each), lparam) };
    find.1
}

/// Keeps `hwnd` in the search when it is the dialog sought, and stops there.
unsafe extern "system" fn each(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let search = std::ptr::with_exposed_provenance_mut::<Search>(lparam.0.cast_unsigned());
    // SAFETY: `dialog_of` passed the address of its live search, alive and not borrowed during its call.
    let find = unsafe { &mut *search };
    let mut owner = 0;
    let mut name = [0_u16; CLASS_LEN];
    // SAFETY: plain reads of a window Windows just listed, into locals.
    let (len, visible) = unsafe {
        GetWindowThreadProcessId(hwnd, Some(&raw mut owner));
        (
            GetClassNameW(hwnd, &mut name),
            IsWindowVisible(hwnd).as_bool(),
        )
    };
    let class = String::from_utf16_lossy(&name[..usize::try_from(len).unwrap_or_default()]);
    let found = owner == find.0 && visible && class == DIALOG_CLASS;
    if found {
        find.1 = Some(hwnd);
    }
    (!found).into()
}

/// The centre of `hwnd`'s box.
fn centre(hwnd: HWND) -> POINT {
    let mut at = RECT::default();
    // SAFETY: a plain read of a live window's box into a local.
    unsafe { GetWindowRect(hwnd, &raw mut at) }.unwrap();
    POINT {
        x: at.left.midpoint(at.right),
        y: at.top.midpoint(at.bottom),
    }
}

/// Whether `at` lies in `area`.
fn inside(area: RECT, at: POINT) -> bool {
    (area.left..area.right).contains(&at.x) && (area.top..area.bottom).contains(&at.y)
}

/// Whether `top` comes before `under` in the z-order, walking down from `top`.
fn above(top: HWND, under: HWND) -> bool {
    let mut at = top;
    // SAFETY: plain reads of the z-order; a window gone mid-walk ends it.
    while let Ok(next) = unsafe { GetWindow(at, GW_HWNDNEXT) } {
        if next == under {
            return true;
        }
        at = next;
    }
    false
}

/// Sends the dialog's own `button` a click message, which waits at most `CLICK_MS`.
fn press(dialog: HWND, button: MESSAGEBOX_RESULT) {
    // SAFETY: the worker's live dialog and one of its own buttons; the send gives up if the dialog hangs.
    unsafe {
        let button = GetDlgItem(Some(dialog), button.0).unwrap();
        SendMessageTimeoutW(
            button,
            BM_CLICK,
            WPARAM(0),
            LPARAM(0),
            SMTO_ABORTIFHUNG,
            CLICK_MS,
            None,
        );
    }
}
