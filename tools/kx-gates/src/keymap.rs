//! Key boxes by UI Automation id: one read of a window's buttons, then each key's screen centre by its board id; and
//! what a screen reader meets on each button, and a press the way it presses.
#![cfg(windows)]

use kx_platform_windows::com::Com;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationCondition, IUIAutomationElement,
    IUIAutomationInvokePattern, TreeScope_Descendants, UIA_AutomationIdPropertyId,
    UIA_ButtonControlTypeId, UIA_ControlTypePropertyId, UIA_InvokePatternId,
};
use windows::core::{BSTR, Interface};

/// What a screen reader meets on one button: its id, name and box, and whether it can be pressed.
#[derive(Debug, Clone, PartialEq)]
pub struct KeyRead {
    /// UI Automation id; a key's board id.
    pub id: String,
    /// What a screen reader says.
    pub name: String,
    /// Screen box, in physical pixels.
    pub rect: RECT,
    /// True when it offers the Invoke pattern, which Narrator and NVDA press.
    pub invokable: bool,
}

/// Every button of one window: its UI Automation id and screen box, in physical pixels.
pub struct KeyMap(Vec<(String, RECT)>);

impl KeyMap {
    /// The map of `buttons`, as UI Automation listed them.
    pub fn new(buttons: Vec<(String, RECT)>) -> Self {
        Self(buttons)
    }

    /// The screen centre of the first button with id `id`; none when it is missing or has no area on screen.
    pub fn at(&self, id: &str) -> Option<POINT> {
        self.rect(id).map(|r| centre(&r))
    }

    /// The screen box of the first button with id `id`; none when it is missing or has no area on screen.
    pub fn rect(&self, id: &str) -> Option<RECT> {
        let (_, r) = self.0.iter().find(|(k, _)| !k.is_empty() && k == id)?;
        (r.right > r.left && r.bottom > r.top).then_some(*r)
    }
}

/// The centre of box `r`.
fn centre(r: &RECT) -> POINT {
    POINT {
        x: r.left + (r.right - r.left) / 2,
        y: r.top + (r.bottom - r.top) / 2,
    }
}

/// A UI Automation client that reads windows' buttons.
pub struct Reader(IUIAutomation);

impl Reader {
    /// Starts COM on this thread and creates the client.
    #[allow(unsafe_code, reason = "Creates the system's UI Automation client.")]
    pub fn new() -> Result<Self, String> {
        // Elements may outlive this client on the thread, so COM stays on.
        Com::start().map_err(|e| e.to_string())?.keep();
        // SAFETY: creates the system's UI Automation client object.
        let auto = unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }
            .map_err(|e| format!("UI Automation: {e}"))?;
        Ok(Self(auto))
    }

    /// Every button inside window `hwnd`, with its id and screen box.
    pub fn read(&self, hwnd: HWND) -> Result<KeyMap, String> {
        let keys = self.walk(hwnd)?;
        Ok(KeyMap::new(
            keys.into_iter().map(|k| (k.id, k.rect)).collect(),
        ))
    }

    /// Every button inside window `hwnd`, as a screen reader meets it.
    #[allow(
        unsafe_code,
        reason = "UI Automation calls on objects this client made."
    )]
    pub fn walk(&self, hwnd: HWND) -> Result<Vec<KeyRead>, String> {
        let button = VARIANT::from(UIA_ButtonControlTypeId.0);
        let mut keys = Vec::new();
        // SAFETY: plain calls on live UI Automation objects; a dead window gives an error.
        unsafe {
            let cond = self
                .0
                .CreatePropertyCondition(UIA_ControlTypePropertyId, &button)
                .map_err(err)?;
            let found = self
                .root(hwnd)?
                .FindAll(TreeScope_Descendants, &cond)
                .map_err(err)?;
            for i in 0..found.Length().map_err(err)? {
                keys.push(read_one(&found.GetElement(i).map_err(err)?)?);
            }
        }
        Ok(keys)
    }

    /// Presses the button with id `id` inside window `hwnd` through its Invoke pattern, as a screen reader does.
    #[allow(
        unsafe_code,
        reason = "UI Automation calls on objects this client made."
    )]
    pub fn invoke(&self, hwnd: HWND, id: &str) -> Result<(), String> {
        let want = VARIANT::from(BSTR::from(id));
        // SAFETY: plain calls on live UI Automation objects; a missing button or pattern gives an error.
        unsafe {
            let cond: IUIAutomationCondition = self
                .0
                .CreatePropertyCondition(UIA_AutomationIdPropertyId, &want)
                .map_err(err)?;
            let el = self
                .root(hwnd)?
                .FindFirst(TreeScope_Descendants, &cond)
                .map_err(err)?;
            let pattern = el.GetCurrentPattern(UIA_InvokePatternId).map_err(err)?;
            let invoke: IUIAutomationInvokePattern = pattern.cast().map_err(err)?;
            invoke.Invoke().map_err(err)
        }
    }

    /// The element of window `hwnd`.
    #[allow(unsafe_code, reason = "A UI Automation call on this client.")]
    fn root(&self, hwnd: HWND) -> Result<IUIAutomationElement, String> {
        // SAFETY: a plain call; a dead window gives an error.
        unsafe { self.0.ElementFromHandle(hwnd) }.map_err(err)
    }
}

/// A UI Automation error as text.
#[allow(clippy::needless_pass_by_value, reason = "It fits `map_err` as it is.")]
fn err(e: windows::core::Error) -> String {
    format!("UI Automation: {e}")
}

/// What one button tells a screen reader.
#[allow(unsafe_code, reason = "UI Automation calls on a live element.")]
fn read_one(el: &IUIAutomationElement) -> Result<KeyRead, String> {
    // SAFETY: plain property reads on a live element; a missing pattern reads as an error, so as not invokable.
    unsafe {
        Ok(KeyRead {
            id: el.CurrentAutomationId().map_err(err)?.to_string(),
            name: el.CurrentName().map_err(err)?.to_string(),
            rect: el.CurrentBoundingRectangle().map_err(err)?,
            invokable: el
                .GetCurrentPattern(UIA_InvokePatternId)
                .is_ok_and(|p| !p.as_raw().is_null()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bx(left: i32, top: i32, right: i32, bottom: i32) -> RECT {
        RECT {
            left,
            top,
            right,
            bottom,
        }
    }

    #[test]
    fn a_key_is_aimed_at_the_centre_of_the_box_ui_automation_gives_for_its_id() {
        let keys = KeyMap::new(vec![
            ("bar.grip".into(), bx(0, 0, 400, 24)),
            ("c-0x001E".into(), bx(100, 200, 148, 248)),
        ]);
        let p = keys.at("c-0x001E").expect("found");
        assert_eq!((p.x, p.y), (124, 224));
        assert!(
            keys.at("c-0x001F").is_none(),
            "a key UI Automation did not list"
        );
    }

    #[test]
    fn a_taller_top_bar_moves_the_aim_with_the_key() {
        let at = |bar: i32| {
            let keys = KeyMap::new(vec![("esc".into(), bx(8, 30 + bar, 40, 62 + bar))]);
            keys.at("esc").map(|p| p.y)
        };
        assert_eq!(at(0), Some(46));
        assert_eq!(
            at(24),
            Some(70),
            "the box comes from the keyboard, not from a key map"
        );
    }

    #[test]
    fn a_box_with_no_area_is_not_on_screen_and_the_first_of_two_ids_wins() {
        let keys = KeyMap::new(vec![
            ("lang".into(), bx(5, 5, 5, 30)),
            ("esc".into(), bx(0, 0, 10, 10)),
            ("esc".into(), bx(50, 50, 60, 60)),
        ]);
        assert!(keys.at("lang").is_none());
        let p = keys.at("esc").expect("found");
        assert_eq!((p.x, p.y), (5, 5));
        assert!(keys.at("").is_none());
    }
}
