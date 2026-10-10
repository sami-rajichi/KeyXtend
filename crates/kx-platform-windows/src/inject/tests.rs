mod fake;

use fake::{Fake, VK_A, VK_LCONTROL, VK_RMENU};

use super::*;

const CTRL: Scan = Scan::new(0x1D);
const SHIFT: Scan = Scan::new(0x2A);
const ALT: Scan = Scan::new(0x38);
const LEFT_WIN: Scan = Scan::new(0xE05B);
const T: Scan = Scan::new(0x14);
/// Scan codes in the fake layout: A (the Q place), 1, Enter, the dead `^` key and Space.
const A: u16 = 0x10;
const ONE: u16 = 0x02;
const ENTER_KEY: u16 = 0x1C;
const DEAD: u16 = 0x1A;
const SPACE: u16 = 0x39;

/// Each event as (code, up): the scan code, the virtual key's scan code, or the unit.
fn shown(events: &[KeyEvent]) -> Vec<(u16, bool)> {
    let code = |k: Key| match k {
        Key::Scan(s) => s.code(),
        Key::Vk { scan, .. } => u16::from(scan),
        Key::Unit(u) => u,
    };
    events.iter().map(|e| (code(e.key), e.up)).collect()
}

fn sends(fake: &Fake) -> Vec<Vec<(u16, bool)>> {
    fake.sends.borrow().iter().map(|s| shown(s)).collect()
}

fn down_up(codes: &[u16]) -> Vec<(u16, bool)> {
    let down = codes.iter().map(|&c| (c, false));
    down.chain(codes.iter().rev().map(|&c| (c, true))).collect()
}

fn text(s: &str) -> Redacted<String> {
    Redacted::new(s.to_owned())
}

fn keys(fake: &Fake, s: &str) -> Result<(), InjectError> {
    type_text(fake, &text(s), TextRoute::Keys)
}

#[test]
fn a_tap_presses_and_releases_the_scan_code() {
    let fake = Fake::default();
    assert_eq!(deliver(&fake, &chord_events(&[], T)), Ok(()));
    assert_eq!(sends(&fake), [down_up(&[0x14])]);
}

#[test]
fn a_chord_goes_in_one_send_with_its_keys_held_around_the_tap() {
    let fake = Fake::default();
    assert_eq!(deliver(&fake, &chord_events(&[CTRL, SHIFT], T)), Ok(()));
    assert_eq!(sends(&fake), [down_up(&[0x1D, 0x2A, 0x14])]);
}

#[test]
fn a_partial_send_releases_the_held_keys_and_says_how_many_went() {
    let fake = Fake::taking(&[3]);
    let got = deliver(&fake, &chord_events(&[CTRL, SHIFT], T));
    assert_eq!(got, Err(InjectError::Refused { sent: 3, total: 6 }));
    let release = vec![(0x14, true), (0x2A, true), (0x1D, true)];
    assert_eq!(sends(&fake), [down_up(&[0x1D, 0x2A, 0x14]), release]);
}

#[test]
fn a_send_that_pressed_nothing_sends_nothing_more() {
    let fake = Fake::taking(&[0]);
    let got = deliver(&fake, &chord_events(&[CTRL], T));
    assert_eq!(got, Err(InjectError::Refused { sent: 0, total: 4 }));
    assert_eq!(sends(&fake).len(), 1);
}

#[test]
fn a_send_cut_after_the_tap_releases_only_the_held_key_left() {
    let fake = Fake::taking(&[3]);
    let got = deliver(&fake, &chord_events(&[CTRL], T));
    assert_eq!(got, Err(InjectError::Refused { sent: 3, total: 4 }));
    assert_eq!(sends(&fake)[1], [(0x1D, true)]);
}

#[test]
fn a_cut_chord_holding_only_alt_or_win_taps_the_mask_key_before_the_release() {
    for held in [ALT, LEFT_WIN] {
        let fake = Fake::taking(&[1]);
        let got = deliver(&fake, &chord_events(&[held], T));
        assert_eq!(got, Err(InjectError::Refused { sent: 1, total: 4 }));
        let mask = Key::vk(keys::MASK_VK, 0);
        let want = [
            mask.event(false),
            mask.event(true),
            Key::Scan(held).event(true),
        ];
        assert_eq!(fake.sends.borrow()[1], want);
    }
}

#[test]
fn a_refused_release_is_retried_once_one_event_at_a_time() {
    let fake = Fake::taking(&[3, 0]);
    let got = deliver(&fake, &chord_events(&[CTRL, SHIFT], T));
    assert_eq!(got, Err(InjectError::Refused { sent: 3, total: 6 }));
    let release = vec![(0x14, true), (0x2A, true), (0x1D, true)];
    let retries = release.iter().map(|&e| vec![e]);
    let want: Vec<_> = [down_up(&[0x1D, 0x2A, 0x14]), release.clone()]
        .into_iter()
        .chain(retries)
        .collect();
    assert_eq!(sends(&fake), want);
}

#[test]
fn a_release_refused_again_is_not_tried_a_third_time() {
    let fake = Fake::taking(&[3, 1, 0, 0]);
    let _ = deliver(&fake, &chord_events(&[CTRL, SHIFT], T));
    let got = sends(&fake);
    assert_eq!(
        got.len(),
        4,
        "the send, the release, then one retry each for the two left"
    );
    assert_eq!(got[2..], [vec![(0x2A, true)], vec![(0x1D, true)]]);
}

#[test]
fn unicode_text_goes_in_one_send_with_each_surrogate_pair_whole() {
    let fake = Fake::default();
    assert_eq!(type_text(&fake, &text("aب😀"), TextRoute::Unicode), Ok(()));
    let mut want = Vec::new();
    for unit in [0x61, 0x0628, 0xD83D, 0xDE00] {
        want.extend([(unit, false), (unit, true)]);
    }
    assert_eq!(sends(&fake), [want]);
}

#[test]
fn empty_text_sends_nothing() {
    let fake = Fake::default();
    for route in [TextRoute::Unicode, TextRoute::Keys] {
        assert_eq!(type_text(&fake, &text(""), route), Ok(()));
    }
    assert!(sends(&fake).is_empty());
}

#[test]
fn the_keys_route_holds_shift_and_altgr_as_the_layout_says() {
    let fake = Fake::default();
    assert_eq!(keys(&fake, "aA@"), Ok(()));
    let mut want = down_up(&[A]);
    want.extend(down_up(&[0x2A, A]));
    want.extend(down_up(&[0x1D, 0x38, 0x0B]));
    assert_eq!(sends(&fake), [want]);
    assert_eq!(fake.layouts.get(), 1, "the layout is read once per text");
}

#[test]
fn the_keys_route_sends_altgr_as_a_physical_one_left_ctrl_then_extended_right_alt() {
    let fake = Fake::default();
    assert_eq!(keys(&fake, "@"), Ok(()));
    let ctrl = Key::Vk {
        vk: VK_LCONTROL,
        scan: 0x1D,
        extended: false,
    };
    let right_alt = Key::Vk {
        vk: VK_RMENU,
        scan: 0x38,
        extended: true,
    };
    let sent = fake.sends.borrow()[0].clone();
    assert_eq!(sent[..2], [ctrl.event(false), right_alt.event(false)]);
    assert_eq!(sent[4..], [right_alt.event(true), ctrl.event(true)]);
}

#[test]
fn with_caps_lock_on_the_keys_route_flips_the_shift_choice() {
    let fake = Fake::caps();
    assert_eq!(keys(&fake, "aA1&"), Ok(()));
    let parts = [
        down_up(&[0x2A, A]),
        down_up(&[A]),
        down_up(&[ONE]),
        down_up(&[0x2A, ONE]),
    ];
    assert_eq!(sends(&fake), [parts.concat()]);
}

#[test]
fn a_dead_key_is_followed_by_space_so_it_types_its_own_character() {
    let fake = Fake::default();
    assert_eq!(keys(&fake, "^a"), Ok(()));
    let parts = [down_up(&[DEAD]), down_up(&[SPACE]), down_up(&[A])];
    assert_eq!(sends(&fake), [parts.concat()]);
}

#[test]
fn a_dead_key_that_types_another_character_is_unmappable_and_nothing_is_sent() {
    let fake = Fake::default();
    assert_eq!(keys(&fake, "a`"), Err(InjectError::Unmappable));
    assert!(sends(&fake).is_empty());
}

#[test]
fn the_keys_route_sends_virtual_keys_with_their_scan_codes() {
    let fake = Fake::default();
    assert_eq!(keys(&fake, "A"), Ok(()));
    let first = fake.sends.borrow()[0][0];
    let shift = Key::Vk {
        vk: keys::VK_SHIFT,
        scan: 0x2A,
        extended: false,
    };
    assert_eq!(first, shift.event(false));
    assert_eq!(fake.sends.borrow()[0][1].key, Key::vk(VK_A, u32::from(A)));
}

#[test]
fn the_keys_route_refuses_a_character_with_no_key_before_sending_anything() {
    let fake = Fake::default();
    for s in ["aé", "a😀", "a\u{1}"] {
        assert_eq!(keys(&fake, s), Err(InjectError::Unmappable), "{s:?}");
    }
    assert!(sends(&fake).is_empty());
}

#[test]
fn with_no_app_in_front_the_keys_route_sends_nothing() {
    let fake = Fake {
        away: true,
        ..Fake::default()
    };
    assert_eq!(keys(&fake, "a"), Err(InjectError::Unmappable));
    assert!(sends(&fake).is_empty());
}

#[test]
fn the_keys_route_types_each_line_break_as_one_enter() {
    let fake = Fake::default();
    assert_eq!(keys(&fake, "a\r\na\na\r"), Ok(()));
    let (a, enter) = (down_up(&[A]), down_up(&[ENTER_KEY]));
    let want = [&a[..], &enter, &a, &enter, &a, &enter].concat();
    assert_eq!(sends(&fake), [want]);
}

#[test]
fn a_partial_keys_route_send_lets_go_of_shift() {
    let fake = Fake::taking(&[1]);
    let got = keys(&fake, "A");
    assert_eq!(got, Err(InjectError::Refused { sent: 1, total: 4 }));
    assert_eq!(sends(&fake)[1], [(0x2A, true)]);
}
