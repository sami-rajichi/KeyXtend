//! The language codes as the pages give them: ISO 639-1's list, which `OpenAI`'s models send on its row and through
//! `OpenRouter` [O2][R1] with Javanese as `jv`, and Whisper's own codes, `jw` for Javanese and `tl` for Filipino.

use super::*;

/// ISO 639-1's two-letter codes, as the registry lists them today.
const ISO_639_1: &str = "aa ab ae af ak am an ar as av ay az ba be bg bi bm bn bo br bs ca ce ch co cr cs cu cv cy \
    da de dv dz ee el en eo es et eu fa ff fi fj fo fr fy ga gd gl gn gu gv ha he hi ho hr ht hu hy hz ia id ie ig ii \
    ik io is it iu ja jv ka kg ki kj kk kl km kn ko kr ks ku kv kw ky la lb lg li ln lo lt lu lv mg mh mi mk ml mn mr \
    ms mt my na nb nd ne ng nl nn no nr nv ny oc oj om or os pa pi pl ps pt qu rm rn ro ru rw sa sc sd se sg si sk sl \
    sm sn so sq sr ss st su sv sw ta te tg th ti tk tl tn to tr ts tt tw ty ug uk ur uz ve vi vo wa wo xh yi yo \
    za zh zu";

/// `OpenAI`'s own models: on its row, and through `OpenRouter`, whose `language` is ISO 639-1 too [R1].
const GPT: [(&str, &str); 2] = [
    ("openai", "gpt-transcribe"),
    ("openrouter", "openai/gpt-transcribe"),
];

#[test]
fn the_iso_list_is_its_183_codes_in_order_each_once() {
    let codes: Vec<&str> = ISO_639_1.split_whitespace().collect();
    let mut once = codes.clone();
    once.sort_unstable();
    once.dedup();
    assert_eq!((codes.len(), once), (183, codes.clone()));
}

#[test]
fn openais_models_send_iso_639_1_codes_with_javanese_as_jv() {
    let iso: Vec<&str> = ISO_639_1.split_whitespace().collect();
    for (id, model) in GPT {
        let m = shipped().get(id).unwrap().model(model).unwrap();
        let odd: Vec<&str> = (m.langs.values())
            .map(String::as_str)
            .filter(|c| !iso.contains(c))
            .collect();
        assert!(odd.is_empty(), "{id} {model}: {odd:?}");
        assert_eq!(m.code("jv"), Some("jv"), "{id} {model}");
    }
    let groq = shipped().get("groq").unwrap();
    let mut whisper = groq.models.iter().map(|m| m.code("jv"));
    assert!(whisper.all(|c| c == Some("jw")), "Whisper's own code");
}

/// Where Whisper's own code is not ISO 639-1: the language, Whisper's code and ISO's.
const NOT_ISO: [(&str, &str, &str); 1] = [("jv", "jw", "jv")];

#[test]
fn the_iso_set_is_whispers_languages_with_only_its_non_iso_codes_changed() {
    let sets: BTreeMap<String, LangCodes> = toml::from_str(LANGS).unwrap();
    let (whisper, iso) = (&sets["whisper"], &sets["iso-639-1"]);
    assert!(whisper.keys().eq(iso.keys()), "the same languages");
    let changed: Vec<(&str, &str, &str)> = (whisper.iter())
        .filter(|&(lang, code)| iso[lang] != *code)
        .map(|(lang, code)| (lang.as_str(), code.as_str(), iso[lang].as_str()))
        .collect();
    assert_eq!(changed, NOT_ISO);
}

#[test]
fn whispers_set_codes_the_owners_languages_as_iso_and_filipino_as_tl() {
    let whisper = &shipped().get("groq").unwrap().models[0].langs;
    let codes = ["en", "fr", "ar", "fil"].map(|l| whisper.get(l).map(String::as_str));
    assert_eq!(codes, [Some("en"), Some("fr"), Some("ar"), Some("tl")]);
}
