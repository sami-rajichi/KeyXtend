//! The tray icon over a fake taskbar: each test has its own window class, and no test touches the real taskbar.

use super::fake_shell::{FakeShell, send};
use super::front::away;
use super::*;
use std::sync::mpsc::{self, Receiver};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowExW, FindWindowW, WM_CONTEXTMENU};
use windows::core::{PCWSTR, w};

/// A mouse move over the icon, which opens nothing.
const MOUSE_MOVE: isize = 0x0200;
/// `NIN_SELECT`, a click on the icon.
const SELECT: isize = 0x0400;

/// A tray over a fake taskbar, with a class no other test uses.
fn rig(test: &str) -> (Arc<FakeShell>, WinTray, Vec<u16>) {
    let class = format!("KeyXtendTest.{}.tray.{test}", std::process::id());
    let shell = Arc::new(FakeShell::default());
    let tray = WinTray::with(&class, Arc::clone(&shell) as Arc<dyn Shell>);
    (shell, tray, wide(&class))
}

fn image() -> TrayImage {
    TrayImage::new(2, 2, vec![0xFF; 16]).unwrap()
}

/// A listener that forwards each click, and where they arrive; a window another app brings to the front meanwhile
/// is not the test's business.
fn clicks() -> (OnTray, Receiver<TrayEvent>) {
    let (tx, rx) = mpsc::channel();
    let on: OnTray = Box::new(move |e| {
        if matches!(e, TrayEvent::Click(_)) {
            let _ = tx.send(e);
        }
    });
    (on, rx)
}

/// The test's icon window, if it exists.
#[allow(unsafe_code, reason = "Finds the test's own window.")]
fn find(class: &[u16]) -> Option<HWND> {
    // SAFETY: the class name ends with a null.
    unsafe { FindWindowExW(None, None, PCWSTR(class.as_ptr()), PCWSTR::null()) }.ok()
}

#[test]
fn an_icon_is_added_with_its_tip_and_version_then_deleted_with_its_window() {
    let (shell, tray, class) = rig("added");
    let icon = tray.show(&image(), "KeyXtend", Box::new(|_| {})).unwrap();
    let (whats, calls) = shell.wait(2);
    assert_eq!(whats, [NIM_ADD.0, NIM_SETVERSION.0]);
    let window = find(&class).expect("the icon's window");
    let add = &calls[0];
    assert_eq!(
        (add.window, add.id, add.msg),
        (window.0.addr(), ICON_ID, CALLBACK_MSG)
    );
    assert_eq!(
        add.flags,
        (NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP).0
    );
    assert!(add.icon && add.tip == "KeyXtend");
    assert_eq!(calls[1].version, NOTIFYICON_VERSION_4);
    drop(icon);
    assert_eq!(shell.take().0, [NIM_DELETE.0]);
    assert!(find(&class).is_none(), "the window went with the icon");
}

#[test]
fn a_click_or_the_menu_key_opens_at_the_anchor_and_a_mouse_move_does_not() {
    let (shell, tray, class) = rig("click");
    let (on, rx) = clicks();
    let _icon = tray.show(&image(), "t", on).unwrap();
    shell.wait(2);
    let window = find(&class).unwrap();
    let anchor = (0x28 << 16) | 0x1E;
    let id = 1 << 16;
    send(window, CALLBACK_MSG, anchor, id | MOUSE_MOVE);
    send(window, CALLBACK_MSG, anchor, id | SELECT);
    send(
        window,
        CALLBACK_MSG,
        anchor,
        id | isize::try_from(WM_CONTEXTMENU).unwrap(),
    );
    let at = TrayEvent::Click(kx_module_api::ports::Point { x: 30, y: 40 });
    assert_eq!(rx.try_iter().collect::<Vec<_>>(), [at, at]);
    assert!(shell.take().0.is_empty(), "clicks call no taskbar");
}

#[test]
fn a_restarted_taskbar_gets_the_icon_again_with_its_latest_picture() {
    let (shell, tray, class) = rig("restart");
    let icon = tray.show(&image(), "old", Box::new(|_| {})).unwrap();
    shell.wait(2);
    icon.set(&image(), "new").unwrap();
    let (whats, calls) = shell.take();
    assert_eq!(whats, [NIM_MODIFY.0]);
    assert_eq!(calls[0].tip, "new");
    send(find(&class).unwrap(), restart_msg(), 0, 0);
    let (whats, calls) = shell.wait(2);
    assert_eq!(whats, [NIM_ADD.0, NIM_SETVERSION.0]);
    assert_eq!(calls[0].tip, "new");
    drop(icon);
    shell.take();
    assert!(find(&class).is_none());
}

#[test]
fn a_taskbar_not_there_yet_gets_the_icon_when_it_starts() {
    let (shell, tray, class) = rig("late");
    shell.refuse(&[NIM_ADD, NIM_MODIFY]);
    let icon = tray.show(&image(), "t", Box::new(|_| {})).unwrap();
    assert_eq!(
        shell.wait(2).0,
        [NIM_ADD.0, NIM_MODIFY.0],
        "no version without an icon"
    );
    icon.set(&image(), "later").unwrap();
    assert!(shell.take().0.is_empty(), "the coming add carries a change");
    shell.refuse(&[]);
    send(find(&class).unwrap(), restart_msg(), 0, 0);
    let (whats, calls) = shell.wait(2);
    assert_eq!(whats, [NIM_ADD.0, NIM_SETVERSION.0]);
    assert_eq!(calls[0].tip, "later");
}

#[test]
fn an_icon_the_taskbar_still_holds_is_changed_when_it_restarts() {
    let (shell, tray, class) = rig("held");
    let _icon = tray.show(&image(), "t", Box::new(|_| {})).unwrap();
    shell.wait(2);
    shell.refuse(&[NIM_ADD]);
    send(find(&class).unwrap(), restart_msg(), 0, 0);
    assert_eq!(shell.wait(3).0, [NIM_ADD.0, NIM_MODIFY.0, NIM_SETVERSION.0]);
}

#[test]
fn a_change_the_taskbar_refuses_is_refused_and_shown_when_it_restarts() {
    let (shell, tray, class) = rig("refused");
    let icon = tray.show(&image(), "t", Box::new(|_| {})).unwrap();
    shell.wait(2);
    shell.refuse(&[NIM_MODIFY]);
    assert_eq!(icon.set(&image(), "u"), Err(failed()));
    shell.refuse(&[]);
    send(find(&class).unwrap(), restart_msg(), 0, 0);
    assert_eq!(shell.wait(3).1[1].tip, "u", "after the refused modify");
}

#[test]
fn a_taskbar_call_that_runs_other_messages_holds_no_lock() {
    let (shell, tray, class) = rig("reenter");
    let _icon = tray.show(&image(), "t", Box::new(|_| {})).unwrap();
    shell.wait(2);
    let window = find(&class).unwrap();
    *shell.restart_in.lock().unwrap() = Some(window.0.addr());
    send(window, restart_msg(), 0, 0);
    let (whats, _) = shell.wait(4);
    assert_eq!(
        whats,
        [NIM_ADD.0, NIM_SETVERSION.0, NIM_ADD.0, NIM_SETVERSION.0],
        "added twice, no deadlock"
    );
}

#[test]
fn a_change_made_while_the_add_is_on_its_way_follows_it() {
    let shell = Arc::new(FakeShell::default());
    let state = Arc::new(Mutex::new(State {
        icon: Arc::new(Icon::new(&image()).unwrap()),
        tip: crate::tray::tip("old"),
        window: 1,
        added: false,
        gone: false,
        changes: 0,
    }));
    *shell.change_in.lock().unwrap() = Some(Arc::clone(&state));
    let here = Here {
        on: Box::new(|_| {}),
        state,
        shell: Arc::clone(&shell) as Arc<dyn Shell>,
        restart: 0,
    };
    here.add();
    let (whats, calls) = shell.take();
    assert_eq!(whats, [NIM_ADD.0, NIM_SETVERSION.0, NIM_MODIFY.0]);
    assert_eq!(
        (calls[0].tip.as_str(), calls[2].tip.as_str()),
        ("old", "newer")
    );
}

#[test]
fn a_listener_that_panics_still_hears_the_next_click() {
    let (_shell, tray, class) = rig("panic");
    let (tx, rx) = mpsc::channel();
    let on: OnTray = Box::new(move |e| {
        let TrayEvent::Click(at) = e else { return };
        assert!(at.x != 1, "a listener bug");
        let _ = tx.send(at.x);
    });
    let _icon = tray.show(&image(), "t", on).unwrap();
    let window = find(&class).unwrap();
    send(window, CALLBACK_MSG, 1, SELECT);
    send(window, CALLBACK_MSG, 2, SELECT);
    assert_eq!(rx.try_iter().collect::<Vec<_>>(), [2]);
}

#[test]
#[allow(unsafe_code, reason = "Finds the taskbar's window to read its class.")]
fn only_a_window_outside_the_taskbar_is_away() {
    let (_shell, tray, class) = rig("away");
    let _icon = tray.show(&image(), "t", Box::new(|_| {})).unwrap();
    assert_eq!(
        away(find(&class).unwrap()),
        Some(TrayEvent::Away),
        "any other window"
    );
    assert_eq!(away(HWND::default()), None, "no window");
    // SAFETY: a constant null-terminated class name; the taskbar is only read.
    if let Ok(taskbar) = unsafe { FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) } {
        assert_eq!(away(taskbar), None);
    }
}
