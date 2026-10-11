//! G8's frames from a `PresentMon` 2.x CSV recorded with `--qpc_time_ms`: each present of the product with its swap
//! chain, its time on the QPC clock and whether it showed, and the slowest gaps between presents.
#![cfg(windows)]

use std::path::Path;
use std::time::Duration;

use serde::Serialize;

use crate::admin::command_line;
use crate::stats::{P99, US_PER_MS, percentile};

/// The column that names the program, lower-cased.
const APP: &str = "application";
/// The column that names a present's swap chain.
const CHAIN: &str = "swapchainaddress";
/// The text in the name of the column `QPC_ARG` writes, whatever the version calls it.
const QPC: &str = "qpc";
/// Asks `PresentMon` for each present's time on the QPC clock, in ms.
pub const QPC_ARG: &str = "--qpc_time_ms";
/// Columns with the time since the previous present, in ms: 2.6, then 2.0 to 2.5.
const GAP: [&str; 2] = ["msbetweenpresents", "frametime"];
/// Columns that say when a frame reached the screen, `NA` when it never did: 2.6, then 2.0 to 2.5.
const SHOWN: [&str; 2] = ["msuntildisplayed", "displayedtime"];
/// A frame that never reached the screen.
const NOT_SHOWN: &str = "NA";
/// The UTF-8 byte order mark `PresentMon` 2.6 starts its CSV with.
const BOM: char = '\u{FEFF}';

/// How much longer to wait for a recording of `secs` seconds, plus `stop_ms` to stop, `gone` after it started.
pub fn stop_wait(secs: u64, stop_ms: u64, gone: Duration) -> Duration {
    let all = Duration::from_secs(secs).saturating_add(Duration::from_millis(stop_ms));
    all.saturating_sub(gone)
}

/// `PresentMon`'s arguments that record `program`'s frames with QPC times for `secs` seconds into CSV `file`, then
/// stop.
pub fn args(program: &str, file: &Path, secs: u64) -> Vec<String> {
    let file = file.display().to_string();
    let secs = secs.to_string();
    let parts = [
        "--process_name",
        program,
        "--output_file",
        &file,
        QPC_ARG,
        "--timed",
        &secs,
        "--terminate_after_timed",
    ];
    parts.map(String::from).to_vec()
}

/// How to get the CSV `file` of `program`'s frames over `secs` seconds, for the result line.
pub fn how(program: &str, file: &Path, secs: u64) -> String {
    format!(
        "name PresentMon (Intel, MIT licence; the owner's yes and kx-licence-check first) in [g8] presentmon_exe and G8 \
         starts it, or start it as administrator just before G8: PresentMon {}",
        command_line(&args(program, file, secs))
    )
}

/// One present of the product.
#[derive(Debug, Clone, PartialEq)]
pub struct Present {
    /// Its swap chain's address, as the CSV writes it.
    pub chain: String,
    /// When the CPU started it, on the QPC clock in µs.
    pub at_us: i64,
    /// Whether it reached the screen.
    pub shown: bool,
}

/// What the CSV says about the product's frames.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Frames {
    /// Frames the product presented.
    pub presented: usize,
    /// Frames that never reached the screen, whether G8's frame rule counts them or not.
    pub lost: usize,
    /// The 99th percentile of the gaps between presents, in ms.
    pub p99_ms: Option<f64>,
    /// Every present, in time order.
    #[serde(skip)]
    pub presents: Vec<Present>,
}

/// Where each column we read sits.
struct Columns {
    app: usize,
    chain: usize,
    time: usize,
    gap: Option<usize>,
    shown: usize,
}

impl Columns {
    /// The columns of `header`; an error names what is missing.
    fn of(header: &str) -> Result<Self, String> {
        let names: Vec<String> = header.split(',').map(|c| c.trim().to_lowercase()).collect();
        let at = |want: &str| names.iter().position(|n| n == want);
        let missing = |what: &str| format!("the CSV has no {what} column");
        Ok(Self {
            app: at(APP).ok_or_else(|| missing("Application"))?,
            chain: at(CHAIN).ok_or_else(|| missing("SwapChainAddress"))?,
            time: names
                .iter()
                .position(|n| n.contains(QPC))
                .ok_or_else(|| format!("{}; record with {QPC_ARG}", missing("QPC time")))?,
            gap: GAP.iter().find_map(|g| at(g)),
            shown: SHOWN
                .iter()
                .find_map(|s| at(s))
                .ok_or_else(|| missing("display time"))?,
        })
    }

    /// The present in row `cells`; an error when it has no time.
    fn present(&self, cells: &[&str]) -> Result<Present, String> {
        let cell = |i: usize| cells.get(i).copied().unwrap_or_default();
        let ms = cell(self.time)
            .parse::<f64>()
            .ok()
            .filter(|t| t.is_finite());
        let ms = ms.ok_or_else(|| format!("a present has no QPC time: {}", cell(self.time)))?;
        Ok(Present {
            chain: cell(self.chain).to_string(),
            at_us: us(ms),
            shown: cell(self.shown) != NOT_SHOWN,
        })
    }
}

/// `ms` milliseconds in whole µs.
#[allow(
    clippy::cast_possible_truncation,
    reason = "QPC times in µs are far below 2^63."
)]
fn us(ms: f64) -> i64 {
    (ms * US_PER_MS).round() as i64
}

/// The frames of `program` (its file name, any case) in `PresentMon`'s `csv`, after any byte order mark; a last row with
/// no line end is still being written, so it is left out.
pub fn parse(csv: &str, program: &str) -> Result<Frames, String> {
    let csv = csv.strip_prefix(BOM).unwrap_or(csv);
    let whole = csv.rfind('\n').map_or(csv, |end| &csv[..=end]);
    let mut lines = whole.lines();
    let cols = Columns::of(lines.next().ok_or("the CSV is empty")?)?;
    let (mut gaps, mut presents) = (Vec::new(), Vec::new());
    for line in lines {
        let cells: Vec<&str> = line.split(',').map(str::trim).collect();
        if !cells
            .get(cols.app)
            .is_some_and(|a| a.eq_ignore_ascii_case(program))
        {
            continue;
        }
        presents.push(cols.present(&cells)?);
        let gap = cols.gap.and_then(|g| cells.get(g)?.parse::<f64>().ok());
        gaps.extend(gap);
    }
    presents.sort_by_key(|p| p.at_us);
    gaps.sort_by(f64::total_cmp);
    Ok(Frames {
        presented: presents.len(),
        lost: presents.iter().filter(|p| !p.shown).count(),
        p99_ms: percentile(&gaps, P99),
        presents,
    })
}

#[cfg(test)]
mod tests;
