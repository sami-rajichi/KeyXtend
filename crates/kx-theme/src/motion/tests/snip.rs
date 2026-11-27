//! Snip's capture windows (design/snip-parts.css): the bar and strip popping in, the countdown, and the record dot's pulse.

use super::*;

#[test]
fn the_snip_windows_pop_and_pulse_as_the_mock_up_and_stand_still_with_reduced_motion() {
    let m = load().expect("motion.toml loads");
    let (swift, ease_out) = (m.full.moves["chips"].bez, m.full.moves["caption"].bez);
    let pop = &m.full.moves["snip_pop"];
    assert_eq!(
        (pop.ms, pop.bez),
        (180, swift),
        ".cbar, .strip popIn .18s cubic-bezier(.2,.8,.2,1)"
    );
    let count = &m.full.moves["snip_count"];
    assert_eq!(
        (count.ms, count.bez),
        (200, ease_out),
        ".count popIn .2s ease-out"
    );
    let dot = &m.full.moves["snip_dot"];
    assert_eq!(
        (dot.ms, dot.bez),
        (1200, m.full.moves["led"].bez),
        ".dot pulse 1.2s ease-in-out"
    );
    assert_eq!(
        m.full.amount.pulse_low, 0.35,
        "@keyframes pulse 50% opacity .35"
    );
    for name in ["snip_pop", "snip_count", "snip_dot"] {
        assert_eq!(
            m.reduced.moves[name].ms, 0,
            "{name}: the dot stops pulsing (edge 23)"
        );
    }
}
