//! G22 bench's rows, its pass rule and the text table it prints, ready for `docs/items/P11.md`. A row holds numbers
//! and a fault's words only, never a transcript.
#![cfg(windows)]

use kx_mod_voice::score::Wrong;
use serde_json::{Value, json};

/// Hundredths in one, as a job's rate cap counts a percent.
const HUNDREDTHS: u32 = 100;
/// Milliseconds in a tenth of a second, and tenths in a second, as the times show.
const MS_PER_TENTH: u32 = 100;
const TENTHS_PER_S: u32 = 10;
/// What a cell shows without a number.
const NONE: &str = "–";
/// The table's head.
const HEAD: [&str; 14] = [
    "Language",
    "Model",
    "Mode",
    "Cores",
    "Cap",
    "Clip",
    "Load",
    "First use",
    "First piece",
    "Total",
    "RTF",
    "Peak",
    "Words wrong",
    "Result",
];
/// The cells a row without numbers leaves empty, between its clip and its result.
const MEASURED: usize = 7;

/// The pass rule: normal rows' highest rate and the most words wrong; weak rows are readings, slow past their rate.
#[derive(Copy, Clone, Debug)]
pub(super) struct Rule {
    /// The highest warm wait over clip length that passes in normal mode.
    pub(super) rtf: f64,
    /// The weak rate past which a row reads slow; a reading, never a fail.
    pub(super) weak_rtf: f64,
    /// The most words wrong, in either mode.
    pub(super) wrong: u16,
}

impl Rule {
    /// The rule in words, for the table's first line.
    pub(super) fn text(&self) -> String {
        format!(
            "normal rows pass at RTF {} with at most {} words wrong; weak rows are readings, slow past RTF {}, that \
             fail only on a fault or words wrong.",
            self.rtf, self.wrong, self.weak_rtf
        )
    }
}

/// What one clip measured in one mode: the cold take's load and wait, the warm take's times and words, the job's peak.
#[derive(Copy, Clone, Debug)]
pub(super) struct Timed {
    /// The cold take's load.
    pub(super) load_ms: u32,
    /// The cold take's wait after Stop: a first dictation's, with any load left.
    pub(super) first_use_ms: u32,
    /// The warm take's Stop to its first piece.
    pub(super) first_ms: Option<u32>,
    /// The warm take's Stop to its last piece.
    pub(super) total_ms: u32,
    /// The job's most memory, in MB.
    pub(super) peak_mb: u32,
    /// The warm take's words wrong; `None` for a language without rules.
    pub(super) wrong: Option<Wrong>,
}

/// Why a row has no numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Miss {
    /// The product marks the default Not for this PC in this mode: its need and the PC's memory, in MB.
    NotHere { need_mb: u32, have_mb: u32 },
    /// The run failed, and why.
    Fault(String),
}

impl Miss {
    /// The words the table and the result line show.
    fn text(&self) -> String {
        match self {
            Self::NotHere { need_mb, have_mb } => {
                format!("needs {need_mb} MB of {have_mb} MB")
            }
            Self::Fault(why) => why.replace('|', "/"),
        }
    }
}

/// One row: a clip's language and default model in one mode, the cores the host counted, the job's cap read back, the
/// clip's length, and what was measured or why not.
#[derive(Clone, Debug)]
pub(super) struct Row {
    pub(super) lang: String,
    pub(super) model: String,
    pub(super) weak: bool,
    pub(super) cores: usize,
    /// The job's processor cap, in hundredths of a percent of the PC.
    pub(super) cap: Option<u32>,
    pub(super) clip_ms: u32,
    pub(super) got: Result<Timed, Miss>,
}

/// How a row stands against the rule.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Verdict {
    /// Within the rule.
    Pass,
    /// Slower than its mode's limit; in weak mode, a reading.
    Slow,
    /// More words wrong than the rule allows.
    Wrong,
    /// The product would not run the default on this mode's PC; in weak mode, a reading.
    NotHere,
    /// Nothing was measured.
    Failed,
}

impl Verdict {
    /// The word the table and the result line show.
    const fn word(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Slow => "slow",
            Self::Wrong => "words wrong",
            Self::NotHere => "not for this PC",
            Self::Failed => "failed",
        }
    }
}

/// The mode's name.
pub(super) const fn mode(weak: bool) -> &'static str {
    if weak { "weak" } else { "normal" }
}

impl Row {
    /// The warm wait over the clip's length; `None` without a measure or a length.
    pub(super) fn rtf(&self) -> Option<f64> {
        let t = self.got.as_ref().ok()?;
        (self.clip_ms > 0).then(|| f64::from(t.total_ms) / f64::from(self.clip_ms))
    }

    /// The row against `rule`: not for this PC or failed without a rate, then too many words wrong, then too slow.
    pub(super) fn verdict(&self, rule: &Rule) -> Verdict {
        let (Ok(t), Some(rtf)) = (&self.got, self.rtf()) else {
            return match self.got {
                Err(Miss::NotHere { .. }) => Verdict::NotHere,
                _ => Verdict::Failed,
            };
        };
        let max = if self.weak { rule.weak_rtf } else { rule.rtf };
        if t.wrong.is_some_and(|w| w.words > rule.wrong) {
            Verdict::Wrong
        } else if rtf > max {
            Verdict::Slow
        } else {
            Verdict::Pass
        }
    }

    /// True when the row lets the run pass: a normal row within the rule, or a weak row slow or not for this PC.
    pub(super) fn passes(&self, rule: &Rule) -> bool {
        match self.verdict(rule) {
            Verdict::Pass => true,
            Verdict::Slow | Verdict::NotHere => self.weak,
            Verdict::Wrong | Verdict::Failed => false,
        }
    }

    /// The row as the result line carries it.
    pub(super) fn json(&self, rule: &Rule) -> Value {
        let t = self.got.as_ref().ok();
        json!({
            "lang": self.lang, "model": self.model, "mode": mode(self.weak), "cores": self.cores, "cap": self.cap,
            "clip_ms": self.clip_ms, "load_ms": t.map(|t| t.load_ms), "first_use_ms": t.map(|t| t.first_use_ms),
            "first_ms": t.and_then(|t| t.first_ms), "total_ms": t.map(|t| t.total_ms), "rtf": self.rtf(),
            "peak_mb": t.map(|t| t.peak_mb), "words_wrong": t.and_then(|t| t.wrong).map(|w| w.words),
            "words": t.and_then(|t| t.wrong).map(|w| w.total), "verdict": self.verdict(rule).word(),
            "error": self.got.as_ref().err().map(Miss::text),
        })
    }

    /// The row's cells, in the head's order.
    fn cells(&self, rule: &Rule) -> Vec<String> {
        let cap = self.cap.map_or_else(|| "none".to_owned(), percent);
        let (lang, model, cores) = (
            self.lang.clone(),
            self.model.clone(),
            self.cores.to_string(),
        );
        let mut cells = vec![
            lang,
            model,
            mode(self.weak).to_owned(),
            cores,
            cap,
            secs(self.clip_ms),
        ];
        match &self.got {
            Ok(t) => cells.extend([
                secs(t.load_ms),
                secs(t.first_use_ms),
                t.first_ms.map_or_else(|| NONE.to_owned(), secs),
                secs(t.total_ms),
                self.rtf()
                    .map_or_else(|| NONE.to_owned(), |r| format!("{r:.2}")),
                format!("{} MB", t.peak_mb),
                t.wrong
                    .map_or_else(|| NONE.to_owned(), |w| format!("{}/{}", w.words, w.total)),
                self.verdict(rule).word().to_owned(),
            ]),
            Err(miss) => {
                cells.extend(std::iter::repeat_n(NONE.to_owned(), MEASURED));
                cells.push(format!("{}: {}", self.verdict(rule).word(), miss.text()));
            }
        }
        cells
    }
}

/// True when every row lets the run pass; a weak row's speed and fit are readings, never a fail.
pub(super) fn pass(rows: &[Row], rule: &Rule) -> bool {
    rows.iter().all(|r| r.passes(rule))
}

/// The table of `rows`: a head, its rule line, then one line per row.
pub(super) fn lines(rows: &[Row], rule: &Rule) -> Vec<String> {
    let mut out = vec![
        line(HEAD.map(str::to_owned)),
        line(HEAD.map(|_| "---".to_owned())),
    ];
    out.extend(rows.iter().map(|r| line(r.cells(rule))));
    out
}

/// One table line of `cells`.
fn line(cells: impl IntoIterator<Item = String>) -> String {
    format!("| {} |", cells.into_iter().collect::<Vec<_>>().join(" | "))
}

/// `ms` in seconds to a tenth, rounded.
fn secs(ms: u32) -> String {
    let tenths = ms.saturating_add(MS_PER_TENTH / 2) / MS_PER_TENTH;
    format!("{}.{} s", tenths / TENTHS_PER_S, tenths % TENTHS_PER_S)
}

/// A rate cap in hundredths of a percent, as a percent.
fn percent(cap: u32) -> String {
    format!("{}.{:02} %", cap / HUNDREDTHS, cap % HUNDREDTHS)
}
