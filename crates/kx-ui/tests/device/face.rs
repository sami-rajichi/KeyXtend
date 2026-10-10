//! A keyboard face over the keyboard module's golden board and states: it puts the window at `HOME` first, follows
//! each later placement, shows the language list at the next relabel once asked, and keeps every placement.

use kx_module_api::ports::{FaceAction, FaceError, FaceReply, KeyboardFace, Point, Rect};
use serde_json::Value;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// The keyboard module's golden files, from this crate.
const GOLDEN: &str = "../kx-mod-keyboard/tests/golden";
/// The golden board, the plain English state and the state with the language list open.
const FILES: [&str; 3] = ["board.json", "state-en.json", "state-list.json"];
/// The language the face's layout speaks.
const LANG: &str = "en";
/// Where the face puts the window first, in physical pixels.
pub const HOME: Point = Point { x: 300, y: 600 };

/// What changes as the test runs.
#[derive(Default)]
struct Live {
    at: Option<Point>,
    ask_list: bool,
    list: bool,
    placed: Vec<Rect>,
}

/// The face the hub draws in the test.
pub struct Face {
    board: String,
    plain: Value,
    listing: Value,
    row: f64,
    live: Mutex<Live>,
}

/// A golden file's text.
fn golden(name: &str) -> Result<String, String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(GOLDEN)
        .join(name);
    std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))
}

/// A golden file as JSON.
fn parsed(name: &str) -> Result<Value, String> {
    serde_json::from_str(&golden(name)?).map_err(|e| format!("{name}: {e}"))
}

impl Face {
    /// The face over the golden files, with its board's row height.
    pub fn new(row: f64) -> Result<Self, String> {
        let [board, plain, listing] = FILES;
        Ok(Self {
            board: golden(board)?,
            plain: parsed(plain)?,
            listing: parsed(listing)?,
            row,
            live: Mutex::default(),
        })
    }

    fn lock(&self) -> MutexGuard<'_, Live> {
        self.live.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Shows the language list at the next relabel, as a click on the language key would.
    pub fn open_list(&self) {
        self.lock().ask_list = true;
    }

    /// Every placement the window reported, oldest first, in physical pixels.
    pub fn placed(&self) -> Vec<Rect> {
        self.lock().placed.clone()
    }
}

impl KeyboardFace for Face {
    fn board_json(&self) -> String {
        self.board.clone()
    }

    fn state_json(&self) -> String {
        let live = self.lock();
        let mut state = if live.list {
            &self.listing
        } else {
            &self.plain
        }
        .clone();
        state["at"] = live
            .at
            .map_or(Value::Null, |p| serde_json::json!({ "x": p.x, "y": p.y }));
        state.to_string()
    }

    fn lang(&self) -> String {
        LANG.to_owned()
    }

    fn row_px(&self) -> f64 {
        self.row
    }

    fn act(&self, action: FaceAction) -> Result<FaceReply, FaceError> {
        let mut live = self.lock();
        let state = match action {
            FaceAction::Placed { rect, .. } => {
                live.placed.push(rect);
                let to = Point {
                    x: rect.x,
                    y: rect.y,
                };
                let at = if live.at.is_some() { to } else { HOME };
                live.at = Some(at);
                at != to
            }
            FaceAction::Relabel => {
                let opens = live.ask_list && !live.list;
                live.list |= live.ask_list;
                opens
            }
            _ => false,
        };
        Ok(FaceReply {
            state,
            ..FaceReply::default()
        })
    }
}
