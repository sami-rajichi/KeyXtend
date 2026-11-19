//! Voice typing's cloud texts in each language (ADR-0047): every cloud id the hub words has its text, the owner's
//! main words, each address fault and cloud reason its own line, and the region and account notes. Money is in
//! `money`, a card's fact with a service's own words in `own`, and the reworded lines in `approved`.

use crate::{all, lang};
use kx_module_api::ports::{
    AddressFault, CloudBand, CloudButton, CloudLine, CloudWhy, KeyCheck, LINK_SCHEME, ModelSide,
    Unavailable, VoiceLang,
};
use kx_ui::i18n::keys::{
    CARD_BILLS, CARD_LABELS, CARD_RULE, CLOUD_WORD, VOICE_ADDRESS, VOICE_ADDRESS_HINT,
    VOICE_ADDRESS_HINT_OWN, VOICE_ADDRESS_LINE, VOICE_BAND, VOICE_CARD, VOICE_CARD_ASK,
    VOICE_CARD_CONFIRM, VOICE_CARD_CUSTOM, VOICE_CARD_HOURS, VOICE_CARD_OLD, VOICE_CARD_ON,
    VOICE_CARD_READ_ON, VOICE_CARD_SUB, VOICE_CARD_UNTESTED, VOICE_CLOUD_FREE, VOICE_CLOUD_HOME,
    VOICE_CLOUD_MODEL, VOICE_CLOUD_OFF, VOICE_CLOUD_READY, VOICE_CLOUD_SUB, VOICE_CLOUD_SWITCH,
    VOICE_CUSTOM, VOICE_CUSTOM_LINE, VOICE_HOURS, VOICE_KEEP_NOTHING, VOICE_KEY, VOICE_KEY_HINT,
    VOICE_KEY_LINE, VOICE_KEY_LINK, VOICE_KEY_SEALED, VOICE_LINE_SAVE, VOICE_LINE_SAVE_NAME,
    VOICE_MODEL_HINT, VOICE_MODEL_LINE, VOICE_MODEL_REFUSED, VOICE_MODELS_SIDE, VOICE_MODELS_SIDES,
    VOICE_MONEY_ANY, VOICE_NOTE_RENEWS, VOICE_ON_BAR_CLOUD, VOICE_SEND_WARNING, VOICE_USE_LOCAL,
    address_fault_id, cloud_band_id, cloud_button_key, cloud_line_id, key_check_id, key_check_why,
    model_side_id,
};
use std::time::Duration;

/// Every cloud reason a note gives, chained by `after`.
fn whys() -> Vec<CloudWhy> {
    std::iter::successors(Some(CloudWhy::Net), after).collect()
}

/// The reason after `why`; its match names every reason, so a new one cannot miss its texts.
fn after(why: &CloudWhy) -> Option<CloudWhy> {
    Some(match why {
        CloudWhy::Net => CloudWhy::Key,
        CloudWhy::Key => CloudWhy::Credit,
        CloudWhy::Credit => CloudWhy::Account,
        CloudWhy::Account => CloudWhy::Region,
        CloudWhy::Region => CloudWhy::Allowance { renews_in: None },
        CloudWhy::Allowance { .. } => CloudWhy::Rate,
        CloudWhy::Rate => CloudWhy::TooBig,
        CloudWhy::TooBig => CloudWhy::Language(front("en")),
        CloudWhy::Language(_) => CloudWhy::Empty,
        CloudWhy::Empty => CloudWhy::Strange,
        CloudWhy::Strange => CloudWhy::Terms,
        CloudWhy::Terms => CloudWhy::Off,
        CloudWhy::Off => CloudWhy::NoLocal,
        CloudWhy::NoLocal => CloudWhy::Fault,
        CloudWhy::Fault => CloudWhy::Unavailable(Unavailable::KeepNothing),
        CloudWhy::Unavailable(Unavailable::KeepNothing) => return None,
    })
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
const FIXED: [&str; 38] = [
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
    VOICE_LINE_SAVE,
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
    keys.extend(CloudLine::ALL.map(|l| format!("{VOICE_LINE_SAVE_NAME}{}", cloud_line_id(l))));
    keys.extend(
        KeyCheck::ALL
            .into_iter()
            .filter(|&c| key_check_why(c, None).is_none())
            .filter_map(key_check_id)
            .map(|id| format!("{VOICE_KEY}{id}")),
    );
    keys.extend(AddressFault::ALL.map(|f| format!("{VOICE_ADDRESS}{}", address_fault_id(f))));
    keys.extend(ModelSide::ALL.map(|s| format!("{VOICE_MODELS_SIDE}{}", model_side_id(s))));
    let bands = CloudBand::ALL.into_iter().filter(|&b| b != CloudBand::Off);
    keys.extend(bands.map(|b| format!("{VOICE_BAND}{}", cloud_band_id(b))));
    let labels = CARD_LABELS.into_iter().chain([CARD_BILLS, CARD_RULE]);
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
    read_as_written(&table);
}

#[test]
fn each_lines_save_reads_short_and_names_its_line_for_readers() {
    read_as_written(&[
        ("cloud.line-save", ["Save", "Enregistrer", "حفظ"]),
        (
            "cloud.line-save-key",
            ["Save the key", "Enregistrer la clé", "حفظ المفتاح"],
        ),
        (
            "cloud.line-save-address",
            ["Save the address", "Enregistrer l’adresse", "حفظ العنوان"],
        ),
        (
            "cloud.line-save-model",
            ["Save the model", "Enregistrer le modèle", "حفظ النموذج"],
        ),
    ]);
}

/// Checks that each key in `table` reads as its English, French and Arabic words.
fn read_as_written(table: &[(&str, [&str; 3])]) {
    for (key, words) in table {
        for (tag, word) in ["en", "fr", "ar"].into_iter().zip(words) {
            let got = lang(tag).text(key);
            assert_eq!(got, *word, "{tag} {key}");
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
}

/// The region and account notes in English, French and Arabic: the service is not offered in the user's country or
/// region, or it asks for a step on its site first.
const NOTES: [(CloudWhy, [&str; 3]); 2] = [
    (
        CloudWhy::Region,
        [
            "The service is not offered in your country or region.",
            "Le service n’est pas proposé dans votre pays ou votre région.",
            "الخدمة غير متاحة في بلدك أو منطقتك.",
        ],
    ),
    (
        CloudWhy::Account,
        [
            "The service asks for a step on its site, such as a verification, a payment method or a spending cap.",
            "Le service demande une étape sur son site, comme une vérification, un moyen de paiement ou un plafond de dépenses.",
            "تطلب الخدمة خطوةً على موقعها، مثل التحقق أو وسيلة دفع أو حدٍّ للإنفاق.",
        ],
    ),
];

#[test]
fn the_region_and_account_notes_say_what_the_service_refused_and_why() {
    for (why, words) in NOTES {
        for (tag, want) in ["en", "fr", "ar"].into_iter().zip(words) {
            assert_eq!(lang(tag).voice_cloud(&why), want, "{tag} {why:?}");
        }
    }
}
