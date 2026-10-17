//! G6's judging, from numbers only: a picture's changed share and UI Automation's percents tell whether a window moved,
//! settled or stood still, percent reads give a speed's rate, and the scroll module's counts tell how it stopped.
#![cfg(windows)]

use kx_module_api::MS_PER_S;
use kx_module_api::ports::{Dir, Reach};

use crate::p5cfg::LINE_READS;
use crate::scroll::{along, percent_moved, reach};
use crate::scrolllog::ScrollLine;

/// UI Automation's (horizontal, vertical) percents, when it reads them.
pub type Pair = Option<(f64, f64)>;

/// Two looks at a window some time apart: the share of its picture that changed between them, and its percents.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Look {
    /// The changed share; none for an empty picture, as the band it shows stays fixed.
    pub share: Option<f64>,
    /// The percents at the first look.
    pub before: Pair,
    /// The percents at the second look.
    pub after: Pair,
}

impl Look {
    /// True when the window scrolled towards `dir`: by its percent where both looks read that way, else by more than
    /// `min` of its picture changing.
    pub fn moved(&self, dir: Dir, min: f64) -> bool {
        if self.reads(dir) {
            percent_moved(dir, self.before, self.after)
        } else {
            self.share.is_some_and(|s| s > min)
        }
    }

    /// True when the window stood still: no percent moved on an axis both looks read, else, with no such axis, at
    /// most `min` of its picture changed.
    pub fn still(&self, min: f64) -> bool {
        let read: Vec<Dir> = [Dir::Down, Dir::Right]
            .into_iter()
            .filter(|&d| self.reads(d))
            .collect();
        if read.is_empty() {
            return self.share.is_some_and(|s| s <= min);
        }
        let turned = Dir::ALL
            .into_iter()
            .any(|d| percent_moved(d, self.before, self.after));
        !turned
    }

    /// True when no vertical percent moved, up or down.
    fn level(&self) -> bool {
        let ways = [Dir::Down, Dir::Up];
        !ways
            .into_iter()
            .any(|d| percent_moved(d, self.before, self.after))
    }

    /// True when both looks read a percent on `dir`'s axis.
    fn reads(&self, dir: Dir) -> bool {
        [self.before, self.after]
            .iter()
            .all(|p| p.and_then(|p| along(dir, p)).is_some())
    }
}

/// True when percent `p` lies short of both ends.
fn inside(p: f64) -> bool {
    [Dir::Down, Dir::Up]
        .into_iter()
        .all(|d| reach(d, Some(p)) == Reach::More)
}

/// A speed's rate, in percent a second, from `reads` of (ms, percent towards its way) taken while it scrolled: the
/// least-squares slope of the reads short of either end; none from fewer than `min_reads` of them.
#[allow(
    clippy::cast_precision_loss,
    reason = "Times and counts are far below 2^52."
)]
pub fn rate(reads: &[(u64, f64)], min_reads: u64) -> Option<f64> {
    let inner: Vec<(f64, f64)> = reads
        .iter()
        .filter(|(_, p)| inside(*p))
        .map(|&(ms, p)| (ms as f64, p))
        .collect();
    if (inner.len() as u64) < min_reads.max(LINE_READS) {
        return None;
    }
    let n = inner.len() as f64;
    let mt = inner.iter().map(|r| r.0).sum::<f64>() / n;
    let mp = inner.iter().map(|r| r.1).sum::<f64>() / n;
    let (num, den) = inner.iter().fold((0.0, 0.0), |(num, den), (t, p)| {
        (num + (t - mt) * (p - mp), den + (t - mt) * (t - mt))
    });
    (den > 0.0).then(|| num / den * f64::from(MS_PER_S))
}

/// True when `fast` ÷ `slow` lies within `slack` of `want`, as a share of it; never when the two ran opposite ways or
/// slow ran not at all.
pub fn ratio_ok(slow: f64, fast: f64, want: f64, slack: f64) -> bool {
    let ratio = fast / slow;
    ratio > 0.0 && (ratio - want).abs() <= want * slack
}

/// True when the sideways cases are judged: the scroll pattern says it scrolls sideways, as the product asks.
pub fn sideways(scrollable: Option<bool>) -> bool {
    scrollable == Some(true)
}

/// Reads `read` every `sample_ms`, after `wait`, until it reads the same twice in a row or `cap_ms` passed; true when
/// it did.
pub fn steady(
    mut read: impl FnMut() -> Pair,
    mut wait: impl FnMut(u64),
    (sample_ms, cap_ms): (u64, u64),
) -> bool {
    let (sample_ms, mut spent) = (sample_ms.max(1), 0);
    let mut last = read();
    while spent + sample_ms <= cap_ms {
        wait(sample_ms);
        spent += sample_ms;
        let now = read();
        if now == last {
            return true;
        }
        last = now;
    }
    false
}

/// True when the end case passed: the pad stopped by itself, its `down` leg moved, and `back`, from Ctrl+End's bottom
/// to now, shows that bottom again: no vertical percent moved where both read one, else a picture within `min` of it.
pub fn end_ok(idle: bool, down: &Look, back: &Look, min: f64) -> bool {
    let at_end = if back.reads(Dir::Down) {
        back.level()
    } else {
        back.share.is_some_and(|s| s <= min)
    };
    idle && down.moved(Dir::Down, min) && at_end
}

/// What G6 did that stops a scroll: its clicks outside our windows, its Stop and third taps, and its runs to the end.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Stops {
    /// Clicks outside our windows.
    pub clicks: u64,
    /// Stop and third taps.
    pub taps: u64,
    /// Runs down that should stop at the document's end.
    pub ends: u64,
}

/// True when the scroll module stopped once for each of G6's clicks and once for each of its taps, so a pad tap never
/// counted as a press nor a page turn of our list as a user stop, at the end at least once after a run there, and
/// never at the limit or as the target went.
pub fn stops_ok(line: &ScrollLine, made: Stops) -> bool {
    let counted = line.press == made.clicks && line.user == made.taps;
    let ended = made.ends == 0 || line.end > 0;
    counted && ended && line.limit == 0 && line.gone == 0
}

/// True when every case judged passed: a case that could not be judged, such as sideways without an overflow, says
/// why in its note and fails nothing.
pub fn pass(verdicts: &[Option<bool>]) -> bool {
    !verdicts.contains(&Some(false))
}

#[cfg(test)]
mod tests;
