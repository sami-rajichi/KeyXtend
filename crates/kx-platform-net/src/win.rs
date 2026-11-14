//! The `WinHTTP` adapter, moved from the spike's `http.rs`: one session per request, TLS and the system's proxy from
//! Windows, redirects handed back unfollowed, no cookies, and every wait capped by the hop's stall time.

mod codes;
mod handle;
mod head;
#[cfg(test)]
mod tests;

use self::handle::{Handle, owned, set};
use crate::range::range_header;
use crate::{AGENT, Body, Hop, Http, NetError, Reply, Target, wait_ms};
use windows::Win32::Networking::WinHttp::{
    WINHTTP_ACCESS_TYPE, WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_DISABLE_COOKIES,
    WINHTTP_FLAG_SECURE, WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2, WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_3,
    WINHTTP_OPTION_DISABLE_FEATURE, WINHTTP_OPTION_RECEIVE_RESPONSE_TIMEOUT,
    WINHTTP_OPTION_REDIRECT_POLICY, WINHTTP_OPTION_REDIRECT_POLICY_NEVER,
    WINHTTP_OPTION_SECURE_PROTOCOLS, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest,
    WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest, WinHttpSetTimeouts,
};
use windows::core::{HSTRING, PCWSTR, w};

/// The only verb this crate sends.
const GET: PCWSTR = w!("GET");
/// The TLS versions a session allows: 1.2 and 1.3.
const TLS: u32 = WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2 | WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_3;
/// The TLS version a Windows without 1.3 in `WinHTTP` allows.
const TLS_OLD: u32 = WINHTTP_FLAG_SECURE_PROTOCOL_TLS1_2;

/// Sends GET requests through `WinHTTP`, over HTTPS only.
#[derive(Clone, Copy, Debug)]
pub struct WinHttp {
    /// How the session reaches a server: through the system's proxy settings, or straight in tests.
    access: WINHTTP_ACCESS_TYPE,
}

impl WinHttp {
    /// A client that reaches servers as the system's proxy settings say.
    #[must_use]
    pub fn new() -> Self {
        Self {
            access: WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
        }
    }

    /// One session with `AGENT`, every wait capped at `ms`, and TLS 1.2 or newer.
    #[allow(unsafe_code, reason = "Opens a WinHTTP session and sets its timeouts.")]
    fn session(&self, ms: i32) -> Result<Handle, NetError> {
        let agent = HSTRING::from(AGENT);
        let none = PCWSTR::null();
        // SAFETY: `agent` lives for the call, no proxy is named, and the handle is owned at once.
        let session = owned(unsafe { WinHttpOpen(&agent, self.access, none, none, 0) })?;
        // SAFETY: `session` is open.
        unsafe { WinHttpSetTimeouts(session.raw(), ms, ms, ms, ms) }.map_err(|e| codes::of(&e))?;
        set(&session, WINHTTP_OPTION_SECURE_PROTOCOLS, TLS)
            .or_else(|_| set(&session, WINHTTP_OPTION_SECURE_PROTOCOLS, TLS_OLD))?;
        Ok(session)
    }
}

impl Default for WinHttp {
    fn default() -> Self {
        Self::new()
    }
}

impl Http for WinHttp {
    fn send(&self, hop: &Hop<'_>) -> Result<Reply, NetError> {
        let wait = wait_ms(hop.stall_s);
        let session = self.session(i32::try_from(wait).unwrap_or(i32::MAX))?;
        let connect = connect(&session, hop.target)?;
        let request = request(&connect, hop.target, wait)?;
        send(&request, hop.from)?;
        let head = head::read(&request)?;
        let body = WinBody {
            request,
            _connect: connect,
            _session: session,
        };
        Ok(Reply {
            head,
            body: Box::new(body),
        })
    }
}

/// A connection to `target`'s host and port.
#[allow(unsafe_code, reason = "Opens a WinHTTP connection.")]
fn connect(session: &Handle, target: &Target) -> Result<Handle, NetError> {
    let host = HSTRING::from(target.host());
    // SAFETY: `session` is open and `host` lives for the call; the handle is owned at once.
    owned(unsafe { WinHttpConnect(session.raw(), &host, target.port(), 0) })
}

/// A secure GET of `target`'s path that hands redirects back, keeps no cookies and waits `wait` ms for its head.
#[allow(unsafe_code, reason = "Opens a WinHTTP request.")]
fn request(connect: &Handle, target: &Target, wait: u32) -> Result<Handle, NetError> {
    let (path, none) = (HSTRING::from(target.path()), PCWSTR::null());
    let accept = std::ptr::null();
    // SAFETY: `connect` is open and `path` lives for the call; the default version, no referrer, no accept types.
    let req = unsafe {
        WinHttpOpenRequest(
            connect.raw(),
            GET,
            &path,
            none,
            none,
            accept,
            WINHTTP_FLAG_SECURE,
        )
    };
    let req = owned(req)?;
    let never = WINHTTP_OPTION_REDIRECT_POLICY_NEVER;
    set(&req, WINHTTP_OPTION_REDIRECT_POLICY, never)?;
    set(
        &req,
        WINHTTP_OPTION_DISABLE_FEATURE,
        WINHTTP_DISABLE_COOKIES,
    )?;
    set(&req, WINHTTP_OPTION_RECEIVE_RESPONSE_TIMEOUT, wait)?;
    Ok(req)
}

/// Sends `req`, asking from byte `from` on when it is past 0, and waits for the answer's head.
#[allow(
    unsafe_code,
    reason = "Sends a WinHTTP request and waits for its answer."
)]
fn send(req: &Handle, from: u64) -> Result<(), NetError> {
    let wide: Option<Vec<u16>> = range_header(from).map(|h| h.encode_utf16().collect());
    // SAFETY: `req` is open; `wide` outlives the call, and a GET sends no body.
    unsafe { WinHttpSendRequest(req.raw(), wide.as_deref(), None, 0, 0, 0) }
        .map_err(|e| codes::of(&e))?;
    // SAFETY: `req` is open and was sent.
    unsafe { WinHttpReceiveResponse(req.raw(), std::ptr::null_mut()) }.map_err(|e| codes::of(&e))
}

/// An answer's body; its handles close request first, then connection, then session.
struct WinBody {
    /// The request the body is read from.
    request: Handle,
    /// Kept open while the body is read.
    _connect: Handle,
    /// Kept open while the body is read.
    _session: Handle,
}

impl Body for WinBody {
    #[allow(unsafe_code, reason = "Reads a WinHTTP answer's body.")]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, NetError> {
        let len = u32::try_from(buf.len()).unwrap_or(u32::MAX);
        let mut n = 0u32;
        // SAFETY: `buf` has at least `len` writable bytes, `n` takes the count, and the request is open.
        unsafe { WinHttpReadData(self.request.raw(), buf.as_mut_ptr().cast(), len, &raw mut n) }
            .map_err(|e| codes::of(&e))?;
        Ok(n as usize)
    }
}
