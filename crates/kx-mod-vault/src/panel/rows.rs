//! A row's buttons by kind (design "Rows"): a kind's pair gives User, Password and Both, one field gives its Insert,
//! and every row ends with Open, so any entry can be opened to edit. A click on the row itself does nothing.

use kx_module_api::ports::{Label, RowButton};

use super::{Fill, Part};
use crate::entry::Entry;
use crate::kinds::tables;

/// The places of `entry`'s fields under its kind's pair, when it holds both.
fn pair(entry: &Entry) -> Option<[usize; 2]> {
    let [user, password] = tables().kind(entry.kind())?.pair.as_ref()?;
    let place = |id: &str| {
        let under = |l: &Label| matches!(l, Label::Table(t) if t == id);
        entry.fields().iter().position(|f| under(&f.label))
    };
    Some([place(user)?, place(password)?])
}

/// The fields a row's hint glimpses: those its fill buttons type that hold a value, so an empty one shows no mask.
pub(crate) fn glimpsed(entry: &Entry) -> Vec<usize> {
    let mut places = match (pair(entry), entry.fields().len()) {
        (Some(both), _) => both.to_vec(),
        (None, 1) => vec![0],
        (None, _) => Vec::new(),
    };
    places.retain(|&i| !entry.fields()[i].value.expose().is_empty());
    places
}

/// `entry`'s row buttons in order: User, Password and Both for its kind's pair, else Insert for one field, then Open.
#[must_use]
pub fn buttons(entry: &Entry) -> Vec<RowButton> {
    let mut all = match (pair(entry), entry.fields().len()) {
        (Some(_), _) => vec![RowButton::User, RowButton::Password, RowButton::Both],
        (None, 1) => vec![RowButton::Insert],
        (None, _) => Vec::new(),
    };
    all.push(RowButton::Open);
    all
}

/// The fill `button` of `entry`'s row asks for; `None` for Open or a button the row lacks.
#[must_use]
pub fn fill(entry: &Entry, button: RowButton) -> Option<Fill> {
    let parts = match (button, pair(entry), entry.fields().len()) {
        (RowButton::User, Some([user, _]), _) => vec![Part::Value(user)],
        (RowButton::Password, Some([_, password]), _) => vec![Part::Value(password)],
        (RowButton::Both, Some([user, password]), _) => {
            vec![Part::Value(user), Part::Tab, Part::Value(password)]
        }
        (RowButton::Insert, None, 1) => vec![Part::Value(0)],
        _ => return None,
    };
    Some(Fill {
        id: entry.id(),
        parts,
    })
}
