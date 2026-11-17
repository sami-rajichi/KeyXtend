//! The worker's command words and exit codes, set once for the host that starts it and the worker itself.

use crate::{
    CLEAN_EXIT, FETCH_JOB, MAX_PIXELS_FLAG, MEDIA_JOB, NET_JOB, REFUSED_EXIT, SOUND_FLAG,
    VAULT_JOB, VOICE_JOB,
};

#[test]
fn a_clean_exit_reads_as_success_and_a_refusal_does_not() {
    assert_eq!(CLEAN_EXIT, 0, "every OS reads 0 as success");
    assert_ne!(REFUSED_EXIT, CLEAN_EXIT);
}

#[test]
fn the_jobs_are_distinct_words_and_the_flags_long_flags() {
    let flag = |w: &str| w.starts_with("--") && !w.contains(char::is_whitespace);
    let jobs = [MEDIA_JOB, VAULT_JOB, VOICE_JOB, FETCH_JOB, NET_JOB];
    for (i, job) in jobs.iter().enumerate() {
        assert!(!job.is_empty() && !flag(job), "{job}");
        assert!(!jobs[..i].contains(job), "{job} twice");
    }
    for f in [MAX_PIXELS_FLAG, SOUND_FLAG] {
        assert!(flag(f), "{f}");
    }
    assert_ne!(MAX_PIXELS_FLAG, SOUND_FLAG);
}
