//! G8's frame times from a `PresentMon` CSV: the product's frames, how many were dropped, and the slowest gaps. Both
//! `PresentMon` 1.x columns and 2.x columns (with or without `--v1_metrics`) are read by name.
#![cfg(windows)]

use std::path::Path;

use serde::Serialize;

use crate::admin::command_line;
use crate::stats::{P99, percentile};

/// The column that names the program, lower-cased.
const APP: &str = "application";
/// Columns with the time since the previous present, in ms: 1.x and 2.x `--v1_metrics`, then 2.x.
const GAP: [&str; 2] = ["msbetweenpresents", "frametime"];
/// The 1.x column that is 1 for a dropped frame.
const DROPPED: &str = "dropped";
/// The value of `DROPPED` for a dropped frame.
const WAS_DROPPED: &str = "1";
/// Columns that say when a frame reached the screen, `NA` when it never did: 1.x, then 2.x.
const SHOWN: [&str; 2] = ["msuntildisplayed", "displayedtime"];
/// A frame that never reached the screen.
const NOT_SHOWN: &str = "NA";
/// `PresentMon`'s arguments that record `program`'s frames for `secs` seconds into CSV `file`, then stop.
pub fn args(program: &str, file: &Path, secs: u64) -> Vec<String> {
    let file = file.display().to_string();
    let secs = secs.to_string();
    let parts = [
        "--process_name",
        program,
        "--output_file",
        &file,
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

/// What the CSV says about the product's frames.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Frames {
    /// Frames the product presented.
    pub presented: usize,
    /// Frames that never reached the screen.
    pub dropped: usize,
    /// The 99th percentile of the gaps between presents, in ms.
    pub p99_ms: Option<f64>,
}

impl Frames {
    /// Frames that never showed; none when the product presented nothing, which proves nothing.
    pub fn drops(&self) -> Option<usize> {
        (self.presented > 0).then_some(self.dropped)
    }
}

/// Where each column we read sits.
struct Columns {
    app: usize,
    gap: usize,
    dropped: Option<usize>,
    shown: Option<usize>,
}

impl Columns {
    /// The columns of `header`; an error names what is missing.
    fn of(header: &str) -> Result<Self, String> {
        let names: Vec<String> = header.split(',').map(|c| c.trim().to_lowercase()).collect();
        let at = |want: &str| names.iter().position(|n| n == want);
        let app = at(APP).ok_or("the CSV has no Application column")?;
        let gap = GAP
            .iter()
            .find_map(|g| at(g))
            .ok_or("the CSV has no frame-gap column")?;
        let dropped = at(DROPPED);
        let shown = SHOWN.iter().find_map(|s| at(s));
        if dropped.is_none() && shown.is_none() {
            return Err("the CSV says nothing about dropped frames".to_string());
        }
        Ok(Self {
            app,
            gap,
            dropped,
            shown,
        })
    }

    /// True when row `cells` is a frame that never reached the screen.
    fn was_dropped(&self, cells: &[&str]) -> bool {
        let is =
            |i: Option<usize>, v: &str| i.and_then(|i| cells.get(i)).is_some_and(|c| c.trim() == v);
        is(self.dropped, WAS_DROPPED) || is(self.shown, NOT_SHOWN)
    }
}

/// The frames of `program` (its file name, any case) in `PresentMon`'s `csv`; a last row with no line end is still being
/// written, so it is left out.
pub fn parse(csv: &str, program: &str) -> Result<Frames, String> {
    let whole = csv.rfind('\n').map_or(csv, |end| &csv[..=end]);
    let mut lines = whole.lines();
    let cols = Columns::of(lines.next().ok_or("the CSV is empty")?)?;
    let (mut gaps, mut dropped) = (Vec::new(), 0);
    for line in lines {
        let cells: Vec<&str> = line.split(',').collect();
        let ours = cells
            .get(cols.app)
            .is_some_and(|a| a.trim().eq_ignore_ascii_case(program));
        if !ours {
            continue;
        }
        dropped += usize::from(cols.was_dropped(&cells));
        if let Some(gap) = cells
            .get(cols.gap)
            .and_then(|g| g.trim().parse::<f64>().ok())
        {
            gaps.push(gap);
        }
    }
    let presented = gaps.len();
    gaps.sort_by(f64::total_cmp);
    Ok(Frames {
        presented,
        dropped,
        p99_ms: percentile(&gaps, P99),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `PresentMon` 1.x rows, as its README documents the columns.
    const V1: &str = "\
Application,ProcessID,SwapChainAddress,Runtime,SyncInterval,PresentFlags,AllowsTearing,PresentMode,Dropped,\
TimeInSeconds,msInPresentAPI,msBetweenPresents,msUntilRenderComplete,msUntilDisplayed
keyxtend.exe,1234,0x1,DXGI,1,0,0,Hardware: Independent Flip,0,0.10,0.05,6.06,0.5,6.1
keyxtend.exe,1234,0x1,DXGI,1,0,0,Hardware: Independent Flip,1,0.11,0.05,12.12,0.5,NA
explorer.exe,99,0x2,DXGI,1,0,0,Composed: Flip,1,0.12,0.05,40.0,0.5,NA
KEYXTEND.EXE,1234,0x1,DXGI,1,0,0,Hardware: Independent Flip,0,0.12,0.05,6.06,0.5,6.0
";

    /// `PresentMon` 2.x rows: no Dropped column, `NA` display time for a frame that never showed.
    const V2: &str = "\
Application,ProcessID,SwapChainAddress,PresentRuntime,SyncInterval,PresentFlags,AllowsTearing,PresentMode,\
FrameType,CPUStartTime,FrameTime,CPUBusy,DisplayedTime
keyxtend.exe,1234,0x1,DXGI,1,0,0,Composed: Flip,Application,10.0,6.06,1.0,6.06
keyxtend.exe,1234,0x1,DXGI,1,0,0,Composed: Flip,Application,16.1,6.06,1.0,6.06
";

    #[test]
    fn version_1_rows_count_the_products_frames_and_drops_in_any_case() {
        let f = parse(V1, "keyxtend.exe").expect("reads");
        assert_eq!((f.presented, f.dropped), (3, 1));
        assert_eq!(f.p99_ms, Some(12.12));
        assert_eq!(f.drops(), Some(1), "a dropped frame counts");
    }

    #[test]
    fn version_2_rows_pass_when_every_frame_showed() {
        let f = parse(V2, "keyxtend.exe").expect("reads");
        assert_eq!((f.presented, f.dropped), (2, 0));
        assert_eq!(f.drops(), Some(0));
        let dropped = V2.replacen(",6.06\nkeyxtend", ",NA\nkeyxtend", 1);
        assert_eq!(parse(&dropped, "keyxtend.exe").expect("reads").dropped, 1);
    }

    #[test]
    fn no_frames_of_the_product_fail_and_a_csv_without_the_columns_is_an_error() {
        assert_eq!(
            parse(V2, "other.exe").expect("reads").drops(),
            None,
            "nothing presented"
        );
        assert!(parse("", "keyxtend.exe").is_err());
        assert!(
            parse("Application,FrameTime\n", "keyxtend.exe").is_err(),
            "no drop column"
        );
        assert!(parse("ProcessID,Dropped,msBetweenPresents\n", "x").is_err());
    }

    #[test]
    fn a_last_row_still_being_written_is_left_out() {
        let cut = V1
            .trim_end_matches('\n')
            .rsplit_once(",6.0")
            .expect("the last row")
            .0;
        let f = parse(cut, "keyxtend.exe").expect("reads");
        assert_eq!(
            (f.presented, f.dropped),
            (2, 1),
            "the cut row is not counted"
        );
    }

    #[test]
    fn g8_starts_presentmon_with_the_arguments_the_how_to_names() {
        let file = Path::new(r"C:\out\pm.csv");
        let got = args("keyxtend.exe", file, 75);
        let want = [
            "--process_name",
            "keyxtend.exe",
            "--output_file",
            r"C:\out\pm.csv",
            "--timed",
            "75",
            "--terminate_after_timed",
        ];
        assert_eq!(got, want);
        let text = how("keyxtend.exe", file, 75);
        assert!(text.contains(&want.join(" ")), "{text}");
        assert!(text.contains("presentmon_exe"), "names the setting");
    }

    #[test]
    fn the_how_to_names_the_program_the_file_and_the_time() {
        let text = how("keyxtend.exe", Path::new(r"C:\out\pm.csv"), 75);
        for part in [
            "--process_name keyxtend.exe",
            r"--output_file C:\out\pm.csv",
            "--timed 75",
            "administrator",
        ] {
            assert!(text.contains(part), "{part}: {text}");
        }
    }
}
