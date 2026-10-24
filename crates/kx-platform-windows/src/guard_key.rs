//! The data key's guard on Windows: DPAPI wraps it for the signed-in user, so only this user on this PC unwraps it.

use std::ptr;
use std::sync::atomic::{Ordering, compiler_fence};

use kx_crypto::{GuardError, KeyGuard, Zeroizing};
use windows::Win32::Foundation::{
    ERROR_INVALID_DATA, ERROR_INVALID_PARAMETER, HLOCAL, LocalFree, NTE_BAD_DATA, NTE_BAD_KEY_STATE,
};
use windows::Win32::Security::Cryptography::{
    CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
};
use windows::core::{HRESULT, PCWSTR};

/// Mixed into every wrap and needed to unwrap, so another program's DPAPI data never passes for our key.
/// Changing it makes every wrapped key unreadable.
const ENTROPY: &[u8] = b"KeyXtend sealed data key";

/// DPAPI's answers for data it declines: too short, invalid, damaged or another user's; any other is `Unavailable`.
/// Our other arguments never change, so an invalid parameter can only be the data.
const REFUSED: [HRESULT; 4] = [
    ERROR_INVALID_PARAMETER.to_hresult(),
    ERROR_INVALID_DATA.to_hresult(),
    NTE_BAD_DATA,
    NTE_BAD_KEY_STATE,
];

/// The key guard on Windows: DPAPI for the current user, which never shows a prompt.
#[derive(Clone, Copy, Debug, Default)]
pub struct WinKeyGuard;

impl KeyGuard for WinKeyGuard {
    fn wrap(&self, key: &[u8]) -> Result<Vec<u8>, GuardError> {
        Ok(protect(key, ENTROPY)?.bytes().to_vec())
    }

    fn unwrap(&self, wrapped: &[u8]) -> Result<Zeroizing<Vec<u8>>, GuardError> {
        Ok(Zeroizing::new(
            unprotect(wrapped, ENTROPY)?.bytes().to_vec(),
        ))
    }
}

/// `key` wrapped by DPAPI for the current user under `entropy`.
#[allow(unsafe_code, reason = "Asks DPAPI to wrap bytes of ours.")]
fn protect(key: &[u8], entropy: &[u8]) -> Result<Output, GuardError> {
    let (data, extra) = (blob(key)?, blob(entropy)?);
    let mut out = Output::default();
    // SAFETY: both blobs point at live slices DPAPI only reads; `out` takes the buffer DPAPI allocates and frees it.
    unsafe {
        CryptProtectData(
            ptr::from_ref(&data),
            PCWSTR::null(),
            Some(ptr::from_ref(&extra)),
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            ptr::from_mut(&mut out.0),
        )
    }
    .map_err(|e| guard_error(e.code()))?;
    Ok(out)
}

/// The bytes DPAPI wrapped as `wrapped` under `entropy`, for the current user only.
#[allow(unsafe_code, reason = "Asks DPAPI to unwrap bytes of ours.")]
fn unprotect(wrapped: &[u8], entropy: &[u8]) -> Result<Output, GuardError> {
    let (data, extra) = (blob(wrapped)?, blob(entropy)?);
    let mut out = Output::default();
    // SAFETY: as in `protect`; no description is asked for, so DPAPI allocates nothing else.
    unsafe {
        CryptUnprotectData(
            ptr::from_ref(&data),
            None,
            Some(ptr::from_ref(&extra)),
            None,
            None,
            CRYPTPROTECT_UI_FORBIDDEN,
            ptr::from_mut(&mut out.0),
        )
    }
    .map_err(|e| guard_error(e.code()))?;
    Ok(out)
}

/// DPAPI's view of `bytes`; `Refused` past `u32::MAX` bytes, which DPAPI cannot take.
fn blob(bytes: &[u8]) -> Result<CRYPT_INTEGER_BLOB, GuardError> {
    Ok(CRYPT_INTEGER_BLOB {
        cbData: u32::try_from(bytes.len()).map_err(|_| GuardError::Refused)?,
        pbData: bytes.as_ptr().cast_mut(),
    })
}

/// How a DPAPI failure reads to the sealed files.
fn guard_error(code: HRESULT) -> GuardError {
    if REFUSED.contains(&code) {
        GuardError::Refused
    } else {
        GuardError::Unavailable
    }
}

/// A buffer DPAPI allocated, wiped and freed once when dropped.
#[derive(Default)]
struct Output(CRYPT_INTEGER_BLOB);

impl Output {
    /// The bytes DPAPI wrote.
    #[allow(unsafe_code, reason = "Reads a buffer DPAPI filled.")]
    fn bytes(&self) -> &[u8] {
        let CRYPT_INTEGER_BLOB { cbData, pbData } = self.0;
        if pbData.is_null() {
            return &[];
        }
        // SAFETY: DPAPI wrote `cbData` bytes at `pbData`, which stay ours until `self` drops.
        unsafe { std::slice::from_raw_parts(pbData, cbData as usize) }
    }
}

impl Drop for Output {
    #[allow(unsafe_code, reason = "Wipes and frees a buffer DPAPI allocated.")]
    fn drop(&mut self) {
        let CRYPT_INTEGER_BLOB { cbData, pbData } = self.0;
        if pbData.is_null() {
            return;
        }
        // SAFETY: as in `bytes`, and nothing else holds the buffer while it is wiped.
        wipe(unsafe { std::slice::from_raw_parts_mut(pbData, cbData as usize) });
        // SAFETY: frees, once, the buffer DPAPI allocated with `LocalAlloc`.
        let _ = unsafe { LocalFree(Some(HLOCAL(pbData.cast()))) };
    }
}

/// Overwrites `bytes` with zeros in a way the optimiser keeps.
#[allow(
    unsafe_code,
    reason = "Volatile writes, which the optimiser never drops."
)]
fn wipe(bytes: &mut [u8]) {
    for byte in bytes {
        // SAFETY: `byte` is a valid, aligned and exclusive reference.
        unsafe { ptr::write_volatile(byte, 0) };
    }
    compiler_fence(Ordering::SeqCst);
}

#[cfg(test)]
mod tests;
