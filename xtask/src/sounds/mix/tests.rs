use super::*;
use crate::sounds::fixture::{BOOK, FORMAT, TAG_FLOAT, TAG_PCM, encode, mono, pcm, sine};
use crate::sounds::recipe::parse;
use crate::sounds::units::samples;
use crate::sounds::{Cut, import_clip};

/// A file in the shipping format: a sine of `hz`, `ms` long, imported so it peaks at the ceiling.
fn part(hz: f64, ms: u32) -> Vec<u8> {
    let wav = encode(TAG_PCM, 48_000, 16, &mono(&sine(hz, 48_000, ms, 0.5)));
    import_clip(&parse(BOOK, FORMAT).unwrap(), &wav, Cut::default())
        .unwrap()
        .bytes
}

#[test]
fn a_join_without_overlap_keeps_both_parts_and_a_silent_gap() {
    let book = parse(BOOK, FORMAT).unwrap();
    let (a, b) = (part(1_000.0, 20), part(500.0, 30));
    let first = shipping(&book, "a.wav", &a).unwrap();
    let second = shipping(&book, "b.wav", &b).unwrap();
    let joined = pcm(&join(&book, &first, &second, 50).unwrap().bytes);
    let (a, b) = (pcm(&a), pcm(&b));
    let at = samples(50.0, 48_000.0);
    assert_eq!(joined.len(), at + b.len());
    assert_eq!(joined[..a.len()], a[..]);
    assert!(
        joined[a.len()..at].iter().all(|s| *s == 0),
        "the gap is not silent"
    );
    // The join fades its own end again, so the second part matches up to its last fade.
    let fade = samples(book.format.fade_ms, 48_000.0);
    let end = joined.len() - fade;
    assert_eq!(joined[at..end], b[..b.len() - fade]);
    let tails = joined[end..].iter().zip(&b[b.len() - fade..]);
    assert!(tails.into_iter().all(|(j, o)| j.abs() <= o.abs()));
}

#[test]
fn a_join_longer_than_the_limit_is_refused() {
    let book = parse(BOOK, FORMAT).unwrap();
    let long = shipping(&book, "a.wav", &part(440.0, 300)).unwrap();
    let err = join(&book, &long, &long, 200).unwrap_err();
    assert!(matches!(err, SoundError::TooLong(_)), "got: {err}");
    assert!(matches!(
        join(&book, &long, &long, u32::MAX),
        Err(SoundError::TooLong(_))
    ));
    assert!(join(&book, &long, &long, 100).is_ok());
}

#[test]
fn only_files_in_the_shipping_format_are_joined() {
    let book = parse(BOOK, FORMAT).unwrap();
    let tone = sine(440.0, 48_000, 10, 0.5);
    let stereo: Vec<Vec<f64>> = tone.iter().map(|s| vec![*s, *s]).collect();
    for wav in [
        encode(TAG_PCM, 44_100, 16, &mono(&tone)),
        encode(TAG_PCM, 48_000, 24, &mono(&tone)),
        encode(TAG_FLOAT, 48_000, 32, &mono(&tone)),
        encode(TAG_PCM, 48_000, 16, &stereo),
    ] {
        let err = shipping(&book, "x.wav", &wav).unwrap_err();
        assert!(matches!(err, SoundError::NotShipping(_)), "got: {err}");
        assert!(err.to_string().contains("x.wav"), "{err}");
    }
    let good = encode(TAG_PCM, 48_000, 16, &mono(&tone));
    assert_eq!(shipping(&book, "x.wav", &good).unwrap().len(), tone.len());
    // kx-ui's build refuses a wrong byte rate and a sound over the limit, so the join does too.
    let mut rate = good.clone();
    rate[28..32].copy_from_slice(&48_000u32.to_le_bytes());
    let long = encode(TAG_PCM, 48_000, 16, &mono(&sine(440.0, 48_000, 401, 0.5)));
    for wav in [rate, long] {
        let err = shipping(&book, "x.wav", &wav).unwrap_err();
        assert!(matches!(err, SoundError::NotShipping(_)), "got: {err}");
    }
    assert!(matches!(
        shipping(&book, "x.wav", b"junk"),
        Err(SoundError::Wav(_))
    ));
}
