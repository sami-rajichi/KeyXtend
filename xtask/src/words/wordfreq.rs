//! Reads wordfreq's `cB` lists (`.msgpack.gz`): a header map, then bucket `i` holding the words whose frequency is
//! about 10^(−i/100), that is `i` centibels below 1.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use flate2::read::GzDecoder;
use kx_mod_predict::pack::layout::STEPS_PER_ZIPF;
use rmpv::Value;

use super::{WordsError, float, io_error};

/// The header key naming the format.
const FORMAT_KEY: &str = "format";
/// The only format read: centibel buckets.
const FORMAT: &str = "cB";
/// The header key naming the version.
const VERSION_KEY: &str = "version";
/// The only version read.
const VERSION: u64 = 1;
/// Centibels per power of ten, so per Zipf unit.
const CB_PER_ZIPF: u32 = 100;
/// wordfreq's Zipf value is this plus log10 of the frequency, so Zipf 0 is a frequency of 10^-9.
const ZIPF_OFFSET: u32 = 9;
/// The centibels of Zipf 0: a bucket's Zipf is this less the bucket, over `CB_PER_ZIPF`.
pub(crate) const ZIPF_TOP_CB: u32 = ZIPF_OFFSET * CB_PER_ZIPF;

/// One word of a list, with its bucket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    /// The word as wordfreq stores it, in lower case.
    pub(crate) word: String,
    /// Its bucket: centibels below a frequency of 1.
    pub(crate) bucket: u32,
}

/// The entries of the list at `path`, most frequent first.
pub(crate) fn read(path: &Path) -> Result<Vec<Entry>, WordsError> {
    let file = File::open(path).map_err(|e| io_error(path, &e))?;
    decode(BufReader::new(file))
        .map_err(|why| WordsError::Wordfreq(format!("{}: {why}", path.display())))
}

/// The entries of the gzipped list in `reader`, most frequent first.
pub(crate) fn decode(reader: impl Read) -> Result<Vec<Entry>, String> {
    let value = rmpv::decode::read_value(&mut GzDecoder::new(reader)).map_err(|e| e.to_string())?;
    let Value::Array(items) = value else {
        return Err("the list is not a msgpack array".to_string());
    };
    let mut items = items.into_iter();
    if !items.next().is_some_and(|h| is_header(&h)) {
        return Err(format!(
            "the header is not {{{FORMAT_KEY}: {FORMAT}, {VERSION_KEY}: {VERSION}}}"
        ));
    }
    let mut entries = Vec::new();
    for (bucket, words) in (0_u32..).zip(items) {
        let Value::Array(words) = words else {
            return Err(format!("bucket {bucket} is not a list"));
        };
        for word in words {
            let word = word
                .as_str()
                .ok_or_else(|| format!("bucket {bucket} holds a non-word"))?;
            entries.push(Entry {
                word: word.to_string(),
                bucket,
            });
        }
    }
    Ok(entries)
}

/// True when `value` is the header of format `cB`, version 1.
fn is_header(value: &Value) -> bool {
    let Value::Map(pairs) = value else {
        return false;
    };
    let get = |key: &str| {
        pairs
            .iter()
            .find(|(k, _)| k.as_str() == Some(key))
            .map(|(_, v)| v)
    };
    get(FORMAT_KEY).and_then(Value::as_str) == Some(FORMAT)
        && get(VERSION_KEY).and_then(Value::as_u64) == Some(VERSION)
}

/// The frequency byte of `bucket`: its Zipf value times the pack's `STEPS_PER_ZIPF`.
pub(crate) fn freq(bucket: u32) -> u8 {
    scaled(bucket, STEPS_PER_ZIPF)
}

/// The Zipf value of `bucket` times `steps`, clamped to a byte.
pub(crate) fn scaled(bucket: u32, steps: u32) -> u8 {
    let above_floor = ZIPF_TOP_CB.saturating_sub(bucket);
    u8::try_from(above_floor.saturating_mul(steps) / CB_PER_ZIPF).unwrap_or(u8::MAX)
}

/// The share of all words that a word of `bucket` takes: 10 to the power of minus its centibels.
pub(crate) fn share(bucket: u32) -> f64 {
    10f64.powf(-f64::from(bucket) / f64::from(CB_PER_ZIPF))
}

/// The bucket of a word seen `count` times in `total` tokens, its share in centibels; `None` when it was never seen.
pub(crate) fn bucket_of(count: u64, total: u64) -> Option<u32> {
    if total < count {
        return None;
    }
    bucket_of_share(float(count) / float(total))
}

/// The bucket of a word taking `share` of all words; `None` for no share or one past all words.
pub(crate) fn bucket_of_share(share: f64) -> Option<u32> {
    (share > 0.0 && share <= 1.0).then(|| whole(-f64::from(CB_PER_ZIPF) * share.log10()))
}

/// `x` rounded to a whole number of centibels, at least 0.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "A share of at least one token in 2^64 is under 2000 centibels, and it is clamped at 0."
)]
fn whole(x: f64) -> u32 {
    x.round().max(0.0) as u32
}
