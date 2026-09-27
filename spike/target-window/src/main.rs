//! P1 spike target: a text box that logs each character it receives with a microsecond timestamp.
//!
//! The log format is in `spike_core::targetlog`; focus losses are logged too, for G2.
#![windows_subsystem = "windows"]

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Mutex, OnceLock};

use spike_core::clock::now_us;
use spike_core::config::TargetConfig;
use spike_core::targetlog;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, DEFAULT_CHARSET, FW_NORMAL,
    OUT_DEFAULT_PRECIS,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW, ES_AUTOVSCROLL, ES_MULTILINE,
    ES_WANTRETURN, GetClientRect, GetMessageW, IDC_IBEAM, LoadCursorW, MSG, MoveWindow,
    PostQuitMessage, RegisterClassW, SendMessageW, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE,
    WM_CHAR, WM_DESTROY, WM_KILLFOCUS, WM_SETFOCUS, WM_SETFONT, WM_SIZE, WNDCLASSW, WS_CHILD,
    WS_EX_CLIENTEDGE, WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_VSCROLL,
};
use windows::core::{HSTRING, PCWSTR, w};

/// The log file, shared with the edit-box subclass.
static LOG: OnceLock<Mutex<File>> = OnceLock::new();
/// The edit box, as a raw handle value.
static EDIT: AtomicIsize = AtomicIsize::new(0);
/// Window class of the main window; the harness finds the window by its title.
const CLASS: PCWSTR = w!("KxsTargetWindow");
/// Id of our subclass on the edit box.
const SUBCLASS_ID: usize = 1;
/// `WM_SETFONT` flag: redraw with the new font now.
const REDRAW: LPARAM = LPARAM(1);

fn log_line(line: &str) {
    if let Some(file) = LOG.get()
        && let Ok(mut f) = file.lock()
    {
        let _ = writeln!(f, "{line}");
        let _ = f.flush();
    }
}

unsafe extern "system" fn edit_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    match msg {
        WM_CHAR => log_line(&targetlog::data(now_us(), wparam.0 as u16)),
        WM_KILLFOCUS => log_line(&targetlog::mark(targetlog::FOCUS_LOST, now_us())),
        _ => {}
    }
    // SAFETY: forwards the same message to the edit box.
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

/// Makes the edit box fill the client area of `main`.
fn fit_edit(main: HWND) {
    let edit = HWND(EDIT.load(Ordering::Relaxed) as *mut _);
    let mut rc = RECT::default();
    // SAFETY: plain calls on our own windows; a null edit handle only makes MoveWindow fail.
    unsafe {
        if GetClientRect(main, &mut rc).is_ok() {
            let _ = MoveWindow(edit, 0, 0, rc.right - rc.left, rc.bottom - rc.top, true);
        }
    }
}

unsafe extern "system" fn main_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let edit = HWND(EDIT.load(Ordering::Relaxed) as *mut _);
    // SAFETY: standard window-procedure calls on our own windows.
    unsafe {
        match msg {
            WM_SIZE => {
                fit_edit(hwnd);
                LRESULT(0)
            }
            WM_SETFOCUS => {
                let _ = SetFocus(Some(edit));
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn main() -> Result<(), String> {
    let cfg = spike_core::config::load()?;
    let log = cfg.resolve(&cfg.target.log);
    if let Some(dir) = log.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log)
        .map_err(|e| e.to_string())?;
    let _ = LOG.set(Mutex::new(file));
    log_line(&targetlog::mark(targetlog::START, now_us()));
    // SAFETY: standard Win32 window creation and message loop on this thread.
    unsafe { run(&cfg.target) }
}

/// Registers the class and opens the main window.
unsafe fn create_main(t: &TargetConfig, inst: HINSTANCE) -> Result<HWND, String> {
    // SAFETY: the caller runs this on the UI thread with our own module handle.
    unsafe {
        let wc = WNDCLASSW {
            lpfnWndProc: Some(main_proc),
            hInstance: inst,
            lpszClassName: CLASS,
            hCursor: LoadCursorW(None, IDC_IBEAM).map_err(|e| e.to_string())?,
            ..Default::default()
        };
        RegisterClassW(&wc);
        let (x, y) = (CW_USEDEFAULT, CW_USEDEFAULT);
        let style = WS_OVERLAPPEDWINDOW | WS_VISIBLE;
        let title = HSTRING::from(t.title.as_str());
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            CLASS,
            &title,
            style,
            x,
            y,
            t.width_px,
            t.height_px,
            None,
            None,
            Some(inst),
            None,
        )
        .map_err(|e| e.to_string())
    }
}

/// Creates the logging edit box inside `main`, sized to its client area.
unsafe fn create_edit(main: HWND, inst: HINSTANCE) -> Result<HWND, String> {
    let style = WS_CHILD
        | WS_VISIBLE
        | WS_VSCROLL
        | WINDOW_STYLE((ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN) as u32);
    // SAFETY: the caller runs this on the UI thread; `main` is our live window.
    unsafe {
        let edit = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            w!("EDIT"),
            w!(""),
            style,
            0,
            0,
            0,
            0,
            Some(main),
            None,
            Some(inst),
            None,
        )
        .map_err(|e| e.to_string())?;
        EDIT.store(edit.0 as isize, Ordering::Relaxed);
        if !SetWindowSubclass(edit, Some(edit_proc), SUBCLASS_ID, 0).as_bool() {
            return Err("SetWindowSubclass failed: characters would not be logged".to_string());
        }
        fit_edit(main);
        Ok(edit)
    }
}

/// Gives `edit` the configured font.
unsafe fn set_font(edit: HWND, t: &TargetConfig) {
    let face = HSTRING::from(t.font.as_str());
    let weight = FW_NORMAL.0 as i32;
    // SAFETY: plain GDI and message calls on our own edit box.
    unsafe {
        let font = CreateFontW(
            t.font_px,
            0,
            0,
            0,
            weight,
            0,
            0,
            0,
            DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS,
            CLEARTYPE_QUALITY,
            0,
            &face,
        );
        SendMessageW(
            edit,
            WM_SETFONT,
            Some(WPARAM(font.0 as usize)),
            Some(REDRAW),
        );
    }
}

unsafe fn run(t: &TargetConfig) -> Result<(), String> {
    // SAFETY: the caller runs this on the UI thread; every handle comes from these calls.
    unsafe {
        let inst: HINSTANCE = GetModuleHandleW(None).map_err(|e| e.to_string())?.into();
        let main = create_main(t, inst)?;
        let edit = create_edit(main, inst)?;
        set_font(edit, t);
        let _ = SetFocus(Some(edit));
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}
