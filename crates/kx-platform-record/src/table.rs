//! The recorder's adapter table (`record.toml`), compiled in and parsed once: Media Foundation's libraries, the
//! encoder's rates, the capture's buffers and the sound's queue, cushion and downmix.

use std::collections::BTreeMap;
use std::sync::LazyLock;

use serde::Deserialize;
use thiserror::Error;

/// The table, compiled in so it is never read from disk.
pub const TEXT: &str = include_str!("record.toml");

/// The shipped table, parsed once; tests prove it parses.
static SHIPPED: LazyLock<Result<Table, TableError>> = LazyLock::new(|| Table::parse(TEXT));

/// The shipped table; `None` when it does not parse, which refuses every recording and copy.
#[must_use]
pub fn table() -> Option<&'static Table> {
    SHIPPED.as_ref().ok()
}

/// Why the table is unusable.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum TableError {
    /// It is not TOML of the expected shape, or names a key it does not know.
    #[error("the record table does not parse: {0}")]
    Parse(String),
    /// A list is empty, or a value that must be above 0 is not.
    #[error("the record table has an empty list or a zero value")]
    Empty,
    /// The cushion is not under the queue, or the drift not under the cushion.
    #[error("the record table's sound times are out of order")]
    Order,
    /// A downmix row is not named by a channel count, has the wrong length, or holds a gain that is not a plain number
    /// of 0 or more.
    #[error("the record table's downmix is wrong")]
    Downmix,
}

/// The recorder's fixed values.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    /// Media Foundation's libraries, loaded by a video job first.
    pub media_dlls: Vec<String>,
    /// The video's values.
    pub video: Video,
    /// The sound's values.
    pub sound: Sound,
}

/// The video's fixed values.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Video {
    /// The H.264 video's bits a second.
    pub bits_s: u32,
    /// The longest wait for the capture's first frame, in ms.
    pub first_frame_ms: u32,
    /// Frames the capture's pool holds.
    pub buffers: u8,
}

/// The sound's fixed values.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sound {
    /// The AAC track's bytes a second.
    pub aac_bytes_s: u32,
    /// The most sound a source keeps waiting, in ms.
    pub queue_ms: u32,
    /// How far the written sound stays behind the picture's clock, in ms.
    pub cushion_ms: u32,
    /// How far a packet may stray from where the last one ended, in ms, and still follow it.
    pub drift_ms: u32,
    /// Each channel's [left, right] gains, by the source's channel count, as written.
    pub downmix: BTreeMap<String, Vec<[f32; 2]>>,
}

impl Table {
    /// Reads and checks the table in `text`.
    ///
    /// # Errors
    /// A `TableError` naming what is wrong.
    pub fn parse(text: &str) -> Result<Self, TableError> {
        let t: Self = toml::from_str(text).map_err(|e| TableError::Parse(e.to_string()))?;
        let (v, s) = (&t.video, &t.sound);
        let counts = [v.bits_s, v.first_frame_ms, u32::from(v.buffers)];
        let rates = [s.aac_bytes_s, s.queue_ms, s.cushion_ms, s.drift_ms];
        let named = t.media_dlls.iter().all(|d| !d.trim().is_empty());
        if t.media_dlls.is_empty() || !named || counts.iter().chain(&rates).any(|n| *n == 0) {
            return Err(TableError::Empty);
        }
        if s.drift_ms >= s.cushion_ms || s.cushion_ms >= s.queue_ms {
            return Err(TableError::Order);
        }
        s.downmix.iter().try_for_each(|(n, rows)| row(n, rows))?;
        Ok(t)
    }

    /// The gains of a source of `channels`, as listed, or `None` when the table lists no such count.
    #[must_use]
    pub fn gains(&self, channels: u16) -> Option<&[[f32; 2]]> {
        let rows = self.sound.downmix.get(&channels.to_string())?;
        Some(rows)
    }
}

/// Refuses a downmix row not named by its own length, or with a gain that is not a plain number of 0 or more.
fn row(count: &str, rows: &[[f32; 2]]) -> Result<(), TableError> {
    let named = count.parse::<usize>().ok() == Some(rows.len());
    let plain = rows.iter().flatten().all(|g| g.is_finite() && *g >= 0.0);
    (named && plain).then_some(()).ok_or(TableError::Downmix)
}

/// Named by path, so the module also resolves where another crate includes this file by `#[path]`.
#[cfg(test)]
#[path = "table/tests.rs"]
mod tests;
