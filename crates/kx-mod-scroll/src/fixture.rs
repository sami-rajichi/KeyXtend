//! Test settings for the rules' unit tests, none of them a default, so no rule's test leans on `defaults.toml`.

use crate::ScrollSettings;

/// The fixture's time limit after the last pad tap, in seconds.
pub(crate) const LIMIT_S: u32 = 60;

/// Valid settings with both stops on and every value off its default.
pub(crate) const fn settings() -> ScrollSettings {
    ScrollSettings {
        slow: 4,
        fast: 12,
        limit_s: LIMIT_S,
        stop_at_end: true,
        stop_on_press: true,
        tick_ms: 20,
        check_ms: 200,
        read_ms: 150,
        edge_px: 5,
    }
}

#[test]
fn the_fixture_is_valid_and_off_the_defaults() {
    use kx_module_api::Settings;
    let s = settings();
    assert_eq!(s.check(), Ok(()));
    let d = ScrollSettings::from_defaults().unwrap();
    assert!(s.slow != d.slow && s.fast != d.fast && s.limit_s != d.limit_s);
    assert!(s.tick_ms != d.tick_ms && s.check_ms != d.check_ms);
}
