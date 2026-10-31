//! The command line: the media job with its count, and every other form refused.

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
fn each_refusal_is_one_short_line_naming_the_usage() {
    for e in [ArgError::Job("voice".to_string()), ArgError::Usage] {
        let line = e.to_string();
        assert!(!line.contains('\n'), "{line}");
        assert!(
            line.contains(MEDIA_JOB) && line.contains(MAX_PIXELS_FLAG),
            "{line}"
        );
    }
}
