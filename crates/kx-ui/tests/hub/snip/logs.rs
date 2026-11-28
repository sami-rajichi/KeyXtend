//! No log line holds what the user typed, the Snips folder, a press's point or a click's kind, id or argument; a
//! window's open times are whole ms by the stage shown, and a countdown second logs numbers as it comes and shows.

use super::drawn::drawn;
use super::{heard, page, rect, screens, shot};
use crate::Rig;
use crate::logs::lines_of;
use kx_module_api::Redacted;
use kx_module_api::ports::{
    CaptureSheet, Countdown, FolderPath, Part, SnipKeyboard, SnipSheet, SnipStage, TextEdit,
};
use kx_ui::snip_acts::CARET;
use kx_ui::{SnipCounted, SnipOpened};

/// What the user typed, as no log line may hold it.
const SECRET: &str = "s3cret-snip";

#[test]
fn no_typed_text_path_point_or_click_reaches_a_log_line() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut editor = drawn();
    let typed = |t: TextEdit| TextEdit {
        text: Redacted::new(SECRET.to_owned()),
        ..t
    };
    editor.body.typing = editor.body.typing.take().map(typed);
    let mut shown = page(7, screens(), SnipStage::Edit(Box::new(editor)));
    let folder = FolderPath::new(std::env::temp_dir()).unwrap();
    let path = folder.expose().display().to_string();
    shown.panel = Some(SnipSheet::Capture(CaptureSheet {
        keyboard: SnipKeyboard::Shown,
        folder: Some(folder),
    }));
    let lines = lines_of(|| {
        heard.shown(&rig, shown);
        assert!(rig.hub.snip_json().contains(SECRET), "QML reads it");
        rig.hub.snip_act(SECRET, SECRET, -7, -7);
        rig.hub.snip_act(CARET, SECRET, -1, 7);
        rig.hub.snip_spot(9, 1234.5, 5678.5, 7);
        rig.hub.snip_press(4321.0, 8765.0, f64::NAN, 7);
        rig.hub.snip_flat(SECRET, shot(2, 2), 7);
        assert!(!rig.hub.snip_fit(SECRET));
    });
    assert_eq!(lines.len(), 6, "each bad call warns once: {lines:?}");
    for bad in [SECRET, path.as_str(), "1234", "5678", "4321", "8765"] {
        assert!(lines.iter().all(|l| !l.contains(bad)), "{lines:?}");
    }
}

#[test]
fn a_windows_open_logs_its_ms_by_the_stage_shown_and_nothing_else() {
    let rig = Rig::new();
    let heard = heard(&rig);
    heard.shown(&rig, page(7, screens(), SnipStage::Edit(Box::new(drawn()))));
    let times = SnipOpened {
        click_ms: 41,
        shown_ms: 3,
        drawn_ms: 27,
        ready_ms: 212,
    };
    let lines = lines_of(|| {
        rig.hub.snip_opened("edit", times);
        rig.hub.snip_opened("pick", times);
        rig.hub.snip_opened(SECRET, times);
    });
    assert_eq!(lines.len(), 3, "one line each: {lines:?}");
    let fields = [
        r#"stage="edit""#,
        "click_ms=41",
        "shown_ms=3",
        "drawn_ms=27",
        "ready_ms=212",
    ];
    assert!(fields.iter().all(|f| lines[0].contains(f)), "{lines:?}");
    let late = &lines[1..];
    assert!(
        late.iter()
            .all(|l| !l.contains("_ms=") && !l.contains("stage=")),
        "no times for a stage gone: {lines:?}"
    );
    assert!(lines.iter().all(|l| !l.contains(SECRET)), "{lines:?}");
}

#[test]
fn a_countdown_second_logs_numbers_as_its_view_comes_and_as_it_shows() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let count = Countdown {
        part: Part::new(rect(1, 1, 4, 3)).unwrap(),
        left_s: 2,
    };
    let second = SnipCounted {
        left_s: 2,
        draws: 5,
        held: false,
        frozen: 1,
    };
    let lines = lines_of(|| {
        heard.shown(&rig, page(7, Vec::new(), SnipStage::Count(count)));
        rig.hub.snip_counted(second);
    });
    let came: Vec<_> = lines
        .iter()
        .filter(|l| l.contains("a countdown second came"))
        .collect();
    assert!(came.len() == 1 && came[0].contains("left_s=2"), "{lines:?}");
    let shown = [
        "left_s=2",
        "draws=5",
        "held=false",
        "frozen=1",
        "a countdown second showed",
    ];
    assert!(
        lines.iter().any(|l| shown.iter().all(|f| l.contains(f))),
        "{lines:?}"
    );
}
