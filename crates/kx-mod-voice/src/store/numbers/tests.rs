//! The card checks' forms (ADR-0052): the first, whose results name no language, still loads with each result naming
//! none; the second keeps each result's clip language, an ISO code; a later one is never read.

use std::sync::Arc;

use kx_module_api::ports::{Date, MAX_BLOB_BYTES};

use super::*;
use crate::store::mem::Mem;
use crate::store::{Damage, Store};

/// The card, driver and file of the results here.
const CARD: &str = "Vulkan0";
const DRIVER: &str = "32.0.15.6094";

fn day() -> Date {
    Date::new(2026, 11, 14).unwrap()
}

fn pass() -> Verdict {
    Verdict::Passed {
        gain: Hundredths(900),
    }
}

/// The result on the clip of `lang`, of file `n`.
fn checked(n: u8, lang: Option<&str>) -> Checked {
    Checked {
        card: CardId::new(CARD).unwrap(),
        driver: DRIVER.into(),
        model: Fingerprint::new([n; Fingerprint::BYTES]),
        lang: lang.map(str::to_owned),
        verdict: pass(),
        day: day(),
    }
}

/// The checks blob of files `files` as the first form wrote it, with no language.
fn first_form(files: &[u8]) -> Zeroizing<Vec<u8>> {
    codec::encode(|w| {
        w.head(CHECKS_MARK, CHECKS_LANGLESS);
        w.count(files.len());
        for n in files {
            let row = checked(*n, None);
            w.text(row.card.as_str());
            w.text(&row.driver);
            w.text(&row.model.hex());
            put_verdict(w, row.verdict);
            w.date(row.day);
        }
    })
}

#[test]
fn a_first_form_blob_loads_with_each_result_naming_no_language() {
    let want = Checks::new(vec![checked(1, None), checked(2, None)]);
    assert_eq!(decode_checks(&first_form(&[1, 2])), Ok(want.clone()));
    let shelf = Arc::new(Mem::default());
    shelf.plant("numbers", "checks", &first_form(&[1, 2]));
    let opened = Store::open(shelf, MAX_BLOB_BYTES).unwrap();
    assert_eq!(opened.checks, want);
    assert_eq!(opened.damaged, Damage::default(), "nothing is dropped");
}

#[test]
fn the_second_form_keeps_each_results_language_or_none() {
    let checks = Checks::new(vec![checked(1, Some("ar")), checked(2, None)]);
    let bytes = encode_checks(&checks);
    assert_eq!(bytes.get(CHECKS_MARK.len()), Some(&CHECKS_VERSION));
    assert_eq!(decode_checks(&bytes), Ok(checks));
}

#[test]
fn a_language_that_is_no_iso_code_reads_damaged_and_a_later_form_newer() {
    let odd = codec::encode(|w| {
        w.head(CHECKS_MARK, CHECKS_VERSION);
        w.count(1);
        w.text(CARD);
        w.text(DRIVER);
        w.text(&Fingerprint::new([1; Fingerprint::BYTES]).hex());
        w.flag(true);
        w.text("EN");
        put_verdict(w, pass());
        w.date(day());
    });
    assert_eq!(decode_checks(&odd), Err(Bad::Damaged));
    let later = codec::encode(|w| w.head(CHECKS_MARK, CHECKS_VERSION + 1));
    assert_eq!(decode_checks(&later), Err(Bad::Newer));
    let older = codec::encode(|w| w.head(CHECKS_MARK, CHECKS_LANGLESS - 1));
    assert_eq!(decode_checks(&older), Err(Bad::Damaged));
}
