//! Tests of the layout source over a fake OS.

use std::cell::{Cell, RefCell};

use super::*;
use crate::keys::ALTGR_VKS;

/// Installed layouts: US English and Arabic (102) AZERTY, a sign-extended variant.
const EN: LayoutId = LayoutId::new(0x0409_0409);
const AR: LayoutId = LayoutId::new(0xFFFF_FFFF_F029_1C01);
/// A layout that is not installed.
const GONE: LayoutId = LayoutId::new(0x0419_0419);
/// Keys: Q, Enter, Left (extended), right Alt (extended), and one with no virtual key.
const Q: Scan = Scan::new(0x10);
const ENTER: Scan = Scan::new(0x1C);
const LEFT: Scan = Scan::new(0xE04B);
const RIGHT_ALT: Scan = Scan::new(0xE038);
const NO_VK: Scan = Scan::new(0x59);
/// Their virtual keys.
const VK_Q: u16 = 0x51;
const VK_RETURN: u16 = 0x0D;
const VK_LEFT: u16 = 0x25;
const VK_RMENU: u16 = 0xA5;
/// The window with the keyboard in the app in front.
const WINDOW: u32 = 7;

/// A fake OS: a layout list, a window in front, and a Q key that answers by modifier state.
#[derive(Default)]
struct Fake {
    layouts: Vec<LayoutId>,
    /// The copy returns one layout fewer than the count, as when one is removed in between.
    shrink: bool,
    focus: Option<u32>,
    refuse: bool,
    asked: Cell<usize>,
    last: Cell<Option<KeyState>>,
    posts: RefCell<Vec<(u32, LayoutId)>>,
}

impl Fake {
    fn new() -> Self {
        Self {
            layouts: vec![EN, AR],
            focus: Some(WINDOW),
            ..Self::default()
        }
    }
}

/// True when virtual key `vk` is down in `state`.
fn down(state: &KeyState, vk: u16) -> bool {
    state[usize::from(vk)] & DOWN != 0
}

/// What Q types in `state`: a text of its own per slot, so a swapped slot fails; `AltGr` is a dead `@`.
fn q_text(state: &KeyState) -> (&'static str, i32) {
    let [ctrl, alt] = ALTGR_VKS;
    if down(state, ctrl) && down(state, alt) {
        return ("@", -1);
    }
    let caps = state[usize::from(VK_CAPITAL)] & TOGGLED != 0;
    match (down(state, VK_SHIFT), caps) {
        (false, false) => ("q", 1),
        (true, false) => ("Q", 1),
        (false, true) => ("C", 1),
        (true, true) => ("c", 1),
    }
}

impl Os for Fake {
    type Window = u32;

    fn list(&self, buf: &mut [LayoutId]) -> usize {
        if buf.is_empty() {
            return self.layouts.len();
        }
        let n = if self.shrink {
            buf.len() - 1
        } else {
            buf.len()
        };
        buf[..n].copy_from_slice(&self.layouts[..n]);
        n
    }

    fn focus(&self) -> Option<u32> {
        self.focus
    }

    fn layout_of(&self, window: u32) -> Option<LayoutId> {
        (window == WINDOW).then_some(AR)
    }

    fn vk_of(&self, key: Scan, _layout: LayoutId) -> u16 {
        match key {
            Q => VK_Q,
            ENTER => VK_RETURN,
            LEFT => VK_LEFT,
            RIGHT_ALT => VK_RMENU,
            _ => 0,
        }
    }

    fn to_unicode(&self, vk: u16, _: Scan, state: &KeyState, _: LayoutId, buf: &mut [u16]) -> i32 {
        self.asked.set(self.asked.get() + 1);
        self.last.set(Some(*state));
        let (text, n) = match vk {
            VK_Q => q_text(state),
            VK_RETURN => ("\r", 1),
            _ => ("", 0),
        };
        for (slot, unit) in buf.iter_mut().zip(text.encode_utf16()) {
            *slot = unit;
        }
        n
    }

    fn facts(&self, _layout: LayoutId) -> LangFacts {
        LangFacts {
            iso: "ar".into(),
            ..LangFacts::default()
        }
    }

    fn post(&self, window: u32, layout: LayoutId) -> Result<(), SwitchError> {
        if self.refuse {
            return Err(SwitchError::Refused);
        }
        self.posts.borrow_mut().push((window, layout));
        Ok(())
    }
}

#[test]
fn every_installed_layout_is_listed_even_past_thirty_two() {
    let many: Vec<LayoutId> = (0..40).map(|n| LayoutId::new(0x0409_0409 + n)).collect();
    let fake = Fake {
        layouts: many.clone(),
        ..Fake::new()
    };
    assert_eq!(installed(&fake), many);
}

#[test]
fn a_layout_removed_between_the_count_and_the_copy_leaves_the_list() {
    let fake = Fake {
        shrink: true,
        ..Fake::new()
    };
    assert_eq!(installed(&fake), [EN]);
}

#[test]
fn each_slot_reads_its_modifier_state_and_dead_slots_are_marked() {
    let want = KeyChars {
        base: "q".into(),
        shift: "Q".into(),
        altgr: "@".into(),
        caps: "C".into(),
        caps_shift: "c".into(),
        dead: vec![Slot::AltGr],
    };
    assert_eq!(keys(&Fake::new(), EN, &[Q]), [want]);
}

#[test]
fn a_control_character_reads_as_empty_but_types_something() {
    let fake = Fake::new();
    assert_eq!(keys(&fake, EN, &[ENTER]), [KeyChars::default()]);
    assert!(types_any(&fake, EN, ENTER, &[]));
    assert!(!types_any(&fake, EN, LEFT, &[]), "an arrow types nothing");
}

#[test]
fn a_key_without_a_virtual_key_types_nothing_and_windows_is_not_asked() {
    let fake = Fake::new();
    assert_eq!(keys(&fake, EN, &[NO_VK]), [KeyChars::default()]);
    assert!(!types_any(&fake, EN, NO_VK, &[]));
    assert_eq!(fake.asked.get(), 0);
}

#[test]
fn an_uninstalled_layout_types_nothing_has_no_facts_and_windows_is_not_asked() {
    let fake = Fake::new();
    assert_eq!(keys(&fake, GONE, &[Q, ENTER]), vec![KeyChars::default(); 2]);
    assert!(!types_any(&fake, GONE, ENTER, &[]));
    assert_eq!(facts(&fake, GONE), LangFacts::default());
    assert_eq!(fake.asked.get(), 0);
    assert_eq!(facts(&fake, AR).iso, "ar");
}

#[test]
fn a_held_modifier_is_down_on_its_side_and_generic_keys_only() {
    let fake = Fake::new();
    assert!(types_any(&fake, EN, Q, &[RIGHT_ALT]));
    let state = fake.last.get().unwrap();
    let [_, alt] = ALTGR_VKS;
    assert!(down(&state, VK_RMENU) && down(&state, alt));
    let others = (0..=u16::from(u8::MAX))
        .filter(|&vk| down(&state, vk))
        .count();
    assert_eq!(others, 2, "only right Alt and Alt");
    assert_eq!(state[usize::from(VK_CAPITAL)] & TOGGLED, 0, "Caps Lock off");
}

#[test]
fn the_layout_in_front_is_that_of_the_window_with_the_keyboard() {
    assert_eq!(foreground(&Fake::new()), Some(AR));
    let empty = Fake {
        focus: None,
        ..Fake::new()
    };
    assert_eq!(foreground(&empty), None);
}

#[test]
fn a_request_goes_to_the_window_with_the_keyboard() {
    let fake = Fake::new();
    assert_eq!(request(&fake, fake.focus, EN), Ok(()));
    assert_eq!(*fake.posts.borrow(), [(WINDOW, EN)]);
}

#[test]
fn a_request_without_a_target_or_for_an_uninstalled_layout_posts_nothing() {
    let fake = Fake::new();
    assert_eq!(request(&fake, None, EN), Err(SwitchError::NoTarget));
    assert_eq!(
        request(&fake, fake.focus, GONE),
        Err(SwitchError::NotInstalled)
    );
    assert!(fake.posts.borrow().is_empty());
    let refusing = Fake {
        refuse: true,
        ..Fake::new()
    };
    assert_eq!(
        request(&refusing, refusing.focus, EN),
        Err(SwitchError::Refused)
    );
}

#[test]
fn the_state_holds_the_keys_the_generic_modifiers_and_caps_lock() {
    let shift = state(&SHIFT, false);
    assert!(down(&shift, VK_SHIFT));
    assert_eq!(shift.iter().filter(|&&b| b != 0).count(), 1);
    let altgr = state(&ALTGR_READ, false);
    for vk in ALTGR_READ.into_iter().chain(ALTGR_VKS) {
        assert!(down(&altgr, vk), "{vk:#04X}");
    }
    let caps = state(&[], true);
    assert_eq!(caps[usize::from(VK_CAPITAL)], TOGGLED);
    assert_eq!(caps.iter().filter(|&&b| b != 0).count(), 1);
    assert_eq!(
        state(&[0x1FF, 0], false),
        [0; KEY_STATES],
        "out of range or none"
    );
}

mod reader;
