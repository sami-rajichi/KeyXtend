//! Key boxes by UI Automation id: one read of a window's buttons, then each key's screen centre by its board id.
#![cfg(windows)]

use kx_platform_windows::com::Com;
use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance};
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, TreeScope_Descendants, UIA_ButtonControlTypeId,
    UIA_ControlTypePropertyId,
};

/// Every button of one window: its UI Automation id and screen box, in physical pixels.
pub struct KeyMap(Vec<(String, RECT)>);

impl KeyMap {
    /// The map of `buttons`, as UI Automation listed them.
    pub fn new(buttons: Vec<(String, RECT)>) -> Self {
        Self(buttons)
    }

    /// The screen centre of the first button with id `id`; none when it is missing or has no area on screen.
    pub fn at(&self, id: &str) -> Option<POINT> {
        let (_, r) = self.0.iter().find(|(k, _)| !k.is_empty() && k == id)?;
        (r.right > r.left && r.bottom > r.top).then(|| centre(r))
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
    #[allow(
        unsafe_code,
        reason = "UI Automation calls on objects this client made."
    )]
    pub fn read(&self, hwnd: HWND) -> Result<KeyMap, String> {
        let err = |e: windows::core::Error| format!("UI Automation: {e}");
        let button = VARIANT::from(UIA_ButtonControlTypeId.0);
        let mut buttons = Vec::new();
        // SAFETY: plain calls on live UI Automation objects; a dead window gives an error.
        unsafe {
            let cond = self
                .0
                .CreatePropertyCondition(UIA_ControlTypePropertyId, &button)
                .map_err(err)?;
            let root = self.0.ElementFromHandle(hwnd).map_err(err)?;
            let found = root.FindAll(TreeScope_Descendants, &cond).map_err(err)?;
            for i in 0..found.Length().map_err(err)? {
                let el = found.GetElement(i).map_err(err)?;
                let id = el.CurrentAutomationId().map_err(err)?.to_string();
                buttons.push((id, el.CurrentBoundingRectangle().map_err(err)?));
            }
        }
        Ok(KeyMap::new(buttons))
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
