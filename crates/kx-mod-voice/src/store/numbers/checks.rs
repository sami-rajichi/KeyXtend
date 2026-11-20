//! The card checks' two blobs (ADR-0052): the checks in their first form, which older builds read as results of no
//! language, and beside them each result's language, which older builds never open.

use kx_module_api::Zeroizing;
use kx_module_api::ports::{Hundredths, Verdict, fits_iso};

use super::{VERSION, card, print};
use crate::counts::{Checked, Checks};
use crate::store::codec::{self, Bad, Mark, Reader, Writer};

/// What starts the card checks' blob.
pub(crate) const CHECKS_MARK: Mark = *b"KXVK";
/// What starts the blob of the checks' languages.
pub(crate) const LANGS_MARK: Mark = *b"KXVL";

/// A verdict's tag, in `Verdict`'s order.
const PASSED: u8 = 0;
const SLOW: u8 = 1;
const DIFFERS: u8 = 2;
const NO_ROOM: u8 = 3;
const FAILED: u8 = 4;

/// `checks` in the first form, each result without its language, so an older build still reads them.
pub(crate) fn encode_checks(checks: &Checks) -> Zeroizing<Vec<u8>> {
    codec::encode(|w| {
        w.head(CHECKS_MARK, VERSION);
        w.count(checks.rows().len());
        checks.rows().iter().for_each(|row| put_row(w, row));
    })
}

/// The checks in `bytes`, each naming no language until `named` finds it.
pub(crate) fn decode_checks(bytes: &[u8]) -> Result<Vec<Checked>, Bad> {
    let mut r = Reader::new(bytes);
    r.head(CHECKS_MARK, VERSION)?;
    let n = r.count()?;
    let rows = (0..n)
        .map(|_| row(&mut r, None))
        .collect::<Result<_, _>>()?;
    r.end()?;
    Ok(rows)
}

/// Each result of `checks` that names a language, with it, in their order.
pub(crate) fn encode_langs(checks: &Checks) -> Zeroizing<Vec<u8>> {
    let rows = checks.rows().iter();
    let named: Vec<_> = rows.filter_map(|c| Some((c.lang.as_deref()?, c))).collect();
    codec::encode(|w| {
        w.head(LANGS_MARK, VERSION);
        w.count(named.len());
        for (lang, row) in &named {
            w.text(lang);
            put_row(w, row);
        }
    })
}

/// The results in `bytes`, each with its language; one that is no ISO code is `Damaged`.
pub(crate) fn decode_langs(bytes: &[u8]) -> Result<Vec<Checked>, Bad> {
    let mut r = Reader::new(bytes);
    r.head(LANGS_MARK, VERSION)?;
    let n = r.count()?;
    let one = |r: &mut Reader<'_>| {
        let lang = r.text()?;
        let lang = fits_iso(lang).then(|| lang.to_owned());
        row(r, Some(lang.ok_or(Bad::Damaged)?))
    };
    let rows = (0..n).map(|_| one(&mut r)).collect::<Result<_, _>>()?;
    r.end()?;
    Ok(rows)
}

/// The checks of `plain`, each taking its language from the same result in `langs`. A result an older build wrote
/// since finds none there, so it serves no language.
pub(crate) fn named(plain: Vec<Checked>, mut langs: Vec<Checked>) -> Checks {
    let take = |row: Checked| {
        let same = langs.iter().position(|l| {
            Checked {
                lang: None,
                ..l.clone()
            } == row
        });
        same.map_or(row, |at| langs.swap_remove(at))
    };
    Checks::new(plain.into_iter().map(take).collect())
}

/// One result's card, driver, file, verdict and day: the first form's row.
fn put_row(w: &mut Writer, row: &Checked) {
    w.text(row.card.as_str());
    w.text(&row.driver);
    w.text(&row.model.hex());
    put_verdict(w, row.verdict);
    w.date(row.day);
}

/// The row `put_row` wrote, on the clip of `lang`.
fn row(r: &mut Reader<'_>, lang: Option<String>) -> Result<Checked, Bad> {
    let (card, driver) = (card(r)?, r.text()?.to_owned());
    let model = print(r)?;
    let (verdict, day) = (verdict(r)?, r.date()?);
    Ok(Checked {
        card,
        driver,
        model,
        lang,
        verdict,
        day,
    })
}

fn put_verdict(w: &mut Writer, verdict: Verdict) {
    match verdict {
        Verdict::Passed { gain } => {
            w.u8(PASSED);
            w.u32(gain.0);
        }
        Verdict::Slow { gain, need } => {
            w.u8(SLOW);
            w.u32(gain.0);
            w.u32(need.0);
        }
        Verdict::Differs { words, allowed } => {
            w.u8(DIFFERS);
            w.u16(words);
            w.u8(allowed);
        }
        Verdict::NoRoom => w.u8(NO_ROOM),
        Verdict::Failed => w.u8(FAILED),
    }
}

fn verdict(r: &mut Reader<'_>) -> Result<Verdict, Bad> {
    Ok(match r.u8()? {
        PASSED => Verdict::Passed {
            gain: Hundredths(r.u32()?),
        },
        SLOW => Verdict::Slow {
            gain: Hundredths(r.u32()?),
            need: Hundredths(r.u32()?),
        },
        DIFFERS => Verdict::Differs {
            words: r.u16()?,
            allowed: r.u8()?,
        },
        NO_ROOM => Verdict::NoRoom,
        FAILED => Verdict::Failed,
        _ => return Err(Bad::Damaged),
    })
}

#[cfg(test)]
mod tests;
