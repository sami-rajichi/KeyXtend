//! Opens the log, the main window and its logging edit box, then runs the message loop.

use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};

use kx_platform_windows::{PlatformError, QpcClock};
use kx_target_window::config::{self, ConfigError, TargetConfig};
use kx_target_window::layout::{self, LayoutError};
use thiserror::Error;
use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{DPI_AWARENESS_CONTEXT_UNAWARE, SetThreadDpiAwarenessContext};
use windows::Win32::UI::WindowsAndMessaging::{
    CW_USEDEFAULT, CreateWindowExW, DispatchMessageW, GetMessageW, IDC_IBEAM, LoadCursorW, MSG,
    RegisterClassW, TranslateMessage, WINDOW_EX_STYLE, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};
use windows::core::{HSTRING, PCWSTR, w};

use crate::{name, procs, switch};

mod edit;

/// Window class of the main window; the gate runner finds the window by its title.
const CLASS: PCWSTR = w!("KxsTargetWindow");

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
    /// The QPC clock could not start.
    #[error(transparent)]
    Clock(PlatformError),
    /// The edit box could not be subclassed.
    #[error("SetWindowSubclass failed: characters would not be logged")]
    Subclass,
    /// The text box could not be given its accessible name.
    #[error("the text box's accessible name: {0}")]
    Name(String),
    /// The `--layout` id is bad, not installed, or Windows did not switch to it.
    #[error(transparent)]
    Layout(#[from] LayoutError),
}

/// Loads the settings and shows the window until it is closed, in the `--layout` layout when one is given, filled,
/// rich or DPI-unaware when the switches ask.
pub fn run() -> Result<(), StartError> {
    let cfg = config::load()?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let wanted = layout::parse(&args, &cfg.layout_arg)?;
    wanted.map(switch::check).transpose()?;
    let clock = QpcClock::new().map_err(StartError::Clock)?;
    procs::start_log(open_log(&cfg.log)?, clock);
    let password = args.contains(&cfg.password_arg);
    procs::set_password(password);
    procs::set_hang_max(cfg.hang_max_ms);
    let filled = !password && args.contains(&cfg.fill_arg);
    let rich = !password && args.contains(&cfg.rich_arg);
    if args.contains(&cfg.unaware_arg) {
        unaware()?;
    }
    let inst = module()?;
    let main = create_main(&cfg, inst)?;
    let edit = edit::create(main, inst, (password, filled, rich))?;
    edit::set_font(edit, &cfg);
    if filled {
        edit::fill(edit, &cfg)?;
    }
    name::set(edit, &cfg.edit_name).map_err(StartError::Name)?;
    procs::focus(edit);
    if let Some(id) = wanted {
        switch::apply(id).inspect_err(procs::note_layout)?;
    }
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

/// Makes this thread's windows DPI-unaware, as an old app's are, so Windows scales them and their points.
#[allow(
    unsafe_code,
    reason = "Sets this thread's DPI awareness before it makes a window."
)]
fn unaware() -> Result<(), StartError> {
    // SAFETY: a plain call on this thread; it makes no window and keeps no pointer.
    let was = unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_UNAWARE) };
    if was.is_invalid() {
        return Err(StartError::Windows(windows::core::Error::from_thread()));
    }
    Ok(())
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
mod tests;
