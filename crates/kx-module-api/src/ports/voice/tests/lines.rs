//! The key page's typed lines (ADR-0049): a key to the Voice panel's line, the line kind its module tells, a click that
//! places a line's caret and a Save, each printing no typed text, caret or key length.

use super::panel::{holds, shown};
use super::{SECRETS, hidden, keeps_secrets};
use crate::ports::voice::*;
use crate::ports::{PanelKey, PanelLine};
use crate::{REDACTED_MARKER, Redacted};

/// A length or caret no output may show.
const LENGTH: u32 = 4417;

#[test]
fn the_lines_come_in_page_order() {
    assert_eq!(
        CloudLine::ALL,
        [CloudLine::Key, CloudLine::Address, CloudLine::Model]
    );
}

#[test]
fn a_key_to_the_voice_line_never_prints_its_text() {
    let word = PanelKey::Word {
        erase: 2,
        text: hidden(1),
    };
    let keys = [VoiceLineKey(PanelKey::Text(hidden(0))), VoiceLineKey(word)];
    let shown = format!("{keys:?}");
    assert!(keeps_secrets(&shown), "{shown}");
    holds(&shown, &["VoiceLineKey", "erase: 2", REDACTED_MARKER]);
    assert_eq!(
        format!("{:?}", VoiceLineKey(PanelKey::Enter)),
        "VoiceLineKey(Enter)"
    );
}

#[test]
fn the_voice_line_says_only_whether_the_panel_is_open_and_the_lines_kind() {
    let lines = [
        VoiceLine(None),
        VoiceLine(Some(PanelLine::None)),
        VoiceLine(Some(PanelLine::Private)),
    ];
    let shown = lines.map(|l| format!("{l:?}"));
    assert_eq!(
        shown,
        [
            "VoiceLine(None)",
            "VoiceLine(Some(None))",
            "VoiceLine(Some(Private))"
        ]
    );
    assert_eq!(format!("{VoiceClose:?}"), "VoiceClose");
}

#[test]
fn a_click_on_a_typed_line_hides_where_its_caret_goes_and_a_save_names_its_line() {
    let acts = [
        VoiceAct {
            round: 3,
            click: VoiceClick::Caret {
                line: CloudLine::Key,
                at: Redacted::new(LENGTH),
            },
        },
        VoiceAct {
            round: 3,
            click: VoiceClick::Save(CloudLine::Address),
        },
    ];
    let shown = format!("{acts:?}");
    assert!(!shown.contains(&LENGTH.to_string()), "{shown}");
    holds(
        &shown,
        &["Caret", "line: Key", "Save(Address)", REDACTED_MARKER],
    );
}

#[test]
fn the_key_page_hides_the_typed_key_and_its_caret_and_names_the_lines_that_save() {
    let sheet = ServiceSheet {
        service: ServiceName::Custom,
        mark: ServiceMark::NeedsKey,
        key: Redacted::new(0),
        typed: Redacted::new(LENGTH),
        check: KeyCheck::Untested,
        renews_at: None,
        address: Some(hidden(1)),
        address_fault: None,
        model: hidden(2),
        model_na: false,
        model_refused: false,
        keep: None,
        site: None,
        tests: false,
        acts: vec![CloudButton::Facts],
        lines: vec![CloudLine::Key, CloudLine::Model],
        typing: CloudLine::Key,
        caret: Redacted::new(LENGTH - 1),
        saves: vec![CloudLine::Key, CloudLine::Model],
        pasted: None,
    };
    let shown = shown(VoiceSheet::Service(sheet));
    for n in [LENGTH, LENGTH - 1] {
        assert!(!shown.contains(&n.to_string()), "{shown}");
    }
    assert!(SECRETS.iter().all(|s| !shown.contains(s)), "{shown}");
    holds(&shown, &["typing: Key", "saves: [Key, Model]"]);
}
