//! The backup and import pages as QML reads them: each place's state, the suggested places and the folder list, the
//! password page by what it asks, and the boxes found or from another computer, by kind, short name and date only.

use super::sheets::row_button;
use super::{Button, Drawn, button, spot, text};
use crate::I18n;
use crate::i18n::keys::{
    FILL_ASK, FILL_DRIVES, FILL_FOLDERS, FILL_FOREIGN, FILL_FOUND, FILL_NO_PLACES, FILL_PICK,
    FILL_PLACE_WAITING, FILL_PLACES, FILL_REPLACE, FILL_WORD, ask_id, fill_button_id, pick_id,
    row_button_id,
};
use kx_module_api::ports::{
    BackupAsk, BackupSheet, BoxState, FillButton, FolderSheet, FoundRow, PasswordAsk,
    PasswordSheet, PlaceKind, PlaceRow, RowButton, Spot,
};
use serde::Serialize;

/// The backup page.
#[derive(Serialize)]
pub(super) struct Backup {
    places: Vec<Line>,
    checks: bool,
    /// The invitation while no place keeps a box.
    none: Option<String>,
    /// Remove place's question, on its row; the panel's Yes and No answer it.
    confirm: Option<Ask>,
    /// Back up now's question for the whole page while the notebook did not read; the panel's Yes and No answer it.
    replace: Option<Question>,
}

/// A question for the whole shown page.
#[derive(Serialize)]
struct Question {
    text: String,
}

/// A question on the row at `row` of the shown page.
#[derive(Serialize)]
struct Ask {
    row: u8,
    text: String,
}

/// A place on the backup page: its box's state, and Remove.
#[derive(Serialize)]
struct Line {
    kind: &'static str,
    name: Drawn,
    state: String,
    waiting: bool,
    buttons: Vec<Button>,
}

/// A place, drive, folder or file to open or pick.
#[derive(Serialize)]
pub(super) struct Row {
    kind: &'static str,
    name: Drawn,
    buttons: Vec<Button>,
}

/// The suggested places, under their heading.
#[derive(Serialize)]
pub(super) struct Spots {
    text: String,
    rows: Vec<Row>,
}

/// The folder list: what it picks, its heading, the folder shown, or This PC at the drives.
#[derive(Serialize)]
pub(super) struct Folders {
    pick: &'static str,
    text: String,
    folder: Drawn,
    drives: bool,
    rows: Vec<Row>,
}

/// The password page: what it asks, said in full, and what is typed so far, which shows (owner).
#[derive(Serialize)]
pub(super) struct Password {
    ask: &'static str,
    text: String,
    typed: Drawn,
    /// The caret, in UTF-16 units of `typed`.
    caret: u32,
}

/// A box found, with Bring back, or one from another computer, which the panel's Yes and No answer.
#[derive(Serialize)]
pub(super) struct Found {
    kind: &'static str,
    name: Drawn,
    text: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    buttons: Vec<Button>,
}

/// The backup page `b`.
pub(super) fn backup(b: &BackupSheet, t: &I18n) -> Backup {
    Backup {
        none: b.places.is_empty().then(|| t.text(FILL_NO_PLACES)),
        places: b.places.iter().map(|p| line(p, t)).collect(),
        checks: b.checks,
        confirm: remove(b, t),
        replace: (b.ask == Some(BackupAsk::Replace)).then(|| Question {
            text: t.text(FILL_REPLACE),
        }),
    }
}

/// Remove place's question, naming its place, as two sticks share a tile; a row the page does not show asks nothing.
fn remove(b: &BackupSheet, t: &I18n) -> Option<Ask> {
    let Some(BackupAsk::Remove(row)) = b.ask else {
        return None;
    };
    let place = b.places.get(usize::from(row))?;
    Some(Ask {
        row,
        text: t.fill_remove_place(place.place.name.expose()),
    })
}

/// Place row `p` with its box's state.
fn line(p: &PlaceRow, t: &I18n) -> Line {
    let (kind, name) = spot(&p.place);
    let (state, waiting) = match &p.state {
        BoxState::Updated(when) => (t.fill_updated(when), false),
        BoxState::Waiting => (t.text(FILL_PLACE_WAITING), true),
    };
    Line {
        kind,
        name,
        state,
        waiting,
        buttons: vec![row_button(t, RowButton::Remove)],
    }
}

/// The suggested places `spots`, each picked by its Open.
pub(super) fn places(spots: &[Spot], t: &I18n) -> Spots {
    Spots {
        text: t.text(FILL_PLACES),
        rows: spots.iter().map(|s| row(s, true, t)).collect(),
    }
}

/// The folder list `f`: a file is picked, and a drive or a folder opens.
pub(super) fn folders(f: &FolderSheet, t: &I18n) -> Folders {
    let folder = f.folder.as_ref().map(|name| text(name.expose()));
    Folders {
        pick: pick_id(f.pick),
        text: t.text(&format!("{FILL_FOLDERS}{}", pick_id(f.pick))),
        drives: folder.is_none(),
        folder: folder.unwrap_or_else(|| text(&t.text(FILL_DRIVES))),
        rows: f
            .rows
            .iter()
            .map(|s| row(s, s.kind == PlaceKind::File, t))
            .collect(),
    }
}

/// Spot `s` with its Open, worded Choose when it `picks`.
fn row(s: &Spot, picks: bool, t: &I18n) -> Row {
    let (kind, name) = spot(s);
    let open = if picks {
        button(t, row_button_id(RowButton::Open), FILL_PICK)
    } else {
        row_button(t, RowButton::Open)
    };
    Row {
        kind,
        name,
        buttons: vec![open],
    }
}

/// The password page `p`; a new password names its least length, and a check its tries left.
pub(super) fn password(p: &PasswordSheet, t: &I18n) -> Password {
    let key = format!("{FILL_ASK}{}", ask_id(p.ask));
    let text_of = match p.ask {
        PasswordAsk::New { min } => t.counted(&key, usize::from(min)),
        PasswordAsk::Check { tries } => t.counted(&key, usize::from(tries)),
        PasswordAsk::Open { .. } | PasswordAsk::Foreign | PasswordAsk::Keepass => t.text(&key),
    };
    Password {
        ask: ask_id(p.ask),
        text: text_of,
        typed: text(p.text.expose()),
        caret: *p.caret.expose(),
    }
}

/// Found box `r`, with Bring back.
pub(super) fn found(r: &FoundRow, t: &I18n) -> Found {
    let bring = format!("{FILL_WORD}{}", fill_button_id(FillButton::BringBack));
    let open = button(t, row_button_id(RowButton::Open), &bring);
    boxed(r, FILL_FOUND, vec![open], t)
}

/// The box from another computer `r`, which asks whether to bring its entries in.
pub(super) fn foreign(r: &FoundRow, t: &I18n) -> Found {
    boxed(r, FILL_FOREIGN, Vec::new(), t)
}

/// The box from another computer in `place`, whose date the system does not tell, which asks the same.
pub(super) fn foreign_undated(place: &Spot, t: &I18n) -> Found {
    let (kind, name) = spot(place);
    Found {
        kind,
        text: t.fill_foreign_undated(place.name.expose()),
        name,
        buttons: Vec::new(),
    }
}

/// Box `r` told by text `key`, with `buttons`.
fn boxed(r: &FoundRow, key: &str, buttons: Vec<Button>, t: &I18n) -> Found {
    let (kind, name) = spot(&r.place);
    Found {
        kind,
        text: t.fill_found(key, r.place.name.expose(), &r.when),
        name,
        buttons,
    }
}
