//! An error's kind: one plain name per variant, letters only, so a log line may hold it and never a value it carries.

use std::collections::BTreeSet;
use std::time::Duration;

use super::*;

/// Every error, one of each variant.
fn every() -> Vec<SpeechError> {
    let retry_in = Duration::from_secs(3600);
    let all = vec![
        SpeechError::Range("pieces"),
        SpeechError::BadPath,
        SpeechError::NoMic,
        SpeechError::MicBusy,
        SpeechError::MicRefused,
        SpeechError::Unplugged,
        SpeechError::NoFile,
        SpeechError::BadFile,
        SpeechError::Language,
        SpeechError::Runner,
        SpeechError::NoCard,
        SpeechError::CardRoom,
        SpeechError::CardFailed,
        SpeechError::NoSound,
        SpeechError::OutOfStep,
        SpeechError::Memory,
        SpeechError::Internal,
        SpeechError::Timeout,
        SpeechError::Gone,
        SpeechError::Down { retry_in },
    ];
    all.iter().copied().for_each(listed);
    all
}

/// Builds only while it names every variant, so a new one fails here until `every` lists it.
const fn listed(error: SpeechError) {
    match error {
        SpeechError::Range(_)
        | SpeechError::BadPath
        | SpeechError::NoMic
        | SpeechError::MicBusy
        | SpeechError::MicRefused
        | SpeechError::Unplugged
        | SpeechError::NoFile
        | SpeechError::BadFile
        | SpeechError::Language
        | SpeechError::Runner
        | SpeechError::NoCard
        | SpeechError::CardRoom
        | SpeechError::CardFailed
        | SpeechError::NoSound
        | SpeechError::OutOfStep
        | SpeechError::Memory
        | SpeechError::Internal
        | SpeechError::Timeout
        | SpeechError::Gone
        | SpeechError::Down { .. } => {}
    }
}

#[test]
fn each_error_has_its_own_kind_of_letters_only_and_never_a_value_it_holds() {
    let all = every();
    let kinds: BTreeSet<&str> = all.iter().map(SpeechError::kind).collect();
    assert_eq!(kinds.len(), all.len(), "one kind per variant");
    for error in &all {
        let kind = error.kind();
        assert!(
            !kind.is_empty() && kind.bytes().all(|b| b.is_ascii_alphabetic()),
            "{kind}"
        );
        assert!(
            format!("{error:?}").starts_with(kind),
            "{kind} names its variant"
        );
    }
    for value in ["pieces", "3600"] {
        assert!(all.iter().all(|e| !e.kind().contains(value)), "{value}");
    }
}
