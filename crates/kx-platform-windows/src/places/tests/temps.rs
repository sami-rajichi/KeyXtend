//! Temp files: each write fills its own, so two writers in one folder never break the box, and a write sweeps the old
//! leftovers of another process's crash.

use super::*;
use std::sync::Arc;
use std::thread;
use tree::{boxed, names, plant};

/// How many times each writer writes.
const ROUNDS: usize = 40;
/// Each writer's box length, long enough for two writes to overlap.
const SIZE: usize = 256 * 1024;

#[test]
fn two_writers_in_one_folder_always_leave_one_writers_whole_box() {
    let tree = Tree::new("places-race");
    let folder = tree.make("OneDrive");
    let places = Arc::new(tree.places());
    let boxes = [boxed(&vec![b'a'; SIZE]), boxed(&vec![b'b'; SIZE])];
    let writer = |mine: usize| {
        let (places, folder, boxes) = (places.clone(), folder.clone(), boxes.clone());
        thread::spawn(move || {
            for _ in 0..ROUNDS {
                assert_eq!(places.write(&folder, &boxes[mine]), Ok(()));
                let now = fs::read(folder.join(box_file())).unwrap();
                assert!(boxes.contains(&now), "a broken box of {} bytes", now.len());
            }
        })
    };
    let writers = [writer(0), writer(1)];
    for done in writers {
        done.join().unwrap();
    }
    assert_eq!(names(&folder), [box_file()], "no temp file left");
}

#[test]
fn a_write_sweeps_another_processs_old_temp_and_keeps_fresh_ones_its_own_and_strangers() {
    let tree = Tree::new("places-sweep");
    let folder = tree.make("OneDrive");
    let (t, me) = (shipped(), std::process::id());
    let other = me.wrapping_add(1);
    let stale = folder.join(t.temp_name(other, 0));
    let fresh = folder.join(t.temp_name(other, 1));
    let mine = folder.join(t.temp_name(me, u64::MAX));
    let stranger = folder.join("~notes.tmp");
    // A second after the Unix epoch is older than any limit; `fresh` is written now.
    for old in [&stale, &mine, &stranger] {
        plant(old, b"half a box", 1);
    }
    fs::write(&fresh, b"half a box").unwrap();
    assert_eq!(tree.places().write(&folder, &boxed(b"box")), Ok(()));
    assert!(!stale.exists(), "a crash's leftover goes");
    for kept in [&fresh, &mine, &stranger] {
        assert!(kept.exists(), "{}", kept.display());
    }
}
