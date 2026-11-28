//! The editor's flattened picture is cut to the kept part from its top left wherever the grab holds more; a grab short
//! of that part on a side keeps its own size there and warns.

use super::{Heard, heard, page, rect, shot};
use crate::Rig;
use crate::logs::warnings_of;
use kx_module_api::ports::{BGRX_BYTES, Crop, Shot, SnipClick, SnipStage};

/// The bytes of a `w` by `h` grab, each one its own place.
fn grab(w: u32, h: u32) -> Shot {
    let len = u8::try_from(usize::try_from(w * h).unwrap() * BGRX_BYTES).unwrap();
    Shot::new(w, h, (0..len).collect()).unwrap()
}

/// An editor of round 8 keeping a 3 by 2 part, and what the hub sends from it.
fn kept_3_by_2(rig: &Rig) -> Heard {
    let heard = heard(rig);
    let mut editor = super::drawn::drawn();
    editor.body.crop = Crop::new(rect(5, 4, 3, 2), 40, 30).unwrap();
    heard.shown(rig, page(8, Vec::new(), SnipStage::Edit(Box::new(editor))));
    heard
}

/// The flattened pictures sent, oldest first.
fn flats(heard: &Heard) -> Vec<Shot> {
    let flat = |c: SnipClick| match c {
        SnipClick::Flat { shot, .. } => Some(shot),
        _ => None,
    };
    heard.sent.clicks().into_iter().filter_map(flat).collect()
}

#[test]
fn a_flattened_picture_larger_than_the_kept_part_is_cut_to_it_from_its_top_left() {
    let rig = Rig::new();
    let heard = kept_3_by_2(&rig);
    rig.hub.snip_flat("save", grab(4, 3), 8);
    rig.hub.snip_flat("copy", shot(3, 2), 8);
    let sent = flats(&heard);
    let rows: Vec<u8> = (0..12).chain(16..28).collect();
    assert_eq!(
        (sent[0].width(), sent[0].height()),
        (3, 2),
        "the kept part's size"
    );
    assert_eq!(
        &sent[0].bgrx().expose()[..],
        &rows[..],
        "its top left, row by row"
    );
    assert_eq!(sent[1], shot(3, 2), "one of its size goes as it is");
}

#[test]
fn a_grab_short_of_the_kept_part_keeps_its_own_size_there_and_warns() {
    let rig = Rig::new();
    let heard = kept_3_by_2(&rig);
    let warned = warnings_of(|| {
        rig.hub.snip_flat("copy", grab(4, 1), 8);
        rig.hub.snip_flat("save", shot(2, 2), 8);
    });
    assert_eq!(warned.len(), 2, "each short grab warns: {warned:?}");
    let sent = flats(&heard);
    let sizes: Vec<_> = sent.iter().map(|s| (s.width(), s.height())).collect();
    assert_eq!(sizes, [(3, 1), (2, 2)], "cut only where it holds more");
    assert_eq!(
        &sent[0].bgrx().expose()[..],
        &(0..12).collect::<Vec<u8>>()[..]
    );
    assert_eq!(sent[1], shot(2, 2), "short on both sides, it goes as it is");
}
