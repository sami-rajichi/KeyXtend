//! The panel as QML reads it: every text worded in the keyboard's language, sizes with its decimal mark, Arabic's
//! plural forms, the filters in click order, pictures as URLs of the kept round, the preview's lines, and no title,
//! search or app in a log line.

use super::{clip, heard, page, pixels, row, shown};
use crate::logs::lines_of;
use crate::words::speaking;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    ClipKind, ClipPage, ClipPanel, ClipView, Date, Filter, PanelNote, Preview, Room, When,
};
use serde_json::{Value, json};
use std::path::PathBuf;

/// Bytes in a gigabyte.
const GB: u64 = 1 << 30;
/// 2.41 MB.
const SIZE: u64 = 2_527_068;

/// The panel showing `page` in a keyboard of language `iso`.
fn read(iso: &str, page: ClipPage) -> Value {
    let (rig, _host) = speaking(iso, 5);
    let (bus, acts, _subs) = heard(&rig);
    shown(&rig, &bus, &acts, page);
    clip(&rig)
}

/// A 2.41 MB row and a 12-byte row of no known app, copied yesterday, on 1.5 GB of a 2 GB cap with 412 GB free.
fn sized() -> ClipPage {
    let mut big = row(7, "rapport");
    big.bytes = Some(SIZE);
    let mut small = row(8, "mot");
    small.app = None;
    small.when.days_ago = 1;
    let mut p = page(3, vec![big, small]);
    p.room = Room {
        used: GB / 2 * 3,
        free: 412 * GB,
        drive: "D:".to_owned(),
    };
    p.cap = 2 * GB;
    p.ticked = 3;
    p
}

/// When a row copied `days_ago` days back on `date` at 14:12 was copied.
fn when(days_ago: u32, date: Date) -> When {
    When {
        days_ago,
        date,
        hour: 14,
        minute: 12,
    }
}

#[test]
fn french_reads_sizes_in_mo_and_go_with_its_comma() {
    let v = read("fr", sized());
    let rows = &v["rows"];
    let (mb, small) = ("2,41\u{a0}Mo", "<\u{a0}0,01\u{a0}Mo");
    assert_eq!(rows[0]["size"], mb);
    assert_eq!(rows[1]["size"], small);
    let today = format!("Notepad · Aujourd’hui 16:32 · {mb}");
    assert_eq!(rows[0]["meta"], today);
    let yesterday = format!("Hier 16:32 · {small}");
    assert_eq!(rows[1]["meta"], yesterday, "no app known");
    assert_eq!(rows[1]["app"], Value::Null);
    let room = "Historique\u{a0}: 1,50\u{a0}Go sur 2,00\u{a0}Go · 412,00\u{a0}Go libres sur D:";
    assert_eq!(v["room"], room);
    assert_eq!(v["words"]["insert"], "Insérer (3)");
    assert_eq!(v["kept"], "Conservés 7 jours");
}

#[test]
fn arabic_reads_sizes_in_its_units_and_the_question_in_every_plural_form() {
    let v = read("ar", sized());
    assert_eq!(v["rows"][0]["size"], "2,41 ميغابايت");
    assert_eq!(v["rows"][1]["size"], "أقل من 0,01 ميغابايت");
    let room = "المحفوظات: 1,50 غيغابايت من 2,00 غيغابايت · 412,00 غيغابايت متوفرة على D:";
    assert_eq!(v["room"], room);
    let stay = "تبقى العناصر المثبّتة.";
    let forms = [
        (0, "لا توجد عناصر غير مثبّتة لحذفها.".to_owned()),
        (1, format!("هل تريد حذف عنصر واحد؟ {stay}")),
        (2, format!("هل تريد حذف عنصرين؟ {stay}")),
        (3, format!("هل تريد حذف 3 عناصر؟ {stay}")),
        (11, format!("هل تريد حذف 11 عنصرًا؟ {stay}")),
        (100, format!("هل تريد حذف 100 عنصر؟ {stay}")),
        (103, format!("هل تريد حذف 103 عناصر؟ {stay}")),
    ];
    for (n, want) in forms {
        let mut p = page(1, vec![row(7, "a")]);
        p.confirm = Some(n);
        let v = read("ar", p);
        assert_eq!(v["confirm"], json!({"text": want, "count": n}), "{n}");
    }
}

#[test]
fn a_rows_day_reads_today_yesterday_a_weekday_then_the_date() {
    let oct = |day| Date::new(2026, 10, day).unwrap();
    let days = [
        (when(0, oct(31)), "Today 14:12"),
        (when(1, oct(30)), "Yesterday 14:12"),
        (when(4, oct(27)), "Tue 14:12"),
        (when(6, oct(25)), "Sun 14:12"),
        (when(7, oct(24)), "24/10/2026"),
        (when(305, Date::new(2025, 12, 30).unwrap()), "30/12/2025"),
    ];
    let rows = (0..).zip(&days).map(|(id, (w, _))| {
        let mut r = row(id, "a");
        r.when = *w;
        r
    });
    let v = read("en", page(1, rows.collect()));
    for (at, (_, want)) in days.iter().enumerate() {
        assert_eq!(v["rows"][at]["when"], *want);
    }
}

#[test]
fn a_row_with_no_title_shows_its_kind_and_readers_hear_its_tick_and_pin() {
    let mut r = row(7, "\u{202e}");
    r.kind = ClipKind::Image;
    r.tick = Some(2);
    r.pinned = true;
    let rows = vec![r, row(u64::MAX, "plain")];
    let v = read("en", page(1, rows));
    let shown = &v["rows"][0];
    assert_eq!(shown["title"], "Image");
    assert_eq!(shown["kind"], "image");
    assert_eq!(shown["name"], "Image, Ticked 2, Pinned");
    assert_eq!(shown["tick"], json!(2));
    assert_eq!(shown["pinned"], json!(true));
    assert_eq!(v["rows"][1]["name"], "plain");
    assert_eq!(
        v["rows"][1]["id"], "18446744073709551615",
        "ids stay exact in JS"
    );
}

#[test]
fn the_page_filters_search_and_footer_are_worded() {
    let mut p = page(1, vec![row(7, "a")]);
    p.filter = Filter::Images;
    p.search = Redacted::new("\u{2068}مرحبا\u{2069}".to_owned());
    p.pages.at = 1;
    p.pages.count = 4;
    p.unpin = true;
    p.unpinned = 0;
    let v = read("en", p);
    let ids: Vec<&Value> = v["filters"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| &f["id"])
        .collect();
    assert_eq!(
        ids,
        ["all", "text", "images", "files", "media"],
        "click order"
    );
    assert_eq!(
        v["filters"][2],
        json!({"id": "images", "label": "Images", "on": true})
    );
    assert_eq!(
        v["search"],
        json!({"text": "مرحبا", "hint": "Type to search", "arabic": true, "rtl": true})
    );
    assert_eq!(v["page"], json!({"text": "2 of 4", "at": 1, "count": 4}));
    assert_eq!(v["words"]["pin"], "Unpin");
    assert_eq!(v["anyUnpinned"], false, "Delete all is off");
    assert_eq!(v["title"], "Clipboard history");
    assert_eq!(
        (v["note"].clone(), v["empty"].clone()),
        (Value::Null, Value::Null)
    );
}

#[test]
fn a_note_and_an_empty_history_are_worded() {
    let mut p = page(1, Vec::new());
    p.note = Some(PanelNote::Disk);
    p.empty = true;
    let v = read("en", p);
    assert_eq!(
        v["note"],
        "The drive is nearly full, so new copies are not kept."
    );
    assert!(
        v["empty"]
            .as_str()
            .unwrap()
            .starts_with("Nothing copied yet.")
    );
    assert_eq!(v["anyUnpinned"], false);
}

#[test]
fn pictures_are_urls_of_the_kept_round() {
    let mut thumbed = row(7, "a");
    thumbed.thumb = Some(pixels(2, [1, 2, 3, 4]));
    let mut p = page(5, vec![thumbed, row(8, "b")]);
    p.preview = Some(Preview::Image(Some(pixels(2, [1, 2, 3, 4]))));
    let v = read("en", p);
    assert_eq!(v["rows"][0]["thumb"], "image://clip/5/7");
    assert_eq!(v["rows"][1]["thumb"], Value::Null);
    assert_eq!(
        v["preview"],
        json!({"kind": "image", "url": "image://clip/5/preview"})
    );
    let mut p = page(6, vec![row(7, "a")]);
    p.preview = Some(Preview::Image(None));
    assert_eq!(
        read("en", p)["preview"],
        json!({"kind": "image", "url": null})
    );
}

#[test]
fn on_the_preview_page_insert_is_plain_whatever_is_ticked() {
    for (tag, ticked, want) in [
        ("en", 3, "Insert"),
        ("en", 0, "Insert"),
        ("fr", 3, "Insérer"),
    ] {
        let mut p = page(1, vec![row(7, "a")]);
        p.ticked = ticked;
        p.preview = Some(Preview::Image(None));
        assert_eq!(read(tag, p)["words"]["insert"], want, "{tag} {ticked}");
    }
    let mut list = page(1, vec![row(7, "a")]);
    list.ticked = 3;
    assert_eq!(
        read("en", list)["words"]["insert"],
        "Insert (3)",
        "the list counts"
    );
}

#[test]
fn a_text_preview_keeps_its_lines_and_drops_hidden_marks() {
    let mut p = page(1, vec![row(7, "a")]);
    let text = "one\r\ntwo\u{202e}\tthree\n\nمرحبا".to_owned();
    p.preview = Some(Preview::Text {
        text: Redacted::new(text),
        chars: 1,
    });
    let v = read("en", p);
    let want = json!({"kind": "text", "text": "one\ntwo three\n\nمرحبا", "chars": "1 character",
        "arabic": true, "rtl": false});
    assert_eq!(v["preview"], want);
}

#[test]
fn a_files_preview_lists_its_paths_without_hidden_marks_and_marks_arabic_ones() {
    let mut p = page(1, vec![row(7, "a")]);
    let paths = vec![
        PathBuf::from("C:\\a\u{202e}\\b.txt"),
        PathBuf::from("D:\\تقارير\\c"),
    ];
    p.preview = Some(Preview::Files(Redacted::new(paths)));
    let v = read("en", p);
    let want = json!({"kind": "files", "paths": [
        {"path": "C:\\a\\b.txt", "arabic": false},
        {"path": "D:\\تقارير\\c", "arabic": true}
    ]});
    assert_eq!(
        v["preview"], want,
        "an Arabic path takes the Arabic font, as a row title does"
    );
}

#[test]
fn no_title_search_or_app_reaches_a_log_line() {
    const SECRETS: [&str; 3] = ["zebra-quartz", "مرحبا", "KeePass"];
    let (rig, _host) = speaking("en", 5);
    let lines = lines_of(|| {
        let (bus, _acts, _subs) = heard(&rig);
        let mut r = row(7, SECRETS[0]);
        r.app = Some(Redacted::new(SECRETS[2].to_owned()));
        let mut p = page(2, vec![r]);
        p.search = Redacted::new(SECRETS[1].to_owned());
        bus.publish(&ClipPanel { open: true }).unwrap();
        bus.publish(&ClipView::new(p)).unwrap();
        let _ = rig.hub.clip_json();
        rig.hub.clip_act("mystery", 1);
        let _ = rig.hub.clip_picture(SECRETS[0]);
        rig.hub.open_folder();
        bus.publish(&ClipPanel { open: false }).unwrap();
    });
    for secret in SECRETS {
        assert!(
            lines.iter().all(|l| !l.contains(secret)),
            "{secret} in {lines:?}"
        );
    }
}
