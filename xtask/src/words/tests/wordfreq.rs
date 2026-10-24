//! The wordfreq decoder and the frequency byte.

use proptest::prelude::*;
use rmpv::Value;

use super::super::fixture;
use super::super::wordfreq::{Entry, decode, freq, scaled, share};

fn entry(word: &str, bucket: u32) -> Entry {
    Entry {
        word: word.to_string(),
        bucket,
    }
}

fn header(format: &str, version: u64) -> Value {
    Value::Map(vec![
        (Value::from("format"), Value::from(format)),
        (Value::from("version"), Value::from(version)),
    ])
}

#[test]
fn reads_a_hand_built_two_bucket_list() {
    let bytes = fixture::list(&[&["the", "of"], &["été"]]);
    let expected = [entry("the", 0), entry("of", 0), entry("été", 1)];
    assert_eq!(decode(&bytes[..]), Ok(expected.to_vec()));
}

#[test]
fn refuses_a_wrong_header() {
    let headers = [
        header("xx", 1),
        header("cB", 2),
        Value::from("cB"),
        Value::Nil,
    ];
    for bad in headers {
        let bytes = fixture::list_with(bad.clone(), &[&["the"]]);
        let err = decode(&bytes[..]).unwrap_err();
        assert!(err.contains("header"), "{bad:?}: {err}");
    }
}

#[test]
fn refuses_what_is_not_a_gzipped_list_of_word_lists() {
    assert!(decode(&b"not gzip"[..]).is_err());
    let mut raw = Vec::new();
    let all = Value::Array(vec![fixture::header(), Value::from(7)]);
    rmpv::encode::write_value(&mut raw, &all).unwrap();
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    std::io::Write::write_all(&mut gz, &raw).unwrap();
    let err = decode(&gz.finish().unwrap()[..]).unwrap_err();
    assert!(err.contains("bucket 0"), "{err}");
}

#[test]
fn the_frequency_byte_is_the_zipf_value_times_the_packs_steps() {
    assert_eq!(freq(0), 225, "a frequency of 1 fits a byte");
    assert_eq!(freq(100), 200);
    assert_eq!(freq(104), 199);
    assert_eq!(freq(900), 0);
    assert_eq!(freq(1000), 0);
    assert_eq!(scaled(0, 30), u8::MAX, "a value over a byte is clamped");
}

#[test]
fn a_bucket_gives_its_share_of_all_words() {
    assert!((share(0) - 1.0).abs() < 1e-12);
    assert!((share(200) - 0.01).abs() < 1e-12);
    assert!((share(150) - 10f64.powf(-1.5)).abs() < 1e-12);
}

proptest! {
    #[test]
    fn any_list_round_trips(buckets in prop::collection::vec(prop::collection::vec("[a-zé']{1,8}", 0..4), 0..6)) {
        let refs: Vec<Vec<&str>> = buckets.iter().map(|b| b.iter().map(String::as_str).collect()).collect();
        let slices: Vec<&[&str]> = refs.iter().map(Vec::as_slice).collect();
        let expected: Vec<Entry> = (0_u32..)
            .zip(&buckets)
            .flat_map(|(i, words)| words.iter().map(move |w| entry(w, i)))
            .collect();
        prop_assert_eq!(decode(&fixture::list(&slices)[..]), Ok(expected));
    }

    #[test]
    fn a_rarer_bucket_never_has_a_higher_byte(a in 0_u32..2000, b in 0_u32..2000, steps in 1_u32..40) {
        let (rare, common) = (a.max(b), a.min(b));
        prop_assert!(scaled(rare, steps) <= scaled(common, steps));
    }
}
