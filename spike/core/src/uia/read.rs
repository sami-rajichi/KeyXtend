//! Reads text facts from UI Automation: line boxes, the selection, and the password flag.

use std::ffi::c_void;

use windows::Win32::Foundation::RECT;
use windows::Win32::System::Com::SAFEARRAY;
use windows::Win32::System::Ole::{
    SafeArrayAccessData, SafeArrayDestroy, SafeArrayGetDim, SafeArrayGetElemsize,
    SafeArrayGetLBound, SafeArrayGetUBound, SafeArrayUnaccessData,
};
use windows::Win32::UI::Accessibility::{
    IUIAutomationElement, IUIAutomationTextPattern, IUIAutomationTextRange, UIA_TextPatternId,
};
use windows::Win32::UI::WindowsAndMessaging::{
    ES_PASSWORD, GWL_STYLE, GetWindowLongPtrW, RealGetWindowClassW,
};

use super::geom::rects_of;

/// The boxes of the visible text lines of `el`, top first.
pub fn lines(el: &IUIAutomationElement) -> Result<Vec<RECT>, String> {
    // SAFETY: plain pattern calls.
    let range = unsafe {
        let text: IUIAutomationTextPattern = el
            .GetCurrentPatternAs(UIA_TextPatternId)
            .map_err(|e| format!("UIA text: {e}"))?;
        text.DocumentRange()
            .map_err(|e| format!("UIA range: {e}"))?
    };
    boxes(&range)
}

/// The screen boxes of `range`, one per line part.
pub fn boxes(range: &IUIAutomationTextRange) -> Result<Vec<RECT>, String> {
    // SAFETY: a plain call; the array we get is ours to read and free.
    unsafe {
        let sa = range
            .GetBoundingRectangles()
            .map_err(|e| format!("UIA boxes: {e}"))?;
        Ok(rects_of(&doubles(sa)?))
    }
}

/// Base window classes whose `ES_PASSWORD` style bit means a password box (adapter table).
const EDIT_CLASSES: [&str; 5] = [
    "Edit",
    "RichEdit20A",
    "RichEdit20W",
    "RichEdit50W",
    "RichEdit60W",
];

/// Longest Win32 window class name, in UTF-16 units.
const CLASS_MAX: usize = 256;

/// The selected text of a text control and its screen boxes. Sensitive: never log the text.
pub struct Sel {
    /// The selected text, capped by the caller.
    pub text: String,
    /// One box per selected line part; empty when the selection is off screen.
    pub boxes: Vec<RECT>,
}

/// Adds at most `left` characters of `part` to `out`, and takes them off `left`.
fn spend(out: &mut String, part: &str, left: &mut usize) {
    let taken: String = part.chars().take(*left).collect();
    *left -= taken.chars().count();
    out.push_str(&taken);
}

/// The selection of `tp`, all ranges together capped at `max_chars`; `None` for a bare caret.
pub fn selection(tp: &IUIAutomationTextPattern, max_chars: usize) -> Result<Option<Sel>, String> {
    let (mut sel, mut left) = (
        Sel {
            text: String::new(),
            boxes: Vec::new(),
        },
        max_chars,
    );
    let err = |e: windows::core::Error| format!("UIA selection: {e}");
    // SAFETY: plain pattern calls; indexes stay below the array length.
    unsafe {
        let ranges = tp.GetSelection().map_err(err)?;
        for i in 0..ranges.Length().map_err(err)? {
            if left == 0 {
                break;
            }
            let range = ranges.GetElement(i).map_err(err)?;
            // A negative length means "all" to UIA, so the budget never goes below 1 here.
            let ask = i32::try_from(left).unwrap_or(i32::MAX);
            spend(
                &mut sel.text,
                &range.GetText(ask).map_err(err)?.to_string(),
                &mut left,
            );
            // A range with no box still counts; the pill just has less to go on.
            sel.boxes.extend(boxes(&range).unwrap_or_default());
        }
    }
    Ok((!sel.text.is_empty()).then_some(sel))
}

/// True when `el` is a password box, from UIA or its edit window's style; an unreadable UIA answer counts as yes.
pub fn is_password(el: &IUIAutomationElement) -> bool {
    // SAFETY: plain property reads; a null or dead handle gives an empty class and style 0.
    unsafe {
        let uia = el.CurrentIsPassword().ok().map(|b| b.as_bool());
        let hwnd = el.CurrentNativeWindowHandle().unwrap_or_default();
        if hwnd.is_invalid() {
            return password_flag(uia, "", 0);
        }
        let mut buf = [0u16; CLASS_MAX];
        // The base class, so an Edit subclassed under another name still counts.
        let len = RealGetWindowClassW(hwnd, &mut buf) as usize;
        let class = String::from_utf16_lossy(&buf[..len.min(CLASS_MAX)]);
        // The style is the low 32 bits of the long; the cut is intended.
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        password_flag(uia, &class, style)
    }
}

/// UIA's password answer (`None` when it could not answer, which fails safe), else the `ES_PASSWORD` bit of an edit window.
pub fn password_flag(uia: Option<bool>, class: &str, style: u32) -> bool {
    let edit = EDIT_CLASSES.iter().any(|c| c.eq_ignore_ascii_case(class));
    uia != Some(false) || (edit && style & ES_PASSWORD as u32 != 0)
}

/// Reads and frees a one-dimensional SAFEARRAY of doubles.
///
/// # Safety
/// `sa` is null or a live array of VT_R8 that we own.
unsafe fn doubles(sa: *mut SAFEARRAY) -> Result<Vec<f64>, String> {
    if sa.is_null() {
        return Ok(Vec::new());
    }
    // SAFETY: the caller hands us a live array; with one dimension of 8-byte elements, its data holds
    // exactly `n` doubles. It is unlocked and freed on every path.
    unsafe {
        let read = || -> windows::core::Result<Vec<f64>> {
            let shaped =
                SafeArrayGetDim(sa) == 1 && SafeArrayGetElemsize(sa) as usize == size_of::<f64>();
            if !shaped {
                return Ok(Vec::new());
            }
            let (lo, hi) = (SafeArrayGetLBound(sa, 1)?, SafeArrayGetUBound(sa, 1)?);
            let n = hi
                .checked_sub(lo)
                .and_then(|d| d.checked_add(1))
                .and_then(|d| usize::try_from(d).ok())
                .unwrap_or(0);
            if n == 0 {
                return Ok(Vec::new());
            }
            let mut data: *mut c_void = std::ptr::null_mut();
            SafeArrayAccessData(sa, &mut data)?;
            let v = std::slice::from_raw_parts(data.cast::<f64>(), n).to_vec();
            SafeArrayUnaccessData(sa)?;
            Ok(v)
        };
        let got = read();
        let _ = SafeArrayDestroy(sa);
        got.map_err(|e| format!("UIA array: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Ole::SafeArrayCreateVector;
    use windows::Win32::System::Variant::{VT_I4, VT_R8};

    const PW: u32 = ES_PASSWORD as u32;

    #[test]
    fn one_budget_covers_every_selected_range() {
        let (mut out, mut left) = (String::new(), 4);
        for part in ["abc", "déf", "ghi"] {
            spend(&mut out, part, &mut left);
        }
        assert_eq!((out.as_str(), left), ("abcd", 0));
    }

    #[test]
    fn an_unreadable_uia_answer_counts_as_a_password() {
        assert!(password_flag(None, "", 0));
    }

    #[test]
    fn office_rich_edit_boxes_are_edit_boxes() {
        assert!(password_flag(Some(false), "RICHEDIT60W", PW));
    }

    #[test]
    fn doubles_reads_a_vector_of_doubles_and_skips_other_shapes() {
        // SAFETY: arrays made here and handed to `doubles`, which frees them; the fill stays in bounds.
        unsafe {
            let sa = SafeArrayCreateVector(VT_R8, 0, 3);
            let mut data = std::ptr::null_mut();
            SafeArrayAccessData(sa, &mut data).expect("lock");
            std::slice::from_raw_parts_mut(data.cast::<f64>(), 3)
                .copy_from_slice(&[1.5, 2.0, -3.0]);
            SafeArrayUnaccessData(sa).expect("unlock");
            assert_eq!(doubles(sa), Ok(vec![1.5, 2.0, -3.0]));
            assert_eq!(doubles(SafeArrayCreateVector(VT_I4, 0, 3)), Ok(vec![]));
            assert_eq!(doubles(std::ptr::null_mut()), Ok(vec![]));
        }
    }

    #[test]
    fn uia_saying_password_is_enough() {
        assert!(password_flag(Some(true), "", 0));
    }

    #[test]
    fn an_edit_box_with_the_password_style_counts_even_when_uia_says_no() {
        assert!(password_flag(Some(false), "Edit", PW));
        assert!(password_flag(Some(false), "edit", PW));
        assert!(password_flag(Some(false), "RichEdit20W", PW));
    }

    #[test]
    fn the_style_bit_means_nothing_outside_an_edit_box() {
        assert!(!password_flag(
            Some(false),
            "Chrome_RenderWidgetHostHWND",
            PW
        ));
    }

    #[test]
    fn neither_source_saying_so_is_not_a_password() {
        assert!(!password_flag(Some(false), "", 0));
        assert!(!password_flag(Some(false), "Edit", 0));
    }
}
