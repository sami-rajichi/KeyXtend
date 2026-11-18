//! Voice typing's cloud texts in each language (ADR-0047): every cloud id the hub words has its text, the owner's
//! main words, each address fault and cloud reason its own line, a card's fact with a service's own name kept as a part
//! of its own so Arabic reads right (ADR-0022, no bidi marks), and money in each language's way.

use crate::{CONTROLS, all, lang};
use kx_module_api::ports::{
    AddressFault, CloudBand, CloudButton, CloudFault, CloudWhy, Fact, FactArg, FactValue,
    Hundredths, KeyCheck, LINK_SCHEME, ModelSide, Unavailable, VoiceLang,
};
use kx_ui::i18n::keys::{
    CARD_BILLS, CARD_LABELS, CLOUD_WORD, FACT_KEEPS, VOICE_ADDRESS, VOICE_ADDRESS_HINT,
    VOICE_ADDRESS_HINT_OWN, VOICE_ADDRESS_LINE, VOICE_BAND, VOICE_CARD, VOICE_CARD_ASK,
    VOICE_CARD_CONFIRM, VOICE_CARD_CUSTOM, VOICE_CARD_HOURS, VOICE_CARD_OLD, VOICE_CARD_ON,
    VOICE_CARD_READ_ON, VOICE_CARD_SUB, VOICE_CARD_UNTESTED, VOICE_CLOUD_FREE, VOICE_CLOUD_HOME,
    VOICE_CLOUD_MODEL, VOICE_CLOUD_OFF, VOICE_CLOUD_READY, VOICE_CLOUD_SUB, VOICE_CLOUD_SWITCH,
    VOICE_CUSTOM, VOICE_CUSTOM_LINE, VOICE_HOURS, VOICE_KEEP_NOTHING, VOICE_KEY, VOICE_KEY_HINT,
    VOICE_KEY_LINE, VOICE_KEY_LINK, VOICE_KEY_SEALED, VOICE_MODEL_HINT, VOICE_MODEL_LINE,
    VOICE_MODEL_REFUSED, VOICE_MODELS_SIDE, VOICE_MODELS_SIDES, VOICE_MONEY_ANY, VOICE_NOTE_RENEWS,
    VOICE_ON_BAR_CLOUD, VOICE_SEND_WARNING, VOICE_USE_LOCAL, address_fault_id, cloud_band_id,
    cloud_button_key, key_check_id, model_side_id,
};
use std::time::Duration;

/// Every cloud reason a note gives.
fn whys() -> Vec<CloudWhy> {
    vec![
        CloudWhy::Net,
        CloudWhy::Key,
        CloudWhy::Credit,
        CloudWhy::Allowance { renews_in: None },
        CloudWhy::Rate,
        CloudWhy::TooBig,
        CloudWhy::Language(front("en")),
        CloudWhy::Empty,
        CloudWhy::Strange,
        CloudWhy::Terms,
        CloudWhy::Off,
        CloudWhy::NoLocal,
        CloudWhy::Fault,
        CloudWhy::Unavailable(Unavailable::KeepNothing),
    ]
}

/// The language of code `iso`, named by Windows.
fn front(iso: &str) -> VoiceLang {
    VoiceLang {
        name: iso.to_owned(),
        iso: iso.to_owned(),
        rtl: iso == "ar",
        arabic: iso == "ar",
    }
}

/// The keys the hub builds for the cloud pages from no id.
const FIXED: [&str; 37] = [
    VOICE_CLOUD_SUB,
    VOICE_CLOUD_SWITCH,
    VOICE_CLOUD_OFF,
    VOICE_CLOUD_FREE,
    VOICE_CUSTOM,
    VOICE_CUSTOM_LINE,
    VOICE_CLOUD_HOME,
    VOICE_CLOUD_READY,
    VOICE_CLOUD_MODEL,
    VOICE_CARD_SUB,
    VOICE_CARD_ON,
    VOICE_CARD_HOURS,
    VOICE_CARD_READ_ON,
    VOICE_CARD_OLD,
    VOICE_CARD_CONFIRM,
    VOICE_CARD_ASK,
    VOICE_CARD_CUSTOM,
    VOICE_CARD_UNTESTED,
    VOICE_MONEY_ANY,
    VOICE_KEY_LINE,
    VOICE_KEY_HINT,
    VOICE_KEY_SEALED,
    VOICE_ADDRESS_HINT,
    VOICE_ADDRESS_HINT_OWN,
    VOICE_MODEL_LINE,
    VOICE_MODEL_HINT,
    VOICE_MODEL_REFUSED,
    VOICE_KEEP_NOTHING,
    VOICE_ON_BAR_CLOUD,
    VOICE_USE_LOCAL,
    VOICE_SEND_WARNING,
    VOICE_NOTE_RENEWS,
    VOICE_HOURS,
    VOICE_MODELS_SIDES,
    "voice.cloud",
    VOICE_ADDRESS_LINE,
    VOICE_KEY_LINK,
];

/// Every key the hub builds for the cloud pages.
fn keys() -> Vec<String> {
    let mut keys: Vec<String> = FIXED.map(str::to_owned).into();
    keys.extend(CloudButton::ALL.map(|b| cloud_button_key(b).to_owned()));
    keys.extend(
        KeyCheck::ALL
            .into_iter()
            .filter_map(key_check_id)
            .map(|id| format!("{VOICE_KEY}{id}")),
    );
    keys.extend(AddressFault::ALL.map(|f| format!("{VOICE_ADDRESS}{}", address_fault_id(f))));
    keys.extend(ModelSide::ALL.map(|s| format!("{VOICE_MODELS_SIDE}{}", model_side_id(s))));
    let bands = CloudBand::ALL.into_iter().filter(|&b| b != CloudBand::Off);
    keys.extend(bands.map(|b| format!("{VOICE_BAND}{}", cloud_band_id(b))));
    let labels = CARD_LABELS.into_iter().chain([CARD_BILLS]);
    keys.extend(labels.map(|l| format!("{VOICE_CARD}{l}")));
    keys
}

#[test]
fn every_cloud_id_the_hub_words_has_its_text_in_each_language() {
    let keys = keys();
    for t in all() {
        let missing: Vec<&String> = keys.iter().filter(|k| !t.has(k)).collect();
        assert!(missing.is_empty(), "{}: {missing:?}", t.lang().tag);
        for why in whys() {
            let line = t.voice_cloud(&why);
            assert!(
                !line.starts_with(CLOUD_WORD),
                "{} {why:?}: {line}",
                t.lang().tag
            );
        }
    }
}

#[test]
fn the_cloud_pages_main_words_read_as_written_in_each_language() {
    let table = [
        (
            "voice.cloud",
            ["Cloud services", "Services cloud", "الخدمات السحابية"],
        ),
        (
            "cloud.switch",
            [
                "Use cloud services",
                "Utiliser les services cloud",
                "استخدام الخدمات السحابية",
            ],
        ),
        ("voice.mark-on", ["Ready", "Prêt", "جاهزة"]),
        (
            "voice.mark-needs-key",
            ["Needs a key", "Clé requise", "يلزم مفتاح"],
        ),
        (
            "voice.mark-untested",
            ["Untested", "Non testé", "غير مُختبَرة"],
        ),
        ("cloud.key-paste", ["Paste", "Coller", "لصق"]),
        ("cloud.key-test", ["Test", "Tester", "اختبار"]),
        ("cloud.card-confirm", ["Confirm", "Confirmer", "تأكيد"]),
        (
            "cloud.use-local",
            [
                "Use the local model",
                "Utiliser le modèle local",
                "استخدام النموذج المحلي",
            ],
        ),
    ];
    for (key, words) in table {
        for (tag, word) in ["en", "fr", "ar"].into_iter().zip(words) {
            let got = lang(tag).text(key);
            assert_eq!(got, word, "{tag} {key}");
        }
    }
}

#[test]
fn each_address_fault_says_its_own_reason_and_the_custom_hint_shows_an_https_host() {
    for t in all() {
        let lines: Vec<String> = AddressFault::ALL
            .map(|f| t.voice_https(&format!("{VOICE_ADDRESS}{}", address_fault_id(f))))
            .into();
        let mut unique = lines.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), lines.len(), "{}: {lines:?}", t.lang().tag);
        let hint = t.voice_https(VOICE_ADDRESS_HINT);
        let host = hint.split(LINK_SCHEME).nth(1).unwrap_or_default();
        let name = host.split('/').next().unwrap_or_default();
        assert!(
            name.contains('.') && name.chars().any(char::is_alphabetic),
            "{hint}"
        );
    }
}

#[test]
fn each_cloud_reason_has_its_own_line_and_names_its_language_or_renewal() {
    for t in all() {
        let lines: Vec<String> = whys().iter().map(|w| t.voice_cloud(w)).collect();
        let mut unique = lines.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), lines.len(), "{}: {lines:?}", t.lang().tag);
    }
    let named = [
        ("en", "The service does not take English."),
        ("fr", "Le service n’écrit pas en anglais."),
        ("ar", "لا تقبل الخدمة اللغة الإنجليزية."),
    ];
    for (tag, want) in named {
        assert_eq!(
            lang(tag).voice_cloud(&CloudWhy::Language(front("en"))),
            want
        );
    }
    let renews = |d| CloudWhy::Allowance { renews_in: Some(d) };
    let en = lang("en");
    assert!(
        en.voice_cloud(&renews(Duration::from_mins(45)))
            .ends_with("45 min.")
    );
    assert!(
        en.voice_cloud(&renews(Duration::from_secs(2 * 3600 + 1)))
            .ends_with("3 h.")
    );
    let ar = lang("ar").voice_cloud(&renews(Duration::from_hours(2)));
    assert!(ar.ends_with("ساعتان."), "{ar}");
    let fault = lang("en").voice_fault(CloudFault::Timeout, &front("en"));
    assert_eq!(fault, lang("en").voice_cloud(&CloudWhy::Net));
}

#[test]
fn a_facts_service_name_stays_its_own_part_with_no_bidi_mark() {
    let fact = Fact {
        id: "logs".to_owned(),
        args: vec![
            FactArg {
                name: "days".to_owned(),
                value: FactValue::Count(30),
            },
            FactArg {
                name: "switch".to_owned(),
                value: FactValue::Text("Settings → Data Controls".to_owned()),
            },
        ],
        warn: false,
    };
    for t in all() {
        let said = t.voice_fact(FACT_KEEPS, &fact);
        let own: Vec<&str> = said
            .parts
            .iter()
            .filter(|p| p.own)
            .map(|p| p.text.as_str())
            .collect();
        assert_eq!(own, ["Settings → Data Controls"], "{}", t.lang().tag);
        assert!(said.text.contains("30"), "{}", said.text);
        assert!(!said.text.contains(CONTROLS), "{}", said.text);
        assert!(said.parts.iter().all(|p| !p.text.trim().is_empty()));
    }
    let ar = lang("ar").voice_fact(FACT_KEEPS, &fact);
    assert!(ar.text.contains("30 يومًا"), "{}", ar.text);
}

#[test]
fn money_reads_in_each_languages_way_with_cents_only_when_it_has_some() {
    let usd = |tag: &str, h| lang(tag).voice_money(Hundredths(h), "USD");
    assert_eq!(usd("en", 60), "$0.60");
    assert_eq!(usd("en", 500), "$5");
    assert_eq!(usd("fr", 60), "0,60\u{a0}$");
    assert_eq!(usd("ar", 167), "1,67 دولار");
    assert_eq!(lang("en").voice_money(Hundredths(69), "EUR"), "0.69 EUR");
}
