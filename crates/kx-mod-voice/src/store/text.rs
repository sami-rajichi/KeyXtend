//! A text's blob: the mark and version, then its id, order, time, pin, language, model, text and recording. The
//! text is written into a wiped buffer and read back into a `Secret`.

use kx_module_api::{Redacted, Zeroizing};

use super::codec::{self, Bad, Mark, Reader, Writer};
use crate::history::{Entry, Sound, Stamp};

/// What starts every text's blob.
pub(crate) const MARK: Mark = *b"KXVT";
/// The form's version, the byte after the mark; a newer one is never read or written over.
pub(crate) const VERSION: u8 = 1;

/// `entry` as bytes, in a wiped buffer.
pub(crate) fn encode(entry: &Entry) -> Zeroizing<Vec<u8>> {
    codec::encode(|w| write(w, entry))
}

fn write(w: &mut Writer, entry: &Entry) {
    w.head(MARK, VERSION);
    w.u64(entry.id);
    w.u64(entry.seq);
    w.i64(entry.at.unix_ms);
    w.i16(entry.at.offset_min);
    w.flag(entry.pinned);
    w.text(&entry.lang);
    w.text(entry.model.expose());
    w.text(entry.text.expose());
    w.flag(entry.sound.is_some());
    if let Some(sound) = entry.sound {
        w.u64(sound.bytes);
        w.u32(sound.blobs);
    }
}

/// The text in `bytes`.
pub(crate) fn decode(bytes: &[u8]) -> Result<Entry, Bad> {
    let mut r = Reader::new(bytes);
    r.head(MARK, VERSION)?;
    let (id, seq) = (r.u64()?, r.u64()?);
    let at = Stamp {
        unix_ms: r.i64()?,
        offset_min: r.i16()?,
    };
    let pinned = r.flag()?;
    let lang = r.text()?.to_owned();
    let model = Redacted::new(r.text()?.to_owned());
    let text = Redacted::new(Zeroizing::new(r.text()?.to_owned()));
    let sound = if r.flag()? {
        Some(sound(&mut r)?)
    } else {
        None
    };
    r.end()?;
    Ok(Entry {
        id,
        seq,
        at,
        text,
        lang,
        model,
        pinned,
        sound,
    })
}

/// A recording's size and blobs; an empty one is `Damaged`.
fn sound(r: &mut Reader<'_>) -> Result<Sound, Bad> {
    let (bytes, blobs) = (r.u64()?, r.u32()?);
    if bytes == 0 || blobs == 0 {
        return Err(Bad::Damaged);
    }
    Ok(Sound { bytes, blobs })
}
