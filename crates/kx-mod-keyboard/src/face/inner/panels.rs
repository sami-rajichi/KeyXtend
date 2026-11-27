//! Our panels in the face's state (spec §7, §8.1, ADR-0057): the one open, as the bus tells, the keys it or Snip's
//! windows take while aimed at, and whether the vault module runs.

use super::Inner;
use crate::panel::{Panel, PanelUse};
use crate::run::ToPanel;
use kx_module_api::ports::FillLive;
use std::collections::HashMap;

impl Inner {
    /// Takes a module's word that `panel` opened, with `clicks` outside clicks heard so far: the one opened last shows
    /// beside Snip's windows and, unless the word only repeats, takes the keys and aims. It closes the language list,
    /// the next relabel redraws a change, and it returns true when newly opened.
    pub(in crate::face) fn panel_opened(&mut self, panel: Panel, clicks: u64) -> bool {
        let new = self.open.replace(panel) != Some(panel);
        if new {
            self.snip_leave();
            self.aim_fresh(clicks);
            self.unshown = true;
        }
        if self.list.take().is_some() {
            self.unshown = true;
        }
        new
    }

    /// Takes a module's word that `panel` closed, which ends its aim, and Snip's windows take the free place unaimed
    /// while their line shows; closing another panel changes nothing.
    pub(in crate::face) fn panel_shut(&mut self, panel: Panel) {
        if self.open == Some(panel) {
            self.open = None;
            self.aim_end();
            self.unshown = true;
            self.snip_back();
        }
    }

    /// Takes the vault module's new state: while it is not running the fill key dims and its panel counts as shut.
    pub(in crate::face) fn heard_fill(&mut self, now: FillLive) {
        if self.vault != now {
            self.vault = now;
            self.unshown = true;
        }
        if !now.live {
            self.panel_shut(Panel::Fill);
        }
    }

    /// True while the vault module runs, so the fill key asks for its panel.
    pub(in crate::face) fn vault_live(&self) -> bool {
        self.vault.live
    }

    /// Key `id` from our panels' key list `uses`, for the panel that holds the place and is aimed at, with `clicks`
    /// outside clicks heard so far: a move, or what a character key gives its search or line, after a burst left open
    /// ends as before a tap. `None` while none is aimed at, or for a key it does not take, as the Voice panel's Esc;
    /// `Pass` for a key Ctrl, Alt or Win acts on.
    pub(in crate::face) fn took(
        &mut self,
        id: &str,
        uses: &HashMap<String, PanelUse>,
        clicks: u64,
    ) -> Option<(Panel, ToPanel)> {
        let panel = self.place()?;
        self.settle_aim(clicks);
        if !self.aimed() {
            return None;
        }
        let &key = uses.get(id)?;
        if !self.takes(panel, key) {
            return None;
        }
        self.ended();
        let table = self.ports.layouts.current();
        let cells = self.layer.cells();
        let io = self.ports.io(&table, &cells);
        let given = match key {
            PanelUse::Nav(nav) => self.typing.moved(&io, nav),
            PanelUse::Char(sc) => self.typing.search(&io, sc),
        };
        Some((panel, given))
    }
}
