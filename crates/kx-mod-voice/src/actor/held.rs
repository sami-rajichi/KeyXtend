//! What the actor holds beside the flow and the panel: `[voice]`, the layout in front, the drives and devices as last
//! read, the clips' languages, the checks, Compare's numbers, History and the counts. It lends them as the panel's
//! `Facts`.

use kx_module_api::ports::{Device, Drive, LayoutChanged, LayoutId, LocalTime};

use crate::VoiceSettings;
use crate::counts::{Checks, Counts, Scores};
use crate::flow::Flow;
use crate::history::History;
use crate::view::Facts;

/// The actor's facts.
pub(super) struct Held {
    pub settings: VoiceSettings,
    /// The layout the keyboard shows; an unknown one until it tells.
    pub front: LayoutChanged,
    pub drives: Vec<Drive>,
    pub devices: Vec<Device>,
    /// The check clips' languages the speech port ships, in order.
    pub clips: Vec<String>,
    pub checks: Checks,
    pub scores: Scores,
    pub history: History,
    pub counts: Counts,
}

impl Held {
    /// The facts at the start: `settings`, the clips' languages, and nothing read yet.
    pub fn new(settings: VoiceSettings, clips: Vec<String>) -> Self {
        Self {
            settings,
            front: LayoutChanged {
                id: LayoutId::default(),
                iso: String::new(),
                name: String::new(),
                rtl: false,
            },
            drives: Vec::new(),
            devices: Vec::new(),
            clips,
            checks: Checks::default(),
            scores: Scores::default(),
            history: History::default(),
            counts: Counts::default(),
        }
    }

    /// The panel's facts at `now`, with Try again and the dictation's state from `flow`.
    pub fn facts<'a>(&'a self, now: LocalTime, flow: &Flow) -> Facts<'a> {
        Facts {
            settings: &self.settings,
            front: &self.front,
            drives: &self.drives,
            devices: &self.devices,
            clips: &self.clips,
            checks: &self.checks,
            scores: &self.scores,
            history: &self.history,
            now,
            again: flow.can_again(),
            dictating: flow.dictating(),
        }
    }
}
