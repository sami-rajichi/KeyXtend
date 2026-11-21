//! The ui holds `Voice` as an event gate only (ADR-0042 decision 7): it hears the views and sends the clicks and the
//! open, the level needs no grant, and its code never types, hears typing done or presses the key. It hears the key's
//! light only to redraw the keyboard at once, in one handler beside its import from the ports, and never renames, makes
//! or sends one.

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
/// The import the light's one other mention must sit in: the list from the ports.
const IMPORT: &str = "use kx_module_api::ports::{";
/// What ends that list.
const IMPORT_END: &str = "};";
/// The one file that hears the light, which names it twice: in its import and in one handler's argument.
const HEARS_LIGHT: &str = "hub/bus.rs";
/// That handler's argument.
const HANDLER: &str = "_: &VoiceLit|";

/// Checks how `file` names the light: in `HEARS_LIGHT` only its import and one handler, nowhere else, never under
/// another name and never sent; true for `HEARS_LIGHT`.
fn hears_light(file: &Path, code: &str) -> bool {
    let shown = file.display();
    assert!(
        !code.contains(&format!("{LIGHT} as")),
        "{shown} renames {LIGHT}"
    );
    assert!(
        !code.contains(&format!("publish(&{LIGHT}")),
        "{shown} sends {LIGHT}"
    );
    let named = code.matches(LIGHT).count();
    if !file.ends_with(HEARS_LIGHT) {
        assert_eq!(named, 0, "{shown} names {LIGHT}");
        return false;
    }
    let handlers = code.matches(HANDLER).count();
    assert_eq!(
        (named, handlers),
        (2, 1),
        "{shown}: only its import and one handler"
    );
    assert!(
        imports_light(code),
        "{shown}: its other mention is the import from the ports"
    );
    true
}

/// True when `code`'s import list from the ports names the light, so its other mention is no stand-in beside a glob.
fn imports_light(code: &str) -> bool {
    let list = code
        .split_once(IMPORT)
        .and_then(|(_, rest)| rest.split_once(IMPORT_END));
    list.is_some_and(|(names, _)| names.split(',').any(|n| n.trim() == LIGHT))
}

#[test]
fn the_lights_other_mention_must_be_its_import_from_the_ports() {
    let handler = format!("bus.subscribe(|{HANDLER} {{}});");
    let real = format!("{IMPORT}\n    PowerChanged, ScrollPage, {LIGHT},\n{IMPORT_END}\n{handler}");
    assert!(imports_light(&real), "the import list");
    let glob = format!("use kx_module_api::ports::*;\nstruct {LIGHT};\n{handler}");
    assert!(!imports_light(&glob), "a stand-in beside a glob import");
    let other = format!("{IMPORT}PowerChanged{IMPORT_END}\nuse other::{LIGHT};\n{handler}");
    assert!(!imports_light(&other), "another crate's light");
}

#[test]
fn the_ui_never_names_typing_or_the_key_and_only_hears_the_light() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let files = rust_files(&src).unwrap();
    assert!(
        files.iter().any(|f| f.ends_with("hub/voice.rs")),
        "the walk reached the hub"
    );
    let mut hearers = 0;
    for file in files {
        let code = fs::read_to_string(&file).unwrap();
        for name in NEVER {
            assert!(!code.contains(name), "{} names {name}", file.display());
        }
        hearers += usize::from(hears_light(&file, &code));
    }
    assert_eq!(hearers, 1, "the walk reached {HEARS_LIGHT}");
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
