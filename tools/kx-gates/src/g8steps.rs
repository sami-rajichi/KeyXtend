//! G8's churn memory step by step: what each step of a measured round changed, and a least-squares slope per round
//! for the round and for each step, so one run shows which step grows.
#![cfg(windows)]

use std::fmt;

use serde::Serialize;

use crate::proc::Memory;

/// A step of a churn round, in the order it runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Step {
    /// The key tips, with the settle after them.
    Tips,
    /// The language list, in the List design only.
    List,
    /// Minimise to the bubble and back.
    Bubble,
    /// A full theme cycle.
    Themes,
}

impl fmt::Display for Step {
    /// The step's name as G8's result writes it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let named = serde_json::to_value(self).map_err(|_| fmt::Error)?;
        f.write_str(named.as_str().unwrap_or_default())
    }
}

/// The `reading` after `step`, kept with it; a failed one says which step it followed.
pub fn read_after(step: Step, reading: Result<Memory, String>) -> Result<(Step, Memory), String> {
    reading
        .map(|m| (step, m))
        .map_err(|e| format!("memory after the {step} step: {e}"))
}

/// One step's growth over the measured rounds.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StepGrowth {
    /// The step.
    pub step: Step,
    /// What the step changed in each round, in MB.
    pub per_round: Vec<Memory>,
    /// The least-squares slope of its running total, in MB per round; the steps' slopes add up to about the round's.
    pub slope: Option<Memory>,
}

/// The least-squares slope of `ys` over evenly spaced rounds: how much it grows per round; none under two points.
pub fn slope(ys: &[f64]) -> Option<f64> {
    if ys.len() < 2 {
        return None;
    }
    let n = count(ys.len());
    let mean_x = (0..ys.len()).map(count).sum::<f64>() / n;
    let mean_y = ys.iter().sum::<f64>() / n;
    let (mut sxy, mut sxx) = (0.0, 0.0);
    for (i, y) in ys.iter().enumerate() {
        let dx = count(i) - mean_x;
        sxy += dx * (y - mean_y);
        sxx += dx * dx;
    }
    Some(sxy / sxx)
}

/// The slope of `readings` per round, for the working set and the commit apart.
pub fn slopes(readings: &[Memory]) -> Option<Memory> {
    let of = |mb: fn(&Memory) -> f64| slope(&readings.iter().map(mb).collect::<Vec<_>>());
    Some(Memory {
        working_mb: of(|m| m.working_mb)?,
        commit_mb: of(|m| m.commit_mb)?,
    })
}

/// Each step's growth over the rounds: `after` holds each round's readings after its steps, in order, and a round
/// starts from the reading after the round before (`per_round`), or from `before`.
pub fn growth(
    before: Memory,
    per_round: &[Memory],
    after: &[Vec<(Step, Memory)>],
) -> Vec<StepGrowth> {
    let starts = std::iter::once(before).chain(per_round.iter().copied());
    let changes: Vec<Vec<(Step, Memory)>> = after
        .iter()
        .zip(starts)
        .map(|(round, start)| changed(start, round))
        .collect();
    let order: Vec<Step> = changes
        .first()
        .map(|r| r.iter().map(|&(s, _)| s).collect())
        .unwrap_or_default();
    let of = |step| -> Vec<Memory> {
        let made = |r: &Vec<(Step, Memory)>| r.iter().find(|(s, _)| *s == step).map(|&(_, m)| m);
        changes.iter().filter_map(made).collect()
    };
    order
        .into_iter()
        .map(|step| {
            let per_round = of(step);
            let slope = slopes(&running(&per_round));
            StepGrowth {
                step,
                per_round,
                slope,
            }
        })
        .collect()
}

/// What each step of a round changed, from the round's `start` and its readings `after` each step.
fn changed(start: Memory, after: &[(Step, Memory)]) -> Vec<(Step, Memory)> {
    let mut last = start;
    after
        .iter()
        .map(|&(step, m)| {
            let change = apply(m, last, |a, b| a - b);
            last = m;
            (step, change)
        })
        .collect()
}

/// The running total of `changes`.
fn running(changes: &[Memory]) -> Vec<Memory> {
    let zero = Memory {
        working_mb: 0.0,
        commit_mb: 0.0,
    };
    changes
        .iter()
        .scan(zero, |sum, &c| {
            *sum = apply(*sum, c, |a, b| a + b);
            Some(*sum)
        })
        .collect()
}

/// `op` on the working sets and on the commits of `a` and `b`.
fn apply(a: Memory, b: Memory, op: fn(f64, f64) -> f64) -> Memory {
    Memory {
        working_mb: op(a.working_mb, b.working_mb),
        commit_mb: op(a.commit_mb, b.commit_mb),
    }
}

/// A count or round number as a float.
#[allow(clippy::cast_precision_loss, reason = "Rounds are far below 2^53.")]
fn count(n: usize) -> f64 {
    n as f64
}

#[cfg(test)]
mod tests;
