//! The drives Windows lists now, of the types the table keeps, each with its volume label, read with the disk dialogs
//! off. A type is read first, so a drive of another type is never touched, and a network drive's label is never read.

use std::ops::RangeInclusive;

use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::Storage::FileSystem::{GetDriveTypeW, GetLogicalDrives, GetVolumeInformationW};
use windows::core::PCWSTR;

use super::Drive;
use super::quiet::Quiet;
use super::table::Table;
use crate::wide;

/// The drive letters, one bit each in `GetLogicalDrives`' mask from the lowest up.
const LETTERS: RangeInclusive<u8> = b'A'..=b'Z';
/// Ends a drive letter's root, as in `E:\`.
const ROOT_END: &str = ":\\";

/// The drives there now, A to Z, of a type `table` keeps.
#[allow(unsafe_code, reason = "Asks Windows which drive letters are in use.")]
pub(super) fn live(table: &Table) -> Vec<Drive> {
    let _quiet = Quiet::new();
    // SAFETY: takes nothing and only returns a bit mask.
    let mask = unsafe { GetLogicalDrives() };
    let used = LETTERS
        .zip(0u32..)
        .filter(|(_, bit)| mask & (1 << bit) != 0);
    used.filter_map(|(letter, _)| drive(letter, table))
        .collect()
}

/// The drive with `letter`, when its type is one `table` keeps.
#[allow(unsafe_code, reason = "Asks Windows for a drive's type.")]
fn drive(letter: u8, table: &Table) -> Option<Drive> {
    let root = format!("{}{ROOT_END}", char::from(letter));
    let name = wide(&root);
    // SAFETY: `name` is NUL-ended and outlives the call, which reads no media.
    let code = unsafe { GetDriveTypeW(PCWSTR(name.as_ptr())) };
    let kind = table.kind(code)?;
    let remote = table.remote(code);
    // A lost share would stall the label's read, so a network drive goes by its letter alone.
    let label = if remote { String::new() } else { label(&name) };
    Some(Drive {
        kind,
        remote,
        root: root.into(),
        label,
    })
}

/// The volume label of the drive whose NUL-ended root is `root`; empty without one or without media.
#[allow(unsafe_code, reason = "Asks Windows for a volume's label.")]
fn label(root: &[u16]) -> String {
    let mut label = vec![0u16; MAX_PATH as usize + 1];
    // SAFETY: `root` is NUL-ended and outlives the call; Windows writes at most `label.len()` units.
    let read = unsafe {
        GetVolumeInformationW(
            PCWSTR(root.as_ptr()),
            Some(&mut label),
            None,
            None,
            None,
            None,
        )
    };
    if read.is_err() {
        return String::new();
    }
    let end = label.iter().position(|u| *u == 0).unwrap_or(label.len());
    String::from_utf16_lossy(&label[..end])
}
