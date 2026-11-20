//! The numbers' blobs, each a mark and a version, then its rows: the counts by day, model and device, a cloud
//! service's too; the card checks by card, driver and file fingerprint, with each one's clip language beside them;
//! Compare's runs by language. They hold ids and numbers, never a text.

mod checks;
mod tags;

use std::collections::BTreeMap;

use kx_module_api::Zeroizing;
use kx_module_api::ports::{CardId, Fingerprint, fits_iso};

use super::codec::{self, Bad, Mark, Reader, Writer};
use crate::counts::{Board, Bucket, Counts, Score, Scores, Tally};
#[cfg(test)]
pub(crate) use checks::{CHECKS_MARK, LANGS_MARK};
pub(crate) use checks::{decode_checks, decode_langs, encode_checks, encode_langs, named};
use tags::{device, model, put_device, put_model};

/// Each numbers form's version, the byte after each mark; a newer one is never read or written over.
pub(crate) const VERSION: u8 = 1;
/// What starts the counts' blob.
pub(crate) const COUNTS_MARK: Mark = *b"KXVC";
/// What starts Compare's results' blob.
pub(crate) const SCORES_MARK: Mark = *b"KXVS";

/// `counts` as bytes.
pub(crate) fn encode_counts(counts: &Counts) -> Zeroizing<Vec<u8>> {
    codec::encode(|w| {
        w.head(COUNTS_MARK, VERSION);
        w.count(counts.rows().len());
        for (bucket, tally) in counts.rows() {
            w.date(bucket.date);
            put_model(w, &bucket.model);
            put_device(w, &bucket.device);
            w.u32(tally.dictations);
            w.u64(tally.sound_ms);
            w.u64(tally.wait_ms);
        }
    })
}

/// The counts in `bytes`; a bucket written twice is `Damaged`.
pub(crate) fn decode_counts(bytes: &[u8]) -> Result<Counts, Bad> {
    let mut r = Reader::new(bytes);
    r.head(COUNTS_MARK, VERSION)?;
    let mut rows = BTreeMap::new();
    for _ in 0..r.count()? {
        let date = r.date()?;
        let model = model(&mut r)?;
        let device = device(&mut r)?;
        let (dictations, sound_ms, wait_ms) = (r.u32()?, r.u64()?, r.u64()?);
        let tally = Tally {
            dictations,
            sound_ms,
            wait_ms,
        };
        let bucket = Bucket {
            date,
            model,
            device,
        };
        if rows.insert(bucket, tally).is_some() {
            return Err(Bad::Damaged);
        }
    }
    r.end()?;
    Ok(Counts::new(rows))
}

/// `scores` as bytes.
pub(crate) fn encode_scores(scores: &Scores) -> Zeroizing<Vec<u8>> {
    codec::encode(|w| {
        w.head(SCORES_MARK, VERSION);
        w.count(scores.boards().len());
        for board in scores.boards() {
            w.text(&board.lang);
            w.date(board.day);
            w.flag(board.auto);
            w.flag(board.card.is_some());
            if let Some(card) = &board.card {
                w.text(card.as_str());
            }
            w.count(board.scores.len());
            board.scores.iter().for_each(|score| put_score(w, score));
        }
    })
}

/// The runs in `bytes`; a language that is no ISO code is `Damaged`.
pub(crate) fn decode_scores(bytes: &[u8]) -> Result<Scores, Bad> {
    let mut r = Reader::new(bytes);
    r.head(SCORES_MARK, VERSION)?;
    let mut boards = Vec::new();
    for _ in 0..r.count()? {
        let lang = r.text()?;
        if !fits_iso(lang) {
            return Err(Bad::Damaged);
        }
        let (lang, day, auto) = (lang.to_owned(), r.date()?, r.flag()?);
        let card = if r.flag()? { Some(card(&mut r)?) } else { None };
        let n = r.count()?;
        let scores = (0..n).map(|_| score(&mut r)).collect::<Result<_, _>>()?;
        boards.push(Board {
            lang,
            day,
            auto,
            card,
            scores,
        });
    }
    r.end()?;
    Ok(Scores::new(boards))
}

fn put_score(w: &mut Writer, score: &Score) {
    w.text(&score.model.hex());
    w.flag(score.wrong.is_some());
    w.u16(score.wrong.unwrap_or_default());
    w.u16(score.words);
    w.u32(score.wait_ms);
    w.u32(score.mb);
}

fn score(r: &mut Reader<'_>) -> Result<Score, Bad> {
    let model = print(r)?;
    let (counted, wrong) = (r.flag()?, r.u16()?);
    let (words, wait_ms, mb) = (r.u16()?, r.u32()?, r.u32()?);
    Ok(Score {
        model,
        wrong: counted.then_some(wrong),
        words,
        wait_ms,
        mb,
    })
}

fn card(r: &mut Reader<'_>) -> Result<CardId, Bad> {
    CardId::new(r.text()?).ok_or(Bad::Damaged)
}

fn print(r: &mut Reader<'_>) -> Result<Fingerprint, Bad> {
    Fingerprint::from_hex(r.text()?).ok_or(Bad::Damaged)
}
