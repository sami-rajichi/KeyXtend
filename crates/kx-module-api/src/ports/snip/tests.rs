//! Snip's events and ports: their names and gates, the setting and light names, and `Debug` that never shows pixels,
//! typed text or a path. The values each port refuses are in `shot`, `record`, `files` and `store`, the pages in
//! `view`.

use super::*;
use crate::{REDACTED_MARKER, Redacted, ServiceId, ServiceKey};

mod cut;
mod editor;
mod files;
mod names;
mod record;
mod shot;
mod store;
mod view;
mod waits;

/// The `ScreenCapture` gate.
const SCREEN: Option<Capability> = Some(Capability::ScreenCapture);
/// The `Snip` event gate.
const SNIP: Option<Capability> = Some(Capability::Snip);
/// Pixel bytes no output may show; `Vec<u8>` would print them as `PRINTED`.
const PIXELS: [u8; 4] = [201, 202, 203, 204];
/// How `Debug` would print the first two of `PIXELS`.
const PRINTED: &str = "201, 202";
/// Texts no output may show: typed text, an Arabic one ("hello"), and a user's name in a path.
const SECRETS: [&str; 3] = ["zebra-quartz", "مرحبا", "Alex"];

/// An event's name and gate.
fn row<E: Event>() -> (&'static str, Option<Capability>) {
    (E::NAME, E::CAPABILITY)
}

/// The snip events, in the order `events_have_their_names` checks.
fn events() -> [(&'static str, Option<Capability>); 8] {
    [
        row::<SnipStart>(),
        row::<SnipView>(),
        row::<SnipAct>(),
        row::<SnipAside>(),
        row::<SnipLit>(),
        row::<SnipLine>(),
        row::<SnipLineKey>(),
        row::<SnipAsk>(),
    ]
}

/// A 2 by 1 shot of `PIXELS` twice.
fn shot() -> Shot {
    Shot::new(2, 1, [PIXELS, PIXELS].concat()).unwrap()
}

/// Secret `at`, redacted.
fn hidden(at: usize) -> Redacted<String> {
    Redacted::new(SECRETS[at].to_owned())
}

/// True when `shown` holds no pixel bytes and no secret.
fn keeps(shown: &str) -> bool {
    !shown.contains(PRINTED) && SECRETS.iter().all(|s| !shown.contains(s))
}

/// Fails unless `shown` holds every one of `parts`.
fn holds(shown: &str, parts: &[&str]) {
    for part in parts {
        assert!(shown.contains(part), "{shown} lacks {part}");
    }
}

#[test]
fn events_have_their_names() {
    let want = [
        "snip-start",
        "snip-view",
        "snip-act",
        "snip-aside",
        "snip-lit",
        "snip-line",
        "snip-line-key",
        "snip-ask",
    ];
    assert_eq!(events().map(|r| r.0), want);
}

#[test]
fn every_event_needs_the_snip_event_gate() {
    let want = [SNIP; 8];
    assert_eq!(
        events().map(|r| r.1),
        want,
        "an event gate, as voice typing's"
    );
}

#[test]
fn the_light_starts_off_and_writes_its_lowercase_name() {
    assert_eq!(SnipLit::default(), SnipLit::Off, "before the module speaks");
    let names = [SnipLit::Off, SnipLit::On].map(|l| serde_json::to_value(l).ok());
    let want = ["off", "on"].map(|n| Some(serde_json::Value::from(n)));
    assert_eq!(names, want, "its lowercase name, as `VoiceLit`'s");
}

#[test]
fn a_keyboard_choice_reads_and_writes_as_its_lowercase_name() {
    use SnipKeyboard::{Bubble, Hidden, Shown};
    assert_eq!(
        SnipKeyboard::ALL,
        [Bubble, Hidden, Shown],
        "the panel's order"
    );
    let names = ["bubble", "hidden", "shown"].map(serde_json::Value::from);
    for (choice, name) in SnipKeyboard::ALL.into_iter().zip(names) {
        assert_eq!(serde_json::to_value(choice).ok(), Some(name.clone()));
        assert_eq!(
            serde_json::from_value::<SnipKeyboard>(name).ok(),
            Some(choice)
        );
    }
    let bad = serde_json::from_value::<SnipKeyboard>(serde_json::Value::from("gone"));
    assert!(bad.is_err());
}

#[test]
fn the_ports_are_named_and_gated() {
    use Capability::{Microphone, OpenFolder, SaveFile};
    let ids = [
        ScreenFreezeKey::ID,
        RecorderKey::ID,
        SnipFolderKey::ID,
        FileAskKey::ID,
        MediaOpenKey::ID,
        MicPassKey::ID,
    ];
    let want = ["screen-freeze", "recorder", "snip-folder"];
    let more = ["file-ask", "media-open", "mic-pass"];
    assert_eq!(ids.map(ServiceId::as_str), [want, more].concat().as_slice());
    let gates = [
        ScreenFreezeKey::CAPABILITY,
        RecorderKey::CAPABILITY,
        SnipFolderKey::CAPABILITY,
        FileAskKey::CAPABILITY,
        MediaOpenKey::CAPABILITY,
        MicPassKey::CAPABILITY,
    ];
    let others = [Some(SaveFile), Some(OpenFolder), Some(Microphone)];
    assert_eq!(gates, [[SCREEN; 3], others].concat().as_slice(), "ADR-0056");
}

#[test]
fn the_line_says_only_whether_a_window_shows_and_its_kind() {
    let lines = [
        SnipLine(None),
        SnipLine(Some(PanelLine::None)),
        SnipLine(Some(PanelLine::Private)),
    ];
    let shown = lines.map(|l| format!("{l:?}"));
    let want = [
        "SnipLine(None)",
        "SnipLine(Some(None))",
        "SnipLine(Some(Private))",
    ];
    assert_eq!(shown, want);
}

#[test]
fn a_key_to_the_editors_line_never_prints_its_text() {
    let word = PanelKey::Word {
        erase: 2,
        text: hidden(1),
    };
    let keys = [SnipLineKey(PanelKey::Text(hidden(0))), SnipLineKey(word)];
    let shown = format!("{keys:?}");
    assert!(keeps(&shown), "{shown}");
    holds(&shown, &["SnipLineKey", "erase: 2", REDACTED_MARKER]);
    let esc = format!("{:?}", SnipLineKey(PanelKey::Esc));
    assert_eq!(
        esc, "SnipLineKey(Esc)",
        "Esc ends a capture or answers Back"
    );
}

#[test]
fn a_flattened_picture_and_a_caret_print_redacted() {
    let acts = [
        SnipAct {
            round: 4,
            click: SnipClick::Flat {
                keep: Keep::Save,
                shot: shot(),
            },
        },
        SnipAct {
            round: 4,
            click: SnipClick::Caret(Redacted::new(4417)),
        },
    ];
    let shown = format!("{acts:?}");
    assert!(keeps(&shown) && !shown.contains("4417"), "{shown}");
    holds(
        &shown,
        &["round: 4", "Flat", "Save", "width: 2", REDACTED_MARKER],
    );
}
