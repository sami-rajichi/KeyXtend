//! The keyboard side of the hub: the face's JSON and actions, the window guard, the window's shape, the pointer
//! and the bubble's spot. The face expects one thread, the Qt thread that also runs the relabel timer, and is
//! called with no lock held, as its actions may send notices back.

use super::{Hub, NULL, json};
use crate::geom::{Area, bubble, outward, screen_at};
use crate::guard::{self, Next};
use kx_module_api::Notice;
use kx_module_api::ports::{Corner, FaceAction, KeyId, KeyboardFace, Point, Rect, physical};
use kx_platform::ui::UiError;
use serde::Serialize;
use std::sync::Arc;

/// What QML redraws after an action.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Redraw {
    /// Read `board_json` again.
    pub board: bool,
    /// Read `state_json` again.
    pub state: bool,
    /// The keyboard still holds the pressed key, so the hold's beat goes on.
    pub holding: bool,
}

/// Which step of a resize the pointer is at.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Resize {
    /// The resize starts.
    Begin,
    /// The pointer moved.
    At,
    /// The resize ends.
    End,
}

impl Hub {
    pub(super) fn face(&self) -> Option<Arc<dyn KeyboardFace>> {
        self.lock().face.clone()
    }

    /// The board to draw, as the face's JSON; `null` while no face is attached.
    #[must_use]
    pub fn board_json(&self) -> String {
        self.face()
            .map_or_else(|| NULL.to_owned(), |f| f.board_json())
    }

    /// The live state, as the face's JSON with an open list shown `[ui] page_rows` rows to a page; `null` while no
    /// face is attached.
    #[must_use]
    pub fn state_json(&self) -> String {
        let rows = self.lock().settings.page_rows;
        self.face()
            .map_or_else(|| NULL.to_owned(), |f| f.state_json(rows))
    }

    /// Does `action` and says what to redraw; a notice in the face's reply goes to the banner (a failed repeat also to
    /// one warning, with no key), and a new state may show a layout in another language. Call it, like every face call
    /// here, on the Qt thread only.
    pub fn act(&self, action: FaceAction) -> Redraw {
        let Some(face) = self.face() else {
            return Redraw::default();
        };
        let tick = action == FaceAction::Tick;
        match face.act(action) {
            Ok(reply) => {
                if reply.state {
                    self.follow(&*face);
                }
                if let Some(key) = reply.notice {
                    if tick {
                        tracing::warn!("a held key's repeat was not sent");
                    }
                    self.notice(Notice {
                        key,
                        module: None,
                        args: Vec::new(),
                    });
                }
                Redraw {
                    board: reply.board,
                    state: reply.state,
                    holding: reply.holding,
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "the keyboard refused an action");
                Redraw::default()
            }
        }
    }

    /// Key `id` goes to the open list, `[ui] page_rows` rows per page.
    pub fn panel_key(&self, id: &str) -> Redraw {
        let rows = self.lock().settings.page_rows;
        self.act(FaceAction::PanelKey {
            key: KeyId::new(id),
            rows,
        })
    }

    /// The window now sits at `rect`, physical pixels, on the screens the `Screens` port reports; its resize corner
    /// hangs `overhang` logical pixels past the plate at size 1.
    pub fn placed(&self, rect: Rect, overhang: f64) -> Redraw {
        let screens = self.ports.screens.work_areas();
        self.act(FaceAction::Placed {
            rect,
            screens,
            overhang,
        })
    }

    /// One step of a resize, following the pointer; nothing happens when the pointer cannot be read.
    pub fn resize(&self, step: Resize) -> Redraw {
        let action = match step {
            Resize::End => FaceAction::ResizeEnd,
            Resize::Begin | Resize::At => {
                let Some(Point { x, y }) = self.cursor() else {
                    return Redraw::default();
                };
                if step == Resize::Begin {
                    FaceAction::ResizeBegin { x, y }
                } else {
                    FaceAction::ResizeAt { x, y }
                }
            }
        };
        self.act(action)
    }

    /// Where the pointer is, in physical pixels; `None` when the system cannot say.
    #[must_use]
    pub fn cursor(&self) -> Option<Point> {
        self.ports
            .screens
            .cursor()
            .map_err(log("the pointer could not be read"))
            .ok()
    }

    /// Where the bubble goes on the screen that holds `at`: the bottom `corner`, sized by the theme, in physical
    /// pixels; `None` with no screen.
    #[must_use]
    pub fn bubble_at(&self, at: Point, corner: Corner) -> Option<Point> {
        let screens = self.ports.screens.work_areas();
        let screen = screen_at(at, &screens)?;
        let [d, _, inset] = self.themes.shape.extra.bubble_px.map(f64::from);
        let (d, inset) = (physical(d, screen.dpi), physical(inset, screen.dpi));
        Some(bubble(screen.rect, corner, d, inset))
    }

    /// Cuts window `title` to `boxes`, in QML units at scale `dpr`; false when it is not shown yet or was refused.
    /// Call it on the UI thread.
    pub fn shape(&self, title: &str, boxes: &[Area], dpr: f64) -> bool {
        let rects: Vec<Rect> = boxes.iter().map(|b| outward(b, dpr)).collect();
        done(
            self.ports.guard.shape(title, &rects),
            "the window could not be shaped",
        )
    }

    /// Lifts our shown windows titled `title` alone in front of our others, as a tip does when it shows; false when none
    /// shows or it was refused. Call it on the UI thread.
    pub fn lift_alone(&self, title: &str) -> bool {
        done(
            self.ports.guard.lift_alone(title),
            "the window could not be lifted",
        )
    }

    /// Guard try number `attempt` (from 1), with the window titled `top` landing in front: true to try again after
    /// `[ui] guard_delay_ms`; once done, the focus goes back to the app the user had in front, and the entry watch asked
    /// next is the run's last. Call it on the UI thread.
    pub fn guard(&self, attempt: u32, top: Option<&str>) -> bool {
        let tries = self.lock().settings.guard_tries;
        let result = self.ports.guard.guard_own(None, top);
        let next = guard::next(&result, attempt, tries);
        self.guard_tried(next == Next::Done);
        match (&result, next) {
            (_, Next::Again) => return true,
            (Ok(0), Next::Done) => tracing::warn!(tries, "no window to guard"),
            (Ok(n), Next::Done) => tracing::info!(guarded = n, "windows guarded"),
            (Err(e), Next::Done) => tracing::warn!(error = %e, "the windows could not be guarded"),
        }
        self.ports.guard.give_back(self.focus);
        false
    }
}

/// A logger for a port error that gives `false`, so the caller can go on.
fn log(what: &'static str) -> impl Fn(UiError) -> bool {
    move |e| {
        tracing::warn!(error = %e, "{what}");
        false
    }
}

/// True for a done window call; a window not shown yet gives false quietly, and a refusal warns with `what`.
pub(super) fn done(result: Result<(), UiError>, what: &'static str) -> bool {
    match result {
        Ok(()) => true,
        Err(UiError::NoWindow) => false,
        Err(e) => log(what)(e),
    }
}

impl Redraw {
    /// The redraw as JSON for QML.
    #[must_use]
    pub fn to_json(self) -> String {
        json(&self)
    }
}
