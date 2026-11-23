//! The card checks' blobs (ADR-0052): the checks keep the first form, passes first, which older builds read, and an
//! old file loads with each result naming no language; each language comes back from the blob beside them, but only
//! onto a result identical in every other field.

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

/// File 1 failed in `ar` and passed in `en`, in that order or the other, and file 2 passed in `en`.
fn split(failed_first: bool) -> Checks {
    let failed = Checked {
        verdict: Verdict::Failed,
        ..checked(1, Some("ar"))
    };
    let file_1 = [failed, checked(1, Some("en"))];
    let file_1 = if failed_first {
        file_1
    } else {
        [file_1[1].clone(), file_1[0].clone()]
    };
    Checks::new([file_1.to_vec(), vec![checked(2, Some("en"))]].concat())
}

/// What an older build keeps of the checks' blob `bytes`: the last row for each card and file.
fn older_read(bytes: &[u8]) -> Vec<Checked> {
    let mut kept: Vec<Checked> = Vec::new();
    for row in decode_checks(bytes).unwrap() {
        kept.retain(|c| (&c.card, &c.model) != (&row.card, &row.model));
        kept.push(row);
    }
    kept
}

#[test]
fn the_checks_blob_puts_each_files_passes_before_its_other_verdicts() {
    let rows = decode_checks(&encode_checks(&split(true))).unwrap();
    let verdicts: Vec<_> = rows.iter().map(|c| (c.model, c.verdict)).collect();
    let (one, two) = (checked(1, None), checked(2, None));
    let want = [
        (one.model, one.verdict),
        (two.model, two.verdict),
        (one.model, Verdict::Failed),
    ];
    assert_eq!(verdicts, want);
}

#[test]
fn an_older_build_keeps_a_non_pass_when_any_language_did_not_pass() {
    for failed_first in [true, false] {
        let kept = older_read(&encode_checks(&split(failed_first)));
        let verdict = |n: u8| {
            kept.iter()
                .find(|c| c.model == checked(n, None).model)
                .unwrap()
                .verdict
        };
        assert_eq!(verdict(1), Verdict::Failed, "failed first: {failed_first}");
        assert_eq!(
            verdict(2),
            checked(2, None).verdict,
            "a file that passed everywhere"
        );
    }
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
fn a_result_an_older_build_rewrote_differently_takes_no_stale_language() {
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
