//! The fill key in the state (ADR-0037 decision 2): dimmed with a tip until the vault module runs, then a plain side
//! key that lights while its panel is open.

use super::*;

/// The state with the vault module in state `fill` and `panel` open, the other modules off.
fn with(fill: FillLive, panel: Option<Panel>) -> StateView {
    let layer = owners_layer();
    let others = Others {
        assist: AssistChanged::OFF,
        scroll: ScrollChanged::OFF,
        layer: &layer,
        power: PowerChanged::OFF,
        fill,
        panel,
    };
    state_with(&none(), false, None, &others)
}

#[test]
fn until_the_vault_runs_the_fill_key_is_dim_and_says_quick_fill_is_off() {
    let k = &with(FillLive::OFF, None).keys["fill"];
    assert_eq!(
        (k.dim, k.paint, k.tip, k.on),
        (true, DIM, Some(FILL_OFF_TIP), false)
    );
}

#[test]
fn a_running_vault_makes_the_fill_key_a_plain_side_key() {
    let k = &with(FillLive { live: true }, None).keys["fill"];
    assert_eq!((k.dim, k.tip, k.on), (false, None, false));
    assert_eq!(k.paint, paint(KeyKind::Act, false));
}

#[test]
fn a_vault_that_is_off_never_lights_the_fill_key() {
    let s = with(FillLive::OFF, Some(Panel::Fill));
    let k = &s.keys["fill"];
    assert_eq!((k.dim, k.tip, k.on), (true, Some(FILL_OFF_TIP), false));
    assert_eq!(s.keys["clip"].tip, None, "only its own key");
}
