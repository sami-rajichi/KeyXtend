//! The card's side of a run: the fit check (edge case 24), a card that fails to load or mid-run (25), one that serves
//! to the end, and one that is gone.

use super::*;

#[test]
fn a_card_short_of_free_memory_is_refused_before_loading_and_the_processor_serves() {
    let (mut engine, lib, tell, rx) = rig(500);
    let notes = write(
        &mut engine,
        &tell,
        &rx,
        2,
        &model(card(1000), "en", 0),
        &[10],
    );
    assert_eq!(
        notes[..3],
        [
            fault(2, VoiceFault::CardRoom),
            loading(2),
            piece(2, 0, "10 en")
        ]
    );
    assert!(!done(&notes[3]).3);
    assert_eq!(lib.script().loads, [(Place::Cpu, 8)]);
}

#[test]
fn a_card_error_mid_run_hands_the_same_sound_to_the_processor() {
    let (mut engine, lib, tell, rx) = rig(4000);
    lib.script().card_runs = Some(1);
    let notes = write(
        &mut engine,
        &tell,
        &rx,
        3,
        &model(card(1000), "en", 0),
        &[100, 200],
    );
    let want = [
        loading(3),
        piece(3, 0, "100 en"),
        fault(3, VoiceFault::CardFailed),
        piece(3, 1, "200 en"),
    ];
    assert_eq!(notes[..4], want);
    assert_eq!((done(&notes[4]).0, done(&notes[4]).3), (2, false));
    let s = lib.script();
    let card = Place::Card("Vulkan0".into());
    assert_eq!(s.loads, [(card.clone(), 8), (Place::Cpu, 8)]);
    let runs: Vec<(Place, usize)> = s.runs.iter().map(|r| (r.0.clone(), r.1)).collect();
    assert_eq!(runs, [(card.clone(), 100), (card, 200), (Place::Cpu, 200)]);
}

#[test]
fn a_card_part_that_fails_to_load_leaves_the_processor_working() {
    let (mut engine, lib, tell, rx) = rig(4000);
    lib.script().card_load = Some(Failed::Device);
    let notes = write(
        &mut engine,
        &tell,
        &rx,
        4,
        &model(card(1000), "en", 0),
        &[10],
    );
    assert_eq!(
        notes[..3],
        [
            loading(4),
            fault(4, VoiceFault::CardFailed),
            piece(4, 0, "10 en")
        ]
    );
    assert!(!done(&notes[3]).3);
}

#[test]
fn a_card_that_serves_to_the_end_says_so_and_a_gone_one_is_no_card() {
    let (mut engine, _, tell, rx) = rig(4000);
    let notes = write(
        &mut engine,
        &tell,
        &rx,
        5,
        &model(card(1000), "en", 0),
        &[10],
    );
    assert!(done(&notes[2]).3);
    let (mut engine, lib, tell, rx) = rig(4000);
    lib.script().found.truncate(1);
    let notes = write(
        &mut engine,
        &tell,
        &rx,
        6,
        &model(card(1000), "en", 0),
        &[10],
    );
    assert_eq!(notes[0], fault(6, VoiceFault::NoCard));
    assert!(!done(&notes[3]).3);
}
