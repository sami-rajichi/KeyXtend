//! The voice gate build's log names, set once for the build that writes them and kx-gates that reads them back.

use crate::gate::{
    DEVICE, FAULT, INDEX, LOAD, OPEN, PIECE, SEND, SERVICE, SOUND_MS, STAGE, START, STOP, TOOK_MS,
};

#[test]
fn the_field_and_stage_names_are_distinct_bare_words() {
    let bare = |w: &str| !w.is_empty() && w.bytes().all(|b| b.is_ascii_lowercase() || b == b'_');
    let fields = [STAGE, INDEX, SOUND_MS, TOOK_MS, DEVICE, SERVICE, FAULT];
    let stages = [START, OPEN, STOP, LOAD, PIECE, SEND];
    for names in [&fields[..], &stages[..]] {
        for (i, name) in names.iter().enumerate() {
            assert!(bare(name), "{name}");
            assert!(!names[..i].contains(name), "{name} twice");
        }
    }
}
