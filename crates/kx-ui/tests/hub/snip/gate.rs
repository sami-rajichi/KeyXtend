//! The ui holds `Snip` as an event gate only: it hears the views and the keyboard's aside, sends the clicks, wakes the
//! keyboard at once when the Snip key's light changes, and never presses the Snip key, touches its line or sends a light.

use super::heard;
use crate::{FakeFace, Rig};
use kx_module_api::ports::{SnipAct, SnipAside, SnipLit, SnipView};
use kx_module_api::{Bus, Capability, Event, EventError};
use kx_test_support::host::{FakeCore, FakeHost};
use kx_test_support::source::rust_files;
use kx_ui::{MANIFEST, Wake};
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// What the ui never names: the Snip key, the line and the keys a Snip window takes, and the keyboard's ask.
const NEVER: [&str; 3] = ["SnipStart", "SnipLine", "SnipAsk"];
/// The key's light, which only the keys' wake names, to hear it.
const LIGHT: &str = "SnipLit";
/// The one file that hears the light.
const HEARS_LIGHT: &str = "hub/bus.rs";

#[test]
fn the_ui_holds_snip_so_its_handle_hears_the_view_and_sends_clicks() {
    assert!(MANIFEST.capabilities.contains(&Capability::Snip));
    let gated = Bus::new(Arc::new(FakeCore::default())).restricted(MANIFEST.capabilities);
    assert_eq!(gated.check::<SnipView>(), Ok(()));
    assert_eq!(gated.check::<SnipAside>(), Ok(()));
    assert_eq!(gated.check::<SnipAct>(), Ok(()));
    let bare = Bus::new(Arc::new(FakeCore::default())).restricted(&[]);
    let refused = EventError::NotGranted(SnipAct::NAME, Capability::Snip);
    assert_eq!(bare.check::<SnipAct>(), Err(refused), "the gate holds");
}

#[test]
fn the_ui_never_presses_the_snip_key_touches_its_line_or_sends_its_light() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = rust_files(&src).unwrap();
    assert!(
        files.iter().any(|f| f.ends_with("hub/snip.rs")),
        "the walk reached the hub"
    );
    for file in files {
        let (code, shown) = (fs::read_to_string(&file).unwrap(), file.display());
        for name in NEVER {
            assert!(!code.contains(name), "{shown} names {name}");
        }
        let lit = code.contains(LIGHT);
        assert!(!lit || file.ends_with(HEARS_LIGHT), "{shown} names {LIGHT}");
        assert!(!code.contains(&format!("{LIGHT} as")), "{shown} renames it");
        assert!(
            !code.contains(&format!("publish(&{LIGHT}")),
            "{shown} sends it"
        );
    }
}

#[test]
fn the_snip_keys_light_redraws_the_keyboard_at_once() {
    let rig = Rig::new();
    let heard = heard(&rig);
    let mut host = FakeHost::new(&MANIFEST);
    rig.attach(Arc::new(FakeFace::default()), &mut host);
    rig.wakes();
    heard.bus.publish(&SnipLit::On).unwrap();
    assert_eq!(rig.wakes(), [Wake::Keys]);
}
