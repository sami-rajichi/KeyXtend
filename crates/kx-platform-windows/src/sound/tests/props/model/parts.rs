//! The checks of the calls that build and tear down an engine: the engine, the master, then the voices, gone in the
//! reverse order, and a voice replaced only in place of the earliest that plays.

use kx_module_api::Mono;
use proptest::prelude::*;

use super::{Engine, Model, Slot, plays, some};

impl Model {
    pub(super) fn made_engine(&mut self) -> Result<(), TestCaseError> {
        prop_assert!(self.engine.is_none(), "the old engine goes first");
        prop_assert!(self.target.is_some(), "only a play builds");
        self.trace.builds.push(self.trace.tries.len());
        let pools = vec![Vec::new(); self.kept.len()];
        self.engine = Some(Engine {
            pools,
            ..Engine::default()
        });
        self.alive += 1;
        Ok(())
    }

    pub(super) fn made_master(&mut self) -> Result<(), TestCaseError> {
        let fault = self.faults.masters.pop_front();
        prop_assert_eq!(fault, None, "a master is made only when no fault waits");
        let engine = some(self.engine.as_mut(), "a master needs an engine")?;
        prop_assert!(!engine.mastered, "one master per engine");
        (engine.master, engine.mastered) = (true, true);
        self.alive += 1;
        Ok(())
    }

    /// A voice of the build fills the pools in order; any other replaces a voice of the clip that plays.
    pub(super) fn made_voice(&mut self, n: u32) -> Result<(), TestCaseError> {
        let engine = some(self.engine.as_mut(), "a voice needs an engine")?;
        prop_assert!(engine.master, "a voice needs the master");
        self.alive += 1;
        let short = engine.pools.iter_mut().find(|p| p.len() < self.voices);
        if let Some(pool) = short {
            let started = Mono::default();
            pool.push(Slot { voice: n, started });
            return Ok(());
        }
        let (at, _) = some(self.target, "only a play replaces a voice")?;
        let pool = some(engine.pools.get(at), "the clip has a pool")?;
        let all = pool.iter().all(|s| plays(&self.ends, s.voice, self.now));
        prop_assert!(
            all,
            "a voice is replaced only while every voice of the clip plays"
        );
        prop_assert!(
            self.trace.fresh.replace(n).is_none(),
            "one new voice per try"
        );
        Ok(())
    }

    /// The voice a new one replaces is the one that started earliest; otherwise voices go before the master.
    pub(super) fn gone_voice(&mut self, n: u32) -> Result<(), TestCaseError> {
        let engine = some(self.engine.as_mut(), "a voice goes with its engine")?;
        self.alive = some(self.alive.checked_sub(1), "a part goes once")?;
        if let Some(fresh) = self.trace.fresh.take() {
            let (at, _) = some(self.target, "only a play replaces a voice")?;
            let pool = some(engine.pools.get_mut(at), "the clip has a pool")?;
            let first = pool.iter_mut().min_by_key(|s| s.started);
            let slot = some(first, "the pool has a voice")?;
            prop_assert_eq!(slot.voice, n, "the voice that started earliest is replaced");
            slot.voice = fresh;
            return Ok(());
        }
        prop_assert!(engine.master, "voices go before the master");
        let mut pools = engine.pools.iter_mut();
        let gone = pools.find_map(|p| Some(p.remove(p.iter().position(|s| s.voice == n)?)));
        prop_assert!(gone.is_some(), "only a voice of the engine goes");
        Ok(())
    }

    pub(super) fn gone_master(&mut self) -> Result<(), TestCaseError> {
        let engine = some(self.engine.as_mut(), "a master goes with its engine")?;
        let bare = engine.pools.iter().all(Vec::is_empty);
        prop_assert!(
            engine.master && bare,
            "the master goes once, after every voice"
        );
        engine.master = false;
        self.alive = some(self.alive.checked_sub(1), "a part goes once")?;
        Ok(())
    }

    /// The engine goes last; one gone without a master was a build that failed on a master fault.
    pub(super) fn gone_engine(&mut self) -> Result<(), TestCaseError> {
        let engine = some(self.engine.take(), "an engine goes once")?;
        prop_assert!(!engine.master, "the engine goes after the master");
        self.alive = some(self.alive.checked_sub(1), "a part goes once")?;
        if !engine.mastered {
            let fault = some(
                self.faults.masters.pop_front(),
                "a build fails on a master fault",
            )?;
            let broke = (self.trace.tries.len(), fault);
            prop_assert!(
                self.trace.broke.replace(broke).is_none(),
                "one failed build per play"
            );
        }
        Ok(())
    }
}
