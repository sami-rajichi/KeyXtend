//! The compiled-in table follows the sound tokens and holds each named file's samples, whole and aligned.

#![allow(
    clippy::float_cmp,
    reason = "the table holds the tokens' gains exactly as written"
)]

use super::*;

#[cfg(feature = "qt")]
mod qt {
    use std::path::Path;

    use kx_theme::assets::SOUND_DIR;
    use kx_theme::sounds::{Sounds, wav};

    use super::*;

    /// The shipped sound tokens.
    fn shipped() -> Sounds {
        Sounds::shipped().unwrap()
    }

    #[test]
    fn the_table_lists_the_four_events_in_order_with_their_files_in_turn() {
        let s = shipped();
        let names: Vec<&str> = table::EVENTS.iter().map(|(e, ..)| *e).collect();
        let events: Vec<&str> = Event::ALL.iter().map(|e| e.name()).collect();
        assert_eq!(names, events);
        for ((event, cue), (_, files, gain_db)) in s.cues.iter().zip(&table::EVENTS) {
            assert_eq!(*files, cue.files, "{}", event.name());
            assert_eq!(*gain_db, cue.gain_db, "{}", event.name());
        }
        assert_eq!(voices(), s.play.voices);
    }

    #[test]
    fn each_clip_holds_its_files_samples_whole_aligned_and_at_the_token_rate() {
        let s = shipped();
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets")
            .join(SOUND_DIR);
        let clips = clips();
        let names: Vec<&str> = table::CLIPS.iter().map(|(n, _)| *n).collect();
        assert_eq!(names, s.files());
        assert_eq!(clips.len(), names.len());
        for (i, (clip, name)) in clips.iter().zip(names).enumerate() {
            let bytes = std::fs::read(dir.join(name)).unwrap();
            let data = wav::samples(&bytes, &s.format).unwrap();
            assert_eq!(
                clip.samples.len(),
                data.len(),
                "{name}: the data chunk's length"
            );
            assert_eq!(clip.samples, data, "{name}");
            assert_eq!(usize::from(clip.id.raw()), i);
            assert_eq!(clip.rate, s.format.rate, "{name}");
            let whole = clip
                .samples
                .len()
                .is_multiple_of(usize::from(Clip::SAMPLE_BYTES));
            assert!(
                !clip.samples.is_empty() && whole,
                "{name}: an even byte count"
            );
            let at = clip.samples.as_ptr().addr();
            assert!(at.is_multiple_of(align_of::<u16>()), "{name}: aligned");
        }
    }

    #[test]
    fn each_event_cues_its_clips_in_turn_at_its_linear_gain() {
        for (event, want) in &shipped().cues {
            let cue = cue(*event).unwrap();
            let names: Vec<&str> = cue
                .clips
                .iter()
                .map(|id| table::CLIPS[usize::from(id.raw())].0)
                .collect();
            assert_eq!(names, want.files, "{}", event.name());
            assert_eq!(cue.gain, gain(want.gain_db), "{}", event.name());
        }
    }
}

#[test]
fn a_balance_in_db_becomes_a_linear_gain() {
    assert_eq!(gain(0.0), 1.0);
    assert!((gain(-20.0) - 0.1).abs() < 1e-6);
    assert!((gain(-6.0) - 0.501).abs() < 1e-3);
}

#[cfg(not(feature = "qt"))]
#[test]
fn without_qt_no_sound_is_compiled_in() {
    assert!(clips().is_empty());
    assert!(Event::ALL.iter().all(|e| cue(*e).is_none()));
    assert_eq!(voices(), 0);
}
