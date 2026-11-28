//! G24's pure decisions: what a window of ours may do during a capture, a length within its slack, the scale the exact
//! checks need, the keyboard and bubble out of a picture, windows settled, a memory back at idle, and the gate's own
//! pictures gone from clipboard history.

use kx_module_api::ports::{BASE_DPI, SnipKeyboard};
use serde_json::{Value, json};

use crate::budget::Part;
use crate::g8churn;
use crate::proc::{MB, Memory};
use crate::winpanel::SKIPPED;

/// A window of ours shown during a capture that the capture could take: not kept out of it.
pub const BARE: &str = "bare";
/// The keyboard shown during a capture, which it steps aside for.
pub const KEYBOARD: &str = "keyboard";
/// A window of ours beside the capture's own while the keyboard is hidden.
pub const SHOWS: &str = "shows";
/// Why the bubble's own drawing is missing when Windows drew it all one colour.
pub const BLANK: &str = "Windows drew the bubble all one colour";
/// A whole in percent.
const PCT: u32 = 100;

/// What one of our shown windows is, by its title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The frozen screen, the picture itself.
    Frozen,
    /// A capture window: bar, hint, strip, countdown, record bar or edge.
    Capture,
    Keyboard,
    Bubble,
    /// Any other window of ours.
    Other,
}

/// One shown window of ours: its kind, and whether it is kept out of captures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seen {
    pub kind: Kind,
    pub shielded: bool,
}

/// Where a window lies as the order check weighs it (ADR-0059).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    /// A frozen screen of the copy.
    Frozen,
    /// A window that must lie over every frozen screen.
    Above,
    /// Any other window, which the check passes over.
    Else,
}

/// Where a window lies: a `frozen` screen of the copy stays one even when the check names it `above`, as `order::ours`
/// names every window of the copy.
pub fn layer(frozen: bool, above: bool) -> Layer {
    if frozen {
        Layer::Frozen
    } else if above {
        Layer::Above
    } else {
        Layer::Else
    }
}

/// How many frozen screens `order` holds when it holds fewer than one on each of `screens`, and never none, so a read
/// that found no frozen screen never passes.
pub fn short(order: &[Layer], screens: usize) -> Option<usize> {
    let frozen = order.iter().filter(|l| **l == Layer::Frozen).count();
    (frozen < screens.max(1)).then_some(frozen)
}

/// Each window of `order`, top first, that must lie over every frozen screen but lies under one: its place, and the
/// topmost frozen screen's.
pub fn under(order: &[Layer]) -> Vec<(usize, usize)> {
    let Some(top) = order.iter().position(|l| *l == Layer::Frozen) else {
        return Vec::new();
    };
    let below = order.iter().enumerate().skip(top);
    below
        .filter(|(_, l)| **l == Layer::Above)
        .map(|(i, _)| (i, top))
        .collect()
}

/// A window named for a report: one of ours (`mine`) by its `title`, and any other, or an untitled one, by its `class`
/// only, as another program's title may name the owner's document.
pub fn named(mine: bool, title: &str, class: &str) -> String {
    if mine && !title.is_empty() {
        format!("'{title}'")
    } else {
        format!("[{class}]")
    }
}

/// The kinds of `need` that no window of `shown` is.
pub fn lacks(need: &[Kind], shown: &[Kind]) -> Vec<Kind> {
    need.iter()
        .copied()
        .filter(|k| !shown.contains(k))
        .collect()
}

/// The faults of our windows `seen` together with the keyboard `way`, judged only while a capture shows: a window but
/// the frozen screen left in the capture, the keyboard shown, or anything beside the capture's own while hidden.
pub fn faults(way: SnipKeyboard, seen: &[Seen]) -> Vec<&'static str> {
    let capture = |s: &Seen| matches!(s.kind, Kind::Frozen | Kind::Capture);
    if !seen.iter().any(capture) {
        return Vec::new();
    }
    let mut found = Vec::new();
    for s in seen {
        let fault = match s.kind {
            Kind::Keyboard => Some(KEYBOARD),
            Kind::Bubble | Kind::Other if way == SnipKeyboard::Hidden => Some(SHOWS),
            Kind::Frozen => None,
            _ => (!s.shielded).then_some(BARE),
        };
        if let Some(f) = fault.filter(|f| !found.contains(f)) {
            found.push(f);
        }
    }
    found
}

/// True when `got_ms` is within `slack_ms` of `want_ms`, either way.
pub fn within(got_ms: u64, want_ms: u64, slack_ms: u64) -> bool {
    got_ms.abs_diff(want_ms) <= slack_ms
}

/// The DPI `dpi` as a percent of 100 %.
fn percent(dpi: u32) -> u32 {
    dpi.saturating_mul(PCT) / BASE_DPI
}

/// Why the exact checks are skipped on a screen at `dpi` when they are judged at `want`; none at that scale.
pub fn off_scale(dpi: u32, want: u32) -> Option<String> {
    (dpi != want).then(|| {
        format!(
            "the screen is at {} %, not {} %: the owner sets it in Settings > Display, as the gate never changes it",
            percent(dpi),
            percent(want)
        )
    })
}

/// A part's result as it stands, or skipped with `off` when the screen is not at the scale the part is judged at.
pub fn at_scale(mut v: Value, off: Option<&str>) -> Value {
    if let Some(why) = off {
        v[SKIPPED] = json!(true);
        v["why"] = json!(why);
    }
    v
}

/// True when the keyboard is out of a full-screen picture: its place holds at most `most` of it as the gate grabbed
/// it, and it no longer showed when the frozen screen came.
pub fn keyboard_out(same: Option<f64>, shown_at_freeze: bool, most: f64) -> bool {
    same.is_some_and(|v| v <= most) && !shown_at_freeze
}

/// A full-screen picture at the bubble's place: the share differing from the gate's grab where the bubble lies clear
/// of the keyboard, the share like the bubble's own drawing, when Windows drew it, and else why not.
#[derive(Debug, Clone, PartialEq)]
pub struct BubbleSeen {
    pub clear: Option<f64>,
    pub like: Option<f64>,
    pub drawing: Option<String>,
}

/// The bubble's verdict with the keyboard `way`: its place holds the grab within `frame_share` where it lies clear of
/// the keyboard, and at most `same_most` of its own drawing; with neither figure it fails.
pub fn bubble(
    way: SnipKeyboard,
    seen: Option<BubbleSeen>,
    (frame_share, same_most): (f64, f64),
) -> Value {
    if way != SnipKeyboard::Bubble {
        return json!({ "ok": true });
    }
    let Some(b) = seen else {
        return json!({ "ok": false, "error": "no bubble showed" });
    };
    let like = b.like.map(|l| l <= same_most);
    let ok = match b.clear {
        Some(d) => d <= frame_share && like != Some(false),
        None => like == Some(true),
    };
    json!({ "differing": b.clear, "own_same": b.like, "drawing": b.drawing, "ok": ok })
}

/// True when some windows were read and each of `got` is `want`.
pub fn settled(got: &[bool], want: bool) -> bool {
    !got.is_empty() && got.iter().all(|&g| g == want)
}

/// How many parts of `copies` were skipped, with the first one's reason; none when none was.
pub fn skipped_note(copies: &[Value]) -> Option<String> {
    let skipped: Vec<&Value> = copies
        .iter()
        .filter_map(Value::as_object)
        .flat_map(|c| c.values())
        .filter(|p| p[SKIPPED] == true)
        .collect();
    let why = skipped.first()?["why"].as_str().unwrap_or_default();
    Some(format!("{} skipped: {why}", skipped.len()))
}

/// Bytes as decimal MB.
#[allow(
    clippy::cast_precision_loss,
    reason = "Picture sizes are far below 2^53 bytes."
)]
fn mb(bytes: usize) -> f64 {
    bytes as f64 / MB
}

/// Windows' history after the gate's Copies: `went` says per Copy whether its picture left, none when history is off.
/// Each Copy that read back a picture of at most `most_mb` decimal MB, the smaller reading of history's limit, must
/// have left it.
pub fn history(went: Option<&[bool]>, bytes: &[Option<usize>], most_mb: f64) -> Value {
    let Some(went) = went else {
        return json!({ "on": false, "ok": true });
    };
    let left = |(i, b): (usize, &Option<usize>)| {
        b.is_some_and(|b| mb(b) <= most_mb) && !went.get(i).copied().unwrap_or(false)
    };
    let missed = bytes.iter().enumerate().filter(|&p| left(p)).count();
    let gone = went.iter().filter(|&&w| w).count();
    let ok = went.len() == bytes.len() && missed == 0;
    json!({ "on": true, "copies": bytes.len(), "went": gone, "missed": missed, "ok": ok })
}

/// The working set's and the commit's growth from `idle` to `after`, each at most `most_mb`.
pub fn back(idle: Memory, after: Memory, most_mb: f64) -> [Part; 2] {
    g8churn::growth(["back_working_mb", "back_commit_mb"], idle, after, most_mb)
}

#[cfg(test)]
mod tests;
