//! The language key in its three designs (spec §4.3, mock-up `langInner`): the Carousel steps back on its left
//! part and on elsewhere, the Cycle steps on, and the List opens the language list. Steps count as `step_to` does,
//! and with one layout installed no design moves, though an open list still closes.

use crate::settings::LangKey;
use kx_module_api::ports::{LangLabel, LayoutId, step_to};
use serde::Serialize;

/// What a click on the language key asks for.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LangClick {
    /// Step this many places along the list.
    Step(i32),
    /// Open or close the language list.
    List,
    /// Nothing: there is no other layout to go to, and the key's tip says so.
    Still,
}

/// True when `count` installed layouts leave the key nothing to switch to.
#[must_use]
pub fn lone(count: usize) -> bool {
    count <= 1
}

/// The click on a language key of design `style`, on its back part when `back`, with `count` layouts installed and
/// the list `open`; an open list closes even when a refresh left one layout or none.
#[must_use]
pub fn click(style: LangKey, back: bool, count: usize, open: bool) -> LangClick {
    match style {
        _ if open => LangClick::List,
        _ if lone(count) => LangClick::Still,
        LangKey::Carousel if back => LangClick::Step(-1),
        LangKey::Carousel | LangKey::Cycle => LangClick::Step(1),
        LangKey::List => LangClick::List,
    }
}

/// One layout's name as the key draws it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct LangName {
    /// Such as `EN` or `ع`.
    pub short: String,
    /// Written right to left.
    pub rtl: bool,
    /// Needs the Arabic font.
    pub arabic: bool,
}

/// The language key and the space bar now.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct LangView {
    /// The layout a step back lands on; none when it is the current one.
    pub prev: Option<LangName>,
    /// The layout shown.
    pub cur: LangName,
    /// The layout a step on lands on; none when it is the current one.
    pub next: Option<LangName>,
    /// The current layout's place in the list, which lights its Cycle dot.
    pub at: Option<usize>,
    /// How many layouts there are: one Cycle dot each.
    pub count: usize,
    /// The space bar's text parts, the language then the layout; the UI joins them.
    pub space: Vec<String>,
    /// Which way the Carousel names turned on the last change: -1 from the left, 1 from the right, 0 not at all.
    pub turn: i8,
}

/// The key's view with `cur` shown among `list`, the names having turned `turn`.
#[must_use]
pub fn lang_view(list: &[LangLabel], cur: &LangLabel, turn: i8) -> LangView {
    let ids: Vec<LayoutId> = list.iter().map(|l| l.id).collect();
    let near = |by: i32| {
        let to = step_to(&ids, Some(cur.id), by).filter(|&id| id != cur.id)?;
        list.iter().find(|l| l.id == to).map(name)
    };
    LangView {
        prev: near(-1),
        cur: name(cur),
        next: near(1),
        at: ids.iter().position(|&id| id == cur.id),
        count: list.len(),
        space: cur.name.clone(),
        turn,
    }
}

/// How the key draws layout `l`'s name.
fn name(l: &LangLabel) -> LangName {
    LangName {
        short: l.short.clone(),
        rtl: l.rtl,
        arabic: l.arabic,
    }
}

/// Which way the Carousel names turn: the step the key asked, used once when the switch lands.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Turn {
    shown: LayoutId,
    asked: i32,
    /// -1 from the left, 1 from the right, 0 not at all.
    way: i8,
}

impl Turn {
    /// Layout `shown` is on the key, and nothing turned.
    #[must_use]
    pub fn new(shown: LayoutId) -> Self {
        Self {
            shown,
            ..Self::default()
        }
    }

    /// The key asked for a step of `by`.
    pub fn asked(&mut self, by: i32) {
        self.asked = by;
    }

    /// Forgets the step asked, as when a layout is picked from the list.
    pub fn clear(&mut self) {
        self.asked = 0;
    }

    /// The layout shown is now `now`, in the list `ids`; true when it changed.
    pub fn seen(&mut self, now: LayoutId, ids: &[LayoutId]) -> bool {
        let was = std::mem::replace(&mut self.shown, now);
        if was == now {
            return false;
        }
        let by = std::mem::take(&mut self.asked);
        let landed = by != 0 && step_to(ids, Some(was), by) == Some(now);
        self.way = if landed {
            by.signum().try_into().unwrap_or_default()
        } else {
            0
        };
        true
    }

    /// Which way the names turned on the last change.
    #[must_use]
    pub fn turn(&self) -> i8 {
        self.way
    }
}

#[cfg(test)]
mod tests;
