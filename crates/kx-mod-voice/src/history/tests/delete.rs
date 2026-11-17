//! Delete and Delete all (edge 32): Delete takes the ticked texts, pinned or not, with their recordings; Delete all
//! asks first, then spares pins and every text kept or pinned since the question.

use super::*;

#[test]
fn delete_all_asks_first_and_spares_pins_and_texts_kept_since() {
    let r = rules(50, MB);
    let mut history = filled(4, &r);
    let ids = ["texte 1", "texte 2", "texte 3", "texte 4"].map(|t| id(&history, t));
    history.pin(&[ids[0]]);
    history.attach(ids[1], sound(7), &r);
    let asked = history.ask_all().unwrap();
    assert_eq!(asked.count(), 3);
    history.pin(&[ids[3]]);
    history.keep(kept(5, "texte 5"), at(5), &r);
    let edit = history.delete_all(asked);
    assert_eq!(texts(&history), ["texte 4", "texte 1", "texte 5"]);
    let gone = BTreeMap::from([(ids[1], Some(sound(7))), (ids[2], None)]);
    assert_eq!(edit.gone, gone);
    assert!(edit.put.is_empty());
}

#[test]
fn delete_all_asks_nothing_when_every_text_is_pinned() {
    let r = rules(50, MB);
    assert!(History::default().ask_all().is_none());
    let mut history = filled(2, &r);
    let ids: Vec<u64> = history.entries().iter().map(|e| e.id).collect();
    history.pin(&ids);
    assert!(history.ask_all().is_none());
}

#[test]
fn delete_takes_the_ticked_texts_pinned_or_not_with_their_recordings() {
    let r = rules(50, MB);
    let mut history = filled(3, &r);
    let ids = ["texte 1", "texte 2", "texte 3"].map(|t| id(&history, t));
    history.pin(&[ids[0]]);
    history.attach(ids[0], sound(4), &r);
    let edit = history.delete(&[ids[0], ids[2]]);
    assert_eq!(texts(&history), ["texte 2"]);
    let gone = BTreeMap::from([(ids[0], Some(sound(4))), (ids[2], None)]);
    assert_eq!(edit.gone, gone);
}
