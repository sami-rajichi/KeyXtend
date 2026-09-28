//! The worker's JSON lines: commands in on stdin, events out on stdout. Transcripts are never shown in `Debug`.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::voicecfg::Engine;

/// What the face or the bench asks the worker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "lowercase")]
pub enum Cmd {
    /// Start recording speech in `lang`.
    Record {
        /// Language code, or `auto`.
        lang: String,
    },
    /// Stop recording and turn the clip into text.
    Stop,
    /// Turn a saved clip into text with `engine` (bench only).
    File {
        /// The WAV file.
        path: String,
        /// Language code, or `auto`.
        lang: String,
        /// Which engine.
        engine: Engine,
    },
    /// End the worker and its server.
    Quit,
}

/// What the worker reports.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "lowercase")]
pub enum Event {
    /// The local model is loaded, `load_ms` after start.
    Ready {
        /// Start-up time in ms.
        load_ms: u64,
    },
    /// Recording started.
    Listening,
    /// Recording stopped; the clip is `audio_ms` long.
    Transcribing {
        /// Clip length in ms.
        audio_ms: u64,
    },
    /// The words from `engine`, `ms` after the clip was sent.
    Text {
        /// The transcript: Sensitive, never logged.
        text: String,
        /// Which engine answered.
        engine: Engine,
        /// Time from sending the clip to the answer, in ms.
        ms: u64,
        /// Clip length in ms.
        audio_ms: u64,
    },
    /// A problem that does not end the recording cycle, such as a clip that could not be kept.
    Note {
        /// Short reason.
        note: String,
    },
    /// Something failed and the cycle ended; the `error` says what.
    Error {
        /// Short reason.
        error: String,
    },
}

impl std::fmt::Debug for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Text {
                text,
                engine,
                ms,
                audio_ms,
            } => write!(
                f,
                "Text {{ {} chars, {engine:?}, {ms} ms, {audio_ms} ms audio }}",
                text.chars().count()
            ),
            Event::Ready { load_ms } => write!(f, "Ready {{ {load_ms} ms }}"),
            Event::Listening => write!(f, "Listening"),
            Event::Transcribing { audio_ms } => write!(f, "Transcribing {{ {audio_ms} ms }}"),
            Event::Note { note } => write!(f, "Note {{ {note} }}"),
            Event::Error { error } => write!(f, "Error {{ {error} }}"),
        }
    }
}

/// `v` as one JSON line, without the line break.
pub fn to_line<T: Serialize>(v: &T) -> Result<String, String> {
    serde_json::to_string(v).map_err(|e| e.to_string())
}

/// Extension of kept clips.
const CLIP_EXT: &str = ".wav";
/// Ends the language in a clip name, `<lang>-<ms>.wav`.
const LANG_END: char = '-';
/// Longest language code, such as `auto`.
const MAX_LANG: usize = 8;

/// True for a language code: 1 to 8 ASCII letters, so it is safe inside a file name.
pub fn valid_lang(lang: &str) -> bool {
    (1..=MAX_LANG).contains(&lang.len()) && lang.bytes().all(|b| b.is_ascii_alphabetic())
}

/// The file name the worker keeps a clip in `lang` under, made at `ms`.
pub fn clip_name(lang: &str, ms: u128) -> String {
    format!("{lang}{LANG_END}{ms}{CLIP_EXT}")
}

/// The language of a kept clip, from its file name.
pub fn clip_lang(name: &str) -> Option<&str> {
    let lang = name.strip_suffix(CLIP_EXT)?.split_once(LANG_END)?.0;
    valid_lang(lang).then_some(lang)
}

/// Byte-order mark some writers put before the first line.
const BOM: char = '\u{FEFF}';

/// One JSON line back into a command or event.
pub fn parse<T: DeserializeOwned>(line: &str) -> Result<T, String> {
    serde_json::from_str(line.trim_start_matches(BOM).trim()).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_round_trip_as_one_json_line() {
        let all = [
            Cmd::Record { lang: "ar".into() },
            Cmd::Stop,
            Cmd::File {
                path: "c.wav".into(),
                lang: "fr".into(),
                engine: Engine::Cloud,
            },
            Cmd::Quit,
        ];
        for c in all {
            let line = to_line(&c).expect("encodes");
            assert!(!line.contains('\n'));
            assert_eq!(parse::<Cmd>(&line), Ok(c));
        }
    }

    #[test]
    fn events_round_trip_and_hide_the_text_in_debug() {
        let e = Event::Text {
            text: "secret words".into(),
            engine: Engine::Local,
            ms: 5,
            audio_ms: 900,
        };
        let line = to_line(&e).expect("encodes");
        assert_eq!(parse::<Event>(&line), Ok(e.clone()));
        let shown = format!("{e:?}");
        assert!(!shown.contains("secret"), "{shown}");
        assert!(shown.contains("12 chars"), "{shown}");
    }

    #[test]
    fn a_bad_line_is_an_error_not_a_panic() {
        assert!(parse::<Cmd>("{\"cmd\":\"fly\"}").is_err());
        assert!(parse::<Event>("not json").is_err());
    }

    #[test]
    fn clip_names_carry_their_language() {
        assert_eq!(clip_name("ar", 17), "ar-17.wav");
        assert_eq!(clip_lang("ar-17.wav"), Some("ar"));
        assert_eq!(clip_lang("ar-17.txt"), None);
        assert_eq!(clip_lang("nolang.wav"), None);
    }

    #[test]
    fn only_short_letter_codes_are_languages() {
        assert!(valid_lang("ar") && valid_lang("auto"));
        for bad in ["..\\x", "C:", "", "a/b", "abcdefghi"] {
            assert!(!valid_lang(bad), "{bad}");
        }
    }

    #[test]
    fn a_note_round_trips() {
        let n = Event::Note {
            note: "clip not kept".into(),
        };
        assert_eq!(parse::<Event>(&to_line(&n).expect("encodes")), Ok(n));
    }

    #[test]
    fn a_leading_byte_order_mark_is_skipped() {
        assert_eq!(parse::<Cmd>("\u{FEFF}{\"cmd\":\"stop\"}"), Ok(Cmd::Stop));
    }
}
