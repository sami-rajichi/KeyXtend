//! The worker's token: a restricted copy of our own primary token, with every privilege but change-notify dropped and
//! Administrators deny-only, then uiAccess cleared and Medium integrity, since a child of a uiAccess process otherwise
//! keeps uiAccess (Forshaw, "Accessing Access Tokens for `UIAccess`", 2019).

use std::ffi::c_void;
use std::os::windows::io::{FromRawHandle, OwnedHandle};

use kx_platform::pictures::SpawnError;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::{
    CreateRestrictedToken, DISABLE_MAX_PRIVILEGE, DuplicateTokenEx, GetTokenInformation,
    SID_AND_ATTRIBUTES, SecurityImpersonation, SetTokenInformation, TOKEN_ACCESS_MASK,
    TOKEN_ADJUST_DEFAULT, TOKEN_ASSIGN_PRIMARY, TOKEN_DUPLICATE, TOKEN_INFORMATION_CLASS,
    TOKEN_QUERY, TokenIntegrityLevel, TokenPrimary, TokenUIAccess, WinBuiltinAdministratorsSid,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

use super::label::{Label, SidRoom, well_known};
use super::{raw, refused, size32};

/// What the copy may do: be read, set and copied, start a process, and take a new default.
const COPY_RIGHTS: TOKEN_ACCESS_MASK = TOKEN_ACCESS_MASK(
    TOKEN_QUERY.0 | TOKEN_DUPLICATE.0 | TOKEN_ASSIGN_PRIMARY.0 | TOKEN_ADJUST_DEFAULT.0,
);

/// Our own primary token, restricted, with uiAccess cleared and Medium integrity, for the worker to start with.
#[allow(unsafe_code, reason = "Opens our own token.")]
pub(super) fn reduced() -> Result<OwnedHandle, SpawnError> {
    // SAFETY: the pseudo handle of our own process needs no closing.
    let ours = open(unsafe { GetCurrentProcess() }, TOKEN_DUPLICATE)?;
    let copy = primary(&restricted(&ours)?)?;
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

/// A copy of `token` with every privilege but change-notify dropped and Administrators deny-only.
#[allow(unsafe_code, reason = "Makes a restricted token into a handle we own.")]
fn restricted(token: &OwnedHandle) -> Result<OwnedHandle, SpawnError> {
    let mut room = SidRoom::default();
    let (admins, _) = well_known(WinBuiltinAdministratorsSid, &mut room)?;
    let deny = [SID_AND_ATTRIBUTES {
        Sid: admins,
        Attributes: 0,
    }];
    let mut made = HANDLE::default();
    // SAFETY: `token` is open with the right to copy; `deny` and the SID in `room` it points to live past the call;
    // the new handle goes straight to an owner.
    unsafe {
        CreateRestrictedToken(
            raw(token),
            DISABLE_MAX_PRIVILEGE,
            Some(&deny),
            None,
            None,
            &raw mut made,
        )
    }
    .map_err(refused("restrict our token"))?;
    // SAFETY: Windows just made this handle, and only the owner closes it.
    Ok(unsafe { OwnedHandle::from_raw_handle(made.0) })
}

/// A primary copy of `token` with `COPY_RIGHTS`, as a start and the settings below need.
#[allow(unsafe_code, reason = "Copies a token of ours into a handle we own.")]
fn primary(token: &OwnedHandle) -> Result<OwnedHandle, SpawnError> {
    let mut copy = HANDLE::default();
    // SAFETY: `token` is open with the right to copy; the new handle goes straight to an owner.
    unsafe {
        DuplicateTokenEx(
            raw(token),
            COPY_RIGHTS,
            None,
            SecurityImpersonation,
            TokenPrimary,
            &raw mut copy,
        )
    }
    .map_err(refused("copy our token"))?;
    // SAFETY: Windows just made this handle, and only the owner closes it.
    Ok(unsafe { OwnedHandle::from_raw_handle(copy.0) })
}

/// Sets the copy's integrity level to Medium, from a label built in place with its SID inside it.
fn set_medium(token: &OwnedHandle) -> Result<(), SpawnError> {
    let mut label = Label::default();
    let size = label.medium()?;
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
pub(super) fn open(process: HANDLE, rights: TOKEN_ACCESS_MASK) -> Result<OwnedHandle, SpawnError> {
    let mut token = HANDLE::default();
    // SAFETY: `process` is open for the call; the token handle goes straight to an owner.
    unsafe { OpenProcessToken(process, rights, &raw mut token) }
        .map_err(refused("open a token"))?;
    // SAFETY: Windows just made this handle, and only the owner closes it.
    Ok(unsafe { OwnedHandle::from_raw_handle(token.0) })
}

/// Sets `class` on `token` from the first `size` bytes of `value`, at most all of it, as step `step`.
#[allow(unsafe_code, reason = "Hands Windows a live value of ours.")]
fn set<T>(
    token: &OwnedHandle,
    class: TOKEN_INFORMATION_CLASS,
    value: &T,
    size: u32,
    step: &'static str,
) -> Result<(), SpawnError> {
    let size = size.min(size32::<T>());
    let value: *const c_void = std::ptr::from_ref(value).cast();
    // SAFETY: `value` is live for the call and spans `size` bytes, as `size` is at most `size_of::<T>()`.
    unsafe { SetTokenInformation(raw(token), class, value, size) }.map_err(refused(step))
}

/// Reads `class` of `token` into `value`, as step `step`.
#[allow(unsafe_code, reason = "Lets Windows fill a value of ours.")]
pub(super) fn get<T>(
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
