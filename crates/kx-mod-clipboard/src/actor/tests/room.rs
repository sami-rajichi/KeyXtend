//! The shelf's room: no storage line shows until a room is read, and it shows from the next good read on.

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
