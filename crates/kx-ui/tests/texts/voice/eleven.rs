//! `ElevenLabs`' card in each language (research [E1] to [E7]): training by default, its history kept, the free
//! allowance in minutes agreeing with its number, its plans' light and heavy prices, the warning naming the profile
//! menu before the switch's own labels, and the free plan's rule under its own label.

use crate::lang;
use kx_mod_voice::services::table;
use kx_module_api::ports::{Fact, FactArg, FactValue, Hundredths};
use kx_ui::i18n::keys::{
    CARD_RULE, FACT_FREE, FACT_KEEPS, FACT_PRICE, FACT_RULE, FACT_TRAINS, FACT_WARN, VOICE_CARD,
};

/// The UI languages, in the order the tables give their words.
const TAGS: [&str; 3] = ["en", "fr", "ar"];

/// Each fact's group, its id and its words in English, French and Arabic.
const LINES: [(&str, &str, [&str; 3]); 3] = [
    (
        FACT_TRAINS,
        "default",
        [
            "Used for training by default",
            "Utilisée pour l’entraînement par défaut",
            "يُستخدم للتدريب افتراضيًا",
        ],
    ),
    (
        FACT_KEEPS,
        "history",
        [
            "Your history, by default; keeping nothing is for Enterprise plans only",
            "Votre historique, par défaut\u{a0}; ne rien garder est réservé aux offres Enterprise",
            "سجلّك افتراضيًا، وعدم الاحتفاظ بأي شيء متاح لخطط المؤسسات فقط",
        ],
    ),
    (
        FACT_RULE,
        "non-commercial",
        [
            "The free plan is for non-commercial use only, with attribution",
            "L’offre gratuite est réservée à un usage non commercial, avec mention de la source",
            "الخطة المجانية للاستخدام غير التجاري فقط، مع نسب المحتوى إلى الخدمة",
        ],
    ),
];

/// The free allowance of `n` minutes a month.
fn minutes(n: u32) -> Fact {
    let count = FactArg {
        name: "minutes".to_owned(),
        value: FactValue::Count(n),
    };
    Fact {
        id: "minutes-month".to_owned(),
        args: vec![count],
        warn: false,
    }
}

/// Minutes a month and their words: Arabic's noun agrees with its number, after "نحو" in the genitive.
const MINUTES: [(u32, [&str; 3]); 5] = [
    (
        1,
        [
            "About 1 minute a month",
            "Environ 1 minute par mois",
            "نحو دقيقة واحدة شهريًا",
        ],
    ),
    (
        2,
        [
            "About 2 minutes a month",
            "Environ 2 minutes par mois",
            "نحو دقيقتين شهريًا",
        ],
    ),
    (
        3,
        [
            "About 3 minutes a month",
            "Environ 3 minutes par mois",
            "نحو 3 دقائق شهريًا",
        ],
    ),
    (
        30,
        [
            "About 30 minutes a month",
            "Environ 30 minutes par mois",
            "نحو 30 دقيقة شهريًا",
        ],
    ),
    (
        100,
        [
            "About 100 minutes a month",
            "Environ 100 minutes par mois",
            "نحو 100 دقيقة شهريًا",
        ],
    ),
];

#[test]
fn its_training_history_and_rule_read_in_each_language() {
    for (group, id, words) in LINES {
        let fact = Fact {
            id: id.to_owned(),
            args: Vec::new(),
            warn: false,
        };
        for (tag, want) in TAGS.into_iter().zip(words) {
            assert_eq!(lang(tag).voice_fact(group, &fact).text, want, "{tag} {id}");
        }
    }
}

#[test]
fn the_free_minutes_agree_with_their_number() {
    for (n, words) in MINUTES {
        for (tag, want) in TAGS.into_iter().zip(words) {
            let said = lang(tag).voice_fact(FACT_FREE, &minutes(n)).text;
            assert_eq!(said, want, "{tag} {n}");
        }
    }
}

#[test]
fn the_rules_label_reads_in_each_language() {
    let words = ["Terms of use", "Conditions d’utilisation", "شروط الاستخدام"];
    for (tag, want) in TAGS.into_iter().zip(words) {
        let label = lang(tag).text(&format!("{VOICE_CARD}{CARD_RULE}"));
        assert_eq!(label, want, "{tag}");
    }
}

/// Light and heavy use on the cheapest plans, Starter and Creator [E1], in hundredths, and their words: Arabic's
/// dollars agree with their number.
const PRICES: [(u32, [&str; 3]); 2] = [
    (600, ["$6", "6\u{a0}$", "6 دولارات"]),
    (2200, ["$22", "22\u{a0}$", "22 دولارًا"]),
];

/// The warning's sentence before the switch in each language, naming the profile menu, not the service's labels.
const WARNED: [&str; 3] = [
    "Trains its models on your voice by default. To stop it, turn off this switch in your profile menu: ",
    "Entraîne ses modèles sur votre voix par défaut. Pour arrêter cela, désactivez ce réglage dans le menu de votre profil\u{a0}: ",
    "تستخدم الخدمة صوتك لتدريب نماذجها افتراضيًا. لإيقاف ذلك، عطِّل هذا الخيار في قائمة ملفك الشخصي: ",
];

#[test]
fn its_warning_names_the_profile_menu_in_each_language_and_its_own_labels_apart() {
    let row = table().expect("the services table").get("elevenlabs");
    let fact = row.and_then(|r| r.facts.warn.clone()).expect("a warning");
    let Some(FactValue::Text(switch)) = fact.args.first().map(|a| a.value.clone()) else {
        panic!("its switch's labels")
    };
    for (tag, want) in TAGS.into_iter().zip(WARNED) {
        let said = lang(tag).voice_fact(FACT_WARN, &fact);
        assert_eq!(said.text, format!("{want}{switch}"), "{tag}");
        let last = said.parts.last().map(|p| (p.text.as_str(), p.own));
        assert_eq!(
            last,
            Some((switch.as_str(), true)),
            "{tag}: its own labels apart"
        );
    }
}

#[test]
fn its_light_and_heavy_prices_read_in_each_language() {
    for (h, words) in PRICES {
        let price = FactArg {
            name: "price".to_owned(),
            value: FactValue::Money {
                amount: Hundredths(h),
                currency: "USD".to_owned(),
            },
        };
        let fact = Fact {
            id: "plain".to_owned(),
            args: vec![price],
            warn: false,
        };
        for (tag, want) in TAGS.into_iter().zip(words) {
            assert_eq!(
                lang(tag).voice_fact(FACT_PRICE, &fact).text,
                want,
                "{tag} {h}"
            );
        }
    }
}
