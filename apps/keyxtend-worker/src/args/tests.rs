//! The command line: the media job with its count, the vault, voice, fetch and speech jobs alone, and every other form
//! refused.

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
    for job in ["ocr", "update", "Media", "Fetch", MAX_PIXELS_FLAG] {
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
fn the_vault_voice_fetch_and_speech_jobs_take_nothing_after_them() {
    let jobs = [
        (VAULT_JOB, Job::Vault),
        (VOICE_JOB, Job::Voice),
        (FETCH_JOB, Job::Fetch),
        (NET_JOB, Job::SpeechNet),
    ];
    for (word, job) in jobs {
        assert_eq!(parse_all(&[word]), Ok(job));
        for extra in [&[word, "x"][..], &[word, MAX_PIXELS_FLAG, "5"], &[word, ""]] {
            assert_eq!(parse_all(extra), Err(ArgError::Alone(word)), "{extra:?}");
        }
    }
    for word in ["Vault", "Voice", "voices", "speech", "Speech-Net"] {
        let job = ArgError::Job(word.to_string());
        assert_eq!(parse_all(&[word]), Err(job));
    }
}

#[test]
fn each_refusal_is_one_short_line_naming_the_usage() {
    let cases = [
        (
            ArgError::Job("ocr".to_string()),
            &[
                MEDIA_JOB,
                MAX_PIXELS_FLAG,
                VAULT_JOB,
                VOICE_JOB,
                FETCH_JOB,
                NET_JOB,
            ][..],
        ),
        (ArgError::Usage, &[MEDIA_JOB, MAX_PIXELS_FLAG]),
        (ArgError::Alone(VAULT_JOB), &[VAULT_JOB]),
        (ArgError::Alone(VOICE_JOB), &[VOICE_JOB]),
        (ArgError::Alone(FETCH_JOB), &[FETCH_JOB]),
        (ArgError::Alone(NET_JOB), &[NET_JOB]),
    ];
    for (e, words) in cases {
        let line = e.to_string();
        assert!(!line.contains('\n'), "{line}");
        assert!(words.iter().all(|w| line.contains(w)), "{line}");
    }
}

#[cfg(feature = "gate")]
#[test]
fn a_gate_build_takes_one_sound_file_after_the_voice_job() {
    let job = parse_all(&[VOICE_JOB, SOUND_FLAG, "clip.wav"]);
    assert_eq!(job, Ok(Job::Played(std::path::PathBuf::from("clip.wav"))));
    for args in [
        &[VOICE_JOB, SOUND_FLAG][..],
        &[VOICE_JOB, SOUND_FLAG, "a.wav", "b.wav"],
    ] {
        assert_eq!(parse_all(args), Err(ArgError::Sound), "{args:?}");
    }
}

#[cfg(not(feature = "gate"))]
#[test]
fn a_build_without_the_gate_refuses_the_sound_flag_the_stage_refuses() {
    let job = parse_all(&[VOICE_JOB, kx_ipc::SOUND_FLAG, "clip.wav"]);
    assert_eq!(job, Err(ArgError::Alone(VOICE_JOB)));
}
