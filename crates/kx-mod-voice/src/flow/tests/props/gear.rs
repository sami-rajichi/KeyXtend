//! The gear's checks in the properties: only what records gets a Cancel, a preparing run ends with no bar, and a run
//! it ended writes nothing after.

use super::*;

impl Seen {
    /// Checks a gear click that found the flow `busy` (and `preparing`) or Compare `recording`: only what records gets
    /// a Cancel, a preparing run also closes with no bar, and a listening or writing one shows Cancel's note.
    pub(super) fn gear(
        &mut self,
        rig: &Rig,
        out: &[Out],
        (busy, preparing): (bool, bool),
        recording: bool,
    ) -> Result<(), TestCaseError> {
        let called: Vec<Out> = (out.iter())
            .filter(|o| matches!(o, Out::Run { .. } | Out::Lent(_)))
            .cloned()
            .collect();
        let run = |call| Out::Run {
            take: rig.take,
            call,
        };
        let want = match (busy, preparing) {
            (true, true) => vec![run(RunCall::Cancel), run(RunCall::Close)],
            (true, false) => vec![run(RunCall::Cancel)],
            _ => (recording
                .then_some(Out::Lent(BarButton::Cancel))
                .into_iter())
            .collect(),
        };
        prop_assert_eq!(called, want, "the gear cancels what records");
        if preparing {
            prop_assert_eq!(rig.shown(), VoiceBar::Hidden, "nothing was recorded");
        } else if busy {
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
