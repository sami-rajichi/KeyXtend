//! The Dictation key's light names in `Key.qml`, pinned to the names the keyboard's state writes `VoiceLit` under.

use kx_module_api::ports::VoiceLit;

/// The key QML file, which reads the light by name.
const KEY: &str = include_str!("../../qml/Key.qml");

/// Where `Key.qml` writes light `lit`'s name; a new light fails to build here until the key draws it.
const fn place(lit: VoiceLit) -> &'static str {
    match lit {
        VoiceLit::Off => "ks.lit ? ks.lit : ",
        VoiceLit::Prep => "prep: light === ",
        VoiceLit::Rec => "rec: light === ",
    }
}

#[test]
fn the_key_reads_each_light_by_the_name_the_state_writes() {
    for lit in [VoiceLit::Off, VoiceLit::Prep, VoiceLit::Rec] {
        let name = serde_json::to_value(lit).expect("a light writes");
        let want = format!("{}\"{}\"", place(lit), name.as_str().expect("a name"));
        assert!(KEY.contains(&want), "{lit:?}: {want}");
    }
}
