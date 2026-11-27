//! The pages Snip's windows draw: the mock-up's keys, tools and inks in order, the dim reasons the spec names, and a
//! view whose pixels, typed text and folder never print.

use super::{PIXELS, SECRETS, hidden, holds, keeps, shot};
use crate::ports::snip::*;
use crate::ports::{Point, Rect};
use crate::{REDACTED_MARKER, Redacted};
use std::path::PathBuf;

/// The 2 by 1 part every page here takes.
fn part() -> Part {
    let rect = Rect {
        x: 0,
        y: 0,
        w: 2,
        h: 1,
    };
    Part::new(rect).unwrap()
}

/// `key`, off and acting.
fn cap<K>(key: K) -> Keycap<K> {
    Keycap {
        key,
        on: false,
        dim: None,
    }
}

/// A typed note and a step at `at`.
fn drawings(at: Point) -> Vec<Drawing> {
    let note = Drawing {
        mark: Mark::Text {
            at,
            text: hidden(1),
            rtl: true,
        },
        ink: Ink::Red,
        size: 2,
    };
    let step = Drawing {
        mark: Mark::Step { at, n: 1 },
        ink: Ink::Blue,
        size: 0,
    };
    vec![note, step]
}

/// The editor holding `shot()`, a typed note, a step and secret text still being typed, at its default place.
pub(super) fn editor() -> Editor {
    let at = Point { x: 1, y: 0 };
    let typing = TextEdit {
        at,
        text: hidden(0),
        caret: Redacted::new(4417),
        rtl: false,
    };
    let edit = PictureEdit {
        crop: Crop::new(part().rect(), 2, 1).unwrap(),
        drawings: drawings(at),
        tool: Tool::Text,
        ink: Ink::Red,
        size: 2,
        pending: None,
        frame: None,
        typing: Some(typing),
        undo: true,
        redo: false,
    };
    Editor {
        picture: shot(),
        body: edit,
        save_dim: Some(Dim::Hidden),
        done: Some(Keep::Copy),
        place: None,
        full: None,
    }
}

/// A page with `stage` over a frozen screen, the question and the Capture page open.
fn page(stage: SnipStage) -> SnipPage {
    let folder = PathBuf::from(format!(r"C:\Users\{}\snips", SECRETS[2]));
    let sheet = CaptureSheet {
        keyboard: SnipKeyboard::Bubble,
        folder: crate::ports::FolderPath::new(folder).ok(),
    };
    SnipPage {
        round: 9,
        frozen: vec![FrozenScreen::new(part().rect(), 96, shot()).unwrap()],
        stage,
        ask: Some(CloseAsk { save_dim: None }),
        note: Some(SnipNote::DiskLow),
        panel: Some(SnipSheet::Capture(sheet)),
        dialog: true,
    }
}

#[test]
fn the_bar_and_the_strips_have_the_mock_ups_keys_in_order() {
    use BarKey::{Close, Full, Region, Settings, VideoFull, VideoRegion, Window};
    let bar = [
        Region,
        Window,
        Full,
        VideoRegion,
        VideoFull,
        Settings,
        Close,
    ];
    assert_eq!(BarKey::ALL, bar, "the spec's order, settings before Close");
    let strip = [
        StripKey::Copy,
        StripKey::Edit,
        StripKey::Save,
        StripKey::Close,
        StripKey::Record,
        StripKey::Mic,
        StripKey::Sound,
        StripKey::Play,
        StripKey::Speed,
    ];
    assert_eq!(StripKey::ALL, strip);
    assert_eq!(
        RecKey::ALL,
        [RecKey::Mic, RecKey::Sound, RecKey::Stop, RecKey::Cancel]
    );
    assert_eq!(AskKey::ALL, [AskKey::Save, AskKey::Drop, AskKey::Back]);
    assert_eq!(
        SnipSpot::ALL,
        [
            SnipSpot::Under,
            SnipSpot::Above,
            SnipSpot::Bottom,
            SnipSpot::Top
        ]
    );
}

#[test]
fn the_editor_has_eleven_tools_and_six_inks_in_the_mock_ups_order() {
    use Ink::{Black, Blue, Green, Red, White, Yellow};
    use Tool::{Arrow, Box, Circle, Cover, Crop, Eraser, Highlighter, Line, Pen, Steps, Text};
    let tools = [
        Crop,
        Pen,
        Highlighter,
        Box,
        Circle,
        Arrow,
        Line,
        Text,
        Cover,
        Steps,
        Eraser,
    ];
    assert_eq!(Tool::ALL, tools);
    assert_eq!(Ink::ALL, [Red, Yellow, Green, Blue, Black, White]);
    let shapes = [
        Shape::Box,
        Shape::Circle,
        Shape::Arrow,
        Shape::Line,
        Shape::Cover,
    ];
    assert_eq!(Shape::ALL, shapes);
    assert_eq!(EditKey::ALL, [EditKey::Undo, EditKey::Redo, EditKey::Close]);
    assert_eq!(Keep::ALL, [Keep::Copy, Keep::Save]);
}

#[test]
fn a_key_dims_only_for_a_reason_the_spec_names() {
    use Dim::{Bare, DiskLow, Hidden, MicRefused, NoMic, NoSound, NoVideo};
    assert_eq!(
        Dim::ALL,
        [Hidden, Bare, NoVideo, NoMic, MicRefused, NoSound, DiskLow]
    );
    let notes = SnipNote::ALL.len();
    assert_eq!(
        notes, 16,
        "too big, freeze failed, disk low, longest, no video, record, save, busy, copy, speed, bare, place, \
         play, folder, mic, sound"
    );
    let last = [
        SnipNote::PlaceRefused,
        SnipNote::PlayFailed,
        SnipNote::FolderFailed,
        SnipNote::MicFailed,
        SnipNote::SoundFailed,
    ];
    assert_eq!(SnipNote::ALL[notes - last.len()..], last);
}

#[test]
fn the_panels_pages_and_rows_come_in_order() {
    use VideoRow::{Countdown, Fps, Longest, Mic, Sound};
    assert_eq!(SnipTab::ALL, [SnipTab::Capture, SnipTab::Video]);
    assert_eq!(VideoRow::ALL, [Countdown, Fps, Longest, Mic, Sound]);
    assert_eq!(FolderKey::ALL, [FolderKey::Change, FolderKey::Open]);
}

#[test]
fn a_view_never_prints_pixels_typed_text_or_a_folder() {
    let picking = Picking {
        screen: part().rect(),
        keys: vec![cap(BarKey::Region)],
        corner: Some(Point { x: 1, y: 0 }),
        within: None,
        windows: vec![part().rect()],
    };
    let views = [
        SnipView::new(page(SnipStage::Pick(picking))),
        SnipView::new(page(SnipStage::Edit(Box::new(editor())))),
    ];
    let shown = format!("{views:?}");
    assert!(keeps(&shown) && !shown.contains("4417"), "{shown}");
    holds(
        &shown,
        &[
            "round: 9",
            "Bubble",
            "DiskLow",
            "Region",
            "rtl: true",
            "dialog: true",
        ],
    );
    let marks = shown.matches(REDACTED_MARKER).count();
    assert!(
        marks >= 7,
        "pixels, folder, both texts and the caret: {shown}"
    );
}

#[test]
fn the_strip_and_the_record_bar_print_their_state() {
    let mut mic = cap(StripKey::Mic);
    mic.dim = Some(Dim::NoMic);
    let taken = Taken {
        part: part(),
        screen: part().rect(),
        spot: SnipSpot::Above,
        keys: vec![cap(StripKey::Record), mic],
        length_ms: None,
        speed: None,
        done: None,
        waits: None,
        frame: None,
    };
    let recording = Recording {
        part: part(),
        screen: part().rect(),
        spot: SnipSpot::Top,
        elapsed_ms: 12_000,
        live: true,
        keys: vec![cap(RecKey::Stop)],
    };
    let stages = [SnipStage::Strip(taken), SnipStage::Record(recording)];
    let shown = format!("{stages:?}");
    holds(
        &shown,
        &["Above", "NoMic", "elapsed_ms: 12000", "live: true", "Stop"],
    );
    assert!(!shown.contains(&format!("{}", PIXELS[0])), "{shown}");
}

#[test]
fn a_sped_videos_strip_and_the_countdown_print_their_state() {
    let sped = Taken {
        part: part(),
        screen: part().rect(),
        spot: SnipSpot::Under,
        keys: vec![cap(StripKey::Speed)],
        length_ms: Some(2_100),
        speed: Speed::new(2),
        done: None,
        waits: Some(StripKey::Play),
        frame: None,
    };
    let count = Countdown {
        part: part(),
        left_s: 3,
    };
    let stages = [SnipStage::Strip(sped), SnipStage::Count(count)];
    let shown = format!("{stages:?}");
    let want = [
        "length_ms: Some(2100)",
        "speed: Some(Speed(2))",
        "waits: Some(Play)",
        "left_s: 3",
    ];
    holds(&shown, &want);
}
