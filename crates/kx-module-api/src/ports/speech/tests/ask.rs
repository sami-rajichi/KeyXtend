//! The asks: a full ask and a listen-only ask pass, each value just past its range is refused by its name, the ranges
//! are the plan's `[voice]` bounds, and no ask prints its model's path. The job's limits and the facts ask are in
//! `job`.

use super::*;
use kx_test_support::paths::native_path;

/// A model path no output may show, written for Windows; wrap it in `native_path`.
pub(super) const FILE: &str = r"D:\Models\parakeet-9913.gguf";

/// A value's name and a change that puts it just past its range.
pub(super) type Case<T> = (&'static str, fn(&mut T));

pub(super) fn job() -> JobLimits {
    JobLimits {
        job_mb: 512,
        idle_min: 5,
        ask_ms: 3_000,
        load_s: 60,
        warm_s: 180,
        piece_wait_s: 120,
        restart_ms: 1_000,
        restart_max_ms: 30_000,
    }
}

fn model() -> SpeechModel {
    let file = Redacted::new(native_path(FILE));
    SpeechModel {
        file,
        lang: "fr-FR".into(),
        runner: "transducer".into(),
        cpu_mb: 1_200,
    }
}

fn card() -> CardUse {
    let id = CardId::new("Vulkan0").unwrap();
    CardUse {
        id,
        need_mb: 900,
        margin_mb: 512,
    }
}

pub(super) fn ask() -> SpeechAsk {
    SpeechAsk {
        job: job(),
        model: Some(model()),
        card: Some(card()),
        threads: 0,
        piece_s: 30,
        pad_ms: 300,
        level_hz: 30,
        record_max_s: 600,
        silence_stop_s: Some(5),
    }
}

fn check_ask() -> CheckAsk {
    let (model, card) = (model(), card());
    CheckAsk {
        job: job(),
        model,
        card,
        threads: 8,
        clip: "ar".into(),
    }
}

/// The name an ask is refused by after `change`, or `None` when it passes.
fn refused(change: impl FnOnce(&mut SpeechAsk)) -> Option<&'static str> {
    let mut ask = ask();
    change(&mut ask);
    match ask.check() {
        Ok(()) => None,
        Err(SpeechError::Range(name)) => Some(name),
        Err(other) => panic!("{other}"),
    }
}

/// The name a check is refused by after `change`, or `None` when it passes.
fn check_refused(change: impl FnOnce(&mut CheckAsk)) -> Option<&'static str> {
    let mut ask = check_ask();
    change(&mut ask);
    ask.check().err().map(|e| match e {
        SpeechError::Range(name) => name,
        other => panic!("{other}"),
    })
}

#[test]
fn a_full_ask_and_a_listen_only_ask_pass() {
    assert_eq!(refused(|_| {}), None);
    let listen = refused(|a| (a.model, a.card) = (None, None));
    assert_eq!(
        listen, None,
        "a job without a model records and hands its sound over"
    );
    assert_eq!(refused(|a| a.card = None), None, "the processor runs it");
    assert_eq!(
        refused(|a| a.silence_stop_s = None),
        None,
        "the silence stop is off"
    );
}

#[test]
fn a_card_without_a_model_is_refused() {
    assert_eq!(refused(|a| a.model = None), Some("card"));
}

#[test]
fn each_value_just_past_its_range_is_refused_by_its_name() {
    let above: [Case<SpeechAsk>; 6] = [
        ("threads", |a| a.threads = SpeechAsk::THREADS.end() + 1),
        ("piece_s", |a| a.piece_s = SpeechAsk::PIECE_S.end() + 1),
        ("pad_ms", |a| a.pad_ms = SpeechAsk::PAD_MS.end() + 1),
        ("level_hz", |a| a.level_hz = SpeechAsk::LEVEL_HZ.end() + 1),
        ("record_max_s", |a| {
            a.record_max_s = SpeechAsk::RECORD_MAX_S.end() + 1;
        }),
        ("silence_stop_s", |a| {
            a.silence_stop_s = Some(SpeechAsk::SILENCE_STOP_S.end() + 1);
        }),
    ];
    let below: [Case<SpeechAsk>; 4] = [
        ("piece_s", |a| a.piece_s = SpeechAsk::PIECE_S.start() - 1),
        ("level_hz", |a| a.level_hz = SpeechAsk::LEVEL_HZ.start() - 1),
        ("record_max_s", |a| {
            a.record_max_s = SpeechAsk::RECORD_MAX_S.start() - 1;
        }),
        ("silence_stop_s", |a| {
            a.silence_stop_s = Some(SpeechAsk::SILENCE_STOP_S.start() - 1);
        }),
    ];
    for (name, change) in above.into_iter().chain(below) {
        assert_eq!(refused(change), Some(name), "{name}");
    }
    assert!(
        *SpeechAsk::SILENCE_STOP_S.start() > 0,
        "0 is no stop: the setting's off is `None`"
    );
}

#[test]
fn every_range_end_is_taken() {
    let ends: [fn(&mut SpeechAsk); 4] = [
        |a| (a.threads, a.pad_ms) = (*SpeechAsk::THREADS.end(), *SpeechAsk::PAD_MS.end()),
        |a| (a.piece_s, a.level_hz) = (*SpeechAsk::PIECE_S.start(), *SpeechAsk::LEVEL_HZ.start()),
        |a| a.record_max_s = *SpeechAsk::RECORD_MAX_S.end(),
        |a| a.silence_stop_s = Some(*SpeechAsk::SILENCE_STOP_S.start()),
    ];
    for change in ends {
        assert_eq!(refused(change), None);
    }
}

#[test]
fn the_ranges_are_the_plans_voice_bounds() {
    let ask = (SpeechAsk::THREADS, SpeechAsk::PIECE_S, SpeechAsk::PAD_MS);
    assert_eq!(ask, (0..=64, 10..=120, 0..=2_000));
    let run = (
        SpeechAsk::LEVEL_HZ,
        SpeechAsk::RECORD_MAX_S,
        SpeechAsk::SILENCE_STOP_S,
    );
    assert_eq!(run, (5..=60, 10..=3_600, 1..=30));
    assert_eq!(
        (JobLimits::JOB_MB, JobLimits::IDLE_MIN),
        (64..=4_096, 1..=120)
    );
    assert_eq!(CardUse::MARGIN_MB, 0..=8_192);
}

#[test]
fn a_tag_is_short_plain_ascii_as_the_frames_cap_it() {
    let over = "a".repeat(*TAG_LEN.end() + 1);
    for tag in ["", "fr FR", "مر", over.as_str()] {
        let (lang, runner) = (tag.to_owned(), tag.to_owned());
        assert_eq!(
            refused(|a| a.model.as_mut().unwrap().lang = lang),
            Some("lang")
        );
        assert_eq!(
            refused(|a| a.model.as_mut().unwrap().runner = runner),
            Some("runner")
        );
    }
    let full = "a".repeat(*TAG_LEN.end());
    assert_eq!(refused(|a| a.model.as_mut().unwrap().runner = full), None);
}

#[test]
fn a_model_with_a_bad_path_or_size_is_refused() {
    let paths = [
        r"models\x.gguf",
        r"\\.\pipe\x",
        r"D:\m\..\x.gguf",
        r"D:\m\x.gguf:s",
    ];
    for path in paths {
        let file = Redacted::new(native_path(path));
        let why = refused(|a| a.model.as_mut().unwrap().file = file);
        assert_eq!(why, Some("file"), "{path}");
    }
    let cpu = |change: fn(&mut SpeechModel)| refused(|a| change(a.model.as_mut().unwrap()));
    assert_eq!(cpu(|m| m.cpu_mb = 0), Some("cpu_mb"));
    assert_eq!(cpu(|m| m.cpu_mb = *SpeechModel::CPU_MB.end()), None);
    assert_eq!(
        cpu(|m| m.cpu_mb = SpeechModel::CPU_MB.end() + 1),
        Some("cpu_mb")
    );
}

#[test]
fn a_card_outside_its_ranges_is_refused() {
    let card = |change: fn(&mut CardUse)| refused(|a| change(a.card.as_mut().unwrap()));
    assert_eq!(card(|c| c.need_mb = 0), Some("need_mb"));
    assert_eq!(card(|c| c.need_mb = *CardUse::NEED_MB.end()), None);
    assert_eq!(
        card(|c| c.need_mb = CardUse::NEED_MB.end() + 1),
        Some("need_mb")
    );
    assert_eq!(card(|c| c.margin_mb = *CardUse::MARGIN_MB.start()), None);
    assert_eq!(card(|c| c.margin_mb = *CardUse::MARGIN_MB.end()), None);
    let margin = card(|c| c.margin_mb = CardUse::MARGIN_MB.end() + 1);
    assert_eq!(margin, Some("margin_mb"));
}

#[test]
fn a_check_needs_a_shipped_clips_language_and_its_limits() {
    assert_eq!(check_refused(|_| {}), None);
    assert_eq!(check_refused(|c| c.clip = "arabic".into()), Some("clip"));
    let threads = SpeechAsk::THREADS.end() + 1;
    assert_eq!(check_refused(|c| c.threads = threads), Some("threads"));
    assert_eq!(check_refused(|c| c.job.idle_min = 0), Some("idle_min"));
    assert_eq!(check_refused(|c| c.model.cpu_mb = 0), Some("cpu_mb"));
    assert_eq!(check_refused(|c| c.card.need_mb = 0), Some("need_mb"));
}

#[test]
fn no_ask_prints_its_models_path() {
    let shown = format!("{:?}{:?}", ask(), check_ask());
    let hidden = !shown.contains("parakeet") && !shown.contains("9913");
    assert!(hidden, "{shown}");
    assert!(
        shown.contains("fr-FR") && shown.contains("Vulkan0"),
        "{shown}"
    );
}
