//! A card serves a language only on a result for the clip that language's check reads (ADR-0052): one in English
//! never serves Arabic, an old result of no language serves none, and a fail in Arabic keeps English (edge case 3).

use kx_module_api::ports::Fingerprint;

use super::*;
use crate::device::Why;

const PASS: Verdict = Verdict::Passed {
    gain: Hundredths(900),
};
const DIFFERS: Verdict = Verdict::Differs {
    words: 4,
    allowed: 1,
};

/// The folder with Cohere Arabic, which knows Arabic and English, its fingerprint read.
fn cohere() -> (Folder, &'static Row) {
    let row = row("cohere-arabic-q8");
    let mut folder = scanned(&[(&row.file, row.bytes)]);
    folder.printed(&at(&row.file), row.bytes, Ok(row.print));
    (folder, row)
}

/// The card's check of file `print` under its driver, on the clip of `lang`.
fn kept(print: Fingerprint, lang: Option<&str>, verdict: Verdict) -> Checked {
    Checked {
        card: card(),
        driver: "1.0".into(),
        model: print,
        lang: lang.map(str::to_owned),
        verdict,
        day: Date::new(2026, 11, 20).unwrap(),
    }
}

/// `device` picked and Cohere Arabic serving English too.
fn both(device: &str) -> VoiceSettings {
    let mut settings = settings(&[("en", "cohere-arabic-q8")]);
    settings.device = device.into();
    settings
}

/// The card and why not, as `iso` is served with `kept` checks.
fn served(settings: &VoiceSettings, kept: Vec<Checked>, iso: &str) -> (bool, Option<Why>) {
    let (folder, _) = cohere();
    let pc = devices(32_768, 8_000);
    let ready = supplied(&folder, settings, &pc, &Checks::new(kept), iso).unwrap();
    (ready.card.is_some(), ready.why)
}

#[test]
fn a_result_in_english_does_not_let_the_card_serve_arabic_and_one_in_arabic_does() {
    let print = cohere().1.print;
    let english = vec![kept(print, Some("en"), PASS)];
    assert_eq!(
        served(&both("Vulkan0"), english, "ar"),
        (false, Some(Why::Unchecked))
    );
    let arabic = vec![kept(print, Some("ar"), PASS)];
    assert_eq!(served(&both("Vulkan0"), arabic, "ar"), (true, None));
}

#[test]
fn an_old_result_without_a_language_serves_nothing_until_a_new_check() {
    let print = cohere().1.print;
    for iso in ["ar", "en"] {
        let old = vec![kept(print, None, PASS)];
        let picked = served(&both("Vulkan0"), old.clone(), iso);
        assert_eq!(picked, (false, Some(Why::Unchecked)), "{iso}");
        assert_eq!(served(&both("auto"), old, iso), (false, None), "{iso}");
    }
}

#[test]
fn edge_3_a_card_failed_in_arabic_writes_arabic_on_the_processor_and_keeps_english() {
    let print = cohere().1.print;
    let rows = vec![
        kept(print, Some("en"), PASS),
        kept(print, Some("ar"), DIFFERS),
    ];
    let arabic = served(&both("Vulkan0"), rows.clone(), "ar");
    assert_eq!(arabic, (false, Some(Why::Failed(DIFFERS))));
    assert_eq!(served(&both("Vulkan0"), rows.clone(), "en"), (true, None));
    assert_eq!(served(&both("auto"), rows.clone(), "ar"), (false, None));
    assert_eq!(served(&both("auto"), rows, "en"), (true, None));
}

#[test]
fn an_other_file_serves_on_the_result_of_the_clip_its_facts_name() {
    let mut folder = scanned(&[("ggml-medium.bin", 1_530 * MB)]);
    folder.printed(&at("ggml-medium.bin"), 1_530 * MB, Ok(stranger()));
    folder.facts_read(stranger(), Ok(facts(&["en", "fr-FR"])));
    let mut settings = settings(&[("fr", "ggml-medium.bin")]);
    settings.device = "Vulkan0".into();
    let pc = devices(32_768, 8_000);
    let card_in = |lang: &str| {
        let checks = Checks::new(vec![kept(stranger(), Some(lang), PASS)]);
        let ready = supplied(&folder, &settings, &pc, &checks, "fr").unwrap();
        ready.card.is_some()
    };
    assert!(!card_in("en"), "French reads its own clip");
    assert!(card_in("fr"));
}
