//! What serves a language (the flow's `Supply`): the pick or the default from the folder, the device rule, and why
//! nothing serves (edges 17, 18, 23, 37).

use super::*;
use crate::VoiceSettings;
use crate::counts::{Checked, Checks};
use crate::flow::{Lack, Supply};
use kx_module_api::ports::{CardId, Date, Device, DeviceKind, Hundredths, Memory, Verdict};

mod langs;

/// The processor with `total_mb` of memory and a card `Vulkan0` with `card_free_mb` free.
fn devices(total_mb: u32, card_free_mb: u32) -> Vec<Device> {
    let memory = |total_mb, free_mb| Memory { total_mb, free_mb };
    let cpu = Device {
        kind: DeviceKind::Cpu,
        name: "CPU".into(),
        memory: memory(total_mb, total_mb / 2),
        driver: String::new(),
        cores: 8,
        built_in: false,
    };
    let card = Device {
        kind: DeviceKind::Card(card()),
        name: "Card".into(),
        memory: memory(8_192, card_free_mb),
        cores: 0,
        built_in: false,
        driver: "1.0".into(),
    };
    vec![cpu, card]
}

fn card() -> CardId {
    CardId::new("Vulkan0").unwrap()
}

/// Settings with `picks` as language and value pairs.
fn settings(picks: &[(&str, &str)]) -> VoiceSettings {
    let mut settings = VoiceSettings::from_defaults().unwrap();
    let pair = |(l, v): &(&str, &str)| ((*l).to_owned(), (*v).to_owned());
    settings.picks = picks.iter().map(pair).collect();
    settings
}

/// What `folder` supplies for `iso` with `settings`, `devices` and `checks`.
fn supplied(
    folder: &Folder,
    settings: &VoiceSettings,
    devices: &[Device],
    checks: &Checks,
    iso: &str,
) -> Result<Ready, Lack> {
    let clips = shipped();
    let stock = Stock {
        settings,
        folder,
        devices,
        checks,
        clips: &clips,
    };
    stock.supply(iso)
}

/// The shipped clips' languages, in their order.
fn shipped() -> Vec<String> {
    ["en", "fr", "ar"].map(String::from).to_vec()
}

use crate::flow::Ready;

#[test]
fn the_default_in_the_folder_serves_with_its_own_code() {
    let parakeet = row("parakeet-v3-q8");
    let mut folder = scanned(&[("renamed.gguf", parakeet.bytes)]);
    folder.printed(&at("renamed.gguf"), parakeet.bytes, Ok(parakeet.print));
    let ready = supplied(&folder, &settings(&[]), &[], &Checks::default(), "fr").unwrap();
    assert_eq!(
        ready.model.file.expose(),
        &Path::new(DIR).join("renamed.gguf")
    );
    assert_eq!(ready.model.lang, parakeet.code("fr").unwrap());
    assert_eq!(ready.model.cpu_mb, parakeet.cpu_mb);
    assert_eq!(
        (ready.name.expose().as_str(), ready.print),
        (parakeet.id.as_str(), parakeet.print)
    );
    assert_eq!((ready.bytes, ready.card), (parakeet.bytes, None));
}

#[test]
fn a_missing_file_or_folder_asks_for_its_download() {
    let parakeet = row("parakeet-v3-q8");
    let missing = Err(Lack::Missing {
        bytes: parakeet.bytes,
    });
    let empty = scanned(&[]);
    assert_eq!(
        supplied(&empty, &settings(&[]), &[], &Checks::default(), "en"),
        missing
    );
    let none = Folder::new("");
    assert_eq!(
        supplied(&none, &settings(&[]), &[], &Checks::default(), "en"),
        missing
    );
    let small = row("whisper-small-q5km");
    let picked = settings(&[("en", &small.id)]);
    let want = Err(Lack::Missing { bytes: small.bytes });
    assert_eq!(
        supplied(&empty, &picked, &[], &Checks::default(), "en"),
        want
    );
}

#[test]
fn a_language_no_model_knows_has_none() {
    let folder = scanned(&[]);
    let got = supplied(&folder, &settings(&[]), &[], &Checks::default(), "tlh");
    assert_eq!(got, Err(Lack::Unknown));
}

#[test]
fn a_model_too_big_for_this_pc_does_not_fit() {
    let parakeet = row("parakeet-v3-q8");
    let folder = scanned(&[(&parakeet.file, parakeet.bytes)]);
    let small_pc = devices(parakeet.cpu_mb, 0);
    let got = supplied(&folder, &settings(&[]), &small_pc, &Checks::default(), "en");
    assert_eq!(got, Err(Lack::NoFit));
}

#[test]
fn a_picked_other_file_serves_the_languages_its_facts_name() {
    let mut folder = scanned(&[("ggml-medium.bin", 1_530 * MB)]);
    let picked = settings(&[("fr", "ggml-medium.bin")]);
    let got = supplied(&folder, &picked, &[], &Checks::default(), "fr");
    assert_eq!(got, Err(Lack::Checking), "its fingerprint is not read yet");
    folder.printed(&at("ggml-medium.bin"), 1_530 * MB, Ok(stranger()));
    let got = supplied(&folder, &picked, &[], &Checks::default(), "fr");
    assert_eq!(got, Err(Lack::Checking), "its facts are not read yet");
    folder.facts_read(stranger(), Ok(facts(&["en", "fr-FR"])));
    let ready = supplied(&folder, &picked, &[], &Checks::default(), "fr").unwrap();
    assert_eq!(ready.model.lang, "fr-FR");
    assert_eq!(ready.model.cpu_mb, 1_530, "its size counts as its memory");
    assert_eq!(
        (ready.name.expose().as_str(), ready.print),
        ("ggml-medium.bin", stranger())
    );
}

#[test]
fn a_picked_file_whose_facts_or_fingerprint_cannot_be_read_is_a_stranger() {
    let picked = settings(&[("fr", "odd.bin")]);
    let mut refused = scanned(&[("odd.bin", 9)]);
    refused.printed(&at("odd.bin"), 9, Ok(stranger()));
    refused.facts_read(stranger(), Err(SpeechError::BadFile));
    let got = supplied(&refused, &picked, &[], &Checks::default(), "fr");
    assert_eq!(got, Err(Lack::Stranger));
    let mut broken = scanned(&[("odd.bin", 9)]);
    broken.printed(&at("odd.bin"), 9, Err(ModelError::Io));
    let got = supplied(&broken, &picked, &[], &Checks::default(), "fr");
    assert_eq!(got, Err(Lack::Stranger));
}

#[test]
fn a_picked_file_that_is_gone_takes_the_default() {
    let parakeet = row("parakeet-v3-q8");
    let folder = scanned(&[(&parakeet.file, parakeet.bytes)]);
    let picked = settings(&[("fr", "gone.gguf")]);
    let ready = supplied(&folder, &picked, &[], &Checks::default(), "fr").unwrap();
    assert_eq!(ready.name.expose(), &parakeet.id);
}

#[test]
fn automatic_runs_on_a_card_once_its_check_passed_and_it_has_room() {
    let parakeet = row("parakeet-v3-q8");
    let unread = scanned(&[(&parakeet.file, parakeet.bytes)]);
    let mut folder = unread.clone();
    folder.printed(&at(&parakeet.file), parakeet.bytes, Ok(parakeet.print));
    let s = settings(&[]);
    let pc = devices(32_768, 8_000);
    assert_eq!(
        supplied(&folder, &s, &pc, &Checks::default(), "en")
            .unwrap()
            .card,
        None
    );
    let passed = Checked {
        card: card(),
        driver: "1.0".into(),
        model: parakeet.print,
        lang: Some("en".into()),
        verdict: Verdict::Passed {
            gain: Hundredths(900),
        },
        day: Date::new(2026, 11, 14).unwrap(),
    };
    let checks = Checks::new(vec![passed]);
    let used = supplied(&folder, &s, &pc, &checks, "en")
        .unwrap()
        .card
        .unwrap();
    assert_eq!((used.id, used.need_mb), (card(), parakeet.card_mb));
    let ready = supplied(&unread, &s, &pc, &checks, "en").unwrap();
    assert_eq!(
        ready.card, None,
        "a file not read yet runs on the processor"
    );
    let crowded = devices(32_768, parakeet.card_mb);
    assert_eq!(
        supplied(&folder, &s, &crowded, &checks, "en").unwrap().card,
        None
    );
}

#[test]
fn download_fetches_the_pick_else_the_default() {
    let folder = scanned(&[]);
    let small = row("whisper-small-q5km");
    let checks = Checks::default();
    let wanted = |settings: &VoiceSettings| {
        let stock = Stock {
            settings,
            folder: &folder,
            devices: &[],
            checks: &checks,
            clips: &[],
        };
        stock.wanted("en").map(|r| r.id.clone())
    };
    assert_eq!(
        wanted(&settings(&[("en", &small.id)])),
        Some(small.id.clone())
    );
    let default = table().unwrap().default_for("en").map(|r| r.id.clone());
    assert_eq!(wanted(&settings(&[("en", "mine.gguf")])), default);
}
