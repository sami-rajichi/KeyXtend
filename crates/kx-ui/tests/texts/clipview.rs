//! The clipboard panel's views the Qt Quick tests draw (`qmltest/fixtures/clip.js`), as the hub writes them in each
//! language: a full page, a short one, one pinned row, an empty history, the Delete-all question with a note, and the
//! preview pages; and the click kinds QML sends to `Ui.clipAct`, by the name QML holds each under (ClipNames.qml).

use crate::qml::{HEAD, LANGS, fixture, hub, var};
use kx_module_api::ports::{
    ClipKind, ClipPage, ClipPanel, ClipRow, ClipView, Date, Filter, MB, Pages, PanelNote, Pixels,
    Preview, Room, When,
};
use kx_module_api::{Bus, Redacted};
use kx_test_support::host::FakeCore;
use kx_ui::clip_acts as acts;
use serde_json::{Map, Value, json};
use std::path::PathBuf;
use std::sync::Arc;

/// The fixture's file name.
pub const FILE: &str = "clip.js";
/// Bytes in a gigabyte.
const GB: u64 = MB << 10;
/// A thumbnail's side and colour in these views.
const THUMB: u32 = 4;
const RED: [u8; 4] = [220, 40, 40, 255];
/// How many lines of each long preview text, and how many paths the files preview holds.
const LINES: u32 = 60;
const PATHS: usize = 12;
/// A path in an Arabic folder, which takes the Arabic font.
const ARABIC_PATH: &str = r"C:\Users\Public\Documents\تقارير\تقرير-سنوي.pdf";

/// Each click kind by the name QML holds it under.
fn acts() -> Value {
    json!({
        "tick": acts::TICK, "filter": acts::FILTER, "page": acts::PAGE, "insert": acts::INSERT,
        "pin": acts::PIN, "delete": acts::DELETE, "deleteAll": acts::DELETE_ALL, "yes": acts::YES,
        "no": acts::NO, "preview": acts::PREVIEW, "back": acts::BACK, "close": acts::CLOSE,
    })
}

/// A row copied `days_ago` days back at 15:10, with no tick, pin, expiry or thumbnail.
fn row(id: u64, kind: ClipKind, title: &str, app: &str, days_ago: u8) -> ClipRow {
    ClipRow {
        id,
        kind,
        title: Redacted::new(title.to_owned()),
        rtl: false,
        arabic: false,
        app: Some(Redacted::new(app.to_owned())),
        when: When {
            days_ago: u32::from(days_ago),
            date: Date::new(2026, 10, 31 - days_ago).expect("a date"),
            hour: 15,
            minute: 10,
        },
        bytes: Some(96),
        tick: None,
        pinned: false,
        expires: false,
        thumb: None,
    }
}

/// The full page's five rows: a long text, an image, Arabic text, media that expires and a pinned link; three ticked.
fn rows() -> Vec<ClipRow> {
    let long = "Meeting moved to Thursday 10:00 in room B2, please bring the printed plan, the budget sheet and \
                the list of open questions from last week";
    let mut text = row(11, ClipKind::Text, long, "Outlook", 0);
    text.tick = Some(2);
    let mut image = row(
        12,
        ClipKind::Image,
        "Screenshot 2026-10-31 163012.png",
        "Snip",
        0,
    );
    image.thumb = Some(picture());
    image.bytes = Some(2_527_068);
    let mut arabic = row(
        13,
        ClipKind::Text,
        "السلام عليكم، أرسلت لك الملف الذي طلبته أمس",
        "WhatsApp",
        0,
    );
    (arabic.rtl, arabic.arabic, arabic.tick) = (true, true, Some(1));
    let mut media = row(
        14,
        ClipKind::Media,
        "demo-recording.mp4",
        "File Explorer",
        1,
    );
    (media.bytes, media.expires, media.tick) = (Some(52 * MB), true, Some(3));
    let mut link = row(
        15,
        ClipKind::Link,
        "https://example.com/keyxtend/notes",
        "Edge",
        4,
    );
    link.pinned = true;
    vec![text, image, arabic, media, link]
}

/// Page 1 of 3 of `rows`, the second highlighted, with 182 MB of 1 GB used and 412 GB free.
fn page(rows: Vec<ClipRow>) -> ClipPage {
    ClipPage {
        round: 7,
        ticked: u16::try_from(rows.iter().filter(|r| r.tick.is_some()).count()).expect("a count"),
        rows,
        highlight: Some(1),
        filter: Filter::All,
        search: Redacted::new(String::new()),
        pages: Pages { at: 0, count: 3 },
        unpin: false,
        preview: None,
        confirm: None,
        unpinned: 40,
        room: Some(Room {
            used: 182 * MB,
            free: 412 * GB,
            drive: "D:".to_owned(),
        }),
        cap: GB,
        note: None,
        kept_days: 7,
        empty: false,
    }
}

/// `LINES` lines of `line` with its number, then one long line that wraps.
fn long(line: &str) -> String {
    let lines = (1..=LINES).map(|n| format!("{n}. {line}"));
    let wraps = line.repeat(8);
    lines.chain([wraps]).collect::<Vec<_>>().join("\n")
}

/// A red thumbnail, `THUMB` pixels a side.
fn picture() -> Pixels {
    let side = usize::try_from(THUMB).expect("a side");
    Pixels::new(THUMB, THUMB, RED.repeat(side * side), THUMB).expect("pixels")
}

/// The first two rows, unticked, on their one page: what the preview pages stand on.
fn short() -> ClipPage {
    let two = rows()
        .into_iter()
        .take(2)
        .map(|r| ClipRow { tick: None, ..r });
    let mut p = page(two.collect());
    (p.ticked, p.pages, p.highlight, p.unpinned) = (0, Pages { at: 0, count: 1 }, Some(0), 2);
    p
}

/// The list views: a full page, a short one, one pinned row, an empty history and the Delete-all question.
fn lists() -> [(&'static str, ClipPage); 5] {
    let mut pinned = page(vec![ClipRow {
        tick: Some(1),
        ..rows().remove(4)
    }]);
    let one = Pages { at: 0, count: 1 };
    (
        pinned.unpin,
        pinned.unpinned,
        pinned.pages,
        pinned.highlight,
    ) = (true, 0, one, Some(0));
    let mut empty = page(Vec::new());
    (empty.empty, empty.highlight, empty.unpinned, empty.pages) = (true, None, 0, Pages::default());
    let mut confirm = page(rows());
    (confirm.confirm, confirm.note) = (Some(40), Some(PanelNote::PinnedFull));
    [
        ("list", page(rows())),
        ("short", short()),
        ("pinned", pinned),
        ("empty", empty),
        ("confirm", confirm),
    ]
}

/// The preview pages: long Latin and Arabic text, an image, one the worker could not draw, and many files, one Arabic.
fn previews() -> [(&'static str, Preview); 5] {
    let latin = long("The panel pages long text one box at a time, so nothing ever scrolls.");
    let arabic = long("تعرض اللوحة النص الطويل صفحةً بعد صفحة، فلا شيء يحتاج إلى التمرير.");
    let text = |t: String| Preview::Text {
        chars: 25_000,
        text: Redacted::new(t),
    };
    let paths = (1..=PATHS).map(|n| {
        PathBuf::from(format!(
            r"C:\Users\Public\Documents\Reports\report-{n:02}.pdf"
        ))
    });
    let paths = paths.chain([PathBuf::from(ARABIC_PATH)]);
    [
        ("text", text(latin)),
        ("arabic", text(arabic)),
        ("image", Preview::Image(Some(picture()))),
        ("drawing", Preview::Image(None)),
        ("files", Preview::Files(Redacted::new(paths.collect()))),
    ]
}

/// The views by name, each as the hub writes it.
fn pages() -> Vec<(&'static str, ClipPage)> {
    let shown = previews().map(|(name, p)| {
        let page = ClipPage {
            preview: Some(p),
            ..short()
        };
        (name, page)
    });
    lists().into_iter().chain(shown).collect()
}

/// Every view in language `tag`, by name.
fn views(tag: &str) -> Value {
    let (hub, _host) = hub(tag);
    let bus = Bus::new(Arc::new(FakeCore::default()));
    let _heard = hub.listen(&bus).expect("the hub listens");
    bus.publish(&ClipPanel { open: true })
        .expect("the panel opens");
    let shown = pages().into_iter().map(|(name, p)| {
        bus.publish(&ClipView::new(p)).expect("the view is sent");
        let v: Value = serde_json::from_str(&hub.clip_json()).expect("panel JSON");
        (name.to_owned(), v)
    });
    Value::Object(shown.collect::<Map<_, _>>())
}

#[test]
fn the_clip_fixture_matches_the_hub() {
    let all: Map<String, Value> = LANGS.iter().map(|t| ((*t).to_owned(), views(t))).collect();
    let text = format!(
        "{HEAD}{}{}",
        var("acts", &acts(), true),
        var("views", &Value::Object(all), true)
    );
    fixture(FILE.to_owned(), &text);
}
