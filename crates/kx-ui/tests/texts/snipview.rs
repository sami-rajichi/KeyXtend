//! Snip's views the Qt Quick tests draw (`qmltest/fixtures/snip.js`), as the hub writes them: picking each way, a
//! corner clicked, dimmed ways, the strips, the countdown, the record bar, a note, a region's frame (`snipframes`), and
//! the editor's, question's and panel's pages (`snipedits`); Snip's texts in each language; and the hub's click kinds,
//! capture windows and ids, which `SnipNames.qml` must hold.

use crate::files::keys_of;
use crate::qml::{HEAD, LANGS, fixture, hub, var};
use kx_module_api::Bus;
use kx_module_api::ports::{
    BASE_DPI, BGRX_BYTES, BarKey, CloseAsk, Countdown, Dim, FrozenScreen, Keep, Keycap, Part,
    Picking, Point, RecKey, Recording, Rect, Shot, SnipNote, SnipPage, SnipSheet, SnipSpot,
    SnipStage, SnipView, Speed, StripKey, Taken,
};
use kx_test_support::host::FakeCore;
use serde_json::{Map, Value};
use std::sync::Arc;

/// The fixture's file name.
pub const FILE: &str = "snip.js";
/// The one screen, inside the offscreen platform's (`qmltest/offscreen.json`), where every part sits.
const SCREEN: Rect = rect(0, 0, 800, 600);
/// The part every strip, countdown and recording sits by.
const PART: Rect = rect(100, 150, 300, 200);
/// A picture's strip keys.
const PICTURE: [StripKey; 4] = [
    StripKey::Copy,
    StripKey::Edit,
    StripKey::Save,
    StripKey::Close,
];
/// A recorded video's strip keys (ADR-0060).
const VIDEO: [StripKey; 5] = [
    StripKey::Play,
    StripKey::Speed,
    StripKey::Copy,
    StripKey::Save,
    StripKey::Close,
];
/// The first corner a region's pick clicked.
const CORNER: Point = Point { x: 120, y: 90 };
/// The windows Window offers.
const SHOWN: [Rect; 2] = [rect(40, 60, 320, 220), rect(420, 80, 300, 240)];

/// A page by name: its stage, whether the screen is frozen, its note, question and panel, and whether a dialog is out.
pub struct Page {
    pub name: &'static str,
    pub stage: SnipStage,
    pub still: bool,
    pub note: Option<SnipNote>,
    pub ask: Option<CloseAsk>,
    pub panel: Option<SnipSheet>,
    pub dialog: bool,
}

/// The box at (`x`, `y`), `w` by `h`.
const fn rect(x: i32, y: i32, w: i32, h: i32) -> Rect {
    Rect { x, y, w, h }
}

/// Page `name` showing `stage` over the frozen screen, with no note or dialog.
pub fn still(name: &'static str, stage: SnipStage) -> Page {
    Page {
        still: true,
        ..live(name, stage, None)
    }
}

/// Page `name` showing `stage` over the live screen, with `note` and no question, panel or dialog.
pub fn live(name: &'static str, stage: SnipStage, note: Option<SnipNote>) -> Page {
    Page {
        name,
        stage,
        still: false,
        note,
        ask: None,
        panel: None,
        dialog: false,
    }
}

/// The keys `all` in order, `on` lit, each with its reason from `dims`.
fn caps<K: Copy + PartialEq>(all: &[K], on: &[K], dims: &[(K, Dim)]) -> Vec<Keycap<K>> {
    let dim = |k: K| dims.iter().find(|(d, _)| *d == k).map(|(_, why)| *why);
    let cap = |&key: &K| Keycap {
        key,
        on: on.contains(&key),
        dim: dim(key),
    };
    all.iter().map(cap).collect()
}

/// The screen frozen, one pixel colour all over.
fn frozen() -> Vec<FrozenScreen> {
    let (w, h) = (SCREEN.w.unsigned_abs(), SCREEN.h.unsigned_abs());
    let bytes = usize::try_from(w * h).expect("a size") * BGRX_BYTES;
    let shot = Shot::new(w, h, vec![0x40; bytes]).expect("a shot");
    vec![FrozenScreen::new(SCREEN, BASE_DPI, shot).expect("a frozen screen")]
}

/// Picking with `way` lit, the `corner` clicked if any, the windows Window offers, and the bar's `dims`.
fn pick(way: BarKey, corner: Option<Point>, windows: &[Rect], dims: &[(BarKey, Dim)]) -> SnipStage {
    let keys = caps(&BarKey::ALL, &[way], dims);
    SnipStage::Pick(Picking {
        screen: SCREEN,
        keys,
        corner,
        within: (way == BarKey::VideoRegion && corner.is_some()).then_some(SCREEN),
        windows: windows.to_vec(),
    })
}

/// A strip under the part with `keys`, `on` lit and `dims`, nothing done.
fn strip(keys: &[StripKey], on: &[StripKey], dims: &[(StripKey, Dim)]) -> Taken {
    let (part, keys) = (Part::new(PART).expect("a part"), caps(keys, on, dims));
    Taken {
        part,
        screen: SCREEN,
        spot: SnipSpot::Under,
        keys,
        length_ms: None,
        speed: None,
        done: None,
        waits: None,
        frame: None,
    }
}

/// A recorded video's strip at `times` its speed, `length_ms` long at that speed.
pub fn recorded(times: u8, length_ms: u32) -> SnipStage {
    SnipStage::Strip(Taken {
        length_ms: Some(length_ms),
        speed: Some(Speed::new(times).expect("a speed")),
        ..strip(&VIDEO, &[], &[])
    })
}

/// Recording the part at `spot`, live for `elapsed_ms`, the mic on and the computer's sound off, with the `dims`.
fn recording(part: Rect, spot: SnipSpot, elapsed_ms: u32, dims: &[(RecKey, Dim)]) -> SnipStage {
    let (part, keys) = (
        Part::new(part).expect("a part"),
        caps(&RecKey::ALL, &[RecKey::Mic], dims),
    );
    SnipStage::Record(Recording {
        part,
        screen: SCREEN,
        spot,
        elapsed_ms,
        live: true,
        keys,
    })
}

/// Picking each way, the second corner, and the video ways dimmed.
fn picking() -> Vec<Page> {
    let dims = |why| [(BarKey::VideoRegion, why), (BarKey::VideoFull, why)];
    vec![
        still("region", pick(BarKey::Region, None, &[], &[])),
        still("corner", pick(BarKey::Region, Some(CORNER), &[], &[])),
        still(
            "video-corner",
            pick(BarKey::VideoRegion, Some(CORNER), &[], &[]),
        ),
        still("window", pick(BarKey::Window, None, &SHOWN, &[])),
        still("full", pick(BarKey::Full, None, &[], &[])),
        still(
            "no-video",
            pick(BarKey::Region, None, &[], &dims(Dim::NoVideo)),
        ),
        still("bare", pick(BarKey::Region, None, &[], &dims(Dim::Bare))),
    ]
}

/// A picture's strip under the part.
pub fn taken() -> SnipStage {
    SnipStage::Strip(strip(&PICTURE, &[], &[]))
}

/// The strips: a picture's, copied, under a dialog, before a video, with its keys refused, and a recorded video's at
/// 1× and 2× and with its player refused.
fn strips() -> Vec<Page> {
    use StripKey::{Close, Mic, Record, Sound};
    let (picture, video) = (PICTURE, [Record, Mic, Sound, Close]);
    let refused = [
        (Record, Dim::DiskLow),
        (Mic, Dim::MicRefused),
        (Sound, Dim::NoSound),
    ];
    let shown = |t: Taken| SnipStage::Strip(t);
    let copied = Taken {
        done: Some(Keep::Copy),
        ..strip(&picture, &[], &[])
    };
    let refused_play = Some(SnipNote::PlayFailed);
    vec![
        still("strip", taken()),
        still("copied", shown(copied)),
        Page {
            dialog: true,
            ..still("low", taken())
        },
        still(
            "ready",
            shown(strip(&video, &[Sound], &[(Mic, Dim::NoMic)])),
        ),
        Page {
            note: Some(SnipNote::DiskLow),
            ..still("refused", shown(strip(&video, &[], &refused)))
        },
        live("length", recorded(1, 75_000), None),
        live("sped", recorded(2, 37_500), None),
        live("play-failed", recorded(1, 75_000), refused_play),
    ]
}

/// The countdown, recording above the part, with its sounds refused, and at the top of a whole screen, a note, and
/// idle.
fn videos() -> Vec<Page> {
    let count = SnipStage::Count(Countdown {
        part: Part::new(PART).expect("a part"),
        left_s: 3,
    });
    let refused = [
        (RecKey::Mic, Dim::MicRefused),
        (RecKey::Sound, Dim::NoSound),
    ];
    vec![
        live("count", count, None),
        live(
            "record",
            recording(PART, SnipSpot::Above, 65_000, &[]),
            None,
        ),
        live(
            "rec-refused",
            recording(PART, SnipSpot::Above, 1_000, &refused),
            None,
        ),
        live(
            "top",
            recording(SCREEN, SnipSpot::Top, 1_000, &[]),
            Some(SnipNote::Bare),
        ),
        live("note", SnipStage::Idle, Some(SnipNote::TooBig)),
        live("idle", SnipStage::Idle, None),
    ]
}

/// Every page as the English hub writes it, by name, each its own round from 1.
fn views() -> Value {
    let (hub, _host) = hub("en");
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = hub.listen(&bus).expect("the hub listens");
    let pages = picking()
        .into_iter()
        .chain(strips())
        .chain(videos())
        .chain(crate::snipedits::pages())
        .chain(crate::snipframes::pages());
    let each = (1..).zip(pages).map(|(round, p)| {
        let frozen = if p.still { frozen() } else { Vec::new() };
        let page = SnipPage {
            round,
            frozen,
            stage: p.stage,
            ask: p.ask,
            note: p.note,
            panel: p.panel,
            dialog: p.dialog,
        };
        bus.publish(&SnipView::new(page)).expect("the view is sent");
        hub.snip_woken();
        let v: Value = serde_json::from_str(&hub.snip_json()).expect("snip JSON");
        (p.name.to_owned(), v)
    });
    Value::Object(each.collect::<Map<_, _>>())
}

/// Snip's own texts in language `tag`, by key, from its files in `snip/`.
fn texts(tag: &str) -> Value {
    let t = crate::lang(tag);
    let files = t.lang().extra.iter().filter(|f| f.contains("\nsnip ="));
    let ours = files
        .flat_map(|f| keys_of(f))
        .map(|k| (k.clone(), Value::String(t.text(&k))));
    Value::Object(ours.collect())
}

#[test]
fn the_snip_fixture_matches_the_hub() {
    let all: Map<String, Value> = LANGS.iter().map(|t| ((*t).to_owned(), texts(t))).collect();
    let parts = [
        var("texts", &Value::Object(all), true),
        var("views", &views(), true),
        var("acts", &crate::snipedits::kinds(), true),
        var("capture", &crate::snipedits::windows(), true),
        var("ids", &crate::snipedits::ids(), true),
    ];
    fixture(FILE.to_owned(), &format!("{HEAD}{}", parts.concat()));
}
