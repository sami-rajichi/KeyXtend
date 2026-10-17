//! What the actor publishes: the state, the outline, a stop and the notice that says why a tap cannot scroll.

use super::Actor;
use crate::{Halt, ID};
use kx_module_api::ports::{Aim, ScrollAimed};
use kx_module_api::{Event, Notice, NoticeArg, args};

impl Actor {
    /// Ends the scroll, by the user for none or by `halt`: it tells the state, keeps the outline only while the pointer
    /// is on the pad, then says why if the halt is told.
    pub(super) fn end(&mut self, halt: Option<Halt>) {
        self.pad.stop();
        self.scroll = None;
        self.stats.stopped(halt);
        self.tell_state();
        if !self.peek {
            self.hide();
        }
        if let Some(halt) = halt {
            self.say(halt);
        }
    }

    /// A tap that could not start: the cause is counted and the user hears why, if the halt is told.
    pub(super) fn refuse(&mut self, halt: Halt) {
        self.stats.stopped(Some(halt));
        self.say(halt);
    }

    /// Takes `aim` as the scroll's target, and moves the outline when the target or its outline changed.
    pub(super) fn follow(&mut self, aim: Aim) {
        if let Some(scroll) = &mut self.scroll {
            scroll.aim = Some(aim);
        }
        if self.shown != Some((aim.id, aim.into())) {
            self.outline(aim);
        }
    }

    /// Outlines `aim`'s box, saying whether it is ours.
    pub(super) fn outline(&mut self, aim: Aim) {
        let aimed = ScrollAimed::from(aim);
        self.shown = Some((aim.id, aimed));
        self.publish(&aimed);
    }

    /// Hides the outline.
    pub(super) fn hide(&mut self) {
        self.shown = None;
        self.publish(&ScrollAimed::NONE);
    }

    /// The pointer went onto the pad or off it: while still, the outline shows what a tap would scroll, or hides.
    pub(super) fn peek(&mut self, on: bool) {
        self.peek = on;
        if self.pad.scrolling() {
            return;
        }
        match on.then(|| self.find()).flatten() {
            Some(Ok(aim)) => self.outline(aim),
            _ => self.hide(),
        }
    }

    /// Tells the state and the outline again, for a module or the UI that started later.
    pub(super) fn ask(&self) {
        self.tell_state();
        let aimed = self.shown.map_or(ScrollAimed::NONE, |(_, aimed)| aimed);
        self.publish(&aimed);
    }

    /// Publishes the live state.
    pub(super) fn tell_state(&self) {
        self.publish(&self.pad.changed());
    }

    /// Publishes `event`; none of the actor's events needs a capability.
    pub(super) fn publish<E: Event>(&self, event: &E) {
        let _ = self.bus.publish(event);
    }

    /// Tells the user why a tap could not scroll; other halts are quiet.
    fn say(&self, halt: Halt) {
        if let Some(key) = halt.key() {
            self.publish(&notice(key));
        }
    }
}

/// The scroll module's notice `key`, naming the module.
pub(crate) fn notice(key: &'static str) -> Notice {
    Notice {
        key,
        module: Some(ID),
        args: vec![(args::MODULE, NoticeArg::Text(ID.to_string()))],
    }
}
