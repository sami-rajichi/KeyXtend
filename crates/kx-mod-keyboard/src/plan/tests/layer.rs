//! The plans of the Shortcuts key, the Power key and the layer's own keys, which never show a shortcut id.

use super::*;
use crate::layer::LayerKey;
use kx_module_api::ports::ShortcutId;

#[test]
fn the_shortcuts_and_power_keys_ask_the_face() {
    let g = Given::new(false).latch(Latch::Ctrl, CTRL);
    assert_eq!(g.plan(Role::Layer, None), Plan::Layer);
    assert_eq!(g.plan(Role::Power, None), Plan::Power);
}

#[test]
fn a_layer_role_without_its_cell_does_nothing() {
    let g = Given::new(false);
    assert_eq!(g.plan(Role::Shortcut, None), Plan::Later, "no id to send");
    assert_eq!(
        g.plan(Role::Next, None),
        Plan::Later,
        "only the layer's cell turns"
    );
}

#[test]
fn a_layer_cell_sends_its_shortcut_or_turns_the_page() {
    let copy = ShortcutId::new("copy").unwrap();
    let cell = LayerKey::Shortcut(copy.clone());
    assert_eq!(layer_plan(&cell), Plan::Shortcut(copy));
    assert_eq!(layer_plan(&LayerKey::Next), Plan::Next);
}

#[test]
fn a_shortcut_plan_never_shows_its_id() {
    let id = ShortcutId::new("select-all").unwrap();
    let plans = [Plan::Shortcut(id), Plan::Next, Plan::Layer, Plan::Power];
    let shown = format!("{plans:?}");
    assert!(!shown.contains("select"), "{shown}");
    assert_eq!(shown, "[Shortcut, Next, Layer, Power]");
}
