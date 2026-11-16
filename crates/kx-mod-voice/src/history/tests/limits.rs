//! The recordings' limit at a later text of the same take: the text stays, and its own recording goes last, only when
//! it alone is past `voice_max_mb`.

use super::*;

/// History with take 1's text and a recording of `bytes`, unpinned again after the limit fell to `voice`.
fn unpinned_past(bytes: u64, voice: u64) -> (History, u64, Rules) {
    let mut r = rules(3, 30);
    let mut history = filled(1, &r);
    let id = history.admit(1, bytes, &r).unwrap();
    history.attach(id, sound(bytes), &r);
    history.pin(&[id]);
    r.voice_bytes = voice;
    history.trim(&r);
    history.unpin(&[id], at(2));
    (history, id, r)
}

#[test]
fn a_later_text_drops_its_own_recording_when_it_alone_is_past_the_limit() {
    let (mut history, id, r) = unpinned_past(11, 0);
    let edit = history.keep(kept(1, "texte 1 bis"), at(3), &r);
    assert!(history.get(id).is_some(), "the text stays");
    assert!(edit.mute.contains_key(&id), "its recording goes");
    assert_eq!(history.voice_bytes(), 0);
}

#[test]
fn a_later_text_keeps_its_own_recording_while_it_fits() {
    let (mut history, id, r) = unpinned_past(11, 20);
    let edit = history.keep(kept(1, "texte 1 bis"), at(3), &r);
    assert!(history.get(id).is_some(), "the text stays");
    assert!(edit.mute.is_empty(), "its recording stays");
    assert_eq!(history.voice_bytes(), 11);
}

#[test]
fn a_later_text_drops_older_recordings_before_its_own() {
    let mut r = rules(3, 30);
    let mut history = filled(1, &r);
    let old = history.admit(1, 11, &r).unwrap();
    history.attach(old, sound(11), &r);
    history.keep(kept(2, "texte 2"), at(2), &r);
    let own = history.admit(2, 15, &r).unwrap();
    history.attach(own, sound(15), &r);
    history.pin(&[old, own]);
    r.voice_bytes = 20;
    history.trim(&r);
    history.unpin(&[own], at(3));
    history.unpin(&[old], at(4));
    let edit = history.keep(kept(2, "texte 2 bis"), at(5), &r);
    assert_eq!(edit.mute.keys().copied().collect::<Vec<_>>(), [old]);
    assert_eq!(history.voice_bytes(), 15, "its own recording stays");
}
