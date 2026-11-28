//! The writer. Each write makes a message-only window on the calling thread to own the clipboard, as
//! `SetClipboardData` needs an owner after `EmptyClipboard`, and ends it once the clipboard closes.

use kx_module_api::ports::{ClipError, ClipPart, ClipWait, ClipboardWriter};
use windows::Win32::System::Ole::CF_UNICODETEXT;

use super::open::{self, Open};
use super::parts::wide_z;
use super::{Table, read, table};

/// The 32-bit number that turns a Windows switch format off.
const OFF: [u8; 4] = 0u32.to_ne_bytes();

/// Sets the Windows clipboard; a write carries our marker and keeps out of Windows' history and cloud, but a kept one.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinClipboardWriter;

impl ClipboardWriter for WinClipboardWriter {
    fn write(&self, parts: &[ClipPart], wait: ClipWait) -> Result<(), ClipError> {
        put_all(parts, wait, Kind::Marked)
    }

    fn write_lent(&self, parts: &[ClipPart], wait: ClipWait) -> Result<(), ClipError> {
        put_all(parts, wait, Kind::Lent)
    }

    fn write_kept(&self, parts: &[ClipPart], wait: ClipWait) -> Result<(), ClipError> {
        put_all(parts, wait, Kind::Kept)
    }
}

/// Which write sets the clipboard, so it sets that write's markers.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// `write`: our marker and the cloud switch.
    Marked,
    /// `write_lent`: those and the presence markers.
    Lent,
    /// `write_kept`: no marker, so our history keeps it, and Windows' too for a picture (ADR-0056 decision 2).
    Kept,
}

/// Sets the clipboard to `parts` with the markers of a `kind` write.
fn put_all(parts: &[ClipPart], wait: ClipWait, kind: Kind) -> Result<(), ClipError> {
    let table = table()?;
    let formats = read::ids(table)?;
    let open = open::open(wait)?;
    open::empty(&open)?;
    // Markers go first, so a part that fails never leaves a half copy that becomes a row. A kept write has none, so one
    // that fails leaves the formats put before it, which both histories may keep.
    mark(&open, table, kind)?;
    let encoded = formats.encode_all(parts);
    encoded
        .iter()
        .try_for_each(|(format, bytes)| open::put(&open, *format, bytes))
}

/// Sets each marker of a `kind` write to 0 on the open clipboard.
fn mark(open: &Open, table: &Table, kind: Kind) -> Result<(), ClipError> {
    for name in marks(table, kind) {
        open::put(open, open::format_id(&name)?, &OFF)?;
    }
    Ok(())
}

/// The markers a `kind` write sets: our marker and the cloud switch, and for a lent text the presence markers too, whose
/// mere presence keeps other clipboard tools away from a text that is never written back; a kept write sets none.
pub(crate) fn marks(table: &Table, kind: Kind) -> Vec<String> {
    if kind == Kind::Kept {
        return Vec::new();
    }
    let present = table.marks().presence();
    let lent_ones = present.names().filter(|_| kind == Kind::Lent);
    [table.marker.clone(), table.cloud_off.clone()]
        .into_iter()
        .chain(lent_ones.map(str::to_owned))
        .collect()
}

/// Puts `text` on the clipboard for a test tool, or only empties it for `None`, in one change; each format in `zero`
/// and the cloud switch hold 0, so a test's text never reaches the user's other devices.
///
/// # Errors
/// `Busy` or `Refused` as `open::open` says, `Refused` when the table does not parse, `Io` when a part cannot be set.
pub fn write_text(text: Option<&str>, zero: &[&str], wait: ClipWait) -> Result<(), ClipError> {
    let cloud = table()?.cloud_off.as_str();
    let open = open::open(wait)?;
    open::empty(&open)?;
    let Some(text) = text else {
        return Ok(());
    };
    open::put(&open, u32::from(CF_UNICODETEXT.0), &wide_z(text))?;
    for name in zero.iter().copied().chain([cloud]) {
        open::put(&open, open::format_id(name)?, &OFF)?;
    }
    Ok(())
}
