//! `AssemblyAI`'s card in each language (research [A3] to [A5]), read from its own row: the EU, no training, a keeping
//! its pages do not state in the other unstated facts' words, $50 of credit that never expires with no card, with
//! Arabic's dollars agreeing with their number after "بقيمة", and the plain price of light and heavy use.

use crate::lang;
use kx_mod_voice::services::{Facts, table};
use kx_module_api::ports::{Fact, FactArg, FactValue, Hundredths};
use kx_ui::i18n::keys::{FACT_FREE, FACT_KEEPS, FACT_PRICE, FACT_TRAINS, FACT_WHERE};

/// The UI languages, in the order the tables give their words.
const TAGS: [&str; 3] = ["en", "fr", "ar"];

/// Its facts' groups, then each one's words in English, French and Arabic, in the card's order.
const CARD: [(&str, [&str; 3]); 6] = [
    (
        FACT_WHERE,
        ["European Union", "Union européenne", "الاتحاد الأوروبي"],
    ),
    (
        FACT_TRAINS,
        [
            "Not used for training",
            "Non utilisée pour l’entraînement",
            "لا يُستخدم للتدريب",
        ],
    ),
    (
        FACT_KEEPS,
        [
            "Not stated on its pages",
            "Non précisé sur ses pages",
            "غير مذكور في صفحات الخدمة",
        ],
    ),
    (
        FACT_FREE,
        [
            "$50 of credit that never expires, no card needed",
            "50\u{a0}$ de crédit sans expiration, sans carte",
            "رصيد دائم بقيمة 50 دولارًا، دون بطاقة دفع",
        ],
    ),
    (FACT_PRICE, ["$4.50", "4,50\u{a0}$", "4,50 دولار"]),
    (FACT_PRICE, ["$18", "18\u{a0}$", "18 دولارًا"]),
];

/// Other amounts of credit in Arabic: one, two in the genitive after "بقيمة", few, and other.
const ARABIC_CREDIT: [(u32, &str); 4] = [
    (100, "رصيد دائم بقيمة دولار واحد"),
    (200, "رصيد دائم بقيمة دولارين"),
    (300, "رصيد دائم بقيمة 3 دولارات"),
    (10_000, "رصيد دائم بقيمة 100 دولار"),
];

/// The shipped row's facts.
fn facts() -> &'static Facts {
    let row = table().expect("the services table").get("assemblyai");
    &row.expect("its row").facts
}

#[test]
fn its_card_reads_in_each_language_from_its_own_row() {
    let f = facts();
    let shown = [&f.place, &f.trains, &f.keeps, &f.free, &f.light, &f.heavy];
    for (fact, (group, words)) in shown.into_iter().zip(CARD) {
        for (tag, want) in TAGS.into_iter().zip(words) {
            let said = lang(tag).voice_fact(group, fact).text;
            assert_eq!(said, want, "{tag} {group}-{}", fact.id);
        }
    }
}

#[test]
fn an_unstated_keeping_says_what_the_other_unstated_facts_say() {
    let unstated = Fact {
        id: "unstated".to_owned(),
        args: Vec::new(),
        warn: false,
    };
    for tag in TAGS {
        let t = lang(tag);
        let keeps = t.voice_fact(FACT_KEEPS, &unstated).text;
        assert_eq!(keeps, t.voice_fact(FACT_TRAINS, &unstated).text, "{tag}");
        assert_eq!(keeps, t.voice_fact(FACT_WHERE, &unstated).text, "{tag}");
    }
}

#[test]
fn arabic_credit_agrees_with_its_number_after_its_preposition() {
    for (cents, start) in ARABIC_CREDIT {
        let credit = FactArg {
            name: "credit".to_owned(),
            value: FactValue::Money {
                amount: Hundredths(cents),
                currency: "USD".to_owned(),
            },
        };
        let fact = Fact {
            id: "credit".to_owned(),
            args: vec![credit],
            warn: false,
        };
        let said = lang("ar").voice_fact(FACT_FREE, &fact).text;
        assert!(said.starts_with(&format!("{start}،")), "{cents}: {said}");
    }
}
