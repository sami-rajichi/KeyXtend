//! Voice typing's events: their names and gates, the place's names, the approved bar's buttons and hints, and `Debug`
//! that never shows a transcript or a text to type. The panel's pages are in `panel`, cloud services in `cloud`.

use super::*;
use crate::capability::is_kebab;
use crate::{REDACTED_MARKER, Redacted, Secret, Zeroizing};
use std::collections::HashSet;

mod cloud;
mod compare;
mod lines;
mod names;
mod note;
mod options;
mod panel;
mod sides;

/// The `Voice` gate.
const VOICE: Option<Capability> = Some(Capability::Voice);
/// Texts no output may show: a transcript, an Arabic one ("hello"), and a folder's or a file's name.
const SECRETS: [&str; 3] = ["zebra-quartz", "مرحبا", "Taxes-4417"];

/// An event's name and gate.
fn row<E: Event>() -> (&'static str, Option<Capability>) {
    (E::NAME, E::CAPABILITY)
}

/// The voice events, in the order `events_have_their_names` checks.
fn events() -> [(&'static str, Option<Capability>); 12] {
    [
        row::<VoiceKey>(),
        row::<VoiceLit>(),
        row::<VoiceOpen>(),
        row::<VoiceAsk>(),
        row::<VoiceView>(),
        row::<VoiceAct>(),
        row::<TextAsk>(),
        row::<TextDone>(),
        row::<VoiceLineKey>(),
        row::<VoiceLine>(),
        row::<VoiceClose>(),
        row::<VoiceLevel>(),
    ]
}

/// Secret `at`, redacted.
fn hidden(at: usize) -> Redacted<String> {
    Redacted::new(SECRETS[at].to_owned())
}

/// Secret `at`, redacted and wiped on drop.
fn secret(at: usize) -> Secret {
    Redacted::new(Zeroizing::new(SECRETS[at].to_owned()))
}

/// True when `shown` holds no secret.
fn keeps_secrets(shown: &str) -> bool {
    SECRETS.iter().all(|s| !shown.contains(s))
}

/// English, as the bar's chip names it.
fn english() -> VoiceLang {
    VoiceLang {
        name: "English".into(),
        iso: "en".into(),
        rtl: false,
        arabic: false,
    }
}

/// A page with `bar` and no panel.
fn page(bar: VoiceBar) -> VoicePage {
    VoicePage {
        round: 7,
        bar,
        panel: None,
        place: BarPlace::Above,
        sounds: true,
        cloud: None,
    }
}

#[test]
fn events_have_their_names() {
    let want = [
        "voice-key",
        "voice-lit",
        "voice-open",
        "voice-ask",
        "voice-view",
        "voice-act",
        "text-ask",
        "text-done",
        "voice-line-key",
        "voice-line",
        "voice-close",
        "voice-level",
    ];
    assert_eq!(events().map(|r| r.0), want);
}

#[test]
fn every_event_needs_voice_but_the_level() {
    let gates = events().map(|r| r.1);
    assert_eq!(gates[..11], [VOICE; 11], "ADR-0042 decision 7, ADR-0049");
    assert_eq!(gates[11], None, "a level holds no content");
}

#[test]
fn names_are_kebab_case_and_unique_in_the_crate() {
    let ours = events().map(|r| r.0);
    assert!(ours.iter().all(|n| is_kebab(n)), "{ours:?}");
    let others: HashSet<_> = names::older().collect();
    assert_eq!(
        others.len(),
        names::older().count(),
        "the older names are unique too"
    );
    let mine: HashSet<_> = ours.iter().copied().collect();
    assert_eq!(mine.len(), ours.len(), "{ours:?}");
    assert!(mine.is_disjoint(&others), "{ours:?}");
}

#[test]
fn the_light_starts_off_and_glows_amber_before_red() {
    assert_eq!(
        VoiceLit::default(),
        VoiceLit::Off,
        "before the module speaks"
    );
    let lights = [VoiceLit::Off, VoiceLit::Prep, VoiceLit::Rec];
    let names = lights.map(|l| serde_json::to_value(l).ok());
    let want = ["off", "prep", "rec"].map(|n| Some(serde_json::Value::from(n)));
    assert_eq!(names, want, "the face's state writes these");
}

#[test]
fn a_place_reads_and_writes_as_its_lowercase_name() {
    assert_eq!(
        BarPlace::ALL,
        [BarPlace::Above, BarPlace::Bottom, BarPlace::Top]
    );
    let names = ["above", "bottom", "top"].map(serde_json::Value::from);
    for (place, name) in BarPlace::ALL.into_iter().zip(names) {
        assert_eq!(serde_json::to_value(place).ok(), Some(name.clone()));
        assert_eq!(serde_json::from_value::<BarPlace>(name).ok(), Some(place));
    }
    let bad = serde_json::from_value::<BarPlace>(serde_json::Value::from("left"));
    assert!(bad.is_err(), "edge 35: back to the default");
}

#[test]
fn the_bar_has_the_approved_mock_ups_buttons() {
    use BarButton::{Again, Anyway, Cancel, Close, Copy, Download, Listen, Service, Stop, Undo};
    let want = [
        Cancel, Stop, Copy, Undo, Anyway, Again, Listen, Download, Service, Close,
    ];
    assert_eq!(
        BarButton::ALL,
        want,
        "Try again shows on a cloud note and in the panel's footer"
    );
    assert!(PanelButton::ALL.contains(&PanelButton::Again));
    assert!(
        PanelButton::ALL.contains(&PanelButton::Rerun),
        "Compare again"
    );
}

#[test]
fn the_bars_text_is_a_secret_wiped_on_drop_that_never_prints() {
    let shown = VoiceText::Shown {
        text: secret(1),
        rtl: true,
        arabic: true,
    };
    let VoiceText::Shown { text, .. } = &shown else {
        unreachable!("a shown text")
    };
    let held: &Zeroizing<String> = text.expose();
    assert_eq!(held.as_str(), SECRETS[1]);
    let printed = format!("{shown:?}{:?}", shown.clone());
    assert!(keeps_secrets(&printed), "{printed}");
    assert!(printed.contains(REDACTED_MARKER), "{printed}");
}

#[test]
fn a_text_bar_hides_its_text_and_a_masked_one_carries_none() {
    let shown = VoiceText::Shown {
        text: secret(0),
        rtl: false,
        arabic: false,
    };
    let text = |text, hint| BarText {
        text,
        hint: Some(hint),
        undo: true,
    };
    let bars = [
        page(VoiceBar::Text(text(shown, BarHint::TooLong))),
        page(VoiceBar::Text(text(VoiceText::Masked, BarHint::Password))),
    ];
    let shown = format!("{:?}", bars.map(VoiceView::new));
    assert!(keeps_secrets(&shown), "{shown}");
    for part in ["round: 7", "TooLong", "Masked", "Password", REDACTED_MARKER] {
        assert!(shown.contains(part), "{shown} lacks {part}");
    }
}

#[test]
fn a_note_carries_the_language_and_size_its_line_and_action_print() {
    let no_model = BarNote {
        hint: BarHint::NoModel {
            lang: english(),
            bytes: 740_000_000,
        },
        act: Some(BarButton::Download),
        mic: false,
    };
    let nothing = BarNote {
        hint: BarHint::Nothing,
        act: Some(BarButton::Listen),
        mic: false,
    };
    let hints = [BarHint::Language(english()), BarHint::NoLanguage(english())];
    let shown = format!("{no_model:?}{nothing:?}{hints:?}");
    let parts = [
        "English",
        "iso: \"en\"",
        "bytes: 740000000",
        "Download",
        "Nothing",
        "Listen",
    ];
    for part in parts {
        assert!(shown.contains(part), "{shown} lacks {part}");
    }
    assert_eq!(shown.matches("English").count(), 3, "{shown}");
}

#[test]
fn a_bad_download_a_file_in_the_way_and_a_file_still_read_each_have_a_hint() {
    let hints = [
        BarHint::Print,
        BarHint::Taken,
        BarHint::Checking,
        BarHint::Language(english()),
    ];
    let shown: HashSet<String> = hints.iter().map(|h| format!("{h:?}")).collect();
    assert_eq!(shown.len(), hints.len(), "{shown:?}");
}

#[test]
fn a_text_ask_and_a_compare_click_hide_their_text() {
    let text = Redacted::new(Zeroizing::new(SECRETS[1].to_owned()));
    let ask = TextAsk {
        round: 3,
        erase: 12,
        text,
        gap_ms: 10,
    };
    let act = VoiceAct {
        round: 3,
        click: VoiceClick::Compare(hidden(0)),
    };
    let shown = format!("{ask:?}{act:?}");
    assert!(keeps_secrets(&shown), "{shown}");
    assert!(
        shown.contains("erase: 12") && shown.contains("gap_ms: 10"),
        "{shown}"
    );
}

#[test]
fn a_text_answer_carries_its_round_and_count() {
    let done = TextDone {
        round: 3,
        count: 41,
        ok: true,
    };
    assert_eq!(
        format!("{done:?}"),
        "TextDone { round: 3, count: 41, ok: true }"
    );
    let level = VoiceLevel { level: 64 };
    assert_eq!(format!("{level:?}"), "VoiceLevel { level: 64 }");
}
