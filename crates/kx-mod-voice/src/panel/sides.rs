//! The Models page's Local | Cloud sides (edge 29), each paging alone, and Use on a cloud model, the one act that moves
//! a language to a service.

use kx_module_api::ports::{ModelSide, VoiceTab};

use super::cloud::edit;
use super::{Do, Key, Page, Panel};
use crate::view::Facts;

impl Panel {
    /// Shows the Models page's `side`; each side keeps the page it was left on, and the ticks go.
    pub(super) fn side(&mut self, side: ModelSide) -> Vec<Do> {
        if self.page != Some(Page::Tab(VoiceTab::Models)) || self.cloud.side.0 == side {
            return Vec::new();
        }
        let left = std::mem::replace(&mut self.at, self.cloud.side.1);
        self.cloud.side = (side, left);
        self.ticks.clear();
        self.hold = None;
        Vec::new()
    }

    /// The Models page's side shown.
    pub(crate) fn models_side(&self) -> ModelSide {
        self.cloud.side.0
    }

    /// The ticked cloud model's service and pick id, on a page that ticks one.
    pub(crate) fn cloud_ticked(&self) -> Option<(String, String)> {
        match self.ticked() {
            Some(Key::Cloud { service, model }) => Some((service.clone(), model.clone())),
            _ => None,
        }
    }

    /// Use on service `service`'s `model` for the language in front, while the rules take it.
    pub(super) fn cloud_pick(&mut self, service: &str, model: &str, facts: &Facts<'_>) -> Vec<Do> {
        self.ticks.clear();
        let iso = facts.iso().to_owned();
        edit(facts, |c, s| c.pick(s, &iso, service, model))
    }
}
