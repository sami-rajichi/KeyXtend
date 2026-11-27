//! The job's processor share, its per-job UI table, and the caps a real job reads back.

use super::*;

/// A test job's memory cap, in MB.
const MB: u32 = 64;

/// `cores` cores at `pct` percent each.
const fn cap(cores: usize, pct: u8) -> CpuCap {
    CpuCap { cores, pct }
}

#[test]
fn each_core_takes_its_share_of_the_whole_pc() {
    assert_eq!(share(cap(2, 30), 16), 375, "0.6 of one processor in 16");
    assert_eq!(share(cap(2, 30), 8), 750);
    assert_eq!(share(cap(2, 30), 2), 3_000);
}

#[test]
fn the_share_is_never_zero_nor_past_the_pc() {
    assert_eq!(share(cap(1, 1), 256), 1, "never 0, which Windows refuses");
    assert_eq!(share(cap(1, 0), 4), 1, "no speed still runs");
    assert_eq!(share(cap(0, 30), 4), 750, "at least one core");
    assert_eq!(
        share(cap(16, 30), 4),
        3_000,
        "no more cores than the PC has"
    );
    assert_eq!(share(cap(2, u8::MAX), 2), 10_000, "no core past its speed");
    assert_eq!(
        share(cap(usize::MAX, 100), usize::MAX),
        10_000,
        "no overflow"
    );
    assert_eq!(share(cap(2, 30), 0), 3_000, "an unknown PC counts one");
}

#[test]
fn the_pc_counts_at_least_the_processors_std_sees() {
    let std = thread::available_parallelism().map_or(1, usize::from);
    assert!(logical() >= std, "{} < {std}", logical());
}

/// A launch of the job `word`.
fn of(word: &str) -> Launch {
    Launch {
        args: vec![word.to_owned()],
        memory_mb: MB,
    }
}

#[test]
fn every_job_keeps_every_ui_limit_and_one_the_table_lacks_too() {
    let words = [
        MediaFrames::JOB,
        VaultFrames::JOB,
        VoiceFrames::JOB,
        FetchFrames::JOB,
        NetFrames::JOB,
        RecordFrames::JOB,
        PickFrames::JOB,
    ];
    assert_eq!(
        JOBS_UI.map(|(word, _)| word),
        words,
        "each job once, in one table"
    );
    for word in words {
        assert_eq!(ui_of(&of(word)), UI, "{word}");
    }
    assert_eq!(ui_of(&of("probe")), UI);
    let none = Launch {
        args: Vec::new(),
        memory_mb: MB,
    };
    assert_eq!(ui_of(&none), UI);
}

#[test]
fn a_normal_job_has_no_processor_cap() {
    assert_eq!(Job::new(MB, UI).unwrap().cpu_cap(), None);
}

#[test]
fn a_weak_job_caps_each_core_and_reads_it_back() {
    let job = Job::new(MB, UI).unwrap();
    job.weaken(cap(2, 30)).unwrap();
    assert_eq!(job.cpu_cap(), Some(share(cap(2, 30), logical())));
}
