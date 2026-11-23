//! The preparing light in the properties: amber only while a dictation or Compare's recording runs with no bar, and an
//! opening that comes late shows Listening and the red light from that moment.

use super::*;

/// Checks the light after `step`, which found the light `was`.
pub(super) fn check(rig: &Rig, step: &Step, was: VoiceLit) -> Result<(), TestCaseError> {
    let shown = rig.shown();
    if rig.light == VoiceLit::Prep {
        prop_assert_eq!(&shown, &VoiceBar::Hidden, "amber behind a bar");
        let runs = rig.flow.dictating() || rig.flow.lent.is_some();
        prop_assert!(runs, "amber with no run");
    }
    let opened = matches!(step, Step::Open(_)) && rig.flow.lent.is_none();
    let lent = matches!(step, Step::Lend(Lend::Listen)) && !rig.flow.dictating();
    if (opened || lent) && was == VoiceLit::Prep {
        let heard = matches!(&shown, VoiceBar::Listening(l) if l.elapsed_ms == 0);
        prop_assert!(heard && rig.lit, "a late opening shows {shown:?}");
    }
    Ok(())
}
