//! The keyboard module's board file, shared by the test binaries that need its row height.

use serde::Deserialize;
use std::path::Path;

/// The keyboard's board, whose row height the face reports and the drawn check measures caps by.
const BOARD: &str = "../kx-mod-keyboard/board.toml";

/// The board file's row height.
#[derive(Deserialize)]
struct Row {
    row_px: f32,
}

/// The keyboard board's own row height at size 1, read from its file.
pub fn row() -> f32 {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(BOARD);
    let text = std::fs::read_to_string(path).expect("the keyboard board reads");
    toml::from_str::<Row>(&text)
        .expect("the keyboard board parses")
        .row_px
}
