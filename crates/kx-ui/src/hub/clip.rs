//! The clipboard panel on the hub (spec §7, ADR-0034): while the module's panel is open it keeps the last `ClipView`
//! and wakes QML once per read, sends clicks and the ui's rows as `ClipAct` through a handle limited to the `ui`
//! module's capabilities, serves the kept round's pictures, and shows the history folder. Nothing here logs a title, a
//! search, a path or a pixel.

pub mod acts;
mod view;

use super::unread::Unread;
use super::{Hub, NULL, Wake, json};
use crate::ui_module::MANIFEST;
use acts::{Dropped, act_of};
use kx_module_api::ports::{ClipAct, ClipPanel, ClipView, Pixels, Preview};
use kx_module_api::{Bus, EventError, Subscription};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

/// The picture provider's name: QML draws `image://clip/<round>/<id>` and `image://clip/<round>/preview`.
pub(crate) const PICTURES: &str = "clip";
/// The picture id that names the preview's image.
const PREVIEW: &str = "preview";
/// What parts a picture id: the round, then the row's id or `PREVIEW`.
const PART: char = '/';
/// The thread that waits on the file manager.
const REVEAL_THREAD: &str = "kx-reveal";

/// The panel's side of the hub: what it shows, whether a wake waits for QML's read, the history's folder and whether
/// a reveal of it is in flight.
#[derive(Default)]
pub(super) struct Clip {
    state: Mutex<State>,
    unread: Unread,
    /// The history's folder, which the app hands over once.
    folder: OnceLock<PathBuf>,
    /// A reveal is in flight.
    revealing: Arc<AtomicBool>,
}

/// Ends the reveal in flight as it drops, even when the reveal's thread panics or never starts.
struct Revealing(Arc<AtomicBool>);

impl Drop for Revealing {
    fn drop(&mut self) {
        self.0.store(false, AtomicOrdering::SeqCst);
    }
}

/// What the panel shows, and the handle that sends its clicks.
#[derive(Default)]
struct State {
    /// The bus limited to the `ui` module's capabilities, once `listen` has it.
    bus: Option<Bus>,
    /// The module's panel is open, as its last `ClipPanel` said.
    open: bool,
    /// The last view heard while the panel was open.
    view: Option<ClipView>,
}

impl Hub {
    /// Hears the panel's state and views through a handle limited to the `ui` module's capabilities, and keeps that
    /// handle for the clicks.
    pub(super) fn hear_clip(self: &Arc<Self>, bus: &Bus) -> Result<Vec<Subscription>, EventError> {
        let gated = bus.restricted(MANIFEST.capabilities);
        let heard = vec![
            self.hear(&gated, |hub, p: &ClipPanel| hub.clip_panel(p.open))?,
            self.hear(&gated, |hub, v: &ClipView| hub.clip_view(v.clone()))?,
        ];
        gated.check::<ClipAct>()?;
        self.clip().bus = Some(gated);
        Ok(heard)
    }

    /// The panel as JSON for QML, every text worded, or `null` while it is closed or has no view yet; reading it lets
    /// the next change wake again. It holds copied text, so never log it.
    #[must_use]
    pub fn clip_json(&self) -> String {
        self.clip.unread.read();
        let view = self.clip().view.clone();
        view.map_or_else(|| NULL.to_owned(), |v| json(&view::of(&v.0, &self.i18n())))
    }

    /// Sends the click `kind` names (one of `acts`) with `arg` on the view of round `round`, and sounds as a key does.
    /// A tick goes only while that view is kept; a bad click sends nothing and warns. Call it on the Qt thread.
    pub fn clip_act(&self, kind: &str, arg: i32, round: i64) {
        let shown = self
            .clip()
            .view
            .as_ref()
            .map(|v| (v.0.round, v.0.rows.len()));
        match act_of(kind, arg, round, shown) {
            Ok(act) => self.send_clip(act),
            Err(Dropped::Stale) => {
                tracing::debug!(kind, "a click on an older clipboard view was dropped");
            }
            Err(Dropped::Bad) => tracing::warn!(
                kind,
                arg,
                "QML sent an unknown clipboard click or a bad argument"
            ),
        }
        self.tapped();
    }

    /// Takes the history's folder from the app, once; a second one is a bug, so it warns and the first stays.
    pub fn set_clip_folder(&self, folder: PathBuf) {
        if self.clip.folder.set(folder).is_err() {
            tracing::warn!("the clipboard folder was handed over twice, so the first stays");
        }
    }

    /// Shows the history's folder in the file manager on a thread of its own, and sounds as a key does; a tap while
    /// one shows is ignored, so a double tap opens one window. True when a reveal started; call it on the Qt thread.
    pub fn open_folder(&self) -> bool {
        self.tapped();
        let Some(folder) = self.clip.folder.get().cloned() else {
            tracing::warn!("the clipboard folder is not known yet, so it cannot open");
            return false;
        };
        if self.clip.revealing.swap(true, AtomicOrdering::SeqCst) {
            return false;
        }
        let (reveal, done) = (
            Arc::clone(&self.ports.reveal),
            Revealing(Arc::clone(&self.clip.revealing)),
        );
        let shown = std::thread::Builder::new()
            .name(REVEAL_THREAD.to_owned())
            .spawn(move || {
                let _done = done;
                if let Err(e) = reveal.show(&folder) {
                    tracing::warn!(error = %e, "the clipboard folder did not open");
                }
            });
        shown
            .inspect_err(|e| tracing::warn!(error = %e, "the clipboard folder did not open, as no thread started"))
            .is_ok()
    }

    /// The picture `id` names, `<round>/<row id>` or `<round>/preview`, from the kept view of that round only.
    #[must_use]
    pub fn clip_picture(&self, id: &str) -> Option<Pixels> {
        let (round, at) = id.split_once(PART)?;
        let round: u32 = round.parse().ok()?;
        let view = self.clip().view.clone()?;
        let page = view.0.as_ref();
        if round != page.round {
            return None;
        }
        if at == PREVIEW {
            return match &page.preview {
                Some(Preview::Image(image)) => image.clone(),
                _ => None,
            };
        }
        let id: u64 = at.parse().ok()?;
        page.rows.iter().find(|r| r.id == id)?.thumb.clone()
    }

    /// A pad page tap: it turns the open panel's page, else our open list's.
    pub(super) fn pad_page(&self, back: bool) {
        if self.clip().open {
            self.send_clip(ClipAct::Page { back });
        } else {
            self.wake(Wake::Page { back });
        }
    }

    /// Tells the module the rows a page holds, `[ui] page_rows`, while its panel is open.
    pub(super) fn clip_rows(&self) {
        if !self.clip().open {
            return;
        }
        let rows = self.lock().settings.page_rows;
        self.send_clip(ClipAct::Rows(u8::try_from(rows).unwrap_or(u8::MAX)));
    }

    /// Hands a new waker the panel's wake still waiting for a read.
    pub(super) fn rewake_clip(&self) {
        if self.clip.unread.waits() {
            self.wake(Wake::Clip);
        }
    }

    /// Takes the panel's state: an open one hears the rows, a closing one drops its view and wakes once.
    fn clip_panel(&self, open: bool) {
        let (was, gone) = {
            let mut s = self.clip();
            let was = std::mem::replace(&mut s.open, open);
            (was, if open { None } else { s.view.take() })
        };
        drop(gone);
        if open {
            self.clip_rows();
        } else if was {
            self.clip_woken();
        }
    }

    /// Keeps `view` while the panel is open and wakes; one heard while closed is late, so it goes.
    fn clip_view(&self, view: ClipView) {
        let last = {
            let mut s = self.clip();
            if !s.open {
                return;
            }
            s.view.replace(view)
        };
        drop(last);
        self.clip_woken();
    }

    /// Wakes QML unless a wake already waits for its read.
    fn clip_woken(&self) {
        if self.clip.unread.arm() {
            self.wake(Wake::Clip);
        }
    }

    /// Sends `act` through the limited handle, outside the lock; nothing goes before `listen`.
    fn send_clip(&self, act: ClipAct) {
        let bus = self.clip().bus.clone();
        if let Some(Err(e)) = bus.map(|b| b.publish(&act)) {
            tracing::warn!(error = %e, "a clipboard click was not sent");
        }
    }

    fn clip(&self) -> MutexGuard<'_, State> {
        self.clip
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests;
