//! The gear's checks in the properties: only what records gets a Cancel, and a run it ended writes nothing after.

use super::*;

impl Seen {
    /// Checks a gear click that found the flow `busy` or Compare `recording`: only what records gets a Cancel, and a
    /// dictation shows Cancel's note and is noted as ended.
    pub(super) fn gear(
        &mut self,
        rig: &Rig,
        out: &[Out],
        busy: bool,
        recording: bool,
    ) -> Result<(), TestCaseError> {
        let called: Vec<&Out> = (out.iter())
            .filter(|o| matches!(o, Out::Run { .. } | Out::Lent(_)))
            .collect();
        let cancel = Out::Run {
            take: rig.take,
            call: RunCall::Cancel,
        };
        let want = if busy {
            Some(cancel)
        } else {
            recording.then_some(Out::Lent(BarButton::Cancel))
        };
        prop_assert_eq!(
            called,
            want.iter().collect::<Vec<_>>(),
            "the gear cancels what records"
        );
        if busy {
            let hint = note(&rig.shown()).map(|(hint, _)| hint);
            prop_assert_eq!(
                hint,
                Some(BarHint::Cancelled),
                "the gear ends it as Cancel does"
            );
            self.geared.insert(rig.take);
        }
        Ok(())
    }

    /// Checks that a run the gear ended is never stopped, typed, kept or counted until Transcribe anyway writes it.
    pub(super) fn after_gear(&mut self, out: &[Out]) -> Result<(), TestCaseError> {
        for effect in out {
            match effect {
                Out::Open { take, .. } => self.live = *take,
                Out::Run {
                    take,
                    call: RunCall::Again(_),
                } => {
                    self.geared.remove(take);
                }
                Out::Run { take, call } => {
                    let wrote = matches!(call, RunCall::Stop | RunCall::Keep);
                    prop_assert!(
                        !wrote || !self.geared.contains(take),
                        "{call:?} after the gear"
                    );
                }
                Out::Type(_) | Out::Count(_) => {
                    prop_assert!(
                        !self.geared.contains(&self.live),
                        "{effect:?} after the gear"
                    );
                }
                Out::Keep(kept) => {
                    prop_assert!(!self.geared.contains(&kept.take), "kept after the gear");
                }
                _ => {}
            }
        }
        Ok(())
    }
}
