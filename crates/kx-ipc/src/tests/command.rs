//! The worker's command words and exit codes, set once for the host that starts it and the worker itself.

use crate::{CLEAN_EXIT, MAX_PIXELS_FLAG, MEDIA_JOB, REFUSED_EXIT, VAULT_JOB};

#[test]
fn a_clean_exit_reads_as_success_and_a_refusal_does_not() {
    assert_eq!(CLEAN_EXIT, 0, "every OS reads 0 as success");
    assert_ne!(REFUSED_EXIT, CLEAN_EXIT);
}

#[test]
fn the_jobs_are_words_and_the_flag_a_long_flag() {
    let flag = |w: &str| w.starts_with("--");
    for job in [MEDIA_JOB, VAULT_JOB] {
        assert!(!job.is_empty() && !flag(job), "{job}");
    }
    assert_ne!(MEDIA_JOB, VAULT_JOB);
    assert!(flag(MAX_PIXELS_FLAG), "{MAX_PIXELS_FLAG}");
}
