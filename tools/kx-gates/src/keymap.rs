//! Key boxes by UI Automation id: one read of a window's buttons, then each key's screen centre by its board id; and
//! what a screen reader meets on each button or at a point, and a press the way it presses.
#![cfg(windows)]

use kx_platform_windows::com::Com;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::System::Variant::{VARIANT, VT_I4};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationCondition, IUIAutomationElement,
    IUIAutomationInvokePattern, ToggleState_On, TreeScope_Descendants, UIA_AutomationIdPropertyId,
    UIA_ButtonControlTypeId, UIA_ControlTypePropertyId, UIA_InvokePatternId,
    UIA_ToggleToggleStatePropertyId,
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

/// `id` when it can name one button; an empty id would match the first button that has none, such as S, M or L.
fn wanted(id: &str) -> Result<&str, String> {
    if id.is_empty() {
        Err("an empty UI Automation id names no button".to_string())
    } else {
        Ok(id)
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
        let el = self.find(hwnd, id)?;
        // SAFETY: plain calls on a live element; a missing pattern gives an error.
        unsafe {
            let pattern = el.GetCurrentPattern(UIA_InvokePatternId).map_err(err)?;
            let invoke: IUIAutomationInvokePattern = pattern.cast().map_err(err)?;
            invoke.Invoke().map_err(err)
        }
    }

    /// Whether the button with id `id` inside window `hwnd` reads as on, as a screen reader reads a latched key;
    /// none when it has no toggle state.
    #[allow(unsafe_code, reason = "A UI Automation call on a live element.")]
    pub fn toggled(&self, hwnd: HWND, id: &str) -> Result<Option<bool>, String> {
        let el = self.find(hwnd, id)?;
        // SAFETY: a plain property read on a live element.
        let v =
            unsafe { el.GetCurrentPropertyValue(UIA_ToggleToggleStatePropertyId) }.map_err(err)?;
        Ok(toggle_state(&v))
    }

    /// The element UI Automation finds at screen point `p`, as Narrator's mouse mode reads what is under the pointer.
    #[allow(unsafe_code, reason = "A UI Automation call on this client.")]
    pub fn at(&self, p: POINT) -> Result<IUIAutomationElement, String> {
        // SAFETY: a plain call; any point is allowed, and it moves no pointer.
        unsafe { self.0.ElementFromPoint(p) }.map_err(err)
    }

    /// The first element with UI Automation id `id` inside window `hwnd`.
    #[allow(
        unsafe_code,
        reason = "UI Automation calls on objects this client made."
    )]
    fn find(&self, hwnd: HWND, id: &str) -> Result<IUIAutomationElement, String> {
        let want = VARIANT::from(BSTR::from(wanted(id)?));
        // SAFETY: plain calls on live UI Automation objects; a missing button gives an error.
        unsafe {
            let cond: IUIAutomationCondition = self
                .0
                .CreatePropertyCondition(UIA_AutomationIdPropertyId, &want)
                .map_err(err)?;
            self.root(hwnd)?
                .FindFirst(TreeScope_Descendants, &cond)
                .map_err(err)
        }
    }

    /// The element of window `hwnd`.
    #[allow(unsafe_code, reason = "A UI Automation call on this client.")]
    fn root(&self, hwnd: HWND) -> Result<IUIAutomationElement, String> {
        // SAFETY: a plain call; a dead window gives an error.
        unsafe { self.0.ElementFromHandle(hwnd) }.map_err(err)
    }
}

/// The toggle state in property value `v`: on or off, or none when the element has no Toggle pattern.
fn toggle_state(v: &VARIANT) -> Option<bool> {
    let state = (v.vt() == VT_I4).then(|| i32::try_from(v).ok()).flatten()?;
    Some(state == ToggleState_On.0)
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
    fn the_aim_follows_the_box_ui_automation_reports() {
        let at = |down: i32| {
            let keys = KeyMap::new(vec![("esc".into(), bx(8, 30 + down, 40, 62 + down))]);
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
    fn a_toggle_state_reads_as_on_or_off_and_a_button_without_one_as_none() {
        use windows::Win32::UI::Accessibility::{ToggleState_Off, ToggleState_On};
        assert_eq!(toggle_state(&VARIANT::from(ToggleState_On.0)), Some(true));
        assert_eq!(toggle_state(&VARIANT::from(ToggleState_Off.0)), Some(false));
        assert_eq!(toggle_state(&VARIANT::default()), None, "no Toggle pattern");
        assert_eq!(
            toggle_state(&VARIANT::from(true)),
            None,
            "not a toggle state"
        );
    }

    #[test]
    fn an_empty_id_names_no_button_so_it_is_refused_before_ui_automation() {
        assert_eq!(wanted("esc"), Ok("esc"));
        let e = wanted("").expect_err("the S, M and L buttons have no id");
        assert!(e.contains("empty"), "{e}");
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
