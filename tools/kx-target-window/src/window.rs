//! Opens the log, the main window and its logging edit box, then runs the message loop.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use kx_target_window::config::{self, ConfigError, TargetConfig};
use thiserror::Error;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, DEFAULT_CHARSET, FW_NORMAL,
    OUT_DEFAULT_PRECIS,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::SetWindowSubclass;
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DispatchMessageW, ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_MULTILINE,
    ES_PASSWORD, ES_WANTRETURN, GetMessageW, IDC_IBEAM, LoadCursorW, MSG, RegisterClassW,
    SendMessageW, TranslateMessage, WINDOW_EX_STYLE, WINDOW_STYLE, WM_SETFONT, WNDCLASSW, WS_CHILD,
    WS_EX_CLIENTEDGE, WS_OVERLAPPEDWINDOW, WS_VISIBLE, WS_VSCROLL,
};
use windows::core::{HSTRING, PCWSTR, w};

use crate::procs;

/// Window class of the main window; the gate runner finds the window by its title.
const CLASS: PCWSTR = w!("KxsTargetWindow");
/// Window class of the system edit box.
const EDIT_CLASS: PCWSTR = w!("EDIT");
/// Id of our subclass on the edit box.
const SUBCLASS_ID: usize = 1;
/// `WM_SETFONT` flag: redraw with the new font now.
const REDRAW: LPARAM = LPARAM(1);

/// Why the window could not start.
#[derive(Debug, Error)]
pub enum StartError {
    /// The settings could not be loaded.
    #[error(transparent)]
    Config(#[from] ConfigError),
    /// The log could not be opened.
    #[error("{}: {source}", .file.display())]
    Log {
        /// The log file.
        file: PathBuf,
        /// The reason.
        source: std::io::Error,
    },
    /// A Windows call failed.
    #[error(transparent)]
    Windows(#[from] windows::core::Error),
    /// The edit box could not be subclassed.
    #[error("SetWindowSubclass failed: characters would not be logged")]
    Subclass,
}

/// Loads the settings and shows the window until it is closed.
pub fn run() -> Result<(), StartError> {
    let cfg = config::load()?;
    procs::start_log(open_log(&cfg.log)?);
    let password = std::env::args().any(|a| a == cfg.password_arg);
    procs::set_password(password);
    let inst = module()?;
    let main = create_main(&cfg, inst)?;
    let edit = create_edit(main, inst, password)?;
    set_font(edit, &cfg);
    procs::focus(edit);
    message_loop();
    Ok(())
}

/// Opens `file` for appending, making its folder first.
fn open_log(file: &Path) -> Result<File, StartError> {
    let fail = |source| StartError::Log {
        file: file.to_path_buf(),
        source,
    };
    if let Some(dir) = file.parent() {
        std::fs::create_dir_all(dir).map_err(fail)?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)
        .map_err(fail)
}

/// This exe's module handle.
#[allow(unsafe_code, reason = "GetModuleHandleW for this exe.")]
fn module() -> Result<HINSTANCE, StartError> {
    // SAFETY: plain call that asks for the module handle of the running exe.
    let module = unsafe { GetModuleHandleW(None) }?;
    Ok(module.into())
}

/// Registers the class and opens the main window.
#[allow(
    unsafe_code,
    reason = "Win32 class registration and window creation on the UI thread."
)]
fn create_main(cfg: &TargetConfig, inst: HINSTANCE) -> Result<HWND, StartError> {
    // SAFETY: called on the UI thread; the stock cursor is shared and needs no release.
    let cursor = unsafe { LoadCursorW(None, IDC_IBEAM) }?;
    let class = WNDCLASSW {
        lpfnWndProc: Some(procs::main_proc),
        hInstance: inst,
        lpszClassName: CLASS,
        hCursor: cursor,
        ..Default::default()
    };
    // SAFETY: `class` names our own window procedure and outlives the call.
    unsafe { RegisterClassW(&raw const class) };
    let title = HSTRING::from(cfg.title.as_str());
    let style = WS_OVERLAPPEDWINDOW | WS_VISIBLE;
    // SAFETY: called on the UI thread with the class registered just above.
    let main = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE(0),
            CLASS,
            &title,
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            cfg.width_px,
            cfg.height_px,
            None,
            None,
            Some(inst),
            None,
        )
    }?;
    Ok(main)
}

/// The edit box style: multi-line, or a single-line password box (Windows ignores `ES_PASSWORD` on multi-line boxes).
fn edit_style(password: bool) -> WINDOW_STYLE {
    if password {
        return WS_CHILD
            | WS_VISIBLE
            | WINDOW_STYLE((ES_PASSWORD | ES_AUTOHSCROLL).cast_unsigned());
    }
    let own = ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN;
    WS_CHILD | WS_VISIBLE | WS_VSCROLL | WINDOW_STYLE(own.cast_unsigned())
}

/// Creates the logging edit box inside `main`, sized to its client area.
#[allow(
    unsafe_code,
    reason = "Win32 window creation and subclassing on the UI thread."
)]
fn create_edit(main: HWND, inst: HINSTANCE, password: bool) -> Result<HWND, StartError> {
    let style = edit_style(password);
    // SAFETY: called on the UI thread; `main` is our live window.
    let edit = unsafe {
        CreateWindowExW(
            WS_EX_CLIENTEDGE,
            EDIT_CLASS,
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
    }?;
    procs::set_edit(edit);
    // SAFETY: `edit` is our live window and `edit_proc` has the callback's signature.
    if !unsafe { SetWindowSubclass(edit, Some(procs::edit_proc), SUBCLASS_ID, 0) }.as_bool() {
        return Err(StartError::Subclass);
    }
    procs::fit_edit(main);
    Ok(edit)
}

/// Gives `edit` the configured font.
#[allow(
    unsafe_code,
    reason = "GDI font creation and a message to our own edit box."
)]
fn set_font(edit: HWND, cfg: &TargetConfig) {
    let face = HSTRING::from(cfg.font.as_str());
    let weight = FW_NORMAL.0.cast_signed();
    // SAFETY: plain GDI call; the font stays alive with the process.
    let font = unsafe {
        CreateFontW(
            cfg.font_px,
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
        )
    };
    let font_handle = Some(WPARAM(font.0 as usize));
    // SAFETY: plain message send to our own edit box.
    unsafe { SendMessageW(edit, WM_SETFONT, font_handle, Some(REDRAW)) };
}

/// Runs the thread's message loop until the window closes.
#[allow(
    unsafe_code,
    reason = "The Win32 message loop of the thread that made the windows."
)]
fn message_loop() {
    let mut msg = MSG::default();
    // SAFETY: standard message loop on the thread that created the windows.
    unsafe {
        while GetMessageW(&raw mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&raw const msg);
            DispatchMessageW(&raw const msg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_password_box_is_single_line_with_the_password_style() {
        let has = |s: WINDOW_STYLE, bits: i32| s.0 & bits.cast_unsigned() != 0;
        assert!(has(edit_style(true), ES_PASSWORD));
        assert!(!has(edit_style(true), ES_MULTILINE));
        assert!(has(edit_style(false), ES_MULTILINE));
        assert!(!has(edit_style(false), ES_PASSWORD));
    }
}
