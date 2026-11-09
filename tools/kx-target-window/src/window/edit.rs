//! The logging text box: a system edit box, a password box, or a rich-edit box, which serves UI Automation's text
//! itself; its font and its fill.

use kx_target_window::config::TargetConfig;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, WPARAM};
use windows::Win32::Graphics::Gdi::{
    CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, DEFAULT_CHARSET, FW_NORMAL,
    OUT_DEFAULT_PRECIS,
};
use windows::Win32::System::LibraryLoader::{LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW};
use windows::Win32::UI::Controls::RichEdit::MSFTEDIT_CLASS;
use windows::Win32::UI::Shell::SetWindowSubclass;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, ES_AUTOHSCROLL, ES_AUTOVSCROLL, ES_MULTILINE, ES_PASSWORD, ES_WANTRETURN,
    SendMessageW, SetWindowTextW, WINDOW_STYLE, WM_SETFONT, WS_CHILD, WS_EX_CLIENTEDGE, WS_HSCROLL,
    WS_VISIBLE, WS_VSCROLL,
};
use windows::core::{HSTRING, PCWSTR, w};

use super::StartError;
use crate::procs;

/// Window class of the system edit box.
const EDIT_CLASS: PCWSTR = w!("EDIT");
/// The system library that holds the rich-edit class, loaded only from System32.
const RICH_LIBRARY: PCWSTR = w!("msftedit.dll");
/// Id of our subclass on the edit box.
const SUBCLASS_ID: usize = 1;
/// `WM_SETFONT` flag: redraw with the new font now.
const REDRAW: LPARAM = LPARAM(1);

/// The box's window class: the rich-edit class when `rich`, else the system edit box's.
pub fn class(rich: bool) -> PCWSTR {
    if rich { MSFTEDIT_CLASS } else { EDIT_CLASS }
}

/// The edit box style: multi-line, unwrapped when `fill`ed, or a single-line password box (Windows ignores
/// `ES_PASSWORD` on multi-line boxes).
pub fn style(password: bool, fill: bool) -> WINDOW_STYLE {
    if password {
        return WS_CHILD
            | WS_VISIBLE
            | WINDOW_STYLE((ES_PASSWORD | ES_AUTOHSCROLL).cast_unsigned());
    }
    let own = ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN;
    let style = WS_CHILD | WS_VISIBLE | WS_VSCROLL | WINDOW_STYLE(own.cast_unsigned());
    if fill {
        style | WS_HSCROLL | WINDOW_STYLE(ES_AUTOHSCROLL.cast_unsigned())
    } else {
        style
    }
}

/// Loads the rich-edit class from System32's library, which stays loaded with the process.
#[allow(
    unsafe_code,
    reason = "Loads a system library by name from System32 only."
)]
fn load_rich() -> Result<(), StartError> {
    // SAFETY: a plain call; the search is limited to System32, so no other folder's copy loads.
    unsafe { LoadLibraryExW(RICH_LIBRARY, None, LOAD_LIBRARY_SEARCH_SYSTEM32) }?;
    Ok(())
}

/// Creates the logging edit box inside `main`, sized to its client area: a password box, filled, or rich.
#[allow(
    unsafe_code,
    reason = "Win32 window creation and subclassing on the UI thread."
)]
pub fn create(
    main: HWND,
    inst: HINSTANCE,
    (password, fill, rich): (bool, bool, bool),
) -> Result<HWND, StartError> {
    if rich {
        load_rich()?;
    }
    // SAFETY: called on the UI thread; `main` is our live window and the class is registered.
    let edit = unsafe {
        CreateWindowExW(
            WS_EX_CLIENTEDGE,
            class(rich),
            w!(""),
            style(password, fill),
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

/// Fills `edit` with the fill switch's numbered lines; they are never logged.
#[allow(unsafe_code, reason = "Sets the text of our own edit box.")]
pub fn fill(edit: HWND, cfg: &TargetConfig) -> Result<(), StartError> {
    let text = HSTRING::from(kx_target_window::fill::text(
        cfg.fill_lines as usize,
        cfg.fill_cols as usize,
    ));
    // SAFETY: plain call on our own edit box; the text outlives it.
    unsafe { SetWindowTextW(edit, &text) }?;
    Ok(())
}

/// Gives `edit` the configured font.
#[allow(
    unsafe_code,
    reason = "GDI font creation and a message to our own edit box."
)]
pub fn set_font(edit: HWND, cfg: &TargetConfig) {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// True when style `s` holds the edit style `bits`.
    fn has(s: WINDOW_STYLE, bits: i32) -> bool {
        s.0 & bits.cast_unsigned() != 0
    }

    #[test]
    fn the_password_box_is_single_line_with_the_password_style() {
        assert!(has(style(true, false), ES_PASSWORD));
        assert!(!has(style(true, false), ES_MULTILINE));
        assert!(has(style(false, false), ES_MULTILINE));
        assert!(!has(style(false, false), ES_PASSWORD));
    }

    #[test]
    fn a_filled_box_keeps_its_long_lines_and_scrolls_both_ways() {
        let filled = style(false, true);
        assert!(has(filled, ES_MULTILINE) && has(filled, ES_AUTOHSCROLL));
        assert!(filled.contains(WS_HSCROLL | WS_VSCROLL));
        let plain = style(false, false);
        assert!(
            !has(plain, ES_AUTOHSCROLL) && !plain.contains(WS_HSCROLL),
            "a plain box wraps"
        );
        assert!(
            !has(style(true, true), ES_MULTILINE),
            "a password box takes no fill"
        );
    }

    /// The text of class name `p`.
    #[allow(unsafe_code, reason = "Reads a static class name.")]
    fn text(p: PCWSTR) -> String {
        // SAFETY: every class name here is a static string that ends in a NUL.
        unsafe { p.to_string() }.unwrap_or_default()
    }

    #[test]
    fn only_the_rich_switch_makes_a_rich_edit_box() {
        assert_eq!(text(class(true)), "RICHEDIT50W");
        assert_eq!(text(class(false)), "EDIT");
    }
}
