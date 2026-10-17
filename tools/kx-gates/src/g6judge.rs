//! G6's judging, from numbers only: a picture's changed share and UI Automation's percents tell whether a window moved
//! or stood still, percent reads give a speed's rate, and the scroll module's counts tell how it stopped.
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
    /// The changed share; none when the window moved or changed size.
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

/// True when `fast` ÷ `slow` lies within `slack` of `want`, as a share of it; never for a slow rate of nothing.
pub fn ratio_ok(slow: f64, fast: f64, want: f64, slack: f64) -> bool {
    slow != 0.0 && ((fast / slow).abs() - want).abs() <= want * slack
}

/// True when the document overflows sideways, so its sideways cases are judged: its horizontal percent reads.
pub fn sideways(pair: Pair) -> bool {
    pair.and_then(|p| along(Dir::Right, p)).is_some()
}

/// True when `pair` reads at `dir`'s end.
pub fn ended(dir: Dir, pair: Pair) -> bool {
    reach(dir, pair.and_then(|p| along(dir, p))) == Reach::End
}

/// True when the end case passed: the pad stopped by itself, its `down` leg moved, and `back`, from the look at the
/// end before it to now, shows the end again: by percent where it reads, else by a picture within `min` of it.
pub fn end_ok(idle: bool, down: &Look, back: &Look, min: f64) -> bool {
    let at_end = if back.reads(Dir::Down) {
        ended(Dir::Down, back.after)
    } else {
        back.share.is_some_and(|s| s <= min)
    };
    idle && down.moved(Dir::Down, min) && at_end
}

/// True when the scroll module stopped once for each of G6's `clicks` outside our windows, so a tap on the pad never
/// stopped it as a press, and at the end at least once.
pub fn stops_ok(line: &ScrollLine, clicks: u64) -> bool {
    line.press == clicks && line.end > 0
}

/// True when every case judged passed: a case that could not be judged, such as sideways without an overflow, says
/// why in its note and fails nothing.
pub fn pass(verdicts: &[Option<bool>]) -> bool {
    !verdicts.contains(&Some(false))
}

#[cfg(test)]
mod tests;
