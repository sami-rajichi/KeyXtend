//! The project's own `words.toml`: the languages, sources and packs it gives, judged by their relations and spec §6.

use flate2::Compression;

use super::super::manifest::{Manifest, Origin};
use super::super::{MANIFEST, WordsError};

/// Spec §6: "Full packs for English, French and Arabic", each with a dictionary and text.
const FULL: [&str; 3] = ["ar", "en", "fr"];

#[test]
fn the_project_manifest_gives_each_full_language_a_dictionary_and_more_words_than_a_small_one() {
    let manifest = Manifest::parse(MANIFEST).unwrap();
    let de = manifest.plan("de").unwrap();
    assert!(de.dict.is_none() && manifest.sources.contains_key(&de.list));
    for iso in FULL {
        let plan = manifest.plan(iso).unwrap();
        assert!(
            plan.dict.is_some() && manifest.sources.contains_key(&plan.list),
            "{iso}"
        );
        assert!(plan.max_words > de.max_words, "{iso}");
        assert!(plan.words_budget_kb > de.words_budget_kb, "{iso}");
    }
}

#[test]
fn the_project_manifest_hardens_curl() {
    let fetch = Manifest::parse(MANIFEST).unwrap().fetch;
    assert_eq!(
        fetch.args.first().map(String::as_str),
        Some("-q"),
        "no curl config file is read"
    );
    let has = |flag: &str| {
        fetch
            .args
            .windows(2)
            .any(|w| w[0] == flag && w[1] == "=https")
    };
    assert!(has("--proto") && has("--proto-redir"), "{:?}", fetch.args);
    let flags = [&fetch.size_flag, &fetch.out_flag, &fetch.url_flag];
    assert_eq!(flags, ["--max-filesize", "--output", "--url"]);
}

#[test]
fn the_project_manifest_gives_each_source_an_https_url_ending_in_its_name_or_a_downloaded_zip() {
    let manifest = Manifest::parse(MANIFEST).unwrap();
    let name = |rel: &str| rel.rsplit('/').next().unwrap().to_owned();
    for rel in manifest.sources.keys() {
        match manifest.origin(rel) {
            Some(Origin::Url(url)) => {
                assert!(
                    url.starts_with("https://") && url.ends_with(&name(rel)),
                    "{url}"
                );
            }
            Some(Origin::Zip(zip)) => {
                assert!(
                    matches!(manifest.origin(zip), Some(Origin::Url(_))),
                    "{rel}"
                );
            }
            None => panic!("{rel} has no origin"),
        }
    }
    assert_eq!(manifest.origin("nope"), None);
}

#[test]
fn an_unknown_language_has_no_plan() {
    let manifest = Manifest::parse(MANIFEST).unwrap();
    for iso in ["xx", "", "de/../en", "{iso}"] {
        assert_eq!(
            manifest.plan(iso).err(),
            Some(WordsError::Lang(iso.to_string()))
        );
    }
}

#[test]
fn the_project_packs_fit_spec_6s_20_mb_at_zips_default_level_and_only_full_ones_get_text() {
    let manifest = Manifest::parse(MANIFEST).unwrap();
    let packs = &manifest.packs;
    assert_eq!(
        packs.download_kb, 20480,
        "spec §6: packs about 20 MB of the download"
    );
    assert_eq!(
        packs.level,
        Compression::default().level(),
        "zip's default level"
    );
    for iso in FULL {
        let text = manifest.plan(iso).unwrap().text.unwrap();
        assert!(text.tatoeba.is_some() && text.wiki.is_some(), "{iso}");
    }
    assert!(manifest.plan("de").unwrap().text.is_none());
}

#[test]
fn every_language_gets_one_pack_the_full_ones_first() {
    let manifest = Manifest::parse(MANIFEST).unwrap();
    let isos = manifest.isos();
    let full = |iso: &String| manifest.plan(iso).unwrap().dict.is_some();
    let firsts = isos.iter().take_while(|i| full(i)).count();
    assert_eq!(firsts, FULL.len(), "{isos:?}");
    assert!(isos.iter().skip(firsts).all(|i| !full(i)), "{isos:?}");
    let distinct: std::collections::BTreeSet<&String> = isos.iter().collect();
    assert_eq!(distinct.len(), isos.len(), "one pack each");
}
