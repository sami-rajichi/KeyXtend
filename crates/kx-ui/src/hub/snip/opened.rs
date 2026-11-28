//! How long Snip's windows took to open and how each countdown second showed, logged at debug for the live checks:
//! numbers only, never a picture, a place or a text.

use super::view::stage_name;
use crate::hub::Hub;

/// A Snip window's open in ms: from the click before its view (negative for none), then from its view to its show, its
/// first frame and the end of its pop.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SnipOpened {
    /// From the click on Snip's windows that led to the view, to the view; negative for none.
    pub click_ms: i32,
    /// From the view to the window's show.
    pub shown_ms: i32,
    /// From the view to the window's first frame, after which its shadow draws.
    pub drawn_ms: i32,
    /// From the view to the end of its pop, when it rests.
    pub ready_ms: i32,
}

/// A countdown second as it showed on the Qt thread.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SnipCounted {
    /// The seconds left as shown; 0 once the countdown hid.
    pub left_s: i32,
    /// The draws the countdown asked for and got since it showed, each new second and its pop's rest.
    pub draws: i32,
    /// Its Cancel is held.
    pub held: bool,
    /// The frozen screens' windows Qt still shows.
    pub frozen: usize,
}

impl Hub {
    /// Logs at debug how the window of stage `stage` opened, by the stage's own name; times for a stage that no longer
    /// shows log none, as that window went before it rested.
    pub fn snip_opened(&self, stage: &str, t: SnipOpened) {
        let now = self.snip_page().map(|p| stage_name(&p.stage));
        let Some(stage) = now.filter(|s| *s == stage) else {
            tracing::debug!("a Snip window's open times came after its stage ended");
            return;
        };
        tracing::debug!(
            stage,
            click_ms = t.click_ms,
            shown_ms = t.shown_ms,
            drawn_ms = t.drawn_ms,
            ready_ms = t.ready_ms,
            "a Snip window opened"
        );
    }

    /// Logs at debug how countdown second `c` showed; set against the line for its view, a gap shows a busy Qt thread.
    pub fn snip_counted(&self, c: SnipCounted) {
        tracing::debug!(
            left_s = c.left_s,
            draws = c.draws,
            held = c.held,
            frozen = c.frozen,
            "a countdown second showed"
        );
    }
}
