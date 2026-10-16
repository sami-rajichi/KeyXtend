//! Property tests of the strict reader: no input makes it panic, and a file it takes gives exactly its one `data`
//! chunk, after a basic PCM `fmt ` chunk in the format.

use proptest::collection::vec;
use proptest::prelude::*;

use kx_theme::sounds::wav::{PCM, samples};

use super::wav::{RATE, fmt, format};

/// Most chunks beside the generated `fmt ` and `data`, and most bytes in one of them.
const MAX_OTHERS: usize = 3;
const MAX_OTHER: usize = 40;
/// Generated files per run; one in a hundred or so is taken, so the exact check runs a few dozen times.
const CASES: u32 = 4096;
/// The small format's channels, bytes per frame and bits.
const CHANNELS: u16 = 1;
const FRAME: u16 = 2;
const BITS: u16 = 16;

/// A generated chunk: its id, its content and, when its header lies, the size it gives.
#[derive(Clone, Debug)]
struct Chunk {
    id: [u8; 4],
    content: Vec<u8>,
    size: Option<u32>,
}

/// The most sample bytes the small format takes.
fn max_data() -> usize {
    format().max_frames() * format().frame_bytes()
}

/// Mostly `None`, now and then any value: a size that lies.
fn lie() -> impl Strategy<Value = Option<u32>> {
    prop_oneof![7 => Just(None), 1 => any::<u32>().prop_map(Some)]
}

/// Mostly `v`, else a value next to it or any value.
fn near16(v: u16) -> impl Strategy<Value = u16> {
    prop_oneof![6 => Just(v), 1 => (v - 1)..=(v + 1), 1 => any::<u16>()]
}

/// Mostly `v`, else a value next to it or any value.
fn near32(v: u32) -> impl Strategy<Value = u32> {
    prop_oneof![6 => Just(v), 1 => (v - 1)..=(v + 1), 1 => any::<u32>()]
}

/// A `fmt ` chunk's content with each field valid or near it, now and then with bytes after the basic ones.
fn near_fmt() -> impl Strategy<Value = Vec<u8>> {
    let byte_rate = RATE * u32::from(FRAME);
    let head = (
        near16(PCM),
        near16(CHANNELS),
        near32(RATE),
        near32(byte_rate),
    );
    let tail = (near16(FRAME), near16(BITS));
    let extra = prop_oneof![6 => Just(Vec::new()), 1 => vec(any::<u8>(), 1..4)];
    (head, tail, extra).prop_map(|((tag, channels, rate, byte_rate), (align, bits), extra)| {
        let mut out = Vec::new();
        out.extend(tag.to_le_bytes());
        out.extend(channels.to_le_bytes());
        out.extend(rate.to_le_bytes());
        out.extend(byte_rate.to_le_bytes());
        out.extend(align.to_le_bytes());
        out.extend(bits.to_le_bytes());
        out.extend(extra);
        out
    })
}

/// A chunk of `id` holding `content`, whose size now and then lies.
fn chunk(
    id: impl Strategy<Value = [u8; 4]>,
    content: impl Strategy<Value = Vec<u8>>,
) -> impl Strategy<Value = Chunk> {
    (id, content, lie()).prop_map(|(id, content, size)| Chunk { id, content, size })
}

/// A near-valid `fmt `, a short `data` and a few other chunks, in any order; the others may repeat either id.
fn chunks() -> impl Strategy<Value = Vec<Chunk>> {
    let fmt = chunk(Just(*b"fmt "), near_fmt());
    let data = chunk(Just(*b"data"), vec(any::<u8>(), 0..=max_data() + 2));
    let id = prop_oneof![2 => any::<[u8; 4]>(), 1 => Just(*b"fmt "), 1 => Just(*b"data")];
    let others = vec(chunk(id, vec(any::<u8>(), 0..MAX_OTHER)), 0..=MAX_OTHERS);
    (fmt, data, others).prop_flat_map(|(fmt, data, mut all)| {
        all.extend([fmt, data]);
        Just(all).prop_shuffle()
    })
}

/// A RIFF/WAVE file of `chunks`, each padded to an even length, with the RIFF size `riff` or the true one, then
/// `tail`.
fn file(chunks: &[Chunk], riff: Option<u32>, tail: &[u8]) -> Vec<u8> {
    let len = |n: usize| u32::try_from(n).expect("a generated file is small");
    let mut body = b"WAVE".to_vec();
    for c in chunks {
        body.extend(c.id);
        body.extend(c.size.unwrap_or(len(c.content.len())).to_le_bytes());
        body.extend(&c.content);
        if c.content.len() % 2 == 1 {
            body.push(0);
        }
    }
    let mut out = b"RIFF".to_vec();
    out.extend(riff.unwrap_or(len(body.len())).to_le_bytes());
    out.extend(body);
    out.extend(tail);
    out
}

proptest! {
    #[test]
    fn any_bytes_give_an_error_or_samples_never_a_panic(bytes in vec(any::<u8>(), 0..512)) {
        let _ = samples(&bytes, &format());
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn a_taken_file_gives_exactly_its_one_data_chunk(
        chunks in chunks(),
        riff in lie(),
        tail in prop_oneof![7 => Just(Vec::new()), 1 => vec(any::<u8>(), 1..3)],
    ) {
        let bytes = file(&chunks, riff, &tail);
        let Ok(data) = samples(&bytes, &format()) else {
            return Ok(());
        };
        let frame = usize::from(FRAME);
        prop_assert!(!data.is_empty() && data.len().is_multiple_of(frame) && data.len() <= max_data());
        let honest = riff.is_none() && tail.is_empty() && chunks.iter().all(|c| c.size.is_none());
        if honest {
            let at = |id: &[u8; 4]| chunks.iter().position(|c| &c.id == id).expect("a taken file has it");
            let count = |id: &[u8; 4]| chunks.iter().filter(|c| &c.id == id).count();
            prop_assert_eq!((count(b"fmt "), count(b"data")), (1, 1));
            prop_assert!(at(b"fmt ") < at(b"data"), "the fmt chunk comes first");
            prop_assert_eq!(&chunks[at(b"fmt ")].content, &fmt(PCM, CHANNELS, RATE, BITS));
            prop_assert_eq!(data, &chunks[at(b"data")].content[..]);
        }
    }
}
