//! The preparing light in the properties: amber only while a dictation runs with no bar, and an opening that comes late
//! shows Listening and the red light from that moment.

use super::*;

/// Checks the light after `step`, which found the light `was`.
pub(super) fn check(rig: &Rig, step: &Step, was: VoiceLit) -> Result<(), TestCaseError> {
    let shown = rig.shown();
    if rig.light == VoiceLit::Prep {
        prop_assert_eq!(&shown, &VoiceBar::Hidden, "amber behind a bar");
        prop_assert!(rig.flow.dictating(), "amber with no dictation");
    }
    if matches!(step, Step::Open(_)) && was == VoiceLit::Prep {
        let heard = matches!(&shown, VoiceBar::Listening(l) if l.elapsed_ms == 0);
        prop_assert!(heard && rig.lit, "a late opening shows {shown:?}");
    }
    Ok(())
}
