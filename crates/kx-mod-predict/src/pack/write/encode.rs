//! The writer's section encoders and the final layout: header, section table, then the sections in order.

use std::collections::BTreeMap;

use fst::MapBuilder;

use super::super::layout::{ENTRY_LEN, ID_BYTES, MAGIC, MAX_ISO, RUN_SHIFT, SCORE_SHIFT, VERSION};
use super::super::{Flags, WordId};
use super::{Word, WriteError};

/// One chain length's contexts, sorted, each with its kept followers and their scores.
pub(super) type Table = BTreeMap<Vec<WordId>, Vec<(WordId, u8)>>;

/// The header's fields.
pub(super) struct Header<'a> {
    /// The language code, already checked.
    pub iso: &'a str,
    /// The pack's flags.
    pub flags: Flags,
    /// The word count.
    pub words: u32,
    /// The highest chain length.
    pub max_chain: u8,
    /// The followers kept per context.
    pub followers: u8,
}

/// Appends `value` as a little-endian u32; `TooBig` when it does not fit.
fn put_u32(out: &mut Vec<u8>, value: usize) -> Result<(), WriteError> {
    let value = u32::try_from(value).map_err(|_| WriteError::TooBig)?;
    out.extend_from_slice(&value.to_le_bytes());
    Ok(())
}

/// The `Lexicon` section: each folded key of the sorted `words` with its run of ids.
pub(super) fn lexicon(words: &[&Word]) -> Result<Vec<u8>, WriteError> {
    let refused = |e: fst::Error| WriteError::Lexicon(e.to_string());
    let mut map = MapBuilder::memory();
    let mut first = 0_u64;
    for run in words.chunk_by(|a, b| a.key == b.key) {
        let count = u64::try_from(run.len()).map_err(|_| WriteError::TooBig)?;
        if let Some(word) = run.first() {
            map.insert(&word.key, first | (count << RUN_SHIFT))
                .map_err(refused)?;
        }
        first += count;
    }
    map.into_inner().map_err(refused)
}

/// The `Words` section: one offset per word and one past the last, then the UTF-8 text.
pub(super) fn spellings(words: &[&Word]) -> Result<Vec<u8>, WriteError> {
    let mut out = Vec::new();
    let mut text = Vec::new();
    for word in words {
        put_u32(&mut out, text.len())?;
        text.extend_from_slice(word.spelling.as_bytes());
    }
    put_u32(&mut out, text.len())?;
    out.extend(text);
    Ok(out)
}

/// One chain section: the two counts, each context's ids and first follower, then the followers with their scores.
pub(super) fn chains(table: &Table) -> Result<Vec<u8>, WriteError> {
    let mut records = Vec::new();
    let mut entries = Vec::new();
    let mut total = 0;
    for (context, followers) in table {
        for id in context {
            records.extend(id.to_le_bytes().into_iter().take(ID_BYTES));
        }
        put_u32(&mut records, total)?;
        total += followers.len();
        for (id, score) in followers {
            entries.extend_from_slice(&(id | u32::from(*score) << SCORE_SHIFT).to_le_bytes());
        }
    }
    let mut out = Vec::new();
    put_u32(&mut out, table.len())?;
    put_u32(&mut out, total)?;
    out.extend(records);
    out.extend(entries);
    Ok(out)
}

/// The whole pack: `header`, the section table, then each of `sections` as (kind, bytes) in order.
pub(super) fn assemble(
    header: &Header<'_>,
    sections: &[(u8, Vec<u8>)],
) -> Result<Vec<u8>, WriteError> {
    let mut out = MAGIC.to_vec();
    out.extend_from_slice(&VERSION.to_le_bytes());
    let mut iso = [0; MAX_ISO];
    let code = header.iso.as_bytes();
    iso.get_mut(..code.len())
        .ok_or(WriteError::BadIso)?
        .copy_from_slice(code);
    out.push(u8::try_from(code.len()).map_err(|_| WriteError::BadIso)?);
    out.extend_from_slice(&iso);
    out.push(header.flags.to_byte());
    out.extend_from_slice(&header.words.to_le_bytes());
    out.extend_from_slice(&[header.max_chain, header.followers]);
    out.push(u8::try_from(sections.len()).map_err(|_| WriteError::TooBig)?);
    let mut at = out.len() + sections.len() * ENTRY_LEN;
    for (kind, body) in sections {
        out.push(*kind);
        put_u32(&mut out, at)?;
        put_u32(&mut out, body.len())?;
        at += body.len();
    }
    for (_, body) in sections {
        out.extend_from_slice(body);
    }
    Ok(out)
}
