//! When the strip shows, moves and hides (P10 design "When the chips show" and "When the chips hide"): one state
//! machine over the looks and the user's acts, on `Mono` time only. Each new strip gets a new round, so a click on an
//! old strip does nothing.

mod input;

pub use input::{Board, Input, Pick, StripConfig};

use kx_module_api::Mono;
use kx_module_api::ports::{ChipsView, SelChip, SelKind, SelSpot, StripSpot};

use crate::chips::{At, chips};

/// A selection by its box and its lines, as Esc put it away.
type Mark = (u64, StripSpot);

/// A strip to show, before it has a round.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Draft {
    /// The keycaps, in `SelChip::ALL`'s order.
    chips: Vec<SelChip>,
    /// Where it goes.
    spot: StripSpot,
    /// The box it acts on.
    box_id: u64,
    /// That box's kind.
    kind: SelKind,
}

/// The strip shown.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Live {
    /// Its round.
    round: u32,
    /// What it shows.
    draft: Draft,
}

impl Live {
    /// The view of this strip, with `acted`'s tick if any.
    fn view(&self, acted: Option<SelChip>) -> ChipsView {
        ChipsView::Shown {
            round: self.round,
            chips: self.draft.chips.clone(),
            spot: self.draft.spot,
            acted,
        }
    }

    /// True when the strip of `round` shows `chip`.
    fn shows(&self, round: u32, chip: SelChip) -> bool {
        self.round == round && self.draft.chips.contains(&chip)
    }
}

/// The caret's Paste while it lasts; its box shows in `Debug` only as redacted.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Caret {
    /// The editable box clicked into.
    box_id: u64,
    /// The caret's box, twice, and its screen.
    spot: StripSpot,
    /// When it ends.
    until: Mono,
}

impl Caret {
    /// True while `spot` shows the same bare caret in the same editable box.
    fn holds(&self, spot: &SelSpot) -> bool {
        let same = spot.box_id == self.box_id && spot.caret == Some(self.spot.first);
        same && spot.kind == SelKind::Editable && !spot.selected
    }
}

/// The strip's state: what the last look saw, the caret's Paste, what the user put away, and the strip shown.
#[derive(Clone, Debug, Default)]
pub struct Strip {
    /// The settings it follows.
    cfg: StripConfig,
    /// Where the keyboard is.
    board: Board,
    /// A panel line is aimed at.
    aimed: bool,
    /// The last look, until one fails.
    last: Option<SelSpot>,
    /// The caret's Paste while it lasts.
    caret: Option<Caret>,
    /// The selection Esc put away; it stays hidden until a look sees another or a press comes.
    dismissed: Option<Mark>,
    /// The strip shown.
    live: Option<Live>,
    /// The last round given.
    round: u32,
}

impl Strip {
    /// A hidden strip that follows `cfg`.
    #[must_use]
    pub fn new(cfg: StripConfig) -> Self {
        Self {
            cfg,
            ..Self::default()
        }
    }

    /// Takes `input` at `now`; the view to send when it changed.
    pub fn step(&mut self, input: Input, now: Mono) -> Option<ChipsView> {
        match input {
            Input::Look { spot, click, clip } => self.look(spot, click, clip, now),
            Input::Lost => (self.last, self.caret) = (None, None),
            Input::Key => self.caret = None,
            // A press starts a new choice: the next look decides, even for the same text.
            Input::Press => (self.last, self.caret, self.dismissed) = (None, None, None),
            Input::Esc => (self.dismissed, self.caret) = (self.last.and_then(mark), None),
            Input::Aimed(on) => self.aimed = on,
            Input::Board(board) => self.board = board,
            Input::Acted { round, chip } => return self.acted(round, chip),
            Input::Tick => {}
        }
        self.settle(now)
    }

    /// Follows `cfg` from `now`; the view to send when it changed.
    pub fn configure(&mut self, cfg: StripConfig, now: Mono) -> Option<ChipsView> {
        self.cfg = cfg;
        self.settle(now)
    }

    /// The click of `chip` on the strip of `round`, when that strip is live and shows it.
    #[must_use]
    pub fn pick(&self, round: u32, chip: SelChip) -> Option<Pick> {
        let live = self.live.as_ref().filter(|l| l.shows(round, chip))?;
        Some(Pick {
            chip,
            box_id: live.draft.box_id,
            kind: live.draft.kind,
        })
    }

    /// When the caret's Paste ends, for the caller's wait.
    #[must_use]
    pub fn ends(&self) -> Option<Mono> {
        self.caret.map(|c| c.until)
    }

    /// The strip shown now.
    #[must_use]
    pub fn view(&self) -> ChipsView {
        self.live
            .as_ref()
            .map_or(ChipsView::Hidden, |l| l.view(None))
    }

    /// Notes a look: a new selection forgets the one put away, a moved caret ends its Paste, and a click into an
    /// editable box with something to paste starts one.
    fn look(&mut self, spot: SelSpot, click: bool, clip: bool, now: Mono) {
        if self.dismissed != mark(spot) {
            self.dismissed = None;
        }
        if self.caret.is_some_and(|c| !c.holds(&spot)) {
            self.caret = None;
        }
        let pastes = !chips(&self.cfg.rules, spot.kind, At::Caret { clip }).is_empty();
        if click && pastes && !spot.selected {
            let until = now.saturating_add(self.cfg.caret);
            self.caret = spot.caret.map(|at| Caret {
                box_id: spot.box_id,
                spot: StripSpot {
                    first: at,
                    last: at,
                    screen: spot.screen,
                },
                until,
            });
        }
        self.last = Some(spot);
    }

    /// Shows the tick of `chip` on the live strip of `round` and ends that strip, sending no `Hidden`, as the ui fades
    /// a ticked strip itself; the next look decides what shows, even the same selection.
    fn acted(&mut self, round: u32, chip: SelChip) -> Option<ChipsView> {
        let live = self.live.take_if(|l| l.shows(round, chip))?;
        (self.last, self.caret) = (None, None);
        Some(live.view(Some(chip)))
    }

    /// Ends a caret's Paste that is over or blocked, then shows what is due, if it changed.
    fn settle(&mut self, now: Mono) -> Option<ChipsView> {
        if self.blocked() || self.caret.is_some_and(|c| now >= c.until) {
            self.caret = None;
        }
        let target = if self.blocked() { None } else { self.due() };
        if self.live.as_ref().map(|l| &l.draft) == target.as_ref() {
            return None;
        }
        let Some(draft) = target else {
            self.live = None;
            return Some(ChipsView::Hidden);
        };
        self.round = self.round.saturating_add(1);
        let live = Live {
            round: self.round,
            draft,
        };
        let view = live.view(None);
        self.live = Some(live);
        Some(view)
    }

    /// Nothing shows: the helper is off, a panel line is aimed at, or the keyboard is away.
    fn blocked(&self) -> bool {
        let away = match self.board {
            Board::Open => false,
            Board::Minimised => !self.cfg.minimised,
            Board::Tray => true,
        };
        !self.cfg.rules.on || self.aimed || away
    }

    /// The strip due: the last look's selection unless put away, else the caret's Paste.
    fn due(&self) -> Option<Draft> {
        let rules = &self.cfg.rules;
        let selection = self.last.and_then(|spot| {
            let line = mark(spot).filter(|m| self.dismissed != Some(*m))?;
            let chips = chips(rules, spot.kind, At::Selection);
            let draft = Draft {
                chips,
                spot: line.1,
                box_id: spot.box_id,
                kind: spot.kind,
            };
            (!draft.chips.is_empty()).then_some(draft)
        });
        selection.or_else(|| {
            let caret = self.caret?;
            let at = chips(rules, SelKind::Editable, At::Caret { clip: true });
            (!at.is_empty()).then_some(Draft {
                chips: at,
                spot: caret.spot,
                box_id: caret.box_id,
                kind: SelKind::Editable,
            })
        })
    }
}

/// The selection `spot` shows, by its box and lines; `None` with no text or no line box. One box told serves as both.
fn mark(spot: SelSpot) -> Option<Mark> {
    let first = spot.first.or(spot.last)?;
    let last = spot.last.or(spot.first)?;
    let line = StripSpot {
        first,
        last,
        screen: spot.screen,
    };
    spot.selected.then_some((spot.box_id, line))
}

#[cfg(test)]
mod props;
#[cfg(test)]
mod tests;
