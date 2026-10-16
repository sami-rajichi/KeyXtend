//! Runs steps on a mixer over the fake system as the speaker thread does, and has the model check each one.

use kx_module_api::Mono;
use proptest::prelude::*;

use super::super::fake::World;
use super::model::{Ask, Model};
use super::{SILENCE_MS, Step, millis};
use crate::sound::Mixer;

/// The mixer, the fake it calls, the model that checks it, and the test clock.
pub(super) struct Sim {
    mixer: Mixer<World>,
    world: World,
    model: Model,
    now: Mono,
}

impl Sim {
    pub(super) fn new() -> Self {
        let world = World::default();
        Self {
            mixer: Mixer::new(world.clone()),
            world,
            model: Model::default(),
            now: Mono::default(),
        }
    }

    /// How many plays the port takes sounded nothing.
    pub(super) fn dropped(&self) -> usize {
        self.model.dropped
    }

    /// Takes each step after its gap.
    pub(super) fn run(&mut self, steps: &[(u32, Step)]) -> Result<(), TestCaseError> {
        for (gap, step) in steps {
            self.pass(*gap)?;
            self.step(step)?;
        }
        Ok(())
    }

    /// Lets `gap` ms pass. The thread wakes when the idle stop is due, as its timer does, and the engine stops then.
    fn pass(&mut self, gap: u32) -> Result<(), TestCaseError> {
        let end = self.now.saturating_add(millis(gap));
        let due = self
            .mixer
            .wait(self.now)
            .map(|w| self.now.saturating_add(w));
        if let Some(due) = due.filter(|&due| due <= end) {
            self.tick(due);
            self.mixer.idle(due);
            self.check(&Ask::Wake)?;
        }
        self.tick(end);
        Ok(())
    }

    /// Takes `step`, then idles as the thread does after each ask; a fault only queues.
    fn step(&mut self, step: &Step) -> Result<(), TestCaseError> {
        step.apply(&mut self.mixer, &self.world, self.now);
        let ask = match step {
            Step::Play(id, gain) => Ask::Play(*id, *gain),
            Step::Prepare(clips, config) => Ask::Prepare(clips.clone(), *config),
            Step::Fault(queue, fault, n) => {
                self.model.faults.add(*queue, *fault, *n);
                return Ok(());
            }
        };
        self.mixer.idle(self.now);
        self.check(&ask)
    }

    /// Has the model check the calls since the last check, then the timer.
    fn check(&mut self, ask: &Ask) -> Result<(), TestCaseError> {
        self.model.begin(ask, self.now);
        for call in self.world.take() {
            self.model.on(&call)?;
        }
        self.model.settle(ask)?;
        let (theirs, ours) = (self.mixer.wait(self.now), self.model.wait(self.now));
        prop_assert_eq!(theirs, ours, "the timer runs to the idle stop");
        Ok(())
    }

    /// Moves the clock to `now`; the fake's voices whose clip ended stop playing.
    fn tick(&mut self, now: Mono) {
        self.now = now;
        let model = &self.model;
        self.world.state().busy.retain(|&v| model.busy(v, now));
    }

    /// Lets a long silence pass, then drops the mixer: the engine stopped, and every part made is gone in order.
    pub(super) fn end(mut self) -> Result<(), TestCaseError> {
        self.pass(SILENCE_MS)?;
        prop_assert_eq!(
            self.mixer.wait(self.now),
            None,
            "a long silence stops the engine"
        );
        let Self {
            mixer,
            world,
            mut model,
            now,
        } = self;
        drop(mixer);
        model.begin(&Ask::Drop, now);
        for call in world.take() {
            model.on(&call)?;
        }
        model.settle(&Ask::Drop)
    }
}
