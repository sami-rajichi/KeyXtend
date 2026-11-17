//! The ui holds `Voice` as an event gate only (ADR-0042 decision 7): it hears the views and sends the clicks and the
//! open, the level needs no grant, and its code never types, hears typing done or presses the key. It hears the key's
//! light only to redraw the keyboard at once, and never makes one, so it never lights it.

use std::fs;
use std::path::Path;

use kx_module_api::ports::{VoiceAct, VoiceAsk, VoiceLevel, VoiceOpen, VoiceView};
use kx_module_api::{Bus, Capability, Event, EventError};
use kx_test_support::host::FakeCore;
use kx_test_support::source::rust_files;
use kx_ui::MANIFEST;
use std::sync::Arc;

/// What `Voice` gates that the ui must never name: the keyboard's typing and its answer, and the key.
const NEVER: [&str; 3] = ["TextAsk", "TextDone", "VoiceKey"];
/// The key's light, which only the keys' wake names, to hear it.
const LIGHT: &str = "VoiceLit";
/// The one file that hears the light.
const HEARS_LIGHT: &str = "hub/bus.rs";

#[test]
fn the_ui_never_names_typing_or_the_key_and_only_hears_the_light() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = rust_files(&src).unwrap();
    assert!(
        files.iter().any(|f| f.ends_with("hub/voice.rs")),
        "the walk reached the hub"
    );
    for file in files {
        let code = fs::read_to_string(&file).unwrap();
        for name in NEVER {
            assert!(!code.contains(name), "{} names {name}", file.display());
        }
        if code.contains(LIGHT) {
            assert!(
                file.ends_with(HEARS_LIGHT),
                "{} names {LIGHT}",
                file.display()
            );
            let made = code.contains(&format!("{LIGHT}::"));
            assert!(
                !made,
                "{} makes a light, which only the voice module sends",
                file.display()
            );
        }
    }
}

#[test]
fn the_ui_holds_voice_so_its_handle_hears_the_view_and_sends_clicks() {
    assert!(MANIFEST.capabilities.contains(&Capability::Voice));
    let gated = Bus::new(Arc::new(FakeCore::default())).restricted(MANIFEST.capabilities);
    assert_eq!(gated.check::<VoiceView>(), Ok(()));
    assert_eq!(gated.check::<VoiceAct>(), Ok(()));
    assert_eq!(gated.check::<VoiceOpen>(), Ok(()));
    assert_eq!(gated.check::<VoiceAsk>(), Ok(()));
    let bare = Bus::new(Arc::new(FakeCore::default())).restricted(&[]);
    let refused = EventError::NotGranted(VoiceAct::NAME, Capability::Voice);
    assert_eq!(bare.check::<VoiceAct>(), Err(refused), "the gate holds");
    assert_eq!(
        bare.check::<VoiceLevel>(),
        Ok(()),
        "the level holds no content"
    );
}
