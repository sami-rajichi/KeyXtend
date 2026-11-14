//! Every row is valid: the trial's exact files, sizes and fingerprints, HTTPS addresses at a fixed revision on a listed
//! host, a download ask the port takes, languages with the model's own codes, a licence, and memory in range.

use super::*;
use crate::settings::VoiceSettings;
use kx_module_api::ports::{
    CardId, CardUse, Fingerprint, LINK_SCHEME, Link, PART_EXT, SpeechModel, TAG_LEN, fits_iso,
};
use std::path::Path;

/// The ten files of `docs/items/P11.md`, with their bytes and SHA-256.
const TRIAL: [(&str, u64, &str); 10] = [
    (
        "whisper-small-Q5_K_M.gguf",
        193_749_056,
        "326cd00c3e7217c751667c7c1600eaf7e0de174e186ca2c16b4bf590251c3c3b",
    ),
    (
        "whisper-large-v3-turbo-Q5_K_M.gguf",
        619_628_096,
        "06fc9923c09fb3ae878ea12c34d1be3ed18fad039ab14b4b045c7b6c40af56b2",
    ),
    (
        "whisper-large-v3-Q5_K_M.gguf",
        1_161_143_008,
        "6053d0fd69a0fd48b8fea5ea7a52b9e0cde389343566fa30e453a1b2b258dc38",
    ),
    (
        "parakeet-tdt-0.6b-v3-Q8_0.gguf",
        739_508_576,
        "5859f77944efcd8eafa23a6350731960b2b55b2203df51f319665c807d802cc7",
    ),
    (
        "canary-1b-v2-Q8_0.gguf",
        1_144_290_016,
        "224f83d1bc487b3303b495a7d6874912fdece93de19d1a04b550829c30a5d289",
    ),
    (
        "nemotron-3.5-asr-streaming-0.6b-Q8_0.gguf",
        751_094_240,
        "b94545b313b3223fda7b2857a52681da813935c2127643d1e9ff0c23d988089c",
    ),
    (
        "Qwen3-ASR-0.6B-Q8_0.gguf",
        850_423_456,
        "f081b2d5e23bd669d92cc331d722a8a0681943b8e6f34b48996fd5c319b5acd8",
    ),
    (
        "Qwen3-ASR-1.7B-Q8_0.gguf",
        2_185_030_624,
        "9a0d81792dfea2d5f278b8a63deb3ea6e02139ce42c2301f32ea19c4f77526b7",
    ),
    (
        "cohere-transcribe-03-2026-Q8_0.gguf",
        2_410_655_232,
        "931916663432fd895423a4291a8400221802b288967ca2d435fc5e3141c9e71e",
    ),
    (
        "cohere-transcribe-arabic-07-2026-Q8_0.gguf",
        2_410_655_136,
        "910de5c9c57f9fd8a280e1701f9cd96f63768878c1ca9e4ecf23638a5e0fef16",
    ),
];

#[test]
fn the_table_holds_exactly_the_trials_ten_files() {
    let rows = shipped().rows();
    assert_eq!(rows.len(), TRIAL.len());
    for (file, bytes, sha) in TRIAL {
        let row = shipped().named(file).unwrap_or_else(|| panic!("{file}"));
        assert_eq!(row.bytes, bytes, "{file}");
        assert_eq!(row.print.hex(), sha, "{file}");
        assert_eq!(
            shipped()
                .printed(&Fingerprint::from_hex(sha).unwrap())
                .map(|r| &r.id),
            Some(&row.id)
        );
    }
}

#[test]
fn every_address_is_https_at_a_fixed_revision_on_a_listed_host() {
    for row in shipped().rows() {
        assert!(!row.links.is_empty(), "{}", row.id);
        for link in &row.links {
            assert!(Link::new(link.clone()).is_ok(), "{link}");
            let rest = link.strip_prefix(LINK_SCHEME).unwrap();
            let (host, path) = rest.split_once('/').unwrap();
            assert!(row.hosts.iter().any(|h| h == host), "{link}");
            assert!(path.ends_with(&format!("/{}", row.file)), "{link}");
            let fixed = path.split('/').any(|p| {
                p.len() == 40
                    && p.bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
            });
            assert!(fixed, "{link} names no commit");
            assert!(!path.contains("/main/"), "{link}");
        }
    }
}

#[test]
fn every_row_makes_a_download_the_port_takes() {
    let s = VoiceSettings::from_defaults().unwrap();
    for row in shipped().rows() {
        let ask = row
            .fetch(Path::new(FOLDER), s.job(), s.min_free_mb, s.stall_s)
            .unwrap();
        assert_eq!(ask.check(), Ok(()), "{}", row.id);
        assert_eq!(
            (ask.bytes, ask.print, ask.links.len()),
            (row.bytes, row.print, row.links.len())
        );
        assert_eq!(ask.file.expose(), &Path::new(FOLDER).join(&row.file));
        let part = ask.part().unwrap();
        assert!(part.to_string_lossy().ends_with(PART_EXT), "{}", row.id);
        assert_eq!(ask.hosts, row.hosts);
    }
}

#[test]
fn every_row_names_its_languages_with_the_models_own_codes() {
    let s = VoiceSettings::from_defaults().unwrap();
    for row in shipped().rows() {
        assert!(!row.langs.is_empty(), "{}", row.id);
        for (lang, code) in &row.langs {
            assert!(fits_iso(lang), "{}: {lang}", row.id);
            assert!(
                TAG_LEN.contains(&code.len()) && code.bytes().all(|b| b.is_ascii_graphic()),
                "{code}"
            );
            let model = row.model(Path::new(FOLDER), lang).unwrap();
            assert_eq!(
                (model.lang.as_str(), model.runner.as_str()),
                (code.as_str(), "transcribe")
            );
            assert_eq!(model.cpu_mb, row.cpu_mb);
            assert_eq!(
                s.ask(Some(model), None).check(),
                Ok(()),
                "{}: {lang}",
                row.id
            );
        }
        assert!(
            row.model(Path::new(FOLDER), "kl").is_none(),
            "no model knows Greenlandic"
        );
    }
}

#[test]
fn the_languages_match_what_the_engine_listed() {
    let nemotron = row("nemotron-0.6b-q8");
    for (lang, code) in [
        ("en", "en-US"),
        ("fr", "fr-FR"),
        ("ar", "ar-AR"),
        ("nb", "nb-NO"),
    ] {
        assert_eq!(nemotron.code(lang), Some(code));
    }
    for id in ["parakeet-v3-q8", "canary-v2-q8"] {
        assert_eq!(
            (row(id).langs.len(), row(id).code("ar")),
            (25, None),
            "{id} knows no Arabic"
        );
    }
    let arabic = row("cohere-arabic-q8");
    assert_eq!(arabic.langs.keys().collect::<Vec<_>>(), ["ar", "en"]);
    let small = row("whisper-small-q5km");
    for (lang, code) in [("jv", "jw"), ("nb", "no"), ("fil", "tl"), ("haw", "haw")] {
        assert_eq!(small.code(lang), Some(code), "{lang}");
    }
    assert_eq!(
        (small.code("yue"), row("whisper-turbo-q5km").code("yue")),
        (None, Some("yue"))
    );
    assert_eq!(row("cohere-q8").langs.len(), 14);
    assert_eq!(row("qwen3-1.7b-q8").code("fil"), Some("fil"));
}

#[test]
fn every_row_has_a_licence_a_credit_and_memory_in_range() {
    let licences = ["MIT", "Apache-2.0", "CC-BY-4.0", "OpenMDW-1.1"];
    for row in shipped().rows() {
        assert!(licences.contains(&row.licence.as_str()), "{}", row.id);
        assert!(!row.credit.is_empty() && !row.name.is_empty(), "{}", row.id);
        assert!(Link::new(row.origin.clone()).is_ok(), "{}", row.id);
        assert!(SpeechModel::CPU_MB.contains(&row.cpu_mb), "{}", row.id);
        assert!(CardUse::NEED_MB.contains(&row.card_mb), "{}", row.id);
        assert!(
            row.id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-.".contains(&b))
        );
    }
    assert_eq!(row("parakeet-v3-q8").licence, "CC-BY-4.0");
    assert_eq!(row("nemotron-0.6b-q8").licence, "OpenMDW-1.1");
    assert!(row("nemotron-0.6b-q8").live && !row("parakeet-v3-q8").live);
}

#[test]
fn a_row_asks_its_card_with_the_margin() {
    let id = CardId::new("Vulkan1").unwrap();
    let card = row("cohere-arabic-q8").card(id.clone(), 512);
    assert_eq!(
        card,
        CardUse {
            id,
            need_mb: 3008,
            margin_mb: 512
        }
    );
}

#[test]
fn files_are_known_by_name_in_any_case_and_by_extension() {
    let t = shipped();
    assert_eq!(
        t.named("PARAKEET-TDT-0.6B-V3-Q8_0.GGUF")
            .map(|r| r.id.as_str()),
        Some("parakeet-v3-q8")
    );
    assert!(t.named("ggml-small-q5_1.bin").is_none());
    assert_eq!(t.exts(), ["gguf", "bin"]);
    for (name, model) in [
        ("x.gguf", true),
        ("x.GGUF", true),
        ("ggml-small.bin", true),
        ("x.exe", false),
        ("gguf", false),
    ] {
        assert_eq!(t.is_model_file(name), model, "{name}");
    }
    let unknown = Fingerprint::new([7; Fingerprint::BYTES]);
    assert!(t.printed(&unknown).is_none());
}
