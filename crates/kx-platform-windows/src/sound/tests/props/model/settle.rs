//! The checks at the end of each step: what a play, a prepare, a wake or the drop may leave, and what holds between
//! steps.

use kx_platform::ui::Clip;
use proptest::prelude::*;

use super::{Ask, Model, fits};
use crate::sound::{Fault, Warn};

impl Model {
    /// Checks the step's calls as a whole, then what holds between steps.
    pub(in super::super) fn settle(&mut self, ask: &Ask) -> Result<(), TestCaseError> {
        match ask {
            Ask::Play(..) => self.settle_play()?,
            Ask::Prepare(clips, _) => self.settle_prepare(clips)?,
            Ask::Wake => {
                let stopped = self.engine.as_ref().is_some_and(|e| e.since.is_none());
                let only = self.trace.calls == 1;
                prop_assert!(
                    stopped && only,
                    "a due wake stops the engine, and does nothing else"
                );
            }
            Ask::Drop => {
                prop_assert!(self.engine.is_none(), "the drop takes the engine");
                prop_assert_eq!(self.alive, 0, "every part made is gone");
            }
        }
        self.held()
    }

    /// NaN or a clip not kept makes no call. Otherwise the tries and builds that `tried` allows, and the failure's
    /// warning once.
    fn settle_play(&mut self) -> Result<(), TestCaseError> {
        if self.target.is_none() {
            let calls = self.trace.calls;
            prop_assert_eq!(calls, 0, "NaN or a clip not kept makes no call");
            return Ok(());
        }
        self.tried()?;
        let t = &self.trace;
        let want: Vec<_> = self
            .due_warn()
            .into_iter()
            .filter(|(w, _)| !t.before.contains(w))
            .collect();
        prop_assert_eq!(
            &t.warns,
            &want,
            "a failure warns its kind once until a play sounds"
        );
        if t.tries.last() != Some(&Ok(())) {
            self.dropped += 1;
        }
        Ok(())
    }

    /// One try, and a retry only after a lost device, with one rebuild between; the engine is made at the first play.
    fn tried(&self) -> Result<(), TestCaseError> {
        let t = &self.trace;
        let lost = |r: &Result<(), Fault>| matches!(r, Err(Fault::Lost(_)));
        let broke = t.broke.map(|(tries, _)| tries);
        let fair = match t.tries.as_slice() {
            [] => broke == Some(0),
            [only] if lost(only) => broke == Some(1),
            [_] => broke.is_none(),
            [first, _] => lost(first) && broke.is_none(),
            _ => false,
        };
        prop_assert!(
            fair,
            "a retry follows a lost device it rebuilt from, and only that: {:?}",
            t.tries
        );
        let first = t.builds.iter().filter(|&&tries| tries == 0).count();
        prop_assert_eq!(
            first,
            usize::from(!t.had_engine),
            "the engine is made at the first play"
        );
        let again = t.builds.len() - first;
        let after_loss = t.tries.first().is_some_and(lost);
        prop_assert_eq!(
            again,
            usize::from(after_loss),
            "a lost device rebuilds once"
        );
        Ok(())
    }

    /// The warning a play's failure calls for, with its code: a failed build's, else the last try's.
    fn due_warn(&self) -> Option<(Warn, Option<i32>)> {
        if let Some((tries, fault)) = self.trace.broke {
            let kind = if tries == 0 {
                Warn::Open
            } else {
                Warn::Rebuild
            };
            return Some((kind, Some(fault.code())));
        }
        let fault = self.trace.tries.last().copied()?.err()?;
        Some((Warn::Play, Some(fault.code())))
    }

    /// A prepare tears the engine down and warns each bad clip's kind once; `made_engine` refuses its builds.
    fn settle_prepare(&self, clips: &[Clip]) -> Result<(), TestCaseError> {
        prop_assert!(self.engine.is_none(), "a prepare tears the engine down");
        let mut want = Vec::new();
        for clip in clips.iter().filter(|c| !fits(c)) {
            let kind = if clip.whole() {
                Warn::Rate
            } else {
                Warn::Bytes
            };
            let warned = self.trace.before.contains(&kind) || want.contains(&(kind, None));
            if !warned {
                want.push((kind, None));
            }
        }
        prop_assert_eq!(&self.trace.warns, &want, "each bad clip's kind warns once");
        Ok(())
    }

    /// Between steps each clip keeps exactly its voices, so no more play at once, and the engine runs while one does.
    fn held(&self) -> Result<(), TestCaseError> {
        let Some(engine) = &self.engine else {
            return Ok(());
        };
        let full = engine.pools.iter().all(|p| p.len() == self.voices);
        prop_assert!(full, "each clip keeps exactly its voices");
        let playing = engine.voices().any(|v| self.busy(v, self.now));
        prop_assert!(
            !playing || engine.since.is_some(),
            "the engine runs while a voice plays"
        );
        Ok(())
    }
}
