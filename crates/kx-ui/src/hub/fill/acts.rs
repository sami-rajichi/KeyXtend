//! The kinds of click QML names to `Hub::fill_act`, and the `FillAct` each one sends. A button, filter or kind goes by
//! the id the panel's JSON gives it, and a row's or a field's place on the page is the argument.

use crate::i18n::keys::{
    entry_kind_id, field_button_id, fill_button_id, fill_filter_id, row_button_id,
};
use kx_module_api::Redacted;
use kx_module_api::ports::{
    EntryKind, FieldButton, FillAct, FillButton, FillClick, FillFilter, RowButton,
};
use std::cmp::Ordering;

/// A row's button `id`, on the row at the argument's place on the page.
pub const ROW: &str = "row";
/// A field's button or line `id`, on the field at the argument's place on the page.
pub const FIELD: &str = "field";
/// Show the filter `id`.
pub const FILTER: &str = "filter";
/// Pick the kind `id` on the New page.
pub const KIND: &str = "kind";
/// Turn a page: back when the argument is below 0, on when above.
pub const PAGE: &str = "page";
/// The panel button `id`.
pub const BUTTON: &str = "button";
/// Answer the shown question: 1 is yes, 0 is no.
pub const ANSWER: &str = "answer";
/// Turn memory checks on with 1, off with 0.
pub const CHECKS: &str = "checks";
/// Put the caret of the line that takes the letters at the argument: UTF-16 units of its text, or dots while masked.
pub const CARET: &str = "caret";
/// A notch of the mouse wheel: it turns a page as `PAGE` does, silently.
pub const WHEEL: &str = "wheel";

/// True for a click that plays no key sound: a caret, whose line sounded, and the wheel, as the scroll pad's page plays
/// none.
pub(super) fn silent(kind: &str) -> bool {
    matches!(kind, CARET | WHEEL)
}

/// A click QML should never send: an unknown kind or id, or an argument or round out of range.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Bad;

/// The act `kind` names with `id` and `arg`, on the view of round `round`.
pub(super) fn act_of(kind: &str, id: &str, arg: i32, round: i64) -> Result<FillAct, Bad> {
    let click = click_of(kind, id, arg)?;
    let round = u32::try_from(round).map_err(|_| Bad)?;
    Ok(FillAct { round, click })
}

/// The click `kind` names with `id` and `arg`.
fn click_of(kind: &str, id: &str, arg: i32) -> Result<FillClick, Bad> {
    Ok(match kind {
        ROW => FillClick::Row {
            row: place(arg)?,
            button: by_id(RowButton::ALL, row_button_id, id)?,
        },
        FIELD => FillClick::Field {
            field: place(arg)?,
            button: by_id(FieldButton::ALL, field_button_id, id)?,
        },
        FILTER => FillClick::Filter(by_id(FillFilter::ALL, fill_filter_id, id)?),
        KIND => FillClick::Kind(by_id(EntryKind::ALL, entry_kind_id, id)?),
        PAGE | WHEEL => match arg.cmp(&0) {
            Ordering::Less => FillClick::Page { back: true },
            Ordering::Greater => FillClick::Page { back: false },
            Ordering::Equal => return Err(Bad),
        },
        BUTTON => FillClick::Button(by_id(FillButton::ALL, fill_button_id, id)?),
        ANSWER => FillClick::Answer(switch(arg)?),
        CHECKS => FillClick::Checks(switch(arg)?),
        CARET => FillClick::Caret {
            at: Redacted::new(u32::try_from(arg).map_err(|_| Bad)?),
        },
        _ => return Err(Bad),
    })
}

/// The member of `all` whose id, by `id_of`, is `id`.
fn by_id<T: Copy, const N: usize>(
    all: [T; N],
    id_of: fn(T) -> &'static str,
    id: &str,
) -> Result<T, Bad> {
    all.into_iter().find(|&t| id_of(t) == id).ok_or(Bad)
}

/// A place on the page.
fn place(arg: i32) -> Result<u8, Bad> {
    u8::try_from(arg).map_err(|_| Bad)
}

/// 1 as on, 0 as off.
fn switch(arg: i32) -> Result<bool, Bad> {
    match arg {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Bad),
    }
}
