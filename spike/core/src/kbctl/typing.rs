//! What a click on a key sends: text, a key press or a latch, and dead keys that wait for the next key.

use kx_mod_keyboard::Role;
use kx_module_api::ports::LayoutSource;
use kx_platform_windows::{WinLayouts, layouts, window};

use super::Kb;
use crate::config::ToolButton;
use crate::facecfg::Action;
use crate::kbview::{chars_of, legend_mods};
use crate::latch::Latch;
use crate::legend::{LegendMods, Slot, slot};
use crate::{langkey, layout, sendkeys};

/// Error when a chip index is past the list.
const NO_CHIP: &str = "no such chip";

/// What a click on a key sends or changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Plan {
    /// The key's scan code, with every latched modifier held around it.
    Chord(u32),
    /// A dead key's scan code, so the app's layout adds the accent to the next key; the listed latches are not held.
    Dead(u32, Vec<Latch>),
    /// A character key's scan code, which ends a dead key's wait; the listed latches are not held.
    Press(u32, Vec<Latch>),
    /// Text typed as Unicode; Shift and AltGr are already in it.
    Text(String),
    /// Caps Lock, which never takes the latches.
    Caps(u32),
    /// A modifier to latch or release.
    Latch(Latch, u32),
    /// The language key, forward or back.
    Lang(bool),
    /// A side key's action.
    Side(Action),
    /// Nothing, for an unknown key.
    Nothing,
}

/// What the face must do after a click.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tapped {
    /// Only redraw.
    Done,
    /// Show this note.
    Note(String),
    /// Run this tool.
    Tool(ToolButton),
    /// Open the Arabic test panel.
    Panel,
}

impl Kb {
    /// What a click on key `id` does, from its board role, with Caps Lock `caps`; `back` is the language key's left third.
    /// Side keys wait for a later version in the product; the spike keeps their test actions.
    pub fn plan(&mut self, id: &str, caps: bool, back: bool) -> Plan {
        let Some(k) = self.board().keys.into_iter().find(|k| k.id == id) else {
            return Plan::Nothing;
        };
        self.load_chars();
        let action = self.cfg.keys.get(id).and_then(|l| l.action);
        match (k.role, k.sc.map(|s| u32::from(s.code()))) {
            (Role::Lang, _) => Plan::Lang(back),
            (Role::Later, _) => action.map_or(Plan::Nothing, Plan::Side),
            (Role::Char, Some(sc)) => self.plan_char(sc, caps),
            (Role::Caps, Some(sc)) => Plan::Caps(sc),
            (Role::Latch(m), Some(sc)) => Plan::Latch(m, sc),
            (Role::Send, Some(sc)) => Plan::Chord(sc),
            _ => Plan::Nothing,
        }
    }

    /// A character key: a shortcut with Ctrl, Alt or Win, a key press around a dead key, else its text.
    fn plan_char(&self, sc: u32, caps: bool) -> Plan {
        let all = self.chars_here();
        let (c, m) = (chars_of(all, sc), legend_mods(&self.latches, caps, all));
        let on = |l| self.latches.is_on(l);
        let alt_like = on(Latch::AltGr) && !m.altgr;
        if on(Latch::Ctrl) || on(Latch::Alt) || on(Latch::Win) || alt_like {
            return Plan::Chord(sc);
        }
        let s = slot(&c, m);
        if c.is_dead(s) {
            return Plan::Dead(sc, unheld(s, m));
        }
        let text = c.get(s);
        if self.dead || !layouts::is_printable(text) {
            Plan::Press(sc, unheld(s, m))
        } else {
            Plan::Text(text.to_string())
        }
    }

    /// Notes that `p` ran: a dead key waits until a key that types something, as Windows does.
    pub(super) fn mark(&mut self, p: &Plan) {
        match p {
            Plan::Dead(..) => self.dead = true,
            Plan::Press(..) | Plan::Text(_) => self.dead = false,
            Plan::Chord(sc) if self.dead => {
                let key = layout::scan(*sc);
                self.dead = !WinLayouts.types_any(self.layout_id(), key, &[]);
            }
            _ => {}
        }
    }

    /// Runs `p`: sends keys, latches or toggles, and says what the face must do next.
    pub fn run(&mut self, id: &str, p: Plan) -> Result<Tapped, String> {
        self.mark(&p);
        match p {
            Plan::Chord(sc) => sendkeys::chord(&self.latches.take(), sc).map(|()| Tapped::Done),
            Plan::Dead(sc, skip) | Plan::Press(sc, skip) => {
                sendkeys::chord(&self.latches.take_but(&skip), sc).map(|()| Tapped::Done)
            }
            Plan::Text(t) => {
                self.latches.take();
                sendkeys::text(&t).map(|()| Tapped::Done)
            }
            Plan::Caps(sc) => sendkeys::tap_scan(sc).map(|()| Tapped::Done),
            Plan::Latch(l, sc) => {
                self.latches.toggle(l, sc);
                Ok(Tapped::Done)
            }
            Plan::Lang(back) => langkey::ask_step(window::foreground(), back).map(|_| {
                self.asked = if back { -1 } else { 1 };
                Tapped::Done
            }),
            Plan::Side(a) => Ok(self.side(id, a)),
            Plan::Nothing => Ok(Tapped::Done),
        }
    }

    /// Types chip `i`'s text, releasing the latches.
    pub fn chip(&mut self, i: usize) -> Result<(), String> {
        let chip = self.cfg.bar.chips.get(i).ok_or(NO_CHIP)?;
        let text = chip.text.clone();
        self.latches.take();
        sendkeys::text(&text)
    }

    /// A side key's action.
    fn side(&mut self, id: &str, a: Action) -> Tapped {
        match a {
            Action::Mic => Tapped::Tool(ToolButton::Mic),
            Action::Snip => Tapped::Tool(ToolButton::Snip),
            Action::Fill => Tapped::Tool(ToolButton::FillPassword),
            Action::Panel => Tapped::Panel,
            Action::Note => Tapped::Note(
                self.cfg
                    .keys
                    .get(id)
                    .and_then(|l| l.note.clone())
                    .unwrap_or_default(),
            ),
            Action::Toggle => {
                if !self.toggled.remove(id) {
                    self.toggled.insert(id.to_string());
                }
                Tapped::Done
            }
        }
    }
}

/// The latches slot `s` ignores, so a key press gives what its legend shows.
fn unheld(s: Slot, m: LegendMods) -> Vec<Latch> {
    let shifted = matches!(s, Slot::Shift | Slot::CapsShift);
    [
        (Latch::AltGr, m.altgr && s != Slot::AltGr),
        (Latch::Shift, m.shift && !shifted),
    ]
    .into_iter()
    .filter_map(|(l, drop)| drop.then_some(l))
    .collect()
}
