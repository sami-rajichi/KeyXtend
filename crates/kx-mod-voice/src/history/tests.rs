//! History (plan task 18): a later text of the same take replaces it, pins sit outside `history_max`, recordings
//! share `voice_max_mb` oldest unpinned first, history off or a hidden run keeps nothing, and Delete all asks first
//! (edge 32).

use std::collections::BTreeMap;

use super::*;
use crate::VoiceSettings;
use kx_module_api::Zeroizing;
use kx_module_api::ports::MB;

mod delete;
mod flow;
mod limits;
mod props;

fn secret(text: &str) -> Secret {
    Redacted::new(Zeroizing::new(text.to_owned()))
}

/// A text of `take` saying `text`, in French by Parakeet.
fn kept(take: u32, text: &str) -> Kept {
    Kept {
        take,
        text: secret(text),
        lang: "fr".into(),
        model: Redacted::new("parakeet-v3-q8".into()),
    }
}

/// Rules keeping `max` unpinned texts and `voice` bytes of recordings.
fn rules(max: usize, voice: u64) -> Rules {
    Rules {
        texts: true,
        max,
        voice: true,
        voice_bytes: voice,
    }
}

/// When the `n`th text was dictated.
fn at(n: u32) -> Stamp {
    Stamp {
        unix_ms: i64::from(n) * 1_000,
        offset_min: 60,
    }
}

/// A recording of `bytes` in one blob.
fn sound(bytes: u64) -> Sound {
    Sound { bytes, blobs: 1 }
}

/// A history holding `texte 1` to `texte n`, each its own take.
fn filled(n: u32, rules: &Rules) -> History {
    let mut history = History::default();
    for take in 1..=n {
        history.keep(kept(take, &format!("texte {take}")), at(take), rules);
    }
    history
}

/// The texts in order.
fn texts(history: &History) -> Vec<String> {
    let text = |e: &Entry| e.text.expose().to_string();
    history.entries().iter().map(text).collect()
}

/// The id of the entry saying `text`.
fn id(history: &History, text: &str) -> u64 {
    let entry = history
        .entries()
        .iter()
        .find(|e| e.text.expose().as_str() == text);
    entry.unwrap().id
}

#[test]
fn a_text_is_kept_newest_first_with_its_time_language_and_model() {
    let r = rules(50, MB);
    let mut history = History::default();
    let edit = history.keep(kept(1, "bonjour"), at(1), &r);
    history.keep(kept(2, "merci"), at(2), &r);
    assert_eq!(texts(&history), ["merci", "bonjour"]);
    let first = &history.entries()[1];
    assert_eq!((first.at, first.lang.as_str()), (at(1), "fr"));
    assert_eq!(first.model.expose(), "parakeet-v3-q8");
    assert!(!first.pinned && first.sound.is_none());
    assert_eq!(edit.put, BTreeSet::from([first.id]));
    assert!(edit.gone.is_empty() && edit.mute.is_empty());
}

#[test]
fn a_later_text_of_the_same_take_replaces_it_in_place() {
    let r = rules(50, MB);
    let mut history = filled(1, &r);
    let first = history.entries()[0].clone();
    let edit = history.keep(kept(1, "bonsoir"), at(9), &r);
    assert_eq!(texts(&history), ["bonsoir"]);
    let now = &history.entries()[0];
    assert_eq!((now.id, now.seq, now.at), (first.id, first.seq, first.at));
    assert_eq!(edit.put, BTreeSet::from([first.id]));
    let other = history.keep(kept(2, "merci"), at(10), &r);
    assert_eq!(texts(&history), ["merci", "bonsoir"]);
    assert!(!other.put.contains(&first.id));
}

#[test]
fn a_take_whose_text_was_deleted_comes_back_as_a_new_text() {
    let r = rules(50, MB);
    let mut history = filled(1, &r);
    let gone = history.entries()[0].id;
    history.delete(&[gone]);
    history.keep(kept(1, "bonsoir"), at(2), &r);
    assert_eq!(texts(&history), ["bonsoir"]);
    assert_ne!(history.entries()[0].id, gone);
}

#[test]
fn history_off_or_a_hidden_run_keeps_nothing() {
    let mut settings = VoiceSettings::from_defaults().unwrap();
    settings.keep_voice = true;
    let hidden = Rules::of(&settings, true);
    settings.history = false;
    let off = Rules::of(&settings, false);
    for r in [hidden, off] {
        assert!(!r.texts && !r.voice);
        let mut history = History::default();
        assert!(history.keep(kept(1, "bonjour"), at(1), &r).is_empty());
        assert!(history.is_empty());
        assert_eq!(history.admit(1, 10, &r), None);
    }
}

#[test]
fn the_rules_follow_the_settings() {
    let mut settings = VoiceSettings::from_defaults().unwrap();
    let r = Rules::of(&settings, false);
    assert_eq!((r.texts, r.voice), (settings.history, settings.keep_voice));
    assert_eq!(r.max, usize::from(settings.history_max));
    assert_eq!(r.voice_bytes, u64::from(settings.voice_max_mb) * MB);
    settings.keep_voice = true;
    assert!(Rules::of(&settings, false).voice);
}

#[test]
fn at_history_max_the_oldest_unpinned_text_goes_and_pins_sit_outside() {
    let r = rules(3, MB);
    let mut history = filled(1, &r);
    let pinned = id(&history, "texte 1");
    history.pin(&[pinned]);
    for take in 2..=4 {
        history.keep(kept(take, &format!("texte {take}")), at(take), &r);
    }
    assert_eq!((history.len(), history.unpinned()), (4, 3));
    let oldest = id(&history, "texte 2");
    let edit = history.keep(kept(5, "texte 5"), at(5), &r);
    assert_eq!(
        texts(&history),
        ["texte 1", "texte 5", "texte 4", "texte 3"]
    );
    assert_eq!(edit.gone, BTreeMap::from([(oldest, None)]));
    assert!(!edit.put.contains(&oldest));
}

#[test]
fn voice_max_mb_drops_the_oldest_unpinned_recording_and_keeps_its_text() {
    let r = rules(50, 30);
    let mut history = filled(4, &r);
    let ids = ["texte 1", "texte 2", "texte 3", "texte 4"].map(|t| id(&history, t));
    history.attach(ids[0], sound(10), &r);
    history.pin(&[ids[0]]);
    history.attach(ids[1], sound(5), &r);
    history.attach(ids[2], sound(5), &r);
    let edit = history.attach(ids[3], sound(15), &r);
    assert_eq!(edit.mute, BTreeMap::from([(ids[1], sound(5))]));
    assert_eq!(edit.put, BTreeSet::from([ids[1], ids[3]]));
    assert_eq!(history.get(ids[1]).unwrap().sound, None);
    assert_eq!(history.len(), 4);
    assert_eq!(history.voice_bytes(), 30);
}

#[test]
fn a_recording_joins_the_last_takes_text_once_within_the_room_pins_leave() {
    let r = rules(50, 30);
    let mut history = filled(2, &r);
    let (one, two) = (id(&history, "texte 1"), id(&history, "texte 2"));
    assert_eq!(history.admit(1, 5, &r), None, "an older take");
    assert_eq!(history.admit(2, 5, &r), Some(two));
    assert_eq!(history.admit(2, 0, &r), None, "an empty recording");
    assert_eq!(history.admit(2, 31, &r), None, "past voice_max_mb");
    history.attach(one, sound(20), &r);
    history.pin(&[one]);
    assert_eq!(
        history.admit(2, 11, &r),
        None,
        "past the room the pin leaves"
    );
    assert_eq!(history.admit(2, 10, &r), Some(two));
    history.attach(two, sound(10), &r);
    assert_eq!(history.admit(2, 1, &r), None, "a text keeps one recording");
}

#[test]
fn a_recording_whose_text_is_gone_has_its_blobs_removed() {
    let r = rules(50, MB);
    let mut history = filled(1, &r);
    let gone = history.entries()[0].id;
    history.delete(&[gone]);
    let edit = history.attach(gone, sound(9), &r);
    assert_eq!(edit.mute, BTreeMap::from([(gone, sound(9))]));
    assert!(edit.put.is_empty() && history.is_empty());
}

#[test]
fn unpin_counts_as_new_so_the_limit_drops_it_last() {
    let r = rules(2, MB);
    let mut history = filled(2, &r);
    let one = id(&history, "texte 1");
    assert_eq!(history.pin(&[one]).put, BTreeSet::from([one]));
    assert!(history.pin(&[one]).is_empty(), "already pinned");
    history.keep(kept(3, "texte 3"), at(3), &r);
    assert_eq!(texts(&history), ["texte 1", "texte 3", "texte 2"]);
    let edit = history.unpin(&[one], at(9));
    assert_eq!(edit.put, BTreeSet::from([one]));
    assert!(edit.gone.is_empty(), "unpin never deletes");
    assert!(history.unpin(&[one], at(10)).is_empty(), "already unpinned");
    assert_eq!(texts(&history), ["texte 1", "texte 3", "texte 2"]);
    assert_eq!(history.get(one).unwrap().at, at(9), "the time of the unpin");
    history.keep(kept(4, "texte 4"), at(10), &r);
    assert_eq!(texts(&history), ["texte 4", "texte 1"]);
}

#[test]
fn a_take_keeps_one_recording_even_after_losing_it() {
    let r = rules(50, 10);
    let mut history = filled(1, &r);
    let one = id(&history, "texte 1");
    assert_eq!(history.admit(1, 8, &r), Some(one));
    history.attach(one, sound(8), &r);
    let edit = history.trim(&rules(50, 4));
    assert_eq!(edit.mute, BTreeMap::from([(one, sound(8))]));
    assert_eq!(
        history.admit(1, 2, &r),
        None,
        "its blob names may still be removed"
    );
    history.keep(kept(1, "texte un"), at(2), &r);
    assert_eq!(history.admit(1, 2, &r), None, "Try again keeps the take");
    history.keep(kept(2, "texte 2"), at(3), &r);
    assert!(history.admit(2, 2, &r).is_some(), "a new take may");
}

#[test]
fn lower_limits_trim_at_once() {
    let wide = rules(50, MB);
    let mut history = filled(4, &wide);
    let ids = ["texte 1", "texte 2", "texte 3", "texte 4"].map(|t| id(&history, t));
    history.attach(ids[2], sound(6), &wide);
    history.attach(ids[3], sound(6), &wide);
    let edit = history.trim(&rules(2, 10));
    assert_eq!(texts(&history), ["texte 4", "texte 3"]);
    assert_eq!(
        edit.gone.keys().copied().collect::<Vec<_>>(),
        [ids[0], ids[1]]
    );
    assert_eq!(edit.mute, BTreeMap::from([(ids[2], sound(6))]));
    assert!(history.trim(&rules(2, 10)).is_empty());
}

#[test]
fn a_loaded_history_orders_pins_first_and_numbers_past_every_id() {
    let r = rules(50, MB);
    let mut history = filled(3, &r);
    let one = id(&history, "texte 1");
    history.pin(&[one]);
    let mut entries = history.entries().to_vec();
    entries.reverse();
    let back = History::new(entries, 0);
    assert_eq!(back.entries(), history.entries());
    assert!(back.next() >= history.next());
    let mut again = back;
    again.keep(kept(1, "neuf"), at(4), &r);
    assert_eq!(again.len(), 4, "a loaded history knows no take of this run");
}

#[test]
fn an_edit_merges_later_changes_and_a_gone_text_is_not_written() {
    let mut first = Edit::default();
    first.put.extend([1, 2]);
    first.gone.insert(3, Some(sound(5)));
    let mut later = Edit::default();
    later.gone.extend([(2, None), (3, None)]);
    later.mute.insert(4, sound(1));
    first.merge(later);
    assert_eq!(first.put, BTreeSet::from([1]));
    assert_eq!(first.gone, BTreeMap::from([(2, None), (3, Some(sound(5)))]));
    assert_eq!(first.mute, BTreeMap::from([(4, sound(1))]));
}
