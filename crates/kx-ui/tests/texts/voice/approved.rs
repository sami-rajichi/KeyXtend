//! The cloud lines as the owner approved them, word for word in each language: a key, address or model refused, an
//! address that must use HTTPS, speech for a password box, the keep-nothing claim, the training mark, the free allowance
//! and the least billed. No Latin run ends an Arabic line on a neutral mark, so `https://` never reads backwards.

use crate::files::keys_of;
use crate::lang;
use kx_module_api::ports::{CloudWhy, Fact, FactArg, FactValue, Hundredths};
use kx_ui::i18n::keys::vars::FREE;
use kx_ui::i18n::keys::{
    FACT_BILLS, FACT_FREE, FACT_PRICE, FACT_WARN, VOICE_CLOUD_FREE, fact_key, warn_mark_key,
};
use kx_ui::{I18n, LANGS};
use std::time::Duration;

/// The UI languages, in the order the table gives their words.
const TAGS: [&str; 3] = ["en", "fr", "ar"];

/// Each reworded line's key and its words in English, French and Arabic, `$https` filled.
const LINES: [(&str, [&str; 3]); 11] = [
    (
        "cloud.key-unfit",
        [
            "This text cannot be sent as a key, so nothing was kept.",
            "Ce texte ne peut pas être envoyé comme clé\u{a0}; rien n’a donc été gardé.",
            "لا يمكن إرسال هذا النص بوصفه مفتاحًا، فلم يُحفَظ شيء.",
        ],
    ),
    (
        "cloud.address-path",
        [
            "The address holds a character that is not allowed, such as a space.",
            "L’adresse contient un caractère non autorisé, comme un espace.",
            "يحتوي العنوان على حرف غير مسموح به، مثل المسافة.",
        ],
    ),
    (
        "cloud.model-refused",
        [
            "This model id is too long or holds a character that is not allowed.",
            "Cet identifiant de modèle est trop long ou contient un caractère non autorisé.",
            "معرّف النموذج هذا طويل جدًا أو يحتوي على حرف غير مسموح به.",
        ],
    ),
    (
        "cloud.note-no-local",
        [
            "Speech for a password box never goes to the cloud, and no local model is ready.",
            "Ce qui est dicté dans un champ de mot de passe ne part jamais vers le cloud, et aucun modèle local n’est prêt.",
            "لا يُرسَل ما يُملى في حقل كلمة المرور إلى السحابة أبدًا، ولا يوجد نموذج محلي جاهز.",
        ],
    ),
    (
        "cloud.address-host",
        [
            "The address needs a host name, not an IP address.",
            "L’adresse doit contenir un nom d’hôte, pas une adresse IP.",
            "يجب أن يحتوي العنوان على اسم مضيف، لا على عنوان IP.",
        ],
    ),
    (
        "cloud.address-https",
        [
            "The address must use HTTPS, such as https://example.com",
            "L’adresse doit utiliser HTTPS, par exemple https://example.com",
            "يجب أن يستخدم العنوان HTTPS، مثل https://example.com",
        ],
    ),
    (
        "cloud.address-hint-own",
        [
            "The HTTPS address of your own resource on this service",
            "L’adresse HTTPS de votre propre ressource chez ce service",
            "عنوان HTTPS لموردك الخاص لدى هذه الخدمة",
        ],
    ),
    (
        "cloud.note-keep-nothing",
        [
            "No host of this model promises to keep nothing.",
            "Aucun hébergeur ne s’engage à ne rien garder.",
            "لا يتعهّد أي مضيف لهذا النموذج بعدم الاحتفاظ بشيء.",
        ],
    ),
    (
        "cloud.warn-mark-trains",
        [
            "Trains by default",
            "Entraînement activé",
            "التدريب مفعّل افتراضيًا",
        ],
    ),
    (
        "cloud.card-free",
        ["Free allowance", "Quota gratuit", "الحصة المجانية"],
    ),
    (
        "cloud.note-allowance",
        [
            "The free allowance is used up for now.",
            "Le quota gratuit est épuisé pour l’instant.",
            "نفدت الحصة المجانية حاليًا.",
        ],
    ),
];

/// The free allowance's row line, short, and the start of its renewal's line, in each language: one term throughout.
const ALLOWANCE: [(&str, &str, &str); 3] = [
    (
        "en",
        "Free: 8",
        "The free allowance is used up. It renews in ",
    ),
    (
        "fr",
        "Gratuit\u{a0}: 8",
        "Le quota gratuit est épuisé. Il se renouvelle dans ",
    ),
    (
        "ar",
        "مجانًا: 8",
        "نفدت الحصة المجانية، وتتجدد بعد مدة قدرها ",
    ),
];

#[test]
fn the_reworded_cloud_lines_read_as_approved_in_each_language() {
    for (key, words) in LINES {
        for (tag, want) in TAGS.into_iter().zip(words) {
            assert_eq!(lang(tag).voice_https(key), want, "{tag} {key}");
        }
    }
}

/// Every warning fact brings its own short mark for its row, so a new warning never shows another's word.
#[test]
fn every_warning_fact_has_its_own_short_mark_in_each_language() {
    let start = fact_key(FACT_WARN, "");
    for l in &LANGS {
        let t = I18n::new(l);
        let keys = l.files().flat_map(keys_of);
        let ids: Vec<String> = keys
            .filter_map(|k| k.strip_prefix(&start).map(str::to_owned))
            .collect();
        assert!(!ids.is_empty(), "{}: a warning fact", l.tag);
        let bare: Vec<&String> = ids.iter().filter(|id| !t.has(&warn_mark_key(id))).collect();
        assert!(bare.is_empty(), "{}: {bare:?}", l.tag);
    }
}

#[test]
fn the_free_allowance_keeps_one_term_on_the_row_and_the_renewal() {
    let renews = CloudWhy::Allowance {
        renews_in: Some(Duration::from_hours(2)),
    };
    for (tag, row, renewal) in ALLOWANCE {
        let t = lang(tag);
        let line = t.voice_with(VOICE_CLOUD_FREE, &[(FREE, "8".to_owned())]);
        assert_eq!(line, row, "{tag}");
        let said = t.voice_cloud(&renews);
        assert!(said.starts_with(renewal), "{tag}: {said}");
    }
}

#[test]
fn the_free_allowance_and_the_least_billed_read_as_approved() {
    let usd = |name: &str, h| FactArg {
        name: name.to_owned(),
        value: FactValue::Money {
            amount: Hundredths(h),
            currency: "USD".to_owned(),
        },
    };
    let price = Fact {
        id: "free-or".to_owned(),
        args: vec![usd("low", 60), usd("high", 167)],
        warn: false,
    };
    let starts = [
        "Free within the free allowance, else ",
        "Gratuit dans la limite du quota gratuit, sinon de ",
        "مجاني ضمن الحصة المجانية، وإلا فمن ",
    ];
    for (tag, start) in TAGS.into_iter().zip(starts) {
        let said = lang(tag).voice_fact(FACT_PRICE, &price).text;
        assert!(said.starts_with(start), "{tag}: {said}");
    }
    let seconds = FactArg {
        name: "seconds".to_owned(),
        value: FactValue::Count(10),
    };
    let bills = Fact {
        id: "least".to_owned(),
        args: vec![seconds],
        warn: false,
    };
    let fr = lang("fr").voice_fact(FACT_BILLS, &bills).text;
    assert!(
        fr.ends_with("les morceaux courts coûtent donc plus cher"),
        "{fr}"
    );
}

/// Together's free line: nothing is free, and a key needs a first purchase of at least $15 [T3].
const FIRST_BUY: [&str; 3] = [
    "None; first purchase at least $15",
    "Aucun\u{a0}; premier achat d’au moins 15\u{a0}$",
    "لا شيء، وأول شراء 15 دولارًا على الأقل",
];

#[test]
fn a_first_purchase_reads_as_nothing_free_and_never_as_free_after_paying() {
    let least = FactArg {
        name: "least".to_owned(),
        value: FactValue::Money {
            amount: Hundredths(1500),
            currency: "USD".to_owned(),
        },
    };
    let free = Fact {
        id: "first-buy".to_owned(),
        args: vec![least],
        warn: false,
    };
    for (tag, want) in TAGS.into_iter().zip(FIRST_BUY) {
        assert_eq!(lang(tag).voice_fact(FACT_FREE, &free).text, want, "{tag}");
    }
}
