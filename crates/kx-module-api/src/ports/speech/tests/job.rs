//! The job's limits: both ends of each range are taken and one past either end is refused by its name, a first wait
//! longer than the longest is refused, and the facts ask checks its path, its size and its job.

use super::ask::{Case, FILE, ask, job};
use super::*;
use kx_test_support::paths::native_path;

/// The name a `Range` refusal gives, or `None` when it passes.
fn named(result: Result<(), SpeechError>) -> Option<&'static str> {
    result.err().map(|e| match e {
        SpeechError::Range(name) => name,
        other => panic!("{other}"),
    })
}

/// The name the limits are refused by after `change`, alone and inside a dictation's ask.
fn refused(change: impl Fn(&mut JobLimits)) -> Option<&'static str> {
    let (mut alone, mut asked) = (job(), ask());
    change(&mut alone);
    change(&mut asked.job);
    let why = named(alone.check());
    assert_eq!(
        named(asked.check()),
        why,
        "devices and dictations refuse alike"
    );
    why
}

/// A value's name and the changes that put it at its range's start and end, just below and just above.
type Edges = (&'static str, [fn(&mut JobLimits); 4]);

/// `$field`'s edges within `$range`.
macro_rules! edges {
    ($field:ident, $range:expr) => {
        (
            stringify!($field),
            [
                |j| j.$field = *$range.start(),
                |j| j.$field = *$range.end(),
                |j| j.$field = $range.start() - 1,
                |j| j.$field = $range.end() + 1,
            ],
        )
    };
}

#[test]
fn both_ends_of_each_job_range_are_taken_and_one_past_is_refused() {
    let cases: [Edges; 8] = [
        edges!(job_mb, JobLimits::JOB_MB),
        edges!(idle_min, JobLimits::IDLE_MIN),
        edges!(ask_ms, JobLimits::ASK_MS),
        edges!(load_s, JobLimits::LOAD_S),
        edges!(warm_s, JobLimits::WARM_S),
        edges!(piece_wait_s, JobLimits::PIECE_WAIT_S),
        edges!(restart_ms, JobLimits::RESTART_MS),
        edges!(restart_max_ms, JobLimits::RESTART_MAX_MS),
    ];
    for (name, [start, end, below, above]) in cases {
        assert_eq!(refused(start), None, "{name} start");
        assert_eq!(refused(end), None, "{name} end");
        assert_eq!(refused(below), Some(name), "{name} below");
        assert_eq!(refused(above), Some(name), "{name} above");
    }
}

#[test]
fn a_first_wait_longer_than_the_longest_is_refused() {
    let order: Case<JobLimits> = ("restart_ms", |j| {
        (j.restart_ms, j.restart_max_ms) = (5_000, 4_999);
    });
    assert_eq!(
        refused(order.1),
        Some(order.0),
        "as [clipboard]'s BAD_RESTART"
    );
    assert_eq!(
        refused(|j| (j.restart_ms, j.restart_max_ms) = (5_000, 5_000)),
        None
    );
}

/// A facts ask for `FILE`.
fn facts() -> FactsAsk {
    FactsAsk {
        job: job(),
        file: Redacted::new(native_path(FILE)),
        size_mb: 1_530,
    }
}

#[test]
fn a_facts_ask_checks_its_path_size_and_job() {
    assert_eq!(facts().check(), Ok(()));
    for path in [r"models\x.gguf", r"D:\m\..\x.gguf", r"D:\m\NUL.gguf"] {
        let file = Redacted::new(native_path(path));
        let ask = FactsAsk { file, ..facts() };
        assert_eq!(ask.check(), Err(SpeechError::BadPath), "{path}");
    }
    let sized = |size_mb| FactsAsk { size_mb, ..facts() }.check();
    let top = *SpeechModel::CPU_MB.end();
    assert_eq!(sized(top), Ok(()));
    assert_eq!(sized(top + 1), Err(SpeechError::Range("size_mb")));
    assert_eq!(sized(0), Err(SpeechError::Range("size_mb")));
    let job = JobLimits {
        idle_min: 0,
        ..job()
    };
    let why = FactsAsk { job, ..facts() }.check();
    assert_eq!(why, Err(SpeechError::Range("idle_min")));
    let shown = format!("{:?}", facts());
    assert!(
        !shown.contains("parakeet") && shown.contains("size_mb: 1530"),
        "{shown}"
    );
}
