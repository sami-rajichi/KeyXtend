//! An answer's status and the headers the rules read, each read into a buffer no longer than the longest address.

use super::codes;
use super::handle::Handle;
use crate::{Head, NetError};
use kx_ipc::MAX_URL_BYTES;
use std::ffi::c_void;
use windows::Win32::Foundation::{ERROR_INSUFFICIENT_BUFFER, WIN32_ERROR};
use windows::Win32::Networking::WinHttp::{
    ERROR_WINHTTP_HEADER_NOT_FOUND, WINHTTP_QUERY_CONTENT_LENGTH, WINHTTP_QUERY_CONTENT_RANGE,
    WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_LOCATION, WINHTTP_QUERY_STATUS_CODE,
    WinHttpQueryHeaders,
};
use windows::core::PCWSTR;

/// UTF-16 units a header value may take with its NUL: the longest address a redirect may name.
const HEAD_UNITS: usize = MAX_URL_BYTES + 1;

/// The status and headers of `req`'s answer.
///
/// # Errors
/// `Server` for a status that is no HTTP code, or a header too long or not UTF-16; `WinHTTP`'s error otherwise.
pub(super) fn read(req: &Handle) -> Result<Head, NetError> {
    Ok(Head {
        status: status(req)?,
        location: text(req, WINHTTP_QUERY_LOCATION)?,
        length: text(req, WINHTTP_QUERY_CONTENT_LENGTH)?,
        range: text(req, WINHTTP_QUERY_CONTENT_RANGE)?,
    })
}

/// The answer's status code.
#[allow(unsafe_code, reason = "Reads a WinHTTP answer's status code.")]
fn status(req: &Handle) -> Result<u16, NetError> {
    let (mut code, mut size) = (0u32, u32::try_from(size_of::<u32>()).unwrap_or_default());
    let buf = Some((&raw mut code).cast::<c_void>());
    let what = WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER;
    let index = std::ptr::null_mut();
    // SAFETY: `req` has an answer; `code` is a u32 of `size` bytes.
    unsafe { WinHttpQueryHeaders(req.raw(), what, PCWSTR::null(), buf, &raw mut size, index) }
        .map_err(|e| codes::of(&e))?;
    u16::try_from(code).map_err(|_| NetError::Server)
}

/// The header `what` names, or none when the answer has none.
#[allow(unsafe_code, reason = "Reads a WinHTTP answer's header.")]
fn text(req: &Handle, what: u32) -> Result<Option<String>, NetError> {
    let mut wide = vec![0u16; HEAD_UNITS];
    let mut size = u32::try_from(HEAD_UNITS * size_of::<u16>()).map_err(|_| NetError::Internal)?;
    let buf = Some(wide.as_mut_ptr().cast::<c_void>());
    let index = std::ptr::null_mut();
    // SAFETY: `req` has an answer; `wide` holds `size` bytes, and WinHTTP writes no more.
    let read =
        unsafe { WinHttpQueryHeaders(req.raw(), what, PCWSTR::null(), buf, &raw mut size, index) };
    match read.map_err(|e| WIN32_ERROR::from_error(&e).map(|c| c.0)) {
        Ok(()) => {
            let units = wide
                .get(..size as usize / size_of::<u16>())
                .ok_or(NetError::Server)?;
            String::from_utf16(units)
                .map(Some)
                .map_err(|_| NetError::Server)
        }
        Err(Some(ERROR_WINHTTP_HEADER_NOT_FOUND)) => Ok(None),
        Err(Some(c)) if c == ERROR_INSUFFICIENT_BUFFER.0 => Err(NetError::Server),
        Err(Some(c)) => Err(codes::kind(c)),
        Err(None) => Err(NetError::Internal),
    }
}
