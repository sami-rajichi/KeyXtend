//! The command line: the media job with its count, the vault job alone, and every other form refused.

use super::*;

fn parse_all(args: &[&str]) -> Result<Job, ArgError> {
    parse(args.iter().map(OsString::from))
}

#[test]
fn the_media_job_takes_its_count() {
    let job = parse_all(&[MEDIA_JOB, MAX_PIXELS_FLAG, "100000000"]);
    assert_eq!(
        job,
        Ok(Job::Media {
            max_pixels: 100_000_000
        })
    );
}

#[test]
fn no_job_is_refused() {
    assert_eq!(parse_all(&[]), Err(ArgError::Job(String::new())));
}

#[test]
fn the_later_jobs_and_unknown_ones_are_refused() {
    for job in ["voice", "ocr", "update", "Media", MAX_PIXELS_FLAG] {
        assert_eq!(
            parse_all(&[job, MAX_PIXELS_FLAG, "5"]),
            Err(ArgError::Job(job.to_string())),
            "{job}"
        );
    }
}

#[test]
fn a_missing_or_wrong_flag_is_refused() {
    for args in [
        &[MEDIA_JOB][..],
        &[MEDIA_JOB, "5"],
        &[MEDIA_JOB, "--max-pixel", "5"],
        &[MEDIA_JOB, "5", MAX_PIXELS_FLAG],
    ] {
        assert_eq!(parse_all(args), Err(ArgError::Usage), "{args:?}");
    }
}

#[test]
fn a_count_that_is_not_above_zero_is_refused() {
    for n in ["0", "-1", "", "ten", "1.5", "18446744073709551616"] {
        assert_eq!(
            parse_all(&[MEDIA_JOB, MAX_PIXELS_FLAG, n]),
            Err(ArgError::Usage),
            "{n}"
        );
    }
}

#[test]
fn an_extra_argument_is_refused() {
    let args = [MEDIA_JOB, MAX_PIXELS_FLAG, "5", "--quiet"];
    assert_eq!(parse_all(&args), Err(ArgError::Usage));
}

#[test]
fn the_vault_job_takes_nothing_after_it() {
    assert_eq!(parse_all(&[VAULT_JOB]), Ok(Job::Vault));
    for extra in [
        &[VAULT_JOB, "x"][..],
        &[VAULT_JOB, MAX_PIXELS_FLAG, "5"],
        &[VAULT_JOB, ""],
    ] {
        assert_eq!(parse_all(extra), Err(ArgError::Alone), "{extra:?}");
    }
    let job = ArgError::Job("Vault".to_string());
    assert_eq!(parse_all(&["Vault"]), Err(job));
}

#[test]
fn each_refusal_is_one_short_line_naming_the_usage() {
    let cases = [
        (
            ArgError::Job("voice".to_string()),
            &[MEDIA_JOB, MAX_PIXELS_FLAG, VAULT_JOB][..],
        ),
        (ArgError::Usage, &[MEDIA_JOB, MAX_PIXELS_FLAG]),
        (ArgError::Alone, &[VAULT_JOB]),
    ];
    for (e, words) in cases {
        let line = e.to_string();
        assert!(!line.contains('\n'), "{line}");
        assert!(words.iter().all(|w| line.contains(w)), "{line}");
    }
}
