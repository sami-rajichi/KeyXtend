//! The card checks' blobs (ADR-0052): the checks keep the first form, which older builds read, and an old file loads
//! with each result naming no language; each language comes back from the blob beside them, but never onto a result
//! an older build wrote since.

use std::sync::Arc;

use kx_module_api::ports::{CardId, Date, Fingerprint, MAX_BLOB_BYTES};

use super::*;
use crate::store::mem::Mem;
use crate::store::{Damage, Opened, Store};

/// The card and driver of the results here.
const CARD: &str = "Vulkan0";
const DRIVER: &str = "32.0.15.6094";

fn day() -> Date {
    Date::new(2026, 11, 14).unwrap()
}

/// A passed result on the clip of `lang`, of file `n`.
fn checked(n: u8, lang: Option<&str>) -> Checked {
    Checked {
        card: CardId::new(CARD).unwrap(),
        driver: DRIVER.into(),
        model: Fingerprint::new([n; Fingerprint::BYTES]),
        lang: lang.map(str::to_owned),
        verdict: Verdict::Passed {
            gain: Hundredths(900),
        },
        day: day(),
    }
}

/// The checks blob of `rows` as the first form writes it, which older builds read: no language.
fn first_form(rows: &[Checked]) -> Zeroizing<Vec<u8>> {
    codec::encode(|w| {
        w.head(CHECKS_MARK, VERSION);
        w.count(rows.len());
        for row in rows {
            w.text(row.card.as_str());
            w.text(&row.driver);
            w.text(&row.model.hex());
            put_verdict(w, row.verdict);
            w.date(row.day);
        }
    })
}

/// The store over a shelf of the checks' blob `checks` and, when given, the languages' blob `langs`.
fn opened(checks: &[u8], langs: Option<&[u8]>) -> Opened {
    let shelf = Arc::new(Mem::default());
    shelf.plant("numbers", "checks", checks);
    if let Some(langs) = langs {
        shelf.plant("numbers", "check-langs", langs);
    }
    Store::open(shelf, MAX_BLOB_BYTES).unwrap()
}

/// Two languages of file 1 and an old result of file 2.
fn mixed() -> Checks {
    Checks::new(vec![
        checked(1, Some("ar")),
        checked(1, Some("en")),
        checked(2, None),
    ])
}

#[test]
fn the_checks_blob_keeps_the_first_form_whatever_the_languages() {
    let plain = [checked(1, None), checked(1, None), checked(2, None)];
    assert_eq!(*encode_checks(&mixed()), *first_form(&plain));
}

#[test]
fn an_old_file_loads_each_result_naming_no_language() {
    let old = vec![checked(1, None), checked(2, None)];
    assert_eq!(decode_checks(&first_form(&old)), Ok(old.clone()));
    let o = opened(&first_form(&old), None);
    assert_eq!(o.checks, Checks::new(old));
    assert_eq!(o.damaged, Damage::default(), "nothing is dropped");
}

#[test]
fn each_language_comes_back_from_the_blob_beside_the_checks() {
    let o = opened(&encode_checks(&mixed()), Some(&encode_langs(&mixed())));
    assert_eq!(o.checks, mixed());
}

#[test]
fn a_result_an_older_build_wrote_since_takes_no_stale_language() {
    let mine = Checks::new(vec![checked(1, Some("ar")), checked(2, Some("en"))]);
    let failed = Checked {
        verdict: Verdict::Failed,
        ..checked(1, None)
    };
    let theirs = [failed.clone(), checked(2, None)];
    let o = opened(&first_form(&theirs), Some(&encode_langs(&mine)));
    assert_eq!(o.checks.rows(), [failed, checked(2, Some("en"))]);
}

#[test]
fn a_damaged_languages_blob_drops_the_languages_alone() {
    let o = opened(&encode_checks(&mixed()), Some(b"junk"));
    let plain = [checked(1, None), checked(1, None), checked(2, None)];
    assert_eq!(o.checks, Checks::new(plain.to_vec()));
    let one = Damage {
        texts: 0,
        numbers: 1,
    };
    assert_eq!(o.damaged, one);
}

#[test]
fn a_language_that_is_no_iso_code_reads_damaged_and_a_later_form_newer() {
    let odd = codec::encode(|w| {
        w.head(LANGS_MARK, VERSION);
        w.count(1);
        w.text("EN");
        put_row(w, &checked(1, None));
    });
    assert_eq!(decode_langs(&odd), Err(Bad::Damaged));
    let later = |mark: Mark| codec::encode(|w| w.head(mark, VERSION + 1));
    assert_eq!(decode_checks(&later(CHECKS_MARK)), Err(Bad::Newer));
    assert_eq!(decode_langs(&later(LANGS_MARK)), Err(Bad::Newer));
}
