//! The language sets: they live in their own file, read with the table, and `OpenAI`'s models send ISO 639-1 codes
//! [O2], so Javanese goes as `jv` where Whisper's own set says `jw`.

use super::*;

#[test]
fn the_language_sets_come_from_their_own_file_alone() {
    let set = "\n[langs]\nmore = { en = \"en\" }\n";
    let inline = parse(&format!("{TEXT}{set}"));
    assert!(matches!(inline, Err(ServicesError::Parse(_))), "{inline:?}");
    let odd = Services::parse(TEXT, "whisper = 1");
    assert!(matches!(odd, Err(ServicesError::Langs(_))), "{odd:?}");
    let none = match Services::parse(TEXT, "") {
        Err(ServicesError::Row {
            why: ServiceError::Model { why, .. },
            ..
        }) => Some(why),
        _ => None,
    };
    assert_eq!(none, Some(ModelError::Set), "a set the file does not name");
}

/// ISO 639-1's two-letter codes, as the registry lists them today.
const ISO_639_1: &str = "aa ab ae af ak am an ar as av ay az ba be bg bi bm bn bo br bs ca ce ch co cr cs cu cv cy \
    da de dv dz ee el en eo es et eu fa ff fi fj fo fr fy ga gd gl gn gu gv ha he hi ho hr ht hu hy hz ia id ie ig ii \
    ik io is it iu ja jv ka kg ki kj kk kl km kn ko kr ks ku kv kw ky la lb lg li ln lo lt lu lv mg mh mi mk ml mn mr \
    ms mt my na nb nd ne ng nl nn no nr nv ny oc oj om or os pa pi pl ps pt qu rm rn ro ru rw sa sc sd se sg si sk sl \
    sm sn so sq sr ss st su sv sw ta te tg th ti tk tl tn to tr ts tt tw ty ug uk ur uz ve vi vo wa wo xh yi yo za zh zu";

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
    let whisper = groq.models.iter().map(|m| m.code("jv"));
    assert!(
        whisper.into_iter().all(|c| c == Some("jw")),
        "Whisper's own code"
    );
}
