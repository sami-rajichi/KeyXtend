//! A call's head and body: its headers go as one block in a wiped buffer, as a value may be a key, and its body in
//! chunks, so each chunk waits at most the call's stall time rather than the whole upload.

use super::codes;
use super::handle::Handle;
use crate::{Call, NetError};
use windows::Win32::Networking::WinHttp::{
    WinHttpReceiveResponse, WinHttpSendRequest, WinHttpWriteData,
};
use zeroize::Zeroizing;

/// Between a header's name and its value.
const COLON: &str = ": ";
/// Ends each header line.
const CRLF: &str = "\r\n";
/// The most body bytes one write sends.
const CHUNK: usize = 64 * 1024;

/// The call's headers as one UTF-16 block of `name: value` lines, wiped when dropped; `Call::check` made each byte
/// visible ASCII or a space.
fn block(call: &Call<'_>) -> Zeroizing<Vec<u16>> {
    let mut wide = Zeroizing::new(Vec::new());
    for f in call.fields {
        wide.extend(f.name.encode_utf16());
        wide.extend(COLON.encode_utf16());
        wide.extend(f.value.iter().map(|b| u16::from(*b)));
        wide.extend(CRLF.encode_utf16());
    }
    wide
}

/// Sends `call`'s head, then its body in chunks, and waits for the answer's head.
///
/// # Errors
/// `Internal` for a body past 4 GiB; `WinHTTP`'s error otherwise.
#[allow(unsafe_code, reason = "Sends a WinHTTP request with its body.")]
pub(super) fn send(req: &Handle, call: &Call<'_>) -> Result<(), NetError> {
    let total = u32::try_from(call.body.len()).map_err(|_| NetError::Internal)?;
    let head = block(call);
    let head = (!head.is_empty()).then_some(&head[..]);
    // SAFETY: `req` is open, `head` outlives the call, and the body follows in writes of `total` bytes in all.
    unsafe { WinHttpSendRequest(req.raw(), head, None, 0, total, 0) }.map_err(|e| codes::of(&e))?;
    for chunk in call.body.chunks(CHUNK) {
        write(req, chunk)?;
    }
    // SAFETY: `req` is open and its body is sent.
    unsafe { WinHttpReceiveResponse(req.raw(), std::ptr::null_mut()) }.map_err(|e| codes::of(&e))
}

/// Writes all of `chunk`, in as many writes as `WinHTTP` takes.
#[allow(unsafe_code, reason = "Writes a WinHTTP request's body.")]
fn write(req: &Handle, mut chunk: &[u8]) -> Result<(), NetError> {
    while !chunk.is_empty() {
        let len = u32::try_from(chunk.len()).map_err(|_| NetError::Internal)?;
        let mut n = 0u32;
        let data = Some(chunk.as_ptr().cast());
        // SAFETY: `req` was sent; `chunk` holds `len` readable bytes and `n` takes the count written.
        unsafe { WinHttpWriteData(req.raw(), data, len, &raw mut n) }.map_err(|e| codes::of(&e))?;
        let n = (n as usize).min(chunk.len());
        if n == 0 {
            return Err(NetError::Net);
        }
        chunk = &chunk[n..];
    }
    Ok(())
}
