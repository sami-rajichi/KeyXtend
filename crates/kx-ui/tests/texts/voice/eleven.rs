//! `ElevenLabs`' card in each language (research [E1] to [E7]): training by default, its history kept, the free
//! allowance in minutes agreeing with its number, its light and heavy prices, and the free plan's rule under its own
//! label.

use crate::lang;
use kx_module_api::ports::{Fact, FactArg, FactValue, Hundredths};
use kx_ui::i18n::keys::{
    CARD_RULE, FACT_FREE, FACT_KEEPS, FACT_PRICE, FACT_RULE, FACT_TRAINS, VOICE_CARD,
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

/// Light and heavy use at $0.20 an hour, in hundredths, and their words: Arabic's dollars agree with their number.
const PRICES: [(u32, [&str; 3]); 2] = [
    (300, ["$3", "3\u{a0}$", "3 دولارات"]),
    (1200, ["$12", "12\u{a0}$", "12 دولارًا"]),
];

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
