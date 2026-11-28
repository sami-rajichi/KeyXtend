//! The keyboard beside a capture (P12): it steps aside as its bubble or hidden while every window of ours stays out of
//! every capture (`shields`), tells the module once that is done so the screens may freeze, and comes back as it was
//! once the capture ends or the editor opens (edges 16 and 17).

mod shields;

use super::{Hub, Mode, Wake};
use kx_module_api::ports::{SnipAside, SnipClick};
use serde::Serialize;

/// How the keyboard steps aside, as QML's mode names it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Way {
    /// It turns into its bubble.
    Bubble,
    /// It and its bubble hide.
    Hidden,
}

/// The keyboard aside: how, the mode it left, and whether a shield of ours was refused.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Aside {
    way: Way,
    before: Mode,
    bare: bool,
}

impl Aside {
    /// How QML shows it: hidden once a shield was refused, so it is never in a picture or video.
    pub fn shown(self) -> Way {
        if self.bare { Way::Hidden } else { self.way }
    }

    /// The mode the aside itself makes: QML minimises a shown keyboard into its bubble.
    fn made(self) -> Mode {
        match (self.way, self.before) {
            (Way::Bubble, Mode::Shown) => Mode::Minimised,
            _ => self.before,
        }
    }
}

impl Hub {
    /// Steps the keyboard aside as the module says, or brings it back: the mode the aside made goes back to the one
    /// before, and one the user chose meanwhile stays. A stay leaves it as it is. QML hears `Wake::Aside`.
    pub(super) fn step_aside(&self, aside: SnipAside) {
        let moved = {
            let mut live = self.lock();
            live.stay = aside == SnipAside::Stay;
            let mode = live.mode;
            let way = match aside {
                SnipAside::Bubble => Some(Way::Bubble),
                SnipAside::Hidden => Some(Way::Hidden),
                SnipAside::Stay | SnipAside::Back => None,
            };
            let last = live.aside.take();
            live.aside = way.map(|way| Aside {
                way,
                before: last.map_or(mode, |a| a.before),
                bare: false,
            });
            if let Some(last) = last.filter(|a| way.is_none() && a.made() == mode) {
                live.mode = last.before;
            }
            live.mode != mode
        };
        if moved {
            self.tell_board();
        }
        self.wake(Wake::Aside);
    }

    /// The Qt thread's answer once QML stepped the keyboard aside at `Wake::Aside`, or after a stay's wake: the module
    /// may freeze the screens. It tells nothing else unless that wake's `shield_aside` `shielded` for an aside still on.
    pub fn aside_shown(&self, shielded: bool) {
        let stay = std::mem::take(&mut self.lock().stay);
        if stay || (shielded && self.aside().is_some()) {
            self.snip_now(SnipClick::Aside);
        }
    }

    /// How the keyboard is aside, as QML shows it, or `None` while it is not.
    pub(super) fn aside(&self) -> Option<Way> {
        self.lock().aside.map(Aside::shown)
    }

    /// The user's mode and how the keyboard is aside, read together.
    pub(super) fn mode_aside(&self) -> (Mode, Option<Way>) {
        let live = self.lock();
        (live.mode, live.aside.map(Aside::shown))
    }

    /// Wakes the new windows with `Wake::Aside` while the keyboard is aside, so they step aside too.
    pub(super) fn rewake_aside(&self) {
        if self.aside().is_some() {
            self.wake(Wake::Aside);
        }
    }
}
