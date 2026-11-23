//! Voice typing's numbers in each language: file sizes to three digits, memory, rooms, seconds and factors with each
//! language's marks and no trailing zeros, and Arabic's counts and factors in every number and case.

use crate::lang;
use kx_module_api::ports::Hundredths;
use kx_ui::i18n::keys::{
    VOICE_CORES, VOICE_FASTER, VOICE_MS, VOICE_SECONDS, VOICE_TEXTS, VOICE_TIMES,
};

#[test]
fn file_sizes_show_three_significant_digits_with_the_languages_mark() {
    let (en, fr, ar) = (lang("en"), lang("fr"), lang("ar"));
    assert_eq!(en.voice_size(739_508_576), "740 MB");
    assert_eq!(en.voice_size(1_161_143_008), "1.16 GB");
    assert_eq!(en.voice_size(2_410_655_232), "2.41 GB");
    assert_eq!(en.voice_size(21_300_000_000), "21.3 GB");
    assert_eq!(en.voice_size(212_000_000_000), "212 GB");
    assert_eq!(
        en.voice_size(999_600_000),
        "1 GB",
        "rounds up past a gigabyte"
    );
    assert_eq!(fr.voice_size(1_161_143_008), "1,16\u{a0}Go");
    assert_eq!(ar.voice_size(1_161_143_008), "1,16 غيغابايت");
    assert_eq!(ar.voice_size(193_749_056), "194 ميغابايت");
}

#[test]
fn memory_rooms_seconds_and_factors_drop_trailing_zeros() {
    let (en, fr, ar) = (lang("en"), lang("fr"), lang("ar"));
    assert_eq!(en.voice_memory(16_384), "16 GB");
    assert_eq!(en.voice_memory(9_626), "9.4 GB");
    assert_eq!(en.voice_memory(512), "0.5 GB");
    assert_eq!(en.voice_room(512), "512 MB");
    assert_eq!(en.voice_room(1_024), "1 GB");
    assert_eq!(en.voice_room(65_536), "64 GB");
    assert_eq!(en.voice_secs(2_100), "2.1 s");
    assert_eq!(
        en.voice_secs(4_000),
        "4.0 s",
        "a wait keeps its decimal, as the mock-up writes it"
    );
    assert_eq!(fr.voice_secs(2_100), "2,1\u{a0}s");
    assert_eq!(
        ar.voice_secs(4_000),
        "4,0 ثانية",
        "a decimal takes the singular"
    );
    assert_eq!(en.voice_times(VOICE_TIMES, Hundredths(150)), "1.5×");
    assert_eq!(en.voice_times(VOICE_TIMES, Hundredths(125)), "1.25×");
    assert_eq!(
        fr.voice_times(VOICE_FASTER, Hundredths(900)),
        "9× plus rapide"
    );
}

#[test]
fn arabic_factors_agree_in_number_and_case() {
    let ar = lang("ar");
    let times = |h: u32| ar.voice_times(VOICE_TIMES, Hundredths(h));
    let faster = |h: u32| ar.voice_times(VOICE_FASTER, Hundredths(h));
    assert_eq!(times(100), "مرة واحدة");
    assert_eq!(times(200), "مرتان", "a value stands as a subject");
    assert_eq!(times(300), "3 مرات");
    assert_eq!(times(1000), "10 مرات");
    assert_eq!(times(150), "1,5 مرة");
    assert_eq!(
        faster(200),
        "أسرع مرتين",
        "an adverb of times is accusative"
    );
    assert_eq!(faster(900), "أسرع 9 مرات");
    assert_eq!(faster(942), "أسرع 9,42 مرة");
}

#[test]
fn arabic_counts_texts_cores_and_seconds_in_every_form() {
    let ar = lang("ar");
    let cases = [
        (VOICE_TEXTS, 0, "لا نصوص"),
        (VOICE_TEXTS, 1, "نص واحد"),
        (VOICE_TEXTS, 2, "نصان"),
        (VOICE_TEXTS, 3, "3 نصوص"),
        (VOICE_TEXTS, 11, "11 نصًا"),
        (VOICE_TEXTS, 100, "100 نص"),
        (VOICE_TEXTS, 103, "103 نصوص"),
        (VOICE_CORES, 2, "نواتان"),
        (VOICE_CORES, 8, "8 أنوية"),
        (VOICE_CORES, 16, "16 نواة"),
        (VOICE_SECONDS, 1, "ثانية واحدة"),
        (VOICE_SECONDS, 9, "9 ثوانٍ"),
        (VOICE_SECONDS, 30, "30 ثانية"),
        (VOICE_MS, 5, "5 ميلي ثوانٍ"),
        (VOICE_MS, 20, "20 ميلي ثانية"),
        (VOICE_MS, 400, "400 ميلي ثانية"),
    ];
    for (key, n, want) in cases {
        assert_eq!(ar.counted(key, n), want, "{key} {n}");
    }
    let (en, fr) = (lang("en"), lang("fr"));
    assert_eq!(en.counted(VOICE_TEXTS, 1), "1 text");
    assert_eq!(en.counted(VOICE_CORES, 8), "8 cores");
    assert_eq!(fr.counted(VOICE_TEXTS, 0), "0 texte");
    assert_eq!(fr.counted(VOICE_CORES, 8), "8\u{a0}cœurs");
}
