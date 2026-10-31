//! The Power key in the state: dimmed with a tip until the power module runs, then a plain side key.

use super::*;

/// The state with the power module in state `power`, the other modules off.
fn with(power: PowerChanged) -> StateView {
    let layer = owners_layer();
    let others = Others {
        assist: AssistChanged::OFF,
        scroll: ScrollChanged::OFF,
        layer: &layer,
        power,
        clip: false,
    };
    state_with(&none(), false, None, &others)
}

#[test]
fn until_the_power_module_runs_the_power_key_is_dim_and_says_it_is_off() {
    let k = &with(PowerChanged::OFF).keys["power"];
    assert_eq!(
        (k.dim, k.paint, k.tip, k.on),
        (true, DIM, Some(POWER_OFF_TIP), false)
    );
}

#[test]
fn a_running_power_module_makes_the_power_key_a_plain_side_key() {
    let k = &with(PowerChanged { live: true }).keys["power"];
    assert_eq!((k.dim, k.tip, k.on), (false, None, false));
    assert_eq!(k.paint, paint(KeyKind::Act, false));
}

#[test]
fn the_later_side_keys_stay_dim_with_no_tip_whatever_the_power_module_says() {
    let mic = &with(PowerChanged { live: true }).keys["mic"];
    assert_eq!((mic.paint, mic.tip, mic.dim), (DIM, None, true));
}
