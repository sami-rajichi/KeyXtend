//! The app's QML names no network address and opens no request. Qt Network loads in the uiAccess process only because
//! Qt Qml and Qt Quick import it, and no TLS backend ships (ADR-0004, ADR-0013).

use std::path::Path;

/// Words that would reach the network from QML: URL schemes and the request object.
const BANNED: [&str; 4] = ["http:", "https:", "ftp:", "XMLHttpRequest"];
/// The shipped QML folder, from this crate.
const QML_DIR: &str = "qml";

/// The banned words in `text`, in any letter case.
fn banned_in(text: &str) -> Vec<&'static str> {
    let lower = text.to_lowercase();
    BANNED
        .into_iter()
        .filter(|word| lower.contains(&word.to_lowercase()))
        .collect()
}

#[test]
fn the_ban_finds_each_word_in_any_letter_case() {
    let text = "source: \"HTTPS://x\"; ftp://y; new XmlHttpRequest(); http://z";
    assert_eq!(banned_in(text), BANNED.to_vec());
    assert!(banned_in("import QtQuick\nImage { source: \"qrc:/x.svg\" }").is_empty());
}

#[test]
fn the_shipped_qml_names_no_network_address() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(QML_DIR);
    let mut read = 0;
    let mut found = Vec::new();
    for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let path = entry.unwrap_or_else(|e| panic!("{e}")).path();
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        read += 1;
        found.extend(
            banned_in(&text)
                .into_iter()
                .map(|w| format!("{}: {w}", path.display())),
        );
    }
    assert!(read > 0, "no QML file in {}", dir.display());
    assert!(found.is_empty(), "{found:#?}");
}
