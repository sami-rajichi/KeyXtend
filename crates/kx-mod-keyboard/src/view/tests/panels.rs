//! The clipboard and Quick-fill keys in the state, the vault running: plain side keys, each lit while its panel is open.

use super::*;

/// The state with `panel` open and the vault running, the other modules off.
fn with(panel: Option<Panel>) -> StateView {
    let layer = owners_layer();
    let others = Others {
        assist: AssistChanged::OFF,
        scroll: ScrollChanged::OFF,
        layer: &layer,
        power: PowerChanged::OFF,
        fill: FillLive { live: true },
        panel,
    };
    state_with(&none(), false, None, &others)
}

#[test]
fn the_clip_key_lights_while_the_panel_is_open_and_never_dims() {
    let shut = &with(None).keys["clip"];
    assert_eq!((shut.on, shut.dim, shut.tip), (false, false, None));
    assert_eq!(shut.paint, paint(KeyKind::Act, false));
    let open = with(Some(Panel::Clip));
    let k = &open.keys["clip"];
    assert_eq!((k.on, k.dim, k.tip), (true, false, None));
    assert_eq!(k.paint, paint(KeyKind::Act, true));
    assert!(!open.keys["shortcuts"].on, "only its own key lights");
    assert!(!open.keys["fill"].on);
}

#[test]
fn the_fill_key_lights_while_quick_fill_is_open_and_never_dims() {
    let shut = &with(None).keys["fill"];
    assert_eq!((shut.on, shut.dim, shut.tip), (false, false, None));
    assert_eq!(shut.paint, paint(KeyKind::Act, false));
    let open = with(Some(Panel::Fill));
    let k = &open.keys["fill"];
    assert_eq!((k.on, k.dim, k.tip), (true, false, None));
    assert_eq!(k.paint, paint(KeyKind::Act, true));
    assert!(!open.keys["clip"].on, "one panel at a time");
}
