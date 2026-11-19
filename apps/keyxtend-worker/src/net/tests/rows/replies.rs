//! Each row's answers read as its own pages say: its text, a bad key, no credit, a spending cap or a 403 behind a
//! paywall as the account's step or else the key, a country refused by its row's code, a language refused in an
//! `error` or in a 422's `detail[]`, too big, too many requests and a service down. The fixtures under
//! `replies/<service>/` copy each page's body with made-up ids and no key; a body the pages do not show is marked as
//! assumed, for the live check.

use super::super::table::{Extra, Got, read_by, tested_by};
use super::*;
use kx_ipc::NetFault::{Account, Credit, Key, Language, Network, Region, Strange, TooBig};

/// The text each case of `CASES` reads, in its order.
pub(super) const TEXTS: [&[u8]; 9] = [
    TEXT,
    OPENAI_TEXT,
    MISTRAL_TEXT,
    TOGETHER_TEXT,
    LEMONFOX_TEXT,
    DEEPINFRA_TEXT,
    ROUTER_TEXT,
    ROUTER_TEXT,
    TEXT,
];

/// `OpenAI`'s reply holds its languages and usage beside the text [O2].
const OPENAI_TEXT: &[u8] = include_bytes!("../replies/openai/text.json");
const OPENAI_KEY: &[u8] = include_bytes!("../replies/openai/key.json");
const OPENAI_CREDIT: &[u8] = include_bytes!("../replies/openai/credit.json");
const OPENAI_SPEND: &[u8] = include_bytes!("../replies/openai/spend.json");
const OPENAI_RATE: &[u8] = include_bytes!("../replies/openai/rate.json");
const OPENAI_COUNTRY: &[u8] = include_bytes!("../replies/openai/country.json");
const OPENAI_BUSY: &[u8] = include_bytes!("../replies/openai/overloaded.json");
/// Mistral's `language` is null even when pinned [M1]; its error bodies are assumed.
const MISTRAL_TEXT: &[u8] = include_bytes!("../replies/mistral/text.json");
const MISTRAL_KEY: &[u8] = include_bytes!("../replies/mistral/key.json");
const MISTRAL_LANGUAGE: &[u8] = include_bytes!("../replies/mistral/language.json");
const MISTRAL_RATE: &[u8] = include_bytes!("../replies/mistral/rate.json");
const MISTRAL_CONTEXT: &[u8] = include_bytes!("../replies/mistral/context.json");
/// Together's statuses are on its error page [T5]; its bodies are assumed.
const TOGETHER_TEXT: &[u8] = include_bytes!("../replies/together/text.json");
const TOGETHER_KEY: &[u8] = include_bytes!("../replies/together/key.json");
const TOGETHER_CREDIT: &[u8] = include_bytes!("../replies/together/credit.json");
const TOGETHER_PAYWALL: &[u8] = include_bytes!("../replies/together/paywall.json");
const TOGETHER_BIG: &[u8] = include_bytes!("../replies/together/too-big.json");
/// Lemonfox documents its text only [L2].
const LEMONFOX_TEXT: &[u8] = include_bytes!("../replies/lemonfox/text.json");
/// `DeepInfra` documents a 422 with `detail[]` only [D3]; its key body is assumed.
const DEEPINFRA_TEXT: &[u8] = include_bytes!("../replies/deepinfra/text.json");
const DEEPINFRA_LANGUAGE: &[u8] = include_bytes!("../replies/deepinfra/language.json");
const DEEPINFRA_INVALID: &[u8] = include_bytes!("../replies/deepinfra/invalid.json");
const DEEPINFRA_KEY: &[u8] = include_bytes!("../replies/deepinfra/key.json");
/// `OpenRouter`'s text with its usage, and its errors with a number for a code [R2][R7].
const ROUTER_TEXT: &[u8] = include_bytes!("../replies/openrouter/text.json");
const ROUTER_KEY: &[u8] = include_bytes!("../replies/openrouter/key.json");
const ROUTER_CREDIT: &[u8] = include_bytes!("../replies/openrouter/credit.json");
const ROUTER_BUDGET: &[u8] = include_bytes!("../replies/openrouter/budget.json");
const ROUTER_RATE: &[u8] = include_bytes!("../replies/openrouter/rate.json");
const ROUTER_LIMITS: &[u8] = include_bytes!("../replies/openrouter/limits.json");

/// One answer: its status, headers and body, and what it reads as.
type Answer<'a> = (u16, Extra<'a>, &'a [u8], Got);

/// The text the rows' fixtures say.
fn said() -> Got {
    Got::Text(SAID.to_owned())
}

/// Checks that `row` reads each answer as its own.
fn reads(row: Row, answers: &[Answer<'_>]) {
    let call = row.call();
    for (status, extra, body, want) in answers {
        let got = match read_by(&call, *status, extra, body) {
            Got::Text(t) => Got::Text(t.trim().to_owned()),
            other => other,
        };
        let shown = String::from_utf8_lossy(body);
        assert_eq!(&got, want, "{} {status} {shown}", row.name);
    }
}

#[test]
fn groq_and_a_custom_address_read_a_403_as_the_key() {
    for row in [GROQ, CUSTOM] {
        reads(
            row,
            &[
                (OK, &[], TEXT, Got::Text("Bonjour tout le monde.".into())),
                (401, &[], KEY_BAD, Got::Fault(Key)),
                (403, &[], KEY_BAD, Got::Fault(Key)),
                (400, &[], LANGUAGE, Got::Fault(Language)),
                (BUSY, &[], CREDIT, Got::Fault(Credit)),
            ],
        );
    }
}

#[test]
fn openais_answers_read_as_its_error_page_says() {
    reads(
        OPENAI,
        &[
            (OK, &[], OPENAI_TEXT, said()),
            (401, &[], OPENAI_KEY, Got::Fault(Key)),
            (BUSY, &[], OPENAI_CREDIT, Got::Fault(Credit)),
            (BUSY, &[], OPENAI_SPEND, Got::Fault(Account)),
            (
                BUSY,
                &[("retry-after", "2")],
                OPENAI_RATE,
                Got::Wait(Some(2)),
            ),
            (403, &[], OPENAI_COUNTRY, Got::Fault(Region)),
            (503, &[], OPENAI_BUSY, Got::Fault(Network)),
        ],
    );
}

#[test]
fn mistrals_answers_read_its_language_refusal_in_a_nested_detail() {
    reads(
        MISTRAL,
        &[
            (OK, &[], MISTRAL_TEXT, said()),
            (401, &[], MISTRAL_KEY, Got::Fault(Key)),
            (422, &[], MISTRAL_LANGUAGE, Got::Fault(Language)),
            (BUSY, &[], MISTRAL_RATE, Got::Wait(None)),
            (400, &[], MISTRAL_CONTEXT, Got::Fault(Strange)),
        ],
    );
}

#[test]
fn togethers_403_is_its_paywall_never_the_key() {
    reads(
        TOGETHER,
        &[
            (OK, &[], TOGETHER_TEXT, said()),
            (401, &[], TOGETHER_KEY, Got::Fault(Key)),
            (402, &[], TOGETHER_CREDIT, Got::Fault(Credit)),
            (403, &[], TOGETHER_PAYWALL, Got::Fault(Account)),
            (413, &[], TOGETHER_BIG, Got::Fault(TooBig)),
            (BUSY, &[], TOGETHER_CREDIT, Got::Wait(None)),
            (503, &[], DOWN, Got::Fault(Network)),
        ],
    );
}

#[test]
fn lemonfox_reads_its_text_and_the_forms_statuses() {
    reads(
        LEMONFOX,
        &[
            (OK, &[], LEMONFOX_TEXT, said()),
            (401, &[], KEY_BAD, Got::Fault(Key)),
            (500, &[], DOWN, Got::Fault(Network)),
            (OK, &[], HOTSPOT, Got::Fault(Strange)),
        ],
    );
}

#[test]
fn deepinfras_422_names_the_language_in_its_detail_list() {
    reads(
        DEEPINFRA,
        &[
            (OK, &[], DEEPINFRA_TEXT, said()),
            (422, &[], DEEPINFRA_LANGUAGE, Got::Fault(Language)),
            (422, &[], DEEPINFRA_INVALID, Got::Fault(Strange)),
            (401, &[], DEEPINFRA_KEY, Got::Fault(Key)),
            (BUSY, &[], DEEPINFRA_INVALID, Got::Wait(None)),
        ],
    );
}

#[test]
fn openrouters_402_is_credit_and_its_capped_key_the_accounts_step() {
    for row in [ROUTER, ROUTER_OPEN] {
        reads(
            row,
            &[
                (OK, &[], ROUTER_TEXT, said()),
                (401, &[], ROUTER_KEY, Got::Fault(Key)),
                (402, &[], ROUTER_CREDIT, Got::Fault(Credit)),
                (403, &[], ROUTER_BUDGET, Got::Fault(Account)),
                (
                    BUSY,
                    &[("retry-after", "4")],
                    ROUTER_RATE,
                    Got::Wait(Some(4)),
                ),
                (524, &[], ROUTER_RATE, Got::Fault(Network)),
                (529, &[], ROUTER_RATE, Got::Fault(Network)),
            ],
        );
    }
    let call = ROUTER.call();
    let tested = tested_by(&call, OK, &[], ROUTER_LIMITS);
    assert_eq!(
        tested,
        Got::Text(String::new()),
        "its key test reads the key's limits"
    );
}

#[test]
fn the_same_403_is_the_key_without_a_paywall_and_the_accounts_step_behind_one() {
    for body in [KEY_BAD, TOGETHER_PAYWALL, ROUTER_BUDGET, b""] {
        assert_eq!(read_by(&walled(false), 403, &[], body), Got::Fault(Key));
        assert_eq!(read_by(&walled(true), 403, &[], body), Got::Fault(Account));
    }
}

#[test]
fn a_region_refusal_reads_by_the_rows_exact_code_and_the_same_403_without_it_as_before() {
    let read = |row: Row, body| read_by(&row.call(), 403, &[], body);
    assert_eq!(read(OPENAI, OPENAI_COUNTRY), Got::Fault(Region));
    assert_eq!(read(OPENAI, OPENAI_KEY), Got::Fault(Key));
    assert_eq!(read(GROQ, OPENAI_COUNTRY), Got::Fault(Key), "no code named");
    assert_eq!(read(TOGETHER, OPENAI_COUNTRY), Got::Fault(Account));
    let near = Row {
        region: Some("unsupported_country"),
        ..OPENAI
    };
    assert_eq!(
        read(near, OPENAI_COUNTRY),
        Got::Fault(Key),
        "the exact code"
    );
    let tested = tested_by(&OPENAI.call(), 403, &[], OPENAI_COUNTRY);
    assert_eq!(tested, Got::Fault(Region), "a key test too");
}
