//! The shortcuts layer in the views: its keys take the side cells with their names, icons and steps, the pad hides,
//! the Shortcuts key lights or dims, and the state carries the page.

use super::*;
use crate::geom::side_area;
use crate::spec::NEXT_ID;
use kx_module_api::ports::{ShortcutInfo, Step};

/// Key name `n`.
fn name(n: &str) -> KeyName {
    KeyName::new(n).unwrap()
}

/// Every id of the owner's pages, each sent as its own key, but Copy all, which is Ctrl+A, then Ctrl+C.
fn known() -> Vec<ShortcutInfo> {
    let ctrl = |k: &str| Step {
        held: vec![name("ctrl")],
        key: name(k),
    };
    let info = |id: &ShortcutId| {
        let steps = match id.as_str() {
            "copy-all" => vec![ctrl("a"), ctrl("c")],
            other => vec![Step {
                held: Vec::new(),
                key: name(other),
            }],
        };
        ShortcutInfo {
            id: id.clone(),
            steps,
        }
    };
    settings()
        .shortcut_pages
        .iter()
        .flatten()
        .map(info)
        .collect()
}

/// The owner's layer, shown on its first page.
fn shown() -> Layer {
    let mut l = owners_layer();
    l.toggle();
    l
}

/// The board at `size` with layer `l`, which knows `known`.
fn layered(l: &Layer, known: &[ShortcutInfo], size: f64) -> BoardView {
    let mut v = board_view(&spec(), &settings(), size);
    with_layer(&mut v, &spec(), l, known);
    v
}

/// The state with layer `l`, the other modules off.
fn with(l: &Layer) -> StateView {
    let others = Others {
        assist: AssistChanged::OFF,
        scroll: ScrollChanged::OFF,
        layer: l,
        power: PowerChanged::OFF,
        clip: false,
    };
    state_with(&none(), false, None, &others)
}

#[test]
fn a_hidden_layer_changes_nothing() {
    let plain = board_view(&spec(), &settings(), 1.0);
    assert_eq!(layered(&owners_layer(), &known(), 1.0), plain);
}

#[test]
fn the_layer_takes_every_side_cell_but_the_shortcuts_keys_and_hides_the_pad() {
    let l = shown();
    let v = layered(&l, &known(), 1.25);
    assert!(v.layer, "the pad hides");
    let side: Vec<&KeyView> = v.keys.iter().filter(|k| k.side).collect();
    let cells = l.cells();
    assert_eq!(side[0].id, "shortcuts", "it stays in its place");
    assert_eq!(side.len(), cells.len() + 1);
    for (k, c) in side[1..].iter().zip(&cells) {
        assert_eq!(k.id, c.key.id());
        assert_eq!(k.cell, side_area(&spec(), c.cell, 1.25), "{}", k.id);
        assert_eq!(k.row, usize::from(c.cell.row - 1), "{}", k.id);
        assert_eq!(k.kind, KeyKind::Act);
        assert!(
            !k.checkable && !k.dim && k.tip.is_none() && !k.repeat,
            "{}",
            k.id
        );
    }
    let plain = board_view(&spec(), &settings(), 1.25).keys;
    let main = |keys: &[KeyView]| keys.iter().filter(|k| !k.side).count();
    assert_eq!(main(&v.keys), main(&plain), "the main block stays");
}

#[test]
fn a_shortcut_key_shows_its_icon_alone_and_names_its_steps() {
    let v = layered(&shown(), &known(), 1.0);
    let k = key(&v, "sc-copy-all");
    assert_eq!(k.icon.as_deref(), Some("shortcuts.copy-all"));
    assert_eq!(k.label.as_deref(), Some("shortcut.copy-all"));
    assert_eq!(k.name, k.label);
    assert!(k.icon_only);
    let want = [[name("ctrl"), name("a")], [name("ctrl"), name("c")]];
    assert_eq!(k.steps, want);
    let alone = key(&v, "sc-copy");
    assert_eq!(alone.steps, [[name("copy")]], "the tapped key last");
}

#[test]
fn the_next_key_has_its_icon_name_and_page_dots_and_no_steps() {
    let v = layered(&shown(), &known(), 1.0);
    let next = key(&v, NEXT_ID);
    assert_eq!(next.icon.as_deref(), Some(NEXT_ICON));
    assert_eq!(next.name.as_deref(), Some("key.layer-next"));
    assert!(next.label.is_none() && !next.icon_only && next.steps.is_empty());
    let dotted: Vec<&str> = v
        .keys
        .iter()
        .filter(|k| k.dots)
        .map(|k| k.id.as_str())
        .collect();
    assert_eq!(dotted, [NEXT_ID], "only Next shows the page dots");
}

#[test]
fn a_shortcut_with_no_known_steps_names_none() {
    let v = layered(&shown(), &[], 1.0);
    assert!(key(&v, "sc-copy").steps.is_empty());
}

#[test]
fn the_shortcuts_key_lights_while_the_layer_shows_and_the_state_carries_the_page() {
    let s = with(&shown());
    let k = &s.keys["shortcuts"];
    assert_eq!((k.on, k.dim, k.tip), (true, false, None));
    assert_eq!(k.paint, paint(KeyKind::Act, true));
    let page = LayerState {
        shown: true,
        page: 0,
        pages: 3,
    };
    assert_eq!(s.layer, page);
    assert_eq!(s.keys["sc-copy"].paint, paint(KeyKind::Act, false));
    assert!(s.keys.contains_key(NEXT_ID));
    let hidden = with(&owners_layer());
    assert!(!hidden.keys["shortcuts"].on && !hidden.layer.shown);
    assert!(!hidden.keys.keys().any(|id| id.starts_with("sc-")));
}

#[test]
fn with_nothing_to_show_the_shortcuts_key_is_dim_and_says_so() {
    let empty = Layer::new(spec().layer_grid(), &[], &HashSet::new());
    let s = with(&empty);
    let k = &s.keys["shortcuts"];
    assert_eq!((k.dim, k.paint, k.tip), (true, DIM, Some(NO_SHORTCUTS_TIP)));
    assert_eq!(s.layer, LayerState::default());
}
