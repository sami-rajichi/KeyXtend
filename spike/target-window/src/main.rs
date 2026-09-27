//! P1 spike target: a text box that logs each character it receives with a microsecond timestamp.
//!
//! Log lines are `<QPC microseconds>\t<UTF-16 unit in hex>`, so the harness can time clicks.
#![windows_subsystem = "windows"]

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Mutex, OnceLock};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, DEFAULT_CHARSET, FW_NORMAL,
    OUT_DEFAULT_PRECIS,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};
use windows::Win32::UI::Input::KeyboardAndMouse::SetFocus;
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DispatchMessageW, ES_AUTOVSCROLL,
    ES_MULTILINE, ES_WANTRETURN, GetMessageW, IDC_IBEAM, LoadCursorW, MSG, MoveWindow,
    PostQuitMessage, RegisterClassW, SendMessageW, TranslateMessage, WINDOW_EX_STYLE,
    WINDOW_STYLE, WM_CHAR, WM_DESTROY, WM_SETFOCUS, WM_SETFONT, WM_SIZE, WNDCLASSW, WS_CHILD,
    WS_EX_CLIENTEDGE, WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_VSCROLL,
};
use windows::core::{HSTRING, w};

/// The log file, shared with the edit-box subclass.
static LOG: OnceLock<Mutex<File>> = OnceLock::new();
/// QPC ticks per second.
static FREQ: OnceLock<i64> = OnceLock::new();
/// The edit box, as a raw handle value.
static EDIT: AtomicIsize = AtomicIsize::new(0);
/// Microseconds per second.
const US: i64 = 1_000_000;

fn now_us() -> i64 {
    let mut count = 0;
    // SAFETY: plain query into a local.
    let _ = unsafe { QueryPerformanceCounter(&mut count) };
    let freq = *FREQ.get().unwrap_or(&1);
    (i128::from(count) * i128::from(US) / i128::from(freq)) as i64
}

fn log_line(line: &str) {
    if let Some(file) = LOG.get() {
        if let Ok(mut f) = file.lock() {
            let _ = writeln!(f, "{line}");
            let _ = f.flush();
        }
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
    if msg == WM_CHAR {
        log_line(&format!("{}\t{:04X}", now_us(), wparam.0 as u16));
    }
    // SAFETY: forwards the same message to the edit box.
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

unsafe extern "system" fn main_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let edit = HWND(EDIT.load(Ordering::Relaxed) as *mut _);
    // SAFETY: standard window-procedure calls on our own windows.
    unsafe {
        match msg {
            WM_SIZE => {
                let (w, h) = ((lparam.0 & 0xFFFF) as i32, ((lparam.0 >> 16) & 0xFFFF) as i32);
                let _ = MoveWindow(edit, 0, 0, w, h, true);
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
    let file = OpenOptions::new().create(true).append(true).open(&log).map_err(|e| e.to_string())?;
    let _ = LOG.set(Mutex::new(file));
    let mut freq = 0;
    // SAFETY: plain query into a local.
    let _ = unsafe { QueryPerformanceFrequency(&mut freq) };
    let _ = FREQ.set(freq.max(1));
    log_line(&format!("# start\t{}", now_us()));
    // SAFETY: standard Win32 window creation and message loop on this thread.
    unsafe { run(&cfg.target) }
}

unsafe fn run(t: &spike_core::config::TargetConfig) -> Result<(), String> {
    // SAFETY: the caller runs this on the UI thread; every handle comes from these calls.
    unsafe {
        let inst = GetModuleHandleW(None).map_err(|e| e.to_string())?;
        let class = w!("KxsTargetWindow");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(main_proc),
            hInstance: inst.into(),
            lpszClassName: class,
            hCursor: LoadCursorW(None, IDC_IBEAM).map_err(|e| e.to_string())?,
            ..Default::default()
        };
        RegisterClassW(&wc);
        let title = HSTRING::from(t.title.as_str());
        let main = CreateWindowExW(
            WINDOW_EX_STYLE(0), class, &title, WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT, CW_USEDEFAULT, t.width_px, t.height_px, None, None, Some(inst.into()), None,
        )
        .map_err(|e| e.to_string())?;
        let style = WS_CHILD | WS_VISIBLE | WS_VSCROLL
            | WINDOW_STYLE((ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN) as u32);
        let edit = CreateWindowExW(
            WS_EX_CLIENTEDGE, w!("EDIT"), w!(""), style, 0, 0, t.width_px, t.height_px,
            Some(main), None, Some(inst.into()), None,
        )
        .map_err(|e| e.to_string())?;
        EDIT.store(edit.0 as isize, Ordering::Relaxed);
        let _ = SetWindowSubclass(edit, Some(edit_proc), 1, 0);
        let font = CreateFontW(
            t.font_px, 0, 0, 0, FW_NORMAL.0 as i32, 0, 0, 0, DEFAULT_CHARSET, OUT_DEFAULT_PRECIS,
            CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY, 0, &HSTRING::from(t.font.as_str()),
        );
        SendMessageW(edit, WM_SETFONT, Some(WPARAM(font.0 as usize)), Some(LPARAM(1)));
        let _ = SetFocus(Some(edit));
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}
