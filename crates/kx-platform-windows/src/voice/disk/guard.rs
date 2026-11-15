//! A models folder made with its rights in one step: full control for the user we run as and for the system,
//! protected so nothing its drive passes down applies, as a folder at a second drive's root often lets every user
//! change it. A folder already there is never touched.

use std::ffi::c_void;
use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Foundation::{HANDLE, HLOCAL, LocalFree, WIN32_ERROR};
use windows::Win32::Security::Authorization::{
    ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION_1,
};
use windows::Win32::Security::{
    GetTokenInformation, PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES, TOKEN_QUERY, TOKEN_USER,
    TokenUser,
};
use windows::Win32::Storage::FileSystem::CreateDirectoryW;
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::{PCWSTR, PWSTR};

/// The folder's rights in Windows' own notation: protected, the user and the system with full control, passed to
/// every folder and file inside; `{user}` is the user's SID.
const RIGHTS: &str = "D:P(A;OICI;FA;;;{user})(A;OICI;FA;;;SY)";
/// The mark `RIGHTS` names the user with.
const USER: &str = "{user}";

/// Makes `folder`, whose parent exists, with the rights `RIGHTS` names; `AlreadyExists` when it is there.
#[allow(unsafe_code, reason = "Makes a folder of ours with its rights.")]
pub(crate) fn make_guarded(folder: &Path) -> io::Result<()> {
    let rights = wide(RIGHTS.replace(USER, &user()?).as_ref());
    let mut made = PSECURITY_DESCRIPTOR::default();
    // SAFETY: `rights` is NUL-ended and outlives the call, which allocates the descriptor and writes `made`.
    unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            PCWSTR(rights.as_ptr()),
            SDDL_REVISION_1,
            &raw mut made,
            None,
        )
    }?;
    let made = Local(made.0);
    let attributes = SECURITY_ATTRIBUTES {
        nLength: u32::try_from(size_of::<SECURITY_ATTRIBUTES>()).unwrap_or(u32::MAX),
        lpSecurityDescriptor: made.0,
        bInheritHandle: false.into(),
    };
    let name = wide(folder.as_os_str());
    // SAFETY: `name` is NUL-ended, and `attributes` and the descriptor in `made` outlive the call.
    let created = unsafe { CreateDirectoryW(PCWSTR(name.as_ptr()), Some(&raw const attributes)) };
    created.map_err(|e| os_error(&e))
}

/// `e` as the system error it carries, so its kind reads as `AlreadyExists`, `NotFound` or `PermissionDenied`.
fn os_error(e: &windows::core::Error) -> io::Error {
    match WIN32_ERROR::from_error(e) {
        Some(code) => io::Error::from_raw_os_error(code.0.cast_signed()),
        None => io::Error::other(e.clone()),
    }
}

/// The SID of the user we run as, in Windows' text form.
#[allow(unsafe_code, reason = "Reads our own token's user.")]
fn user() -> io::Result<String> {
    let mut token = HANDLE::default();
    // SAFETY: our own process's pseudo-handle, and a local Windows writes once.
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &raw mut token) }?;
    let token = Owned(token);
    let mut len = 0;
    // SAFETY: a size query with no buffer; it fails by design and only writes `len`.
    let _ = unsafe { GetTokenInformation(token.0, TokenUser, None, 0, &raw mut len) };
    let bytes = usize::try_from(len).map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;
    let mut buf = vec![0_usize; bytes.div_ceil(size_of::<usize>())];
    // SAFETY: `buf` holds at least `len` writable bytes, pointer-aligned, and Windows writes no more.
    unsafe {
        GetTokenInformation(
            token.0,
            TokenUser,
            Some(buf.as_mut_ptr().cast()),
            len,
            &raw mut len,
        )
    }?;
    // SAFETY: Windows wrote a `TOKEN_USER` at the start of the aligned buffer, which outlives the read.
    let sid = unsafe { (*buf.as_ptr().cast::<TOKEN_USER>()).User.Sid };
    let mut text = PWSTR::null();
    // SAFETY: the SID lives in `buf`; Windows allocates the text and writes `text`.
    unsafe { ConvertSidToStringSidW(sid, &raw mut text) }?;
    let text = Local(text.0.cast());
    // SAFETY: a NUL-ended string Windows made, freed when `text` drops.
    unsafe { PWSTR(text.0.cast()).to_string() }
        .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))
}

/// `text` as UTF-16 with an ending NUL.
fn wide(text: &std::ffi::OsStr) -> Vec<u16> {
    text.encode_wide().chain([0]).collect()
}

/// Memory Windows allocated for us, freed once when dropped.
struct Local(*mut c_void);

impl Drop for Local {
    #[allow(unsafe_code, reason = "Frees memory Windows allocated for us.")]
    fn drop(&mut self) {
        // SAFETY: allocated by Windows for this call's result and freed once, here.
        let _ = unsafe { LocalFree(Some(HLOCAL(self.0))) };
    }
}

/// A handle of ours, closed once when dropped.
struct Owned(HANDLE);

impl Drop for Owned {
    #[allow(unsafe_code, reason = "Closes our own token handle.")]
    fn drop(&mut self) {
        // SAFETY: opened by `user` and closed once, here.
        let _ = unsafe { windows::Win32::Foundation::CloseHandle(self.0) };
    }
}
