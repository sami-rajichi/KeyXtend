//! Money in each language's way: cents only when it has some, Arabic's dollars agreeing with their number, and a dual
//! after "من" or "إلى" in the genitive.

use crate::lang;
use kx_module_api::ports::{Fact, FactArg, FactValue, Hundredths};
use kx_ui::i18n::keys::FACT_PRICE;

#[test]
fn money_reads_in_each_languages_way_with_cents_only_when_it_has_some() {
    let usd = |tag: &str, h| lang(tag).voice_money(Hundredths(h), "USD");
    assert_eq!(usd("en", 60), "$0.60");
    assert_eq!(usd("en", 500), "$5");
    assert_eq!(usd("fr", 60), "0,60\u{a0}$");
    assert_eq!(usd("ar", 167), "1,67 دولار");
    assert_eq!(lang("en").voice_money(Hundredths(69), "EUR"), "0.69 EUR");
}

/// Dollars in Arabic and the words they take: the noun agrees with a whole number, as CLDR's forms read it, and every
/// fraction takes the singular, whatever its whole part.
const DOLLARS: [(u32, &str); 11] = [
    (100, "دولار واحد"),
    (200, "دولاران"),
    (300, "3 دولارات"),
    (1100, "11 دولارًا"),
    (1500, "15 دولارًا"),
    (10_000, "100 دولار"),
    (10_300, "103 دولارات"),
    (60, "0,60 دولار"),
    (167, "1,67 دولار"),
    (720, "7,20 دولار"),
    (1620, "16,20 دولار"),
];

#[test]
fn arabic_dollars_agree_with_their_number_and_a_fraction_takes_the_singular() {
    for (h, want) in DOLLARS {
        assert_eq!(lang("ar").voice_money(Hundredths(h), "USD"), want, "{h}");
    }
    let usd = |tag: &str| lang(tag).voice_money(Hundredths(1500), "USD");
    assert_eq!((usd("en"), usd("fr")), ("$15".into(), "15\u{a0}$".into()));
}

/// A price argument `name` of `h` hundredths of a dollar.
fn usd(name: &str, h: u32) -> FactArg {
    FactArg {
        name: name.to_owned(),
        value: FactValue::Money {
            amount: Hundredths(h),
            currency: "USD".to_owned(),
        },
    }
}

/// The price fact `id` from `low` to `high` hundredths, with a fee for the range that takes one.
fn range(id: &str, low: u32, high: u32) -> Fact {
    let fee = FactArg {
        name: "fee".to_owned(),
        value: FactValue::Number(Hundredths(550)),
    };
    Fact {
        id: id.to_owned(),
        args: vec![fee, usd("low", low), usd("high", high)],
        warn: false,
    }
}

#[test]
fn two_dollars_after_a_preposition_read_in_the_genitive_and_alone_in_the_nominative() {
    for id in ["free-or", "range", "range-fee"] {
        let said = lang("ar").voice_fact(FACT_PRICE, &range(id, 200, 200)).text;
        assert!(said.contains("من دولارين إلى دولارين"), "{id}: {said}");
    }
    let said = lang("ar")
        .voice_fact(FACT_PRICE, &range("range", 200, 1500))
        .text;
    assert_eq!(said, "من دولارين إلى 15 دولارًا حسب النموذج");
    let plain = Fact {
        id: "plain".to_owned(),
        args: vec![usd("price", 200)],
        warn: false,
    };
    assert_eq!(lang("ar").voice_fact(FACT_PRICE, &plain).text, "دولاران");
    let en = lang("en")
        .voice_fact(FACT_PRICE, &range("range", 200, 1500))
        .text;
    assert_eq!(en, "$2 to $15 by model");
}
