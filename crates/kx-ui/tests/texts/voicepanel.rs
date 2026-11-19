//! The Voice panel's views the Qt Quick tests draw (`qmltest/fixtures/voicepanel.js`), as the hub writes them in each
//! language: every page, a second page of models with a note, the model in use ticked, a slow language, the folder
//! question, Compare's steps, History with two ticks, its question and empty, three pages of Options, Paste at once
//! chosen on the third, the drives, and the cloud pages.

use crate::qml::{HEAD, LANGS, fixture, hub, var};
use kx_module_api::ports::{
    BarHint, BarPlace, Ends, FolderList, Hundredths, OptionRow, OptionUnit, OptionValue, Pages,
    PanelButton, VoiceBar, VoiceLang, VoicePage, VoicePanel, VoiceSheet, VoiceView,
};
use kx_module_api::{Bus, Redacted};
use kx_test_support::host::FakeCore;
use serde_json::{Map, Value};
use std::sync::Arc;

mod cloud;
mod device;
mod long;
mod sheets;

/// The fixture's file name.
pub const FILE: &str = "voicepanel.js";

/// The language in front for a keyboard in `tag`, as Windows names it.
fn front(tag: &str) -> VoiceLang {
    let (name, rtl) = match tag {
        "fr" => ("français", false),
        "ar" => ("العربية", true),
        _ => ("English", false),
    };
    VoiceLang {
        name: name.to_owned(),
        iso: tag.to_owned(),
        rtl,
        arabic: rtl,
    }
}

/// An option `key` with `value`, live.
const fn opt(key: &'static str, value: OptionValue) -> OptionRow {
    OptionRow {
        key,
        value,
        live: true,
    }
}

/// A number `value` in `unit` with its ends.
const fn number(value: u32, unit: OptionUnit, low: bool, high: bool) -> OptionValue {
    OptionValue::Number {
        value,
        auto: None,
        unit,
        ends: Ends { low, high },
    }
}

/// Options' first page: its seconds stepper dimmed under its switch, and the bar's place at its first choice.
fn options_first() -> VoiceSheet {
    let mut under = opt(
        "silence_stop_s",
        number(3, OptionUnit::Seconds, false, false),
    );
    under.live = false;
    let place = OptionValue::Choice {
        name: "above",
        ends: Ends {
            low: true,
            high: false,
        },
    };
    let done = number(9000, OptionUnit::MsAsSeconds, false, false);
    VoiceSheet::Options(vec![
        opt("sounds", OptionValue::Switch(true)),
        opt("silence_stop", OptionValue::Switch(false)),
        under,
        opt("done_ms", done),
        opt("place", place),
    ])
}

/// Options' second page, Threads on Automatic and the texts kept at their most.
fn options_second() -> VoiceSheet {
    let threads = OptionValue::Number {
        value: 0,
        auto: Some(0),
        unit: OptionUnit::Count,
        ends: Ends {
            low: true,
            high: false,
        },
    };
    let gain = OptionValue::Factor {
        value: Hundredths(150),
        ends: Ends::default(),
    };
    VoiceSheet::Options(vec![
        opt("threads", threads),
        opt("type_suffix", OptionValue::Switch(true)),
        opt("card_gain", gain),
        opt("history_max", number(500, OptionUnit::Texts, false, true)),
        opt("card_diff", number(1, OptionUnit::Words, false, false)),
    ])
}

/// Options' third page: Paste at once chosen, its hint on the row below.
fn options_insert() -> VoiceSheet {
    let pair = OptionValue::Pair {
        names: ["type", "paste"],
        second: true,
    };
    VoiceSheet::Options(vec![
        opt("debounce_ms", number(400, OptionUnit::Ms, false, false)),
        opt("insert", pair),
        opt("insert", OptionValue::Hint),
        opt("type_suffix", OptionValue::Switch(true)),
        opt("type_gap_ms", number(5, OptionUnit::Ms, false, false)),
    ])
}

/// The drives, to pick the models folder.
fn drives() -> VoiceSheet {
    let rows = [r"C:\", r"D:\"].map(|d| Redacted::new(d.to_owned()));
    VoiceSheet::Folders(FolderList {
        folder: None,
        rows: rows.into(),
    })
}

/// A view's panel: its sheet, page `at` of `count`, `ticked` rows, the buttons that act and a note.
struct Shown {
    sheet: VoiceSheet,
    pages: (u16, u16),
    ticked: u16,
    acts: Vec<PanelButton>,
    note: Option<BarHint>,
}

/// `sheet` on one page, nothing ticked, with `acts` acting.
fn shown(sheet: VoiceSheet, acts: &[PanelButton]) -> Shown {
    Shown {
        sheet,
        pages: (0, 1),
        ticked: 0,
        acts: acts.to_vec(),
        note: None,
    }
}

/// `s` on page `at` of `count`, with `ticked` rows.
fn paged(mut s: Shown, at: u16, count: u16, ticked: u16) -> Shown {
    (s.pages, s.ticked) = ((at, count), ticked);
    s
}

/// The Models pages the tests draw, by name.
fn model_panels() -> Vec<(&'static str, Shown)> {
    use PanelButton::{Change, Delete, Open, Use};
    let mut more = paged(shown(sheets::models(1, false), &[Change, Open]), 1, 2, 0);
    more.note = Some(BarHint::Net);
    let first = shown(sheets::models(0, false), &[Change, Open, Use, Delete]);
    vec![
        ("models", paged(first, 0, 2, 1)),
        ("modelsMore", more),
        (
            "modelsUsed",
            paged(shown(sheets::models(0, true), &[Change, Open]), 0, 2, 1),
        ),
        ("modelsSlow", shown(sheets::slow(), &[Change, Open])),
        ("modelsOffer", shown(sheets::offer(), &[])),
    ]
}

/// Every other page the tests draw, by name.
fn other_panels() -> Vec<(&'static str, Shown)> {
    use PanelButton::{Again, Cancel, Check, Copy, Delete, DeleteAll, Insert, Pin, Rerun, Use};
    let lists = [Insert, Copy, Pin, Delete, DeleteAll];
    let results = |free| paged(shown(sheets::results(free), &[Rerun, Use]), 0, 1, 1);
    let history = |ask, empty, acts: &[PanelButton], pages, ticked| {
        paged(shown(sheets::history(ask, empty), acts), 0, pages, ticked)
    };
    vec![
        ("home", shown(sheets::home(), &[Again])),
        (
            "device",
            paged(shown(device::device(), &[Check, Use]), 0, 1, 1),
        ),
        ("compare", shown(sheets::ready(), &[])),
        ("comparing", shown(sheets::running(), &[Cancel])),
        ("results", results(false)),
        ("freeResults", results(true)),
        ("history", history(false, false, &lists, 3, 2)),
        ("historyAsk", history(true, false, &[], 3, 0)),
        ("historyEmpty", history(false, true, &[], 1, 0)),
        ("options", paged(shown(options_first(), &[]), 0, 5, 0)),
        ("optionsMore", paged(shown(options_second(), &[]), 1, 5, 0)),
        (
            "optionsInsert",
            paged(shown(options_insert(), &[]), 2, 5, 0),
        ),
        ("folders", shown(drives(), &[])),
    ]
}

/// Every panel the tests draw in language `tag`, by name.
fn panels(tag: &str) -> Vec<(String, Shown)> {
    let fixed = model_panels()
        .into_iter()
        .chain(other_panels())
        .chain(cloud::panels())
        .map(|(name, s)| (name.to_owned(), s));
    fixed.chain(long::panels(tag)).collect()
}

/// Every view in language `tag`, by name, each its own round from 1, the bar hidden.
fn views(tag: &str) -> Value {
    let (hub, _host) = hub(tag);
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = hub.listen(&bus).expect("the hub listens");
    let shown = (1..).zip(panels(tag)).map(|(round, (name, s))| {
        let panel = VoicePanel {
            sheet: s.sheet,
            lang: front(tag),
            pages: Pages {
                at: s.pages.0,
                count: s.pages.1,
            },
            ticked: s.ticked,
            acts: s.acts,
            note: s.note,
        };
        let page = VoicePage {
            round,
            bar: VoiceBar::Hidden,
            panel: Some(panel),
            place: BarPlace::Above,
            sounds: true,
            cloud: None,
        };
        bus.publish(&VoiceView::new(page))
            .expect("the view is sent");
        let v: Value = serde_json::from_str(&hub.voice_json()).expect("voice JSON");
        (name, v)
    });
    Value::Object(shown.collect::<Map<_, _>>())
}

#[test]
fn the_voice_panel_fixture_matches_the_hub() {
    let all: Map<String, Value> = LANGS.iter().map(|t| ((*t).to_owned(), views(t))).collect();
    let text = format!("{HEAD}{}", var("views", &Value::Object(all), true));
    fixture(FILE.to_owned(), &text);
}
