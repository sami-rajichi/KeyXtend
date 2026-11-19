//! Gemini's card in each language (research [N3] to [N5]): training while the free allowance is used, the paid tier's
//! limited keeping, no card needed in one meaning in all three, the paid tier's price for light and heavy use with
//! Arabic's dollars agreeing with their number, its rule for the EEA and for age, and its own warning and mark naming
//! the one free-allowance term; the paid tier is one term too.

use crate::lang;
use kx_mod_voice::services::table;
use kx_module_api::ports::{Fact, FactArg, FactValue, Hundredths};
use kx_ui::i18n::keys::{
    FACT_FREE, FACT_KEEPS, FACT_PRICE, FACT_RULE, FACT_TRAINS, FACT_WARN, warn_mark_key,
};

/// The UI languages, in the order the tables give their words.
const TAGS: [&str; 3] = ["en", "fr", "ar"];
/// The one free-allowance term, in each language (ruling 8).
const FREE_TERM: [&str; 3] = ["free allowance", "quota gratuit", "الحصة المجانية"];
/// The paid tier's one term, in each language, as Google's own pages name it.
const PAID_TERM: [&str; 3] = ["paid tier", "niveau payant", "المستوى المدفوع"];

/// Each fact's group, its id and its words in English, French and Arabic.
const LINES: [(&str, &str, [&str; 3]); 4] = [
    (
        FACT_TRAINS,
        "free-allowance",
        [
            "Used for training while you use the free allowance",
            "Utilisée pour l’entraînement tant que vous utilisez le quota gratuit",
            "يُستخدم للتدريب ما دمت ضمن الحصة المجانية",
        ],
    ),
    (
        FACT_KEEPS,
        "paid-limited",
        [
            "On the paid tier: your requests, for a limited time to check for abuse",
            "Au niveau payant\u{a0}: vos requêtes, pendant un temps limité pour détecter les abus",
            "في المستوى المدفوع: طلباتك لمدة محدودة لكشف إساءة الاستخدام",
        ],
    ),
    (
        FACT_FREE,
        "no-card",
        [
            "No card needed",
            "Sans carte bancaire",
            "دون الحاجة إلى بطاقة دفع",
        ],
    ),
    (
        FACT_RULE,
        "eea-paid-18",
        [
            "Paid use only in the EEA, Switzerland and the UK; users must be 18 or older",
            "Usage payant seulement dans l’EEE, en Suisse et au Royaume-Uni\u{a0}; utilisateurs de 18\u{a0}ans ou plus",
            "الاستخدام المدفوع فقط في المنطقة الاقتصادية الأوروبية وسويسرا والمملكة المتحدة، ويجب ألا يقل عمر المستخدم عن 18 عامًا",
        ],
    ),
];

/// Its warning, then its short mark, in each language [N3][N4].
const WARNING: [(&str, &str); 3] = [
    (
        "While you use the free allowance, Google uses your voice to improve its products, and human reviewers may read what you say.",
        "Trains on the free allowance",
    ),
    (
        "Tant que vous utilisez le quota gratuit, Google utilise votre voix pour améliorer ses produits, et des relecteurs humains peuvent lire ce que vous dites.",
        "Entraînement avec le quota gratuit",
    ),
    (
        "ما دمت ضمن الحصة المجانية، تستخدم Google صوتك لتحسين منتجاتها، وقد يطّلع مراجعون بشريون على ما تقوله.",
        "التدريب ضمن الحصة المجانية",
    ),
];

/// Light and heavy use at the paid tier's $0.005 a minute [N3], in hundredths, and their words.
const PRICES: [(u32, [&str; 3]); 2] = [
    (
        450,
        [
            "$4.50 on the paid tier",
            "4,50\u{a0}$ au niveau payant",
            "4,50 دولار في المستوى المدفوع",
        ],
    ),
    (
        1800,
        [
            "$18 on the paid tier",
            "18\u{a0}$ au niveau payant",
            "18 دولارًا في المستوى المدفوع",
        ],
    ),
];

/// A fact `id` with no values.
fn bare(id: &str) -> Fact {
    Fact {
        id: id.to_owned(),
        args: Vec::new(),
        warn: false,
    }
}

/// Gemini's own warning, as its row gives it.
fn own_warning() -> Fact {
    let row = table().expect("the services table").get("gemini");
    row.and_then(|r| r.facts.warn.clone()).expect("a warning")
}

#[test]
fn its_training_keeping_free_allowance_and_rule_read_in_each_language() {
    for (group, id, words) in LINES {
        for (tag, want) in TAGS.into_iter().zip(words) {
            let said = lang(tag).voice_fact(group, &bare(id)).text;
            assert_eq!(said, want, "{tag} {id}");
        }
    }
}

#[test]
fn its_warning_and_mark_read_in_each_language_with_the_one_free_allowance_term() {
    let fact = own_warning();
    let lines = TAGS.into_iter().zip(WARNING).zip(FREE_TERM);
    for ((tag, (warning, mark)), term) in lines {
        let t = lang(tag);
        assert_eq!(t.voice_fact(FACT_WARN, &fact).text, warning, "{tag}");
        assert_eq!(t.text(&warn_mark_key(&fact.id)), mark, "{tag}: its mark");
        assert!(warning.contains(term), "{tag}: {term}");
    }
    let trains = LINES[0].2;
    assert!(
        trains
            .iter()
            .zip(FREE_TERM)
            .all(|(line, term)| line.contains(term))
    );
}

#[test]
fn its_light_and_heavy_prices_read_in_each_language_on_the_paid_tier() {
    for (h, words) in PRICES {
        let price = FactArg {
            name: "price".to_owned(),
            value: FactValue::Money {
                amount: Hundredths(h),
                currency: "USD".to_owned(),
            },
        };
        let fact = Fact {
            id: "paid-plain".to_owned(),
            args: vec![price],
            warn: false,
        };
        for ((tag, want), term) in TAGS.into_iter().zip(words).zip(PAID_TERM) {
            let said = lang(tag).voice_fact(FACT_PRICE, &fact).text;
            assert_eq!(said, want, "{tag} {h}");
            assert!(said.contains(term), "{tag}: {term}");
        }
    }
    let keeps = LINES[1].2;
    assert!(keeps.iter().zip(PAID_TERM).all(|(l, t)| l.contains(t)));
}
