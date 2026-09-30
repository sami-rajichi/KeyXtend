//! A click on a key: kx-mod-keyboard's `Typing` plans and sends it (P3); the spike adds the language step,
//! chips and its side-key test actions, which the product leaves to later versions.

use kx_mod_keyboard::Io;
use kx_mod_keyboard::Tapped as Typed;
use kx_platform_windows::{WinInjector, WinLayouts, window};

use super::Kb;
use crate::config::ToolButton;
use crate::facecfg::Action;
use crate::{langkey, sendkeys};

/// Error when a chip index is past the list.
const NO_CHIP: &str = "no such chip";

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
    /// A click on key `id`, on the language key's left third when `back`: the product types it, the spike steps
    /// the language and runs the side keys' test actions.
    pub fn tap(&mut self, id: &str, back: bool) -> Result<Tapped, String> {
        let table = self.table();
        let io = Io {
            injector: &WinInjector,
            source: &WinLayouts,
            table: &table,
        };
        match self.typing.tap(&io, id, back).map_err(|e| e.to_string())? {
            Typed::Done => Ok(Tapped::Done),
            Typed::Lang { back } => langkey::ask_step(window::foreground(), back).map(|_| {
                self.asked = if back { -1 } else { 1 };
                Tapped::Done
            }),
            Typed::Later => {
                let action = self.cfg.keys.get(id).and_then(|l| l.action);
                Ok(action.map_or(Tapped::Done, |a| self.side(id, a)))
            }
        }
    }

    /// Types chip `i`'s text; chips come to the product in P7.
    pub fn chip(&mut self, i: usize) -> Result<(), String> {
        let chip = self.cfg.bar.chips.get(i).ok_or(NO_CHIP)?;
        let text = chip.text.clone();
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
