//! The kinds of click QML names to `Hub::clip_act`, and the `ClipAct` each one sends.

use crate::i18n::keys::CLIP_FILTERS;
use kx_module_api::ports::ClipAct;
use std::cmp::Ordering;

/// Tick or untick the row at the argument's place on the page.
pub const TICK: &str = "tick";
/// Show the filter at the argument's place in `CLIP_FILTERS`, the list the panel's JSON gives.
pub const FILTER: &str = "filter";
/// Turn a page: back when the argument is below 0, on when above.
pub const PAGE: &str = "page";
/// Insert the ticked rows or the previewed entry.
pub const INSERT: &str = "insert";
/// Pin, or unpin when every target is pinned.
pub const PIN: &str = "pin";
/// Delete the ticked rows or the previewed entry.
pub const DELETE: &str = "delete";
/// Ask to delete every unpinned entry.
pub const DELETE_ALL: &str = "delete-all";
/// Yes to the Delete-all question.
pub const YES: &str = "yes";
/// No to the Delete-all question.
pub const NO: &str = "no";
/// Open the preview page.
pub const PREVIEW: &str = "preview";
/// Leave the preview page.
pub const BACK: &str = "back";
/// Close the panel.
pub const CLOSE: &str = "close";

/// Why a click sends nothing.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) enum Dropped {
    /// It names a row of a view the hub no longer keeps, or of none as the panel closes: a harmless late click.
    Stale,
    /// An unknown kind or an argument out of range: QML sent what it never should.
    Bad,
}

/// The act `kind` names with `arg`, from the view of round `round`; a tick needs `shown`, the kept view's round and
/// row count, and that round.
pub(super) fn act_of(
    kind: &str,
    arg: i32,
    round: i64,
    shown: Option<(u32, usize)>,
) -> Result<ClipAct, Dropped> {
    Ok(match kind {
        TICK => {
            let (kept, rows) = shown.ok_or(Dropped::Stale)?;
            let round = u32::try_from(round).ok().filter(|&r| r == kept);
            let round = round.ok_or(Dropped::Stale)?;
            let row = u8::try_from(arg).ok().filter(|&r| usize::from(r) < rows);
            ClipAct::Tick {
                round,
                row: row.ok_or(Dropped::Bad)?,
            }
        }
        FILTER => {
            let at = usize::try_from(arg).map_err(|_| Dropped::Bad)?;
            ClipAct::Filter(CLIP_FILTERS.get(at).ok_or(Dropped::Bad)?.0)
        }
        PAGE => match arg.cmp(&0) {
            Ordering::Less => ClipAct::Page { back: true },
            Ordering::Greater => ClipAct::Page { back: false },
            Ordering::Equal => return Err(Dropped::Bad),
        },
        INSERT => ClipAct::Insert,
        PIN => ClipAct::Pin,
        DELETE => ClipAct::Delete,
        DELETE_ALL => ClipAct::DeleteAll,
        YES => ClipAct::Confirm(true),
        NO => ClipAct::Confirm(false),
        PREVIEW => ClipAct::Preview,
        BACK => ClipAct::Back,
        CLOSE => ClipAct::Close,
        _ => return Err(Dropped::Bad),
    })
}
