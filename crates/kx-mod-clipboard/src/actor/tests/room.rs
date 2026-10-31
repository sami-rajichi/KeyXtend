//! The shelf's room: no storage line shows until a room is read, it shows from the next good read on, and a copy
//! asks only the free bytes.

use kx_module_api::ports::SealedError;
use kx_platform_fake::ShelfCall;

use super::rig::Rig;

#[test]
fn no_storage_line_shows_until_a_room_is_read() {
    let mut rig = Rig::new();
    rig.fakes.shelf.fail(ShelfCall::Room, SealedError::Io);
    rig.open();
    assert_eq!(rig.page().room, None, "no half-empty line");
    rig.text("a");
    assert!(rig.page().room.is_some(), "read again after a change");
}

#[test]
fn a_copy_asks_only_the_drives_free_bytes_and_never_walks_the_shelf() {
    let mut rig = Rig::new();
    let walks = rig.fakes.shelf.calls(ShelfCall::Room);
    rig.text("a");
    rig.text("b");
    assert_eq!(rig.fakes.shelf.calls(ShelfCall::Room), walks, "no walk");
    assert_eq!(rig.fakes.shelf.calls(ShelfCall::Free), 2, "one ask each");
}
