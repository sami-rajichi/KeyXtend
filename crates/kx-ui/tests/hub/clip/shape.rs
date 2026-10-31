//! The panel's JSON holds only what QML draws, so no text is worded twice or carried unread.

use super::{clip, heard, page, row, shown};
use crate::words::speaking;
use kx_module_api::ports::Preview;
use serde_json::Value;

/// The keys of object `v`, sorted.
fn keys(v: &Value) -> Vec<String> {
    let mut all: Vec<String> = v
        .as_object()
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default();
    all.sort_unstable();
    all
}

#[test]
fn rows_the_pager_the_buttons_and_the_question_hold_only_what_qml_reads() {
    let (rig, _host) = speaking("en", 5);
    let (bus, acts, _subs) = heard(&rig);
    let mut p = page(1, vec![row(7, "a")]);
    p.confirm = Some(1);
    shown(&rig, &bus, &acts, p);
    let v = clip(&rig);
    let row = [
        "arabic", "expires", "id", "kind", "meta", "name", "pinned", "rtl", "thumb", "tick",
        "title",
    ];
    assert_eq!(
        keys(&v["rows"][0]),
        row,
        "the meta line alone carries app, time and size"
    );
    assert_eq!(
        keys(&v["page"]),
        ["at", "count"],
        "the pager words its own number"
    );
    assert_eq!(keys(&v["confirm"]), ["text"]);
    let words = [
        "back",
        "close",
        "delete",
        "deleteAll",
        "expires",
        "insert",
        "no",
        "openFolder",
        "pin",
        "preview",
        "window",
        "yes",
    ];
    assert_eq!(keys(&v["words"]), words, "the pager names its own buttons");
}

#[test]
fn readers_hear_an_image_preview_by_its_kind_in_every_language() {
    for (iso, want) in [("en", "Image"), ("fr", "Image"), ("ar", "صورة")] {
        let (rig, _host) = speaking(iso, 5);
        let (bus, acts, _subs) = heard(&rig);
        let mut p = page(1, vec![row(7, "a")]);
        p.preview = Some(Preview::Image(None));
        shown(&rig, &bus, &acts, p);
        assert_eq!(clip(&rig)["preview"]["name"], want, "{iso}");
    }
}
