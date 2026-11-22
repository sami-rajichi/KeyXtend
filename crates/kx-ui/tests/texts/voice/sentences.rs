//! The bar's and Compare's sentences as the owner approved them, word for word in each language: the bar's line for
//! the address the user types or the model on this PC, and Compare's send warning, whose names end with "and".

use crate::{items, lang};
use kx_ui::i18n::keys::vars::{NAME, NAMES};
use kx_ui::i18n::keys::{
    VOICE_CARD_CUSTOM, VOICE_ON_BAR_CLOUD, VOICE_ON_DEVICE, VOICE_SEND_WARNING,
};

/// The UI languages, in the order the tables give their words.
const TAGS: [&str; 3] = ["en", "fr", "ar"];

/// The bar's line while the address the user types takes the voice.
const BAR: [&str; 3] = [
    "Your voice goes to the address you type at Stop",
    "Votre voix partira vers l’adresse que vous saisissez à l’arrêt",
    "يُرسَل صوتك إلى العنوان الذي تكتبه عند الإيقاف",
];

/// Compare's warning for Groq and the typed address, then for Groq, `ElevenLabs` and the typed address; Arabic joins
/// و to the next word, as CLDR's "{0} و{1}" does.
const SEND: [[&str; 3]; 2] = [
    [
        "Compare sends your voice to Groq and the address you type at Stop.",
        "Comparer enverra votre voix à Groq et l’adresse que vous saisissez à l’arrêt.",
        "سترسل المقارنة صوتك إلى Groq والعنوان الذي تكتبه عند الإيقاف.",
    ],
    [
        "Compare sends your voice to Groq, ElevenLabs and the address you type at Stop.",
        "Comparer enverra votre voix à Groq, ElevenLabs et l’adresse que vous saisissez à l’arrêt.",
        "سترسل المقارنة صوتك إلى Groq وElevenLabs والعنوان الذي تكتبه عند الإيقاف.",
    ],
];

#[test]
fn the_bar_sends_the_voice_to_the_typed_address_in_the_approved_words() {
    for (tag, bar) in TAGS.iter().zip(BAR) {
        let t = lang(tag);
        let custom = t.text(VOICE_CARD_CUSTOM);
        assert_eq!(
            t.voice_with(VOICE_ON_BAR_CLOUD, &[(NAME, custom)]),
            bar,
            "{tag}"
        );
    }
}

/// The bar's line while Whisper Turbo on this PC writes (owner, 2026-11-22).
const DEVICE: [&str; 3] = [
    "Whisper Turbo writes what you say",
    "Whisper Turbo transcrit ce que vous dites",
    "يكتب Whisper Turbo ما تقوله",
];

#[test]
fn the_bar_names_the_model_on_this_pc_in_the_approved_words() {
    for (tag, bar) in TAGS.iter().zip(DEVICE) {
        let t = lang(tag);
        let name = "Whisper Turbo".to_owned();
        assert_eq!(t.voice_with(VOICE_ON_DEVICE, &[(NAME, name)]), bar, "{tag}");
    }
}

#[test]
fn compares_warning_ends_its_names_with_and_in_each_language() {
    for (i, tag) in TAGS.iter().enumerate() {
        let t = lang(tag);
        let custom = t.text(VOICE_CARD_CUSTOM);
        for (names, send) in [&["Groq"][..], &["Groq", "ElevenLabs"]].iter().zip(SEND) {
            let mut names = items(names);
            names.push(custom.clone());
            let line = t.voice_with(VOICE_SEND_WARNING, &[(NAMES, t.and_list(&names))]);
            assert_eq!(line, send[i], "{tag}");
        }
    }
}

#[test]
fn a_list_of_one_is_that_name_and_of_none_is_empty() {
    for tag in TAGS {
        let t = lang(tag);
        assert_eq!(t.and_list(&items(&["Groq"])), "Groq", "{tag}");
        assert_eq!(t.and_list(&[]), "", "{tag}");
    }
}
