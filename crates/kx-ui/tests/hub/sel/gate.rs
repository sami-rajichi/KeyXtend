//! The ui holds `Selection` as an event gate only (ADR-0041 decision 3): it hears `ChipsView` and sends `ChipAct`, and
//! its code never sends an edit, hears one done, watches the selection or opens a link.

use std::fs;
use std::path::Path;

use kx_module_api::ports::{ChipAct, ChipsView};
use kx_module_api::{Bus, Capability, Event, EventError};
use kx_test_support::host::FakeCore;
use kx_test_support::source::rust_files;
use kx_ui::MANIFEST;
use std::sync::Arc;

/// What `Selection` and the module's other grants gate that the ui must never name: the keyboard's edits and their
/// answer, the selection watch and the web link.
const NEVER: [&str; 4] = ["EditAsk", "EditDone", "SelectionWatch", "WebLink"];

#[test]
fn the_ui_never_names_an_edit_the_watch_or_the_link() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = rust_files(&src).unwrap();
    assert!(
        files.iter().any(|f| f.ends_with("hub/sel.rs")),
        "the walk reached the hub"
    );
    for file in files {
        let code = fs::read_to_string(&file).unwrap();
        for name in NEVER {
            assert!(!code.contains(name), "{} names {name}", file.display());
        }
    }
}

#[test]
fn the_ui_holds_selection_so_its_handle_hears_the_strip_and_sends_clicks() {
    assert!(MANIFEST.capabilities.contains(&Capability::Selection));
    let gated = Bus::new(Arc::new(FakeCore::default())).restricted(MANIFEST.capabilities);
    assert_eq!(gated.check::<ChipsView>(), Ok(()));
    assert_eq!(gated.check::<ChipAct>(), Ok(()));
    let bare = Bus::new(Arc::new(FakeCore::default())).restricted(&[]);
    let refused = EventError::NotGranted(ChipAct::NAME, Capability::Selection);
    assert_eq!(bare.check::<ChipAct>(), Err(refused), "the gate holds");
}
