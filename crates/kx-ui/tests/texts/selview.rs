//! The selection strip's views the Qt Quick tests draw (`qmltest/fixtures/sel.js`), as the hub writes them in each
//! language: every keycap with both legends, icons only, words only, and Copy ticked.

use crate::qml::{HEAD, LANGS, fixture, hub, var};
use kx_module_api::Bus;
use kx_module_api::ports::{BASE_DPI, ChipsView, Legend, Rect, Screen, SelChip, StripSpot};
use kx_test_support::host::FakeCore;
use serde_json::{Map, Value};
use std::sync::Arc;

/// The fixture's file name.
pub const FILE: &str = "sel.js";
/// The selected line every view sits by.
const LINE: Rect = Rect {
    x: 100,
    y: 300,
    w: 200,
    h: 20,
};
/// Its screen's work area, at 100 %.
const AREA: Rect = Rect {
    x: 0,
    y: 0,
    w: 1920,
    h: 1040,
};

/// Each view by name: the keycap ticked, if any, and the legend.
const KINDS: [(&str, Option<SelChip>, Legend); 4] = [
    ("both", None, Legend::Both),
    ("icons", None, Legend::Icons),
    ("words", None, Legend::Words),
    ("ticked", Some(SelChip::Copy), Legend::Both),
];

/// Every view in language `tag`, by name, each its own round from 1.
fn views(tag: &str) -> Value {
    let (hub, _host) = hub(tag);
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = hub.listen(&bus).expect("the hub listens");
    let screen = Screen {
        rect: AREA,
        dpi: BASE_DPI,
        primary: true,
    };
    let spot = StripSpot {
        first: LINE,
        last: LINE,
        screen,
    };
    let shown = (1..).zip(KINDS).map(|(round, (name, acted, legend))| {
        let chips = SelChip::ALL.to_vec();
        let view = ChipsView::Shown {
            round,
            chips,
            spot,
            acted,
            legend,
        };
        bus.publish(&view).expect("the view is sent");
        let v: Value = serde_json::from_str(&hub.sel_json()).expect("strip JSON");
        (name.to_owned(), v)
    });
    Value::Object(shown.collect::<Map<_, _>>())
}

#[test]
fn the_sel_fixture_matches_the_hub() {
    let all: Map<String, Value> = LANGS.iter().map(|t| ((*t).to_owned(), views(t))).collect();
    let text = format!("{HEAD}{}", var("views", &Value::Object(all), true));
    fixture(FILE.to_owned(), &text);
}
