//! The fit against this PC (plan edge case 23) and a file no row names (edge case 37).

use super::*;
use crate::settings::VoiceSettings;
use kx_module_api::Redacted;
use kx_module_api::ports::{MB, Memory, ModelFacts, ModelState, SpeechModel};

/// A processor with `total_mb` of memory, half of it free.
fn pc(total_mb: u32) -> Memory {
    Memory {
        total_mb,
        free_mb: total_mb / 2,
    }
}

/// The facts an other file tells.
fn facts(langs: &[&str]) -> ModelFacts {
    ModelFacts {
        runner: "transcribe".into(),
        langs: langs.iter().map(|l| (*l).to_owned()).collect(),
    }
}

#[test]
fn a_model_too_big_for_the_pc_is_not_for_this_pc_with_the_reason() {
    let arabic = row("cohere-arabic-q8");
    let job_mb = 512;
    let need_mb = arabic.cpu_mb + job_mb;
    assert_eq!(
        arabic.not_here(&pc(need_mb), job_mb),
        None,
        "exactly enough fits"
    );
    let short = pc(need_mb - 1);
    assert_eq!(
        arabic.not_here(&short, job_mb),
        Some(ModelState::NotHere {
            need_mb,
            have_mb: need_mb - 1,
            free_mb: short.free_mb
        }),
        "the fit goes by the total, the reason shows the free memory too"
    );
    assert_eq!(
        row("whisper-small-q5km").not_here(&short, job_mb),
        None,
        "a small model still fits"
    );
}

#[test]
fn the_usable_models_follow_the_order_and_none_usable_is_empty() {
    let t = shipped();
    let usable = |total, lang| {
        t.usable(lang, &pc(total), 512)
            .into_iter()
            .map(|r| r.id.as_str())
            .collect::<Vec<_>>()
    };
    assert_eq!(usable(8192, "ar").len(), 8);
    assert_eq!(
        usable(2000, "ar"),
        ["whisper-small-q5km"],
        "only Whisper Small fits 2 GB"
    );
    assert_eq!(
        usable(2400, "ar"),
        [
            "whisper-turbo-q5km",
            "nemotron-0.6b-q8",
            "whisper-small-q5km"
        ]
    );
    assert_eq!(
        usable(2700, "fr"),
        [
            "parakeet-v3-q8",
            "whisper-turbo-q5km",
            "whisper-small-q5km",
            "nemotron-0.6b-q8"
        ]
    );
    assert!(
        usable(1024, "en").is_empty(),
        "none usable: the panel says so"
    );
    assert!(usable(65_536, "kl").is_empty());
}

#[test]
fn a_file_no_row_names_is_an_other_file_capped_at_its_size_plus_job_mb() {
    let t = shipped();
    assert!(t.named("ggml-small-q5_1.bin").is_none());
    let s = VoiceSettings::from_defaults().unwrap();
    let other = Other {
        name: "ggml-small-q5_1.bin".into(),
        bytes: 190 * MB + 1,
    };
    assert_eq!(
        other.need_mb(),
        Some(191),
        "its size, rounded up to a whole MB"
    );
    assert_eq!(other.cap_mb(s.job_mb), Some(191 + 512));
    let ask = other.facts(&folder(), s.job()).unwrap();
    assert_eq!((ask.size_mb, ask.check()), (191, Ok(())));
    assert_eq!(ask.file.expose(), &folder().join("ggml-small-q5_1.bin"));
    for (bytes, need) in [
        (1, Some(1)),
        (MB, Some(1)),
        (MB + 1, Some(2)),
        (0, None),
        (65_537 * MB, None),
    ] {
        assert_eq!(
            Other {
                name: "x.gguf".into(),
                bytes
            }
            .need_mb(),
            need,
            "{bytes}"
        );
    }
}

#[test]
fn an_other_file_serves_the_languages_its_facts_name() {
    let s = VoiceSettings::from_defaults().unwrap();
    let other = Other {
        name: "my-model.gguf".into(),
        bytes: 600 * MB,
    };
    let told = facts(&["en-US", "fr-FR", "ar", "PT_br"]);
    assert_eq!(code_in(&told, "fr"), Some("fr-FR"));
    assert_eq!(code_in(&told, "ar"), Some("ar"));
    assert_eq!(code_in(&told, "pt"), Some("PT_br"));
    assert_eq!(code_in(&told, "de"), None);
    let model = other.model(&folder(), &told, "fr").unwrap();
    let want = SpeechModel {
        file: Redacted::new(folder().join("my-model.gguf")),
        lang: "fr-FR".into(),
        runner: "transcribe".into(),
        cpu_mb: 600,
    };
    assert_eq!(model, want);
    assert_eq!(s.ask(Some(model), None).check(), Ok(()));
    assert!(other.model(&folder(), &told, "de").is_none());
}
