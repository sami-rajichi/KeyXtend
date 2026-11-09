//! UI Automation reads that tell a failed read from a part that is not there, so a failed read never passes as gone.
//! An element destroyed as it is read is gone, as the panel and the banner redraw their pages.
#![cfg(windows)]

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    IUIAutomationCondition, IUIAutomationElement, TreeScope, TreeScope_Descendants,
    TreeScope_Subtree, UIA_AutomationIdPropertyId, UIA_CONTROLTYPE_ID, UIA_ControlTypePropertyId,
    UIA_E_ELEMENTNOTAVAILABLE,
};
use windows::core::BSTR;

use crate::keymap::{KeyRead, Reader, read_raw};
use crate::{product, win};

/// True while window `w` is one of the product's shown windows.
pub fn shown(w: HWND) -> bool {
    product::shown().contains(&w)
}

/// `read` of a window, none when it failed because the window went meanwhile; a failure while `still()` holds stays
/// an error.
pub fn kept<T>(read: Result<T, String>, still: impl FnOnce() -> bool) -> Result<Option<T>, String> {
    match read {
        Ok(v) => Ok(Some(v)),
        Err(_) if !still() => Ok(None),
        Err(e) => Err(e),
    }
}

/// Calls `f` every `poll_ms` until it gives a value or fails, for up to `wait_ms`: none when the wait ran out.
pub fn poll_read<T>(
    wait_ms: u64,
    poll_ms: u64,
    mut f: impl FnMut() -> Result<Option<T>, String>,
) -> Result<Option<T>, String> {
    win::poll_until(wait_ms, poll_ms, || f().transpose()).transpose()
}

/// UI Automation's error as text.
#[allow(clippy::needless_pass_by_value, reason = "It fits `map_err` as it is.")]
fn text(e: windows::core::Error) -> String {
    format!("UI Automation: {e}")
}

/// `read`'s value; none when nothing came back, or its element was destroyed as it was read, which is gone and no
/// failed read. Any other failure is an error.
fn present<T>(read: windows::core::Result<T>) -> Result<Option<T>, String> {
    match read {
        Ok(v) => Ok(Some(v)),
        Err(e) if e.code().is_ok() || e.code().0.cast_unsigned() == UIA_E_ELEMENTNOTAVAILABLE => {
            Ok(None)
        }
        Err(e) => Err(text(e)),
    }
}

/// The values of `reads` that came, in order; a failure ends it with its error.
fn gather<T>(reads: impl IntoIterator<Item = windows::core::Result<T>>) -> Result<Vec<T>, String> {
    let mut values = Vec::new();
    for read in reads {
        values.extend(present(read)?);
    }
    Ok(values)
}

/// What `read` gives for each element `cond` finds in `scope` of window `hwnd`: none when no element matches, and an
/// error when a read failed.
#[allow(
    unsafe_code,
    reason = "UI Automation calls on objects this client made."
)]
pub(crate) fn each<T>(
    reader: &Reader,
    hwnd: HWND,
    scope: TreeScope,
    cond: &IUIAutomationCondition,
    read: impl Fn(&IUIAutomationElement) -> windows::core::Result<T>,
) -> Result<Vec<T>, String> {
    // SAFETY: plain calls on live UI Automation objects; a dead window gives an error.
    unsafe {
        let Some(root) = present(reader.auto().ElementFromHandle(hwnd))? else {
            return Ok(Vec::new());
        };
        let Some(found) = present(root.FindAll(scope, cond))? else {
            return Ok(Vec::new());
        };
        let count = present(found.Length())?.unwrap_or(0);
        gather((0..count).map(|i| found.GetElement(i).and_then(|el| read(&el))))
    }
}

/// The screen boxes of every element with UI Automation id `id` in window `hwnd`.
#[allow(
    unsafe_code,
    reason = "UI Automation calls on objects this client made."
)]
pub fn boxes(reader: &Reader, hwnd: HWND, id: &str) -> Result<Vec<RECT>, String> {
    let want = VARIANT::from(BSTR::from(id));
    // SAFETY: a plain call on this client, with a live value.
    let cond = unsafe {
        reader
            .auto()
            .CreatePropertyCondition(UIA_AutomationIdPropertyId, &want)
    }
    .map_err(text)?;
    // SAFETY: a plain property read on a live element.
    let rect = |el: &IUIAutomationElement| unsafe { el.CurrentBoundingRectangle() };
    each(reader, hwnd, TreeScope_Descendants, &cond, rect)
}

/// The names of every element in window `hwnd`, as a screen reader meets them.
#[allow(
    unsafe_code,
    reason = "UI Automation calls on objects this client made."
)]
pub fn names(reader: &Reader, hwnd: HWND) -> Result<Vec<String>, String> {
    // SAFETY: a plain call on this client.
    let cond = unsafe { reader.auto().CreateTrueCondition() }.map_err(text)?;
    // SAFETY: a plain property read on a live element.
    let name = |el: &IUIAutomationElement| unsafe { el.CurrentName() }.map(|n| n.to_string());
    each(reader, hwnd, TreeScope_Subtree, &cond, name)
}

/// Every element of control type `kind` in window `hwnd`, as a screen reader meets it.
#[allow(
    unsafe_code,
    reason = "UI Automation calls on objects this client made."
)]
pub fn items(
    reader: &Reader,
    hwnd: HWND,
    kind: UIA_CONTROLTYPE_ID,
) -> Result<Vec<KeyRead>, String> {
    let want = VARIANT::from(kind.0);
    // SAFETY: a plain call on this client, with a live value.
    let cond = unsafe {
        reader
            .auto()
            .CreatePropertyCondition(UIA_ControlTypePropertyId, &want)
    }
    .map_err(text)?;
    each(reader, hwnd, TreeScope_Descendants, &cond, read_raw)
}

#[cfg(test)]
mod tests {
    use windows::Win32::Foundation::E_FAIL;
    use windows::core::{Error, HRESULT};

    use super::*;

    /// The error UI Automation gives for an element destroyed as it is read.
    fn vanished() -> Error {
        Error::from_hresult(HRESULT(UIA_E_ELEMENTNOTAVAILABLE.cast_signed()))
    }

    #[test]
    fn an_element_destroyed_mid_read_is_gone_and_a_real_failure_stays_an_error() {
        assert_eq!(present(Ok(3)), Ok(Some(3)));
        assert_eq!(present::<u8>(Err(vanished())), Ok(None));
        assert_eq!(present::<u8>(Err(Error::empty())), Ok(None), "no element");
        let failed = present::<u8>(Err(Error::from_hresult(E_FAIL)));
        assert!(
            failed
                .expect_err("a real failure")
                .starts_with("UI Automation")
        );
    }

    #[test]
    fn a_list_leaves_out_what_vanished_and_ends_on_a_real_failure() {
        let reads = [Ok(1), Err(vanished()), Ok(2)];
        assert_eq!(gather(reads), Ok(vec![1, 2]));
        let reads = [Ok(1), Err(Error::from_hresult(E_FAIL)), Ok(2)];
        assert!(gather(reads).is_err());
    }

    #[test]
    fn a_failed_read_is_an_error_while_its_window_shows_and_nothing_once_it_went() {
        assert_eq!(kept(Ok(3), || true), Ok(Some(3)));
        assert_eq!(kept(Ok(3), || false), Ok(Some(3)));
        let failed = || Err::<u8, _>("UI Automation: failed".to_string());
        assert_eq!(kept(failed(), || true), Err("UI Automation: failed".into()));
        assert_eq!(kept(failed(), || false), Ok(None));
    }

    #[test]
    fn a_poll_stops_on_a_value_or_a_failure_and_gives_none_when_the_wait_ran_out() {
        let mut tries = 0;
        let late = poll_read(0, 1, || {
            tries += 1;
            Ok::<Option<u8>, String>(None)
        });
        assert_eq!((late, tries), (Ok(None), 1));
        assert_eq!(poll_read(0, 1, || Ok(Some(7))), Ok(Some(7)));
        let failed = poll_read(0, 1, || Err::<Option<u8>, _>("no read".to_string()));
        assert_eq!(failed, Err("no read".to_string()));
    }
}
