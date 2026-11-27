//! Snip's event names stay kebab-case and unique in the crate, beside every older name and voice typing's.

use super::events;
use crate::Event;
use crate::capability::is_kebab;
use crate::ports::voice::tests::names::older;
use crate::ports::{
    TextAsk, TextDone, VoiceAct, VoiceAsk, VoiceClose, VoiceKey, VoiceLevel, VoiceLine,
    VoiceLineKey, VoiceLit, VoiceOpen, VoiceView,
};
use std::collections::HashSet;

/// Voice typing's event names.
fn voice() -> [&'static str; 12] {
    [
        VoiceKey::NAME,
        VoiceLit::NAME,
        VoiceOpen::NAME,
        VoiceAsk::NAME,
        VoiceView::NAME,
        VoiceAct::NAME,
        TextAsk::NAME,
        TextDone::NAME,
        VoiceLineKey::NAME,
        VoiceLine::NAME,
        VoiceClose::NAME,
        VoiceLevel::NAME,
    ]
}

#[test]
fn names_are_kebab_case_and_unique_in_the_crate() {
    let ours = events().map(|r| r.0);
    assert!(ours.iter().all(|n| is_kebab(n)), "{ours:?}");
    let mine: HashSet<_> = ours.iter().copied().collect();
    assert_eq!(mine.len(), ours.len(), "{ours:?}");
    let others: HashSet<_> = older().chain(voice()).collect();
    assert_eq!(others.len(), older().count() + voice().len());
    assert!(mine.is_disjoint(&others), "{ours:?}");
}
