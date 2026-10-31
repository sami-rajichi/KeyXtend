//! The clipboard key in the state: a plain side key, lit while the clipboard module last said its panel is open.

use super::*;

/// The state with the clipboard panel open or shut, the other modules off.
fn with(clip: bool) -> StateView {
    let layer = owners_layer();
    let others = Others {
        assist: AssistChanged::OFF,
        scroll: ScrollChanged::OFF,
        layer: &layer,
        power: PowerChanged::OFF,
        clip,
    };
    state_with(&none(), false, None, &others)
}

#[test]
fn the_clip_key_lights_while_the_panel_is_open_and_never_dims() {
    let shut = &with(false).keys["clip"];
    assert_eq!((shut.on, shut.dim, shut.tip), (false, false, None));
    assert_eq!(shut.paint, paint(KeyKind::Act, false));
    let open = with(true);
    let k = &open.keys["clip"];
    assert_eq!((k.on, k.dim, k.tip), (true, false, None));
    assert_eq!(k.paint, paint(KeyKind::Act, true));
    assert!(!open.keys["shortcuts"].on, "only its own key lights");
}
