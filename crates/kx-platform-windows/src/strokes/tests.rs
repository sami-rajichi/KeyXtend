use super::*;
use proptest::prelude::*;

const CTRL: Scan = Scan::new(0x1D);
const SHIFT: Scan = Scan::new(0x2A);
const RIGHT_ALT: Scan = Scan::new(0xE038);
const T: Scan = Scan::new(0x14);
const SECRET: &str = "hunter2";

/// Each event as (code, up, extended), readable in a failed assertion.
fn shown(events: &[KeyEvent]) -> Vec<(u16, bool, bool)> {
    let code = |k: Key| match k {
        Key::Scan(s) => s.code(),
        Key::Vk { vk, .. } => vk,
        Key::Unit(u) => u,
    };
    events
        .iter()
        .map(|e| (code(e.key), e.up, e.key.extended()))
        .collect()
}

fn scans(codes: &[Scan]) -> Vec<Key> {
    codes.iter().map(|&s| Key::Scan(s)).collect()
}

#[test]
fn a_chord_holds_its_keys_around_the_tap_and_right_alt_stays_extended() {
    let got = shown(&combo(&scans(&[CTRL, RIGHT_ALT, T])));
    let want = [
        (0x1D, false, false),
        (0xE038, false, true),
        (0x14, false, false),
        (0x14, true, false),
        (0xE038, true, true),
        (0x1D, true, false),
    ];
    assert_eq!(got, want);
}

#[test]
fn a_virtual_key_combo_presses_in_order_and_releases_in_reverse() {
    let keys = [Key::vk(0x11, 0x1D), Key::vk(0x43, 0x2E)];
    let got = shown(&combo(&keys));
    let want = [
        (0x11, false, false),
        (0x43, false, false),
        (0x43, true, false),
        (0x11, true, false),
    ];
    assert_eq!(got, want);
}

#[test]
fn a_virtual_key_keeps_its_scan_byte_and_extended_flag() {
    let right_alt = Key::vk(0xA5, 0xE038);
    let want = Key::Vk {
        vk: 0xA5,
        scan: 0x38,
        extended: true,
    };
    assert_eq!(right_alt, want);
    assert!(right_alt.extended());
    assert!(Key::vk(keys::EXTENDED_VKS[0], 0x49).extended(), "Page Up");
    assert!(!Key::vk(0x41, 0x1E).extended(), "A");
}

#[test]
fn a_scan_key_is_extended_only_with_the_e0_prefix_and_a_unit_never() {
    assert!(Key::Scan(RIGHT_ALT).extended());
    assert!(!Key::Scan(CTRL).extended());
    assert!(!Key::Unit(0xE038).extended());
}

#[test]
fn each_unit_is_pressed_and_released_so_a_surrogate_pair_stays_whole() {
    let got = shown(&unicode("aب😀"));
    let mut want = Vec::new();
    for unit in [0x61, 0x0628, 0xD83D, 0xDE00] {
        want.extend([(unit, false, false), (unit, true, false)]);
    }
    assert_eq!(got, want);
    assert!(unicode("").is_empty());
}

#[test]
fn debug_output_never_shows_what_was_typed() {
    let mut events = unicode(SECRET);
    events.extend(combo(&[Key::Scan(T), Key::vk(0x41, 0x1E)]));
    let shown = format!("{events:?}");
    assert!(!shown.chars().any(|c| c.is_ascii_digit()), "{shown}");
    assert!(shown.contains(REDACTED_MARKER), "{shown}");
}

#[test]
fn nothing_is_still_down_after_a_whole_chord() {
    assert!(still_down(&combo(&scans(&[CTRL, SHIFT, T]))).is_empty());
    assert!(still_down(&[]).is_empty());
}

#[test]
fn a_cut_chord_releases_what_went_down_newest_first() {
    let events = combo(&scans(&[CTRL, SHIFT, T]));
    let got = shown(&still_down(&events[..3]));
    let want = [
        (0x14, true, false),
        (0x2A, true, false),
        (0x1D, true, false),
    ];
    assert_eq!(got, want);
}

#[test]
fn letting_go_of_alt_or_win_taps_the_mask_key_first() {
    let mask = Key::vk(keys::MASK_VK, 0);
    let alt_win = [
        Scan::new(0x38),
        RIGHT_ALT,
        Scan::new(0xE05B),
        Scan::new(0xE05C),
    ];
    let vks = keys::ALT_WIN_VKS.map(|vk| Key::vk(vk, 0));
    for held in alt_win.map(Key::Scan).into_iter().chain(vks) {
        let events = combo(&[Key::Scan(CTRL), held, Key::Scan(T)]);
        let ups = [held.event(true), Key::Scan(CTRL).event(true)];
        let want = [[mask.event(false), mask.event(true)], ups].concat();
        assert_eq!(releases(&events[..2]), want);
    }
}

#[test]
fn letting_go_of_other_keys_needs_no_mask() {
    let events = combo(&scans(&[CTRL, SHIFT, T]));
    assert_eq!(releases(&events[..3]), still_down(&events[..3]));
    assert!(releases(&events).is_empty());
}

fn key() -> impl Strategy<Value = Key> {
    prop_oneof![
        (0..=u16::MAX).prop_map(|c| Key::Scan(Scan::new(c))),
        (0x08..0xFFu16, 0..0xE1FFu32).prop_map(|(vk, mapped)| Key::vk(vk, mapped)),
        (0..=u16::MAX).prop_map(Key::Unit),
    ]
}

proptest! {
    #[test]
    fn a_cut_send_plus_its_releases_leaves_nothing_down(
        keys in prop::collection::vec(key(), 0..6),
        cut in 0usize..=12,
    ) {
        let events = combo(&keys);
        let sent = &events[..cut.min(events.len())];
        let release = still_down(sent);
        prop_assert!(release.iter().all(|e| e.up));
        let all: Vec<KeyEvent> = sent.iter().chain(&release).copied().collect();
        for k in &keys {
            let count = |up| all.iter().filter(|e| e.key == *k && e.up == up).count();
            prop_assert_eq!(count(false), count(true));
        }
        prop_assert!(still_down(&all).is_empty());
        let masked = releases(sent);
        prop_assert!(masked.ends_with(&release));
        prop_assert!(still_down(&[sent, &masked].concat()).is_empty());
    }
}
