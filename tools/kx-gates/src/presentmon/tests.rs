//! Tests of the `PresentMon` CSV reader: its columns, each present's time and swap chain, and G8's arguments.

use super::*;

/// `PresentMon` 2.x rows under `--qpc_time_ms`, as its README names the columns; the second frame never showed.
const V2: &str = "\
Application,ProcessID,SwapChainAddress,PresentRuntime,SyncInterval,PresentFlags,AllowsTearing,PresentMode,\
FrameType,CPUStartQPCTime,FrameTime,CPUBusy,DisplayedTime
keyxtend.exe,1234,0x1,DXGI,1,0,0,Composed: Flip,Application,5000.25,6.06,1.0,6.06
explorer.exe,99,0x2,DXGI,1,0,0,Composed: Flip,Application,5001.0,40.0,1.0,NA
KEYXTEND.EXE,1234,0x1,DXGI,1,0,0,Composed: Flip,Application,5006.31,12.12,1.0,NA
";

/// `PresentMon` 2.6's own header line, from a real capture without `--qpc_time_ms`, and two rows.
const V2_6: &str = "\
Application,ProcessID,SwapChainAddress,PresentRuntime,SyncInterval,PresentFlags,AllowsTearing,PresentMode,TimeInMs,\
MsBetweenSimulationStart,MsBetweenPresents,MsBetweenDisplayChange,MsInPresentAPI,MsRenderPresentLatency,\
MsUntilDisplayed,CPUStartTimeInMs,MsBetweenAppStart,MsCPUBusy,MsCPUWait,MsGPULatency,MsGPUTime,MsGPUBusy,MsGPUWait,\
MsAnimationError,AnimationTime,MsFlipDelay,MsAllInputToPhotonLatency,MsClickToPhotonLatency\r
keyxtend.exe,1234,0x1,DXGI,1,0,0,Composed: Flip,10.0,NA,6.06,6.06,0.1,1.0,6.0,9.0,6.06,1.0,0.1,0.5,0.5,0.5,0.1,0.0,\
10.0,0.0,NA,NA\r
keyxtend.exe,1234,0x1,DXGI,1,0,0,Composed: Flip,16.1,NA,12.12,NA,0.1,1.0,NA,15.0,6.06,1.0,0.1,0.5,0.5,0.5,0.1,0.0,\
16.1,0.0,NA,NA\r
";

/// The 2.6 file as `--qpc_time_ms` would write it; the column's exact 2.6 name is unknown here, and any name with QPC
/// in it is read.
fn v2_6_qpc() -> String {
    V2_6.replacen("CPUStartTimeInMs", "CPUStartQPCTimeInMs", 1)
}

#[test]
fn each_present_of_the_product_keeps_its_swap_chain_its_qpc_time_in_us_and_whether_it_showed() {
    let f = parse(V2, "keyxtend.exe").expect("reads");
    assert_eq!((f.presented, f.lost), (2, 1), "in any case, not explorer");
    assert_eq!(f.p99_ms, Some(12.12));
    let want = [("0x1", 5_000_250, true), ("0x1", 5_006_310, false)];
    let got = f
        .presents
        .iter()
        .map(|p| (p.chain.as_str(), p.at_us, p.shown));
    assert_eq!(got.collect::<Vec<_>>(), want);
}

#[test]
fn version_2_6_files_read_with_or_without_the_byte_order_mark_they_start_with() {
    for csv in [v2_6_qpc(), format!("\u{FEFF}{}", v2_6_qpc())] {
        let f = parse(&csv, "keyxtend.exe").expect("reads");
        assert_eq!((f.presented, f.lost), (2, 1));
        assert_eq!(f.p99_ms, Some(12.12));
        assert_eq!(f.presents[1].at_us, 15_000, "CPU start, in µs");
    }
}

#[test]
fn a_file_recorded_without_qpc_times_is_refused_and_names_the_flag() {
    for csv in [V2_6.to_string(), format!("\u{FEFF}{V2_6}")] {
        let e = parse(&csv, "keyxtend.exe").expect_err("no QPC column");
        assert!(e.contains(QPC_ARG), "{e}");
    }
}

#[test]
fn presents_are_read_in_time_order_and_a_present_without_a_time_is_an_error() {
    let swapped = V2.replacen("5000.25", "5009.0", 1);
    let f = parse(&swapped, "keyxtend.exe").expect("reads");
    let at: Vec<_> = f.presents.iter().map(|p| p.at_us).collect();
    assert_eq!(at, [5_006_310, 5_009_000]);
    let blank = V2.replacen("5000.25", "NA", 1);
    assert!(parse(&blank, "keyxtend.exe").is_err());
}

#[test]
fn no_frames_of_the_product_read_as_none_and_a_csv_without_the_columns_is_an_error() {
    let none = parse(V2, "other.exe").expect("reads");
    assert_eq!((none.presented, none.lost), (0, 0));
    assert!(parse("", "keyxtend.exe").is_err());
    let no_shown = "Application,SwapChainAddress,CPUStartQPCTime,FrameTime\n";
    assert!(
        parse(no_shown, "keyxtend.exe").is_err(),
        "no display column"
    );
    let no_chain = "Application,CPUStartQPCTime,FrameTime,DisplayedTime\n";
    assert!(parse(no_chain, "keyxtend.exe").is_err(), "no swap chain");
}

#[test]
fn a_last_row_still_being_written_is_left_out() {
    let cut = V2
        .trim_end_matches('\n')
        .rsplit_once(",NA")
        .expect("the last row")
        .0;
    let f = parse(cut, "keyxtend.exe").expect("reads");
    assert_eq!((f.presented, f.lost), (1, 0), "the cut row is not counted");
}

#[test]
fn g8_starts_presentmon_with_qpc_times_and_the_arguments_the_how_to_names() {
    let file = Path::new(r"C:\out\pm.csv");
    let got = args("keyxtend.exe", file, 75);
    let want = [
        "--process_name",
        "keyxtend.exe",
        "--output_file",
        r"C:\out\pm.csv",
        QPC_ARG,
        "--timed",
        "75",
        "--terminate_after_timed",
    ];
    assert_eq!(got, want);
    let text = how("keyxtend.exe", file, 75);
    assert!(text.contains(&want.join(" ")), "{text}");
    for part in ["presentmon_exe", "administrator"] {
        assert!(text.contains(part), "{part}: {text}");
    }
}

#[test]
fn g8_waits_out_the_recording_and_its_stop_time_from_its_start() {
    let gone = |s| Duration::from_secs(s);
    assert_eq!(stop_wait(90, 15_000, gone(60)), gone(45));
    assert_eq!(stop_wait(90, 0, gone(30)), gone(60), "no stop time");
    assert_eq!(
        stop_wait(90, 15_000, gone(200)),
        Duration::ZERO,
        "long done"
    );
}
