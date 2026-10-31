//! What a process's token allows, read back before the worker runs: uiAccess, the integrity level, whether it can use
//! Administrators, and whether it holds any privilege beyond change-notify.

use std::mem::offset_of;
use std::os::windows::io::OwnedHandle;

use kx_platform::pictures::SpawnError;
use windows::Win32::Foundation::{HANDLE, LUID};
use windows::Win32::Security::{
    EqualSid, GetTokenInformation, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW, PSID,
    SE_CHANGE_NOTIFY_NAME, SID_AND_ATTRIBUTES, TOKEN_GROUPS, TOKEN_INFORMATION_CLASS,
    TOKEN_PRIVILEGES, TOKEN_QUERY, TokenGroups, TokenIntegrityLevel, TokenPrivileges,
    TokenUIAccess, WELL_KNOWN_SID_TYPE, WinBuiltinAdministratorsSid,
};
use windows::Win32::System::SystemServices::SE_GROUP_USE_FOR_DENY_ONLY;
use windows::core::PCWSTR;

use super::label::{Label, MEDIUM, SidRoom, well_known};
use super::token::{get, open};
use super::{raw, refused};

/// What a process's token allows.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(super) struct Rights {
    /// Whether it has uiAccess.
    pub ui_access: bool,
    /// Its integrity level's last SID part.
    pub level: u32,
    /// Whether it can use Administrators: the group is there and not deny-only, so enabled or able to be.
    pub admin: bool,
    /// Whether it holds a privilege beyond change-notify; held is enough, as the process could enable it.
    pub privileged: bool,
}

impl Rights {
    /// Whether the worker may run with these rights: no uiAccess, at most Medium, no Administrators, no privilege.
    pub(super) const fn reduced(self) -> bool {
        !self.ui_access && self.level <= MEDIUM && !self.admin && !self.privileged
    }
}

/// A token list's entry: plain data, valid for any bytes.
pub(super) trait Entry {}
impl Entry for SID_AND_ATTRIBUTES {}
impl Entry for LUID_AND_ATTRIBUTES {}

/// What the token of `process` allows.
pub(super) fn rights(process: HANDLE) -> Result<Rights, SpawnError> {
    of(&open(process, TOKEN_QUERY)?)
}

/// What `token`, open for queries, allows.
pub(super) fn of(token: &OwnedHandle) -> Result<Rights, SpawnError> {
    let mut ui: u32 = 0;
    get(token, TokenUIAccess, &mut ui, "read uiAccess")?;
    let mut label = Label::default();
    get(token, TokenIntegrityLevel, &mut label, "read the integrity")?;
    let held = read(token, TokenPrivileges, "read the privileges")?;
    let held = listed(&held, offset_of!(TOKEN_PRIVILEGES, Privileges)).ok_or(SpawnError::Rights)?;
    Ok(Rights {
        ui_access: ui != 0,
        level: label.level().ok_or(SpawnError::Rights)?,
        admin: can_use(token, WinBuiltinAdministratorsSid)?,
        privileged: beyond(held, change_notify()?),
    })
}

/// Whether a group with `attributes` can be used: it is not deny-only.
pub(super) const fn usable(attributes: u32) -> bool {
    attributes & SE_GROUP_USE_FOR_DENY_ONLY.cast_unsigned() == 0
}

/// Whether `held` holds a privilege other than `kept`.
pub(super) fn beyond(held: &[LUID_AND_ATTRIBUTES], kept: LUID) -> bool {
    held.iter().any(|p| p.Luid != kept)
}

/// The id of change-notify, the one privilege a restricted token keeps.
#[allow(unsafe_code, reason = "Looks a privilege's id up into a local.")]
pub(super) fn change_notify() -> Result<LUID, SpawnError> {
    let mut id = LUID::default();
    // SAFETY: a static privilege name and a local to write to.
    unsafe { LookupPrivilegeValueW(PCWSTR::null(), SE_CHANGE_NOTIFY_NAME, &raw mut id) }
        .map_err(refused("look up a privilege"))?;
    Ok(id)
}

/// Whether `token`, open for queries, holds the well-known group `kind` in a form it can use.
#[allow(unsafe_code, reason = "Compares SIDs Windows wrote with one of ours.")]
pub(super) fn can_use(token: &OwnedHandle, kind: WELL_KNOWN_SID_TYPE) -> Result<bool, SpawnError> {
    let groups = read(token, TokenGroups, "read the groups")?;
    let at = offset_of!(TOKEN_GROUPS, Groups);
    let groups: &[SID_AND_ATTRIBUTES] = listed(&groups, at).ok_or(SpawnError::Rights)?;
    let mut room = SidRoom::default();
    let (want, _) = well_known(kind, &mut room)?;
    // SAFETY: both SIDs are valid and live for the call: `want` in `room`, each group's in the list Windows wrote.
    let is = |sid: PSID| unsafe { EqualSid(want, sid) }.is_ok();
    Ok(groups.iter().any(|g| usable(g.Attributes) && is(g.Sid)))
}

/// The whole of `class` of `token`, in a pointer-aligned buffer of the size Windows asks for.
#[allow(unsafe_code, reason = "Lets Windows fill a buffer of ours.")]
fn read(
    token: &OwnedHandle,
    class: TOKEN_INFORMATION_CLASS,
    step: &'static str,
) -> Result<Vec<usize>, SpawnError> {
    let mut len = 0;
    // SAFETY: a size query with no buffer; it fails by design and only writes `len`.
    let _ = unsafe { GetTokenInformation(raw(token), class, None, 0, &raw mut len) };
    let bytes = usize::try_from(len).map_err(|_| SpawnError::Rights)?;
    let mut buf = vec![0_usize; bytes.div_ceil(size_of::<usize>())];
    let at = Some(buf.as_mut_ptr().cast());
    // SAFETY: `buf` holds at least `len` writable bytes, pointer-aligned, for the call; Windows writes no more.
    unsafe { GetTokenInformation(raw(token), class, at, len, &raw mut len) }
        .map_err(refused(step))?;
    Ok(buf)
}

/// The entries of a list Windows wrote into `buf`: a `u32` count first, then that many `E`s from byte `at`. `None`
/// when they would not fit in `buf` or would sit out of line.
#[allow(unsafe_code, reason = "Views a list Windows wrote into our buffer.")]
pub(super) fn listed<E: Entry>(buf: &[usize], at: usize) -> Option<&[E]> {
    let bytes = size_of_val(buf);
    let lined = align_of::<E>() <= align_of::<usize>() && at.is_multiple_of(align_of::<E>());
    if !lined || bytes < size_of::<u32>() {
        return None;
    }
    // SAFETY: `buf` starts with at least a `u32`'s initialised bytes, aligned as a `u32` needs.
    let count = usize::try_from(unsafe { buf.as_ptr().cast::<u32>().read() }).ok()?;
    let end = count.checked_mul(size_of::<E>())?.checked_add(at)?;
    if end > bytes {
        return None;
    }
    // SAFETY: the `count` entries lie wholly inside `buf` from byte `at`, aligned for `E`; every byte is initialised,
    // any bytes are a valid `E` (`Entry`), and the slice borrows `buf`.
    Some(unsafe { std::slice::from_raw_parts(buf.as_ptr().cast::<u8>().add(at).cast(), count) })
}
