//! The writer. Each write makes a message-only window on the calling thread to own the clipboard, as
//! `SetClipboardData` needs an owner after `EmptyClipboard`, and ends it once the clipboard closes.

use kx_module_api::ports::{ClipError, ClipPart, ClipWait, ClipboardWriter};
use windows::Win32::System::Ole::CF_UNICODETEXT;

use super::open::{self, Open};
use super::parts::wide_z;
use super::{Table, read, table};

/// The 32-bit number that turns a Windows switch format off.
const OFF: [u8; 4] = 0u32.to_ne_bytes();

/// Sets the Windows clipboard; every write carries our marker and keeps out of Windows' history and cloud.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinClipboardWriter;

impl ClipboardWriter for WinClipboardWriter {
    fn write(&self, parts: &[ClipPart], wait: ClipWait) -> Result<(), ClipError> {
        let table = table()?;
        let formats = read::ids(table)?;
        let open = open::open(wait)?;
        open::empty(&open)?;
        // The marker goes first, so a part that fails never leaves a half copy that becomes a row.
        mark(&open, table)?;
        let encoded = formats.encode_all(parts);
        encoded
            .iter()
            .try_for_each(|(format, bytes)| open::put(&open, *format, bytes))
    }
}

/// Sets our marker and the cloud switch to 0 on the open clipboard.
fn mark(open: &Open, table: &Table) -> Result<(), ClipError> {
    for name in [&table.marker, &table.cloud_off] {
        open::put(open, open::format_id(name)?, &OFF)?;
    }
    Ok(())
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
