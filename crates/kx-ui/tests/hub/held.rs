//! A held key through the hub (spec §4.6): a repeat that went out says so apart from a redraw, and a failed one
//! logs one warning with no key.

use crate::Rig;
use crate::logs::warnings_of;
use kx_module_api::keys;
use kx_module_api::ports::{FaceAction, FaceReply, KeyId};

#[test]
fn a_repeat_that_went_out_says_so_apart_from_a_redraw() {
    let (rig, face, _host) = Rig::with_face();
    face.answer(Ok(FaceReply {
        repeated: true,
        ..FaceReply::default()
    }));
    let redraw = rig.hub.act(FaceAction::Tick);
    assert!(redraw.repeated && !redraw.state && !redraw.board);
    assert_eq!(
        redraw.to_json(),
        r#"{"board":false,"state":false,"repeated":true}"#
    );
}

#[test]
fn a_held_keys_failed_repeat_logs_one_warning_with_no_key() {
    let (rig, face, _host) = Rig::with_face();
    face.answer(Ok(FaceReply {
        notice: Some(keys::KEYS_NOT_SENT),
        ..FaceReply::default()
    }));
    let warned = warnings_of(|| {
        rig.hub.act(FaceAction::Tick);
    });
    assert_eq!(warned, ["a held key's repeat was not sent"]);
    let tap = FaceAction::Tap {
        id: KeyId::new("a"),
        back: false,
    };
    let warned = warnings_of(|| {
        rig.hub.act(tap);
    });
    assert!(warned.is_empty(), "a click's banner is enough");
}
