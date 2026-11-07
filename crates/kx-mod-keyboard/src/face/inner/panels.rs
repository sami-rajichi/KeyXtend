//! Our panels in the face's state (spec §7, §8.1): the one open, as the bus tells, the keys it takes while aimed at, and
//! whether the vault module runs.

use super::Inner;
use crate::panel::{Panel, PanelUse};
use crate::run::ToPanel;
use kx_module_api::ports::FillLive;
use std::collections::HashMap;

impl Inner {
    /// Takes a module's word that `panel` opened, with `clicks` outside clicks heard so far; the next relabel redraws a
    /// change. The one opened last takes the keys, so one shows at a time, it aims unless the word only repeats, and it
    /// closes the language list.
    pub(in crate::face) fn panel_opened(&mut self, panel: Panel, clicks: u64) {
        if self.open.replace(panel) != Some(panel) {
            self.aim_fresh(clicks);
            self.unshown = true;
        }
        if self.list.take().is_some() {
            self.unshown = true;
        }
    }

    /// Takes a module's word that `panel` closed, which ends the aim; closing another panel changes nothing.
    pub(in crate::face) fn panel_shut(&mut self, panel: Panel) {
        if self.open == Some(panel) {
            self.open = None;
            self.aim_end();
            self.unshown = true;
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

    /// Key `id` from our panels' key list `uses`, for the panel open now and aimed at, with `clicks` outside clicks
    /// heard so far: a move, or what a character key gives its search or line, after a burst left open ends as before a
    /// tap. `None` while no panel is open or aimed at, or for a key it does not take; `Pass` for a key Ctrl, Alt or Win
    /// acts on.
    pub(in crate::face) fn took(
        &mut self,
        id: &str,
        uses: &HashMap<String, PanelUse>,
        clicks: u64,
    ) -> Option<(Panel, ToPanel)> {
        let panel = self.open?;
        self.settle_aim(clicks);
        if !self.aimed() {
            return None;
        }
        let &key = uses.get(id)?;
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
