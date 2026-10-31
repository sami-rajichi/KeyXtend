//! The worker's token: our own primary token, copied with uiAccess cleared and Medium integrity, since a child of a
//! uiAccess process otherwise keeps uiAccess (Forshaw, "Accessing Access Tokens for `UIAccess`", 2019); and the
//! check that reads a process's token back.

use std::ffi::c_void;
use std::os::windows::io::{FromRawHandle, OwnedHandle};

use kx_platform::pictures::SpawnError;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::{
    CreateWellKnownSid, DuplicateTokenEx, GetSidSubAuthority, GetSidSubAuthorityCount,
    GetTokenInformation, IsValidSid, PSID, SECURITY_MAX_SID_SIZE, SID_AND_ATTRIBUTES,
    SecurityImpersonation, SetTokenInformation, TOKEN_ACCESS_MASK, TOKEN_ADJUST_DEFAULT,
    TOKEN_ASSIGN_PRIMARY, TOKEN_DUPLICATE, TOKEN_INFORMATION_CLASS, TOKEN_MANDATORY_LABEL,
    TOKEN_QUERY, TokenIntegrityLevel, TokenPrimary, TokenUIAccess, WinMediumLabelSid,
};
use windows::Win32::System::SystemServices::{SE_GROUP_INTEGRITY, SECURITY_MANDATORY_MEDIUM_RID};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

use super::{raw, refused, size32};

/// The integrity level's last SID part the worker runs at: Medium, S-1-16-8192.
pub(super) const MEDIUM: u32 = SECURITY_MANDATORY_MEDIUM_RID.cast_unsigned();
/// What the copy may do: be read, set and copied, start a process, and take a new default.
const COPY_RIGHTS: TOKEN_ACCESS_MASK = TOKEN_ACCESS_MASK(
    TOKEN_QUERY.0 | TOKEN_DUPLICATE.0 | TOKEN_ASSIGN_PRIMARY.0 | TOKEN_ADJUST_DEFAULT.0,
);
/// A SID buffer of the largest size, aligned as a SID's parts are.
type SidRoom = [u32; SECURITY_MAX_SID_SIZE as usize / size_of::<u32>()];

/// An integrity label with room for its SID behind it, as `TokenIntegrityLevel` writes it.
#[repr(C)]
#[derive(Default)]
struct Label {
    label: TOKEN_MANDATORY_LABEL,
    sid: SidRoom,
}

/// What a process's token allows.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Rights {
    /// Whether it has uiAccess.
    pub ui_access: bool,
    /// Its integrity level's last SID part.
    pub level: u32,
}

impl Rights {
    /// Whether the worker may run with these rights: no uiAccess and at most Medium integrity.
    pub(super) const fn reduced(self) -> bool {
        !self.ui_access && self.level <= MEDIUM
    }
}

/// A copy of our own primary token with uiAccess cleared and Medium integrity, for the worker to start with.
#[allow(unsafe_code, reason = "Copies our own token into a handle we own.")]
pub(super) fn reduced() -> Result<OwnedHandle, SpawnError> {
    // SAFETY: the pseudo handle of our own process needs no closing.
    let ours = open(unsafe { GetCurrentProcess() }, TOKEN_DUPLICATE)?;
    let mut copy = HANDLE::default();
    // SAFETY: `ours` is open with the right to copy; the new handle goes straight to an owner.
    unsafe {
        DuplicateTokenEx(
            raw(&ours),
            COPY_RIGHTS,
            None,
            SecurityImpersonation,
            TokenPrimary,
            &raw mut copy,
        )
    }
    .map_err(refused("copy our token"))?;
    // SAFETY: Windows just made this handle, and only the owner closes it.
    let copy = unsafe { OwnedHandle::from_raw_handle(copy.0) };
    let off: u32 = 0;
    set(
        &copy,
        TokenUIAccess,
        &off,
        size32::<u32>(),
        "clear uiAccess",
    )?;
    set_medium(&copy)?;
    Ok(copy)
}

/// What the token of `process` allows.
#[allow(unsafe_code, reason = "Reads the SID Windows wrote into our buffer.")]
pub(super) fn rights(process: HANDLE) -> Result<Rights, SpawnError> {
    let token = open(process, TOKEN_QUERY)?;
    let mut ui: u32 = 0;
    get(&token, TokenUIAccess, &mut ui, "read uiAccess")?;
    let mut label = Label::default();
    get(
        &token,
        TokenIntegrityLevel,
        &mut label,
        "read the integrity level",
    )?;
    let sid = label.label.Label.Sid;
    let bad = SpawnError::Rights;
    // SAFETY: Windows wrote the label and its SID into `label`, which `sid` points into and which lives here; the
    // SID is checked before its parts are read, and the last part is within its count.
    let level = unsafe {
        if !IsValidSid(sid).as_bool() {
            return Err(bad);
        }
        let count = u32::from(*GetSidSubAuthorityCount(sid));
        *GetSidSubAuthority(sid, count.checked_sub(1).ok_or(bad)?)
    };
    Ok(Rights {
        ui_access: ui != 0,
        level,
    })
}

/// Sets the copy's integrity level to Medium.
#[allow(unsafe_code, reason = "Builds a well-known SID in our buffer.")]
fn set_medium(token: &OwnedHandle) -> Result<(), SpawnError> {
    let mut room = SidRoom::default();
    let sid = PSID(room.as_mut_ptr().cast());
    let mut len = SECURITY_MAX_SID_SIZE;
    // SAFETY: `room` holds `len` bytes, aligned for a SID, and lives past the set below.
    unsafe { CreateWellKnownSid(WinMediumLabelSid, None, Some(sid), &raw mut len) }
        .map_err(refused("make the Medium label"))?;
    let label = TOKEN_MANDATORY_LABEL {
        Label: SID_AND_ATTRIBUTES {
            Sid: sid,
            Attributes: SE_GROUP_INTEGRITY.cast_unsigned(),
        },
    };
    let size = size32::<TOKEN_MANDATORY_LABEL>().saturating_add(len);
    set(
        token,
        TokenIntegrityLevel,
        &label,
        size,
        "set Medium integrity",
    )
}

/// Opens the token of `process` with `rights`.
#[allow(unsafe_code, reason = "Opens a token into a handle we own.")]
fn open(process: HANDLE, rights: TOKEN_ACCESS_MASK) -> Result<OwnedHandle, SpawnError> {
    let mut token = HANDLE::default();
    // SAFETY: `process` is open for the call; the token handle goes straight to an owner.
    unsafe { OpenProcessToken(process, rights, &raw mut token) }
        .map_err(refused("open a token"))?;
    // SAFETY: Windows just made this handle, and only the owner closes it.
    Ok(unsafe { OwnedHandle::from_raw_handle(token.0) })
}

/// Sets `class` on `token` from `value`, `size` bytes, as step `step`.
#[allow(unsafe_code, reason = "Hands Windows a live value of ours.")]
fn set<T>(
    token: &OwnedHandle,
    class: TOKEN_INFORMATION_CLASS,
    value: &T,
    size: u32,
    step: &'static str,
) -> Result<(), SpawnError> {
    let value: *const c_void = std::ptr::from_ref(value).cast();
    // SAFETY: `value` is live for the call, and `size` covers it and the SID it points to.
    unsafe { SetTokenInformation(raw(token), class, value, size) }.map_err(refused(step))
}

/// Reads `class` of `token` into `value`, as step `step`.
#[allow(unsafe_code, reason = "Lets Windows fill a value of ours.")]
fn get<T>(
    token: &OwnedHandle,
    class: TOKEN_INFORMATION_CLASS,
    value: &mut T,
    step: &'static str,
) -> Result<(), SpawnError> {
    let mut len = 0;
    let buf: *mut c_void = std::ptr::from_mut(value).cast();
    // SAFETY: `buf` is `size_of::<T>()` writable bytes for the call; Windows writes no more.
    unsafe { GetTokenInformation(raw(token), class, Some(buf), size32::<T>(), &raw mut len) }
        .map_err(refused(step))
}
