use std::cell::{Cell, RefCell};

use super::*;

/// The layout the fake reports in front; lookups in any other layout fail the test.
const FRONT: u32 = 0x040C_040C;
const CTRL: Scan = Scan::new(0x1D);
const SHIFT: Scan = Scan::new(0x2A);
const T: Scan = Scan::new(0x14);
/// `VK_A`, `VK_0` and `VK_RETURN`.
const VK_A: u16 = 0x41;
const VK_0: u16 = 0x30;
const VK_RETURN: u16 = 0x0D;
/// `VkKeyScanEx` answers of a French AZERTY layout: 'a', 'A', '@' (`AltGr`+0), Enter, and Ctrl-only ones.
const KEYMAP: [(char, i16); 6] = [
    ('a', 0x0041),
    ('A', 0x0141),
    ('@', 0x0630),
    ('\r', 0x000D),
    ('\n', 0x020D),
    ('\u{1}', 0x0241),
];
/// `MapVirtualKeyExW` scan codes in that layout: Shift, Ctrl, Alt, A (the Q place), 0 and Enter.
const SCANS: [(u16, u32); 6] = [
    (keys::VK_SHIFT, 0x2A),
    (keys::ALTGR_VKS[0], 0x1D),
    (keys::ALTGR_VKS[1], 0x38),
    (VK_A, 0x10),
    (VK_0, 0x0B),
    (VK_RETURN, 0x1C),
];

/// Records each batch; the first send takes only `take` events when set.
#[derive(Default)]
struct Fake {
    take: Option<usize>,
    sends: RefCell<Vec<Vec<KeyEvent>>>,
    layouts: Cell<usize>,
}

impl Fake {
    fn taking(take: usize) -> Self {
        Self {
            take: Some(take),
            ..Self::default()
        }
    }

    fn sends(&self) -> Vec<Vec<(u16, bool)>> {
        self.sends.borrow().iter().map(|s| shown(s)).collect()
    }
}

impl Os for Fake {
    type Layout = u32;

    fn send(&self, events: &[KeyEvent]) -> usize {
        let first = self.sends.borrow().is_empty();
        self.sends.borrow_mut().push(events.to_vec());
        match self.take {
            Some(take) if first => take.min(events.len()),
            _ => events.len(),
        }
    }

    fn layout(&self) -> u32 {
        self.layouts.set(self.layouts.get() + 1);
        FRONT
    }

    fn key_scan(&self, unit: u16, layout: u32) -> i16 {
        assert_eq!(layout, FRONT);
        let hit = KEYMAP
            .iter()
            .find(|(c, _)| u32::from(*c) == u32::from(unit));
        hit.map_or(-1, |&(_, raw)| raw)
    }

    fn scan_of(&self, vk: u16, layout: u32) -> u32 {
        assert_eq!(layout, FRONT);
        SCANS.iter().find(|(v, _)| *v == vk).map_or(0, |&(_, s)| s)
    }
}

/// Each event as (code, up): the scan code, the virtual key's scan code, or the unit.
fn shown(events: &[KeyEvent]) -> Vec<(u16, bool)> {
    let code = |k: Key| match k {
        Key::Scan(s) => s.code(),
        Key::Vk { scan, .. } => u16::from(scan),
        Key::Unit(u) => u,
    };
    events.iter().map(|e| (code(e.key), e.up)).collect()
}

fn down_up(codes: &[u16]) -> Vec<(u16, bool)> {
    let down = codes.iter().map(|&c| (c, false));
    down.chain(codes.iter().rev().map(|&c| (c, true))).collect()
}

fn text(s: &str) -> Redacted<String> {
    Redacted::new(s.to_owned())
}

#[test]
fn a_tap_presses_and_releases_the_scan_code() {
    let fake = Fake::default();
    assert_eq!(deliver(&fake, &chord_events(&[], T)), Ok(()));
    assert_eq!(fake.sends(), [down_up(&[0x14])]);
}

#[test]
fn a_chord_goes_in_one_send_with_its_keys_held_around_the_tap() {
    let fake = Fake::default();
    assert_eq!(deliver(&fake, &chord_events(&[CTRL, SHIFT], T)), Ok(()));
    assert_eq!(fake.sends(), [down_up(&[0x1D, 0x2A, 0x14])]);
}

#[test]
fn a_partial_send_releases_the_held_keys_and_says_how_many_went() {
    let fake = Fake::taking(3);
    let got = deliver(&fake, &chord_events(&[CTRL, SHIFT], T));
    assert_eq!(got, Err(InjectError::Refused { sent: 3, total: 6 }));
    let release = vec![(0x14, true), (0x2A, true), (0x1D, true)];
    assert_eq!(fake.sends(), [down_up(&[0x1D, 0x2A, 0x14]), release]);
}

#[test]
fn a_send_that_pressed_nothing_sends_nothing_more() {
    let fake = Fake::taking(0);
    let got = deliver(&fake, &chord_events(&[CTRL], T));
    assert_eq!(got, Err(InjectError::Refused { sent: 0, total: 4 }));
    assert_eq!(fake.sends().len(), 1);
}

#[test]
fn a_send_cut_after_the_tap_releases_only_the_held_key_left() {
    let fake = Fake::taking(3);
    let got = deliver(&fake, &chord_events(&[CTRL], T));
    assert_eq!(got, Err(InjectError::Refused { sent: 3, total: 4 }));
    assert_eq!(fake.sends()[1], [(0x1D, true)]);
}

#[test]
fn unicode_text_goes_in_one_send_with_each_surrogate_pair_whole() {
    let fake = Fake::default();
    assert_eq!(type_text(&fake, &text("aب😀"), TextRoute::Unicode), Ok(()));
    let mut want = Vec::new();
    for unit in [0x61, 0x0628, 0xD83D, 0xDE00] {
        want.extend([(unit, false), (unit, true)]);
    }
    assert_eq!(fake.sends(), [want]);
}

#[test]
fn empty_text_sends_nothing() {
    let fake = Fake::default();
    for route in [TextRoute::Unicode, TextRoute::Keys] {
        assert_eq!(type_text(&fake, &text(""), route), Ok(()));
    }
    assert!(fake.sends().is_empty());
}

#[test]
fn the_keys_route_holds_shift_and_altgr_as_the_layout_says() {
    let fake = Fake::default();
    assert_eq!(type_text(&fake, &text("aA@"), TextRoute::Keys), Ok(()));
    let mut want = down_up(&[0x10]);
    want.extend(down_up(&[0x2A, 0x10]));
    want.extend(down_up(&[0x1D, 0x38, 0x0B]));
    assert_eq!(fake.sends(), [want]);
    assert_eq!(fake.layouts.get(), 1, "the layout is read once per text");
}

#[test]
fn the_keys_route_sends_virtual_keys_with_their_scan_codes() {
    let fake = Fake::default();
    assert_eq!(type_text(&fake, &text("A"), TextRoute::Keys), Ok(()));
    let first = fake.sends.borrow()[0][0];
    let shift = Key::Vk {
        vk: keys::VK_SHIFT,
        scan: 0x2A,
        extended: false,
    };
    assert_eq!(first, shift.event(false));
}

#[test]
fn the_keys_route_refuses_a_character_with_no_key_before_sending_anything() {
    let fake = Fake::default();
    for s in ["aé", "a😀", "a\u{1}"] {
        let got = type_text(&fake, &text(s), TextRoute::Keys);
        assert_eq!(got, Err(InjectError::Unmappable), "{s:?}");
    }
    assert!(fake.sends().is_empty());
}

#[test]
fn the_keys_route_types_each_line_break_as_one_enter() {
    let fake = Fake::default();
    assert_eq!(
        type_text(&fake, &text("a\r\na\na\r"), TextRoute::Keys),
        Ok(())
    );
    let (a, enter) = (down_up(&[0x10]), down_up(&[0x1C]));
    let want = [&a[..], &enter, &a, &enter, &a, &enter].concat();
    assert_eq!(fake.sends(), [want]);
}

#[test]
fn a_partial_keys_route_send_lets_go_of_shift() {
    let fake = Fake::taking(1);
    let got = type_text(&fake, &text("A"), TextRoute::Keys);
    assert_eq!(got, Err(InjectError::Refused { sent: 1, total: 4 }));
    assert_eq!(fake.sends()[1], [(0x2A, true)]);
}
