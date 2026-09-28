//! One HTTP request through WinHTTP: to the local server, or over HTTPS to the opt-in cloud engine.

use std::ffi::c_void;

use windows::Win32::Networking::WinHttp::{
    WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_ACCESS_TYPE_NO_PROXY, WINHTTP_FLAG_SECURE,
    WINHTTP_OPEN_REQUEST_FLAGS, WINHTTP_OPTION_REDIRECT_POLICY,
    WINHTTP_OPTION_REDIRECT_POLICY_NEVER, WINHTTP_QUERY_FLAG_NUMBER, WINHTTP_QUERY_STATUS_CODE,
    WinHttpCloseHandle, WinHttpConnect, WinHttpOpen, WinHttpOpenRequest, WinHttpQueryHeaders,
    WinHttpReadData, WinHttpReceiveResponse, WinHttpSendRequest, WinHttpSetOption,
    WinHttpSetTimeouts,
};
use windows::core::{HSTRING, PCWSTR, w};

/// Who we say we are.
const AGENT: PCWSTR = w!("KeyXtend-spike-worker");
/// Sends a clip.
const POST: PCWSTR = w!("POST");
/// Asks whether the server is ready.
const GET: PCWSTR = w!("GET");
/// Largest answer read, in bytes; a transcript is far smaller.
const MAX_REPLY: usize = 1024 * 1024;
/// Bytes read per call.
const CHUNK: usize = 16 * 1024;
/// HTTP OK.
pub const HTTP_OK: u32 = 200;
/// Ends each header line, and each line of a form body.
pub const CRLF: &str = "\r\n";
/// Start of the header line that names a body's type, in a request or a form part.
pub const CONTENT_TYPE: &str = "Content-Type: ";
/// Start of the header line that carries an API key.
const BEARER: &str = "Authorization: Bearer ";

/// One request.
pub struct Request<'a> {
    /// Host name or address.
    pub host: &'a str,
    /// Port.
    pub port: u16,
    /// HTTPS when true.
    pub secure: bool,
    /// Path on the host.
    pub path: &'a str,
    /// Type of the body; none for a GET.
    pub content_type: Option<&'a str>,
    /// API key sent as a bearer token.
    pub bearer: Option<&'a str>,
    /// The body; empty for a GET.
    pub body: &'a [u8],
    /// Longest wait for each step, in ms.
    pub timeout_ms: u64,
}

/// The answer.
pub struct Reply {
    /// HTTP status code.
    pub status: u32,
    /// The body, at most `MAX_REPLY` bytes.
    pub body: Vec<u8>,
}

/// A WinHTTP handle, closed once when dropped.
struct Handle(*mut c_void);

impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: the handle came from WinHTTP, is not null, and is closed only here.
        let _ = unsafe { WinHttpCloseHandle(self.0) };
    }
}

/// `h` as an owned handle, or the last error for `what`.
fn owned(h: *mut c_void, what: &str) -> Result<Handle, String> {
    if h.is_null() {
        Err(format!("{what}: {}", windows::core::Error::from_thread()))
    } else {
        Ok(Handle(h))
    }
}

/// Sends `r` as a POST and reads the answer.
pub fn post(r: &Request) -> Result<Reply, String> {
    exchange(r, POST, MAX_REPLY)
}

/// Sends `r` as a GET and reads the answer.
pub fn get(r: &Request) -> Result<Reply, String> {
    exchange(r, GET, MAX_REPLY)
}

/// One request and an answer of at most `max` bytes; the handles close in reverse order.
fn exchange(r: &Request, verb: PCWSTR, max: usize) -> Result<Reply, String> {
    let (_session, _connect, req) = open(r, verb)?;
    send(&req, r)?;
    Ok(Reply {
        status: status(&req)?,
        body: read(&req, max)?,
    })
}

/// The header lines for `r`, each ending in CRLF.
fn headers(r: &Request) -> String {
    let line = |start: &str, v: Option<&str>| v.map(|v| format!("{start}{v}{CRLF}"));
    [line(BEARER, r.bearer), line(CONTENT_TYPE, r.content_type)]
        .into_iter()
        .flatten()
        .collect()
}

/// A session, a connection and a request that never follows redirects, so a key cannot leak to another host.
fn open(r: &Request, verb: PCWSTR) -> Result<(Handle, Handle, Handle), String> {
    let (access, flags) = if r.secure {
        (WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, WINHTTP_FLAG_SECURE)
    } else {
        (WINHTTP_ACCESS_TYPE_NO_PROXY, WINHTTP_OPEN_REQUEST_FLAGS(0))
    };
    // SAFETY: constant agent string, no proxy names; the handle is owned.
    let session = owned(
        unsafe { WinHttpOpen(AGENT, access, PCWSTR::null(), PCWSTR::null(), 0) },
        "http open",
    )?;
    let ms = i32::try_from(r.timeout_ms).unwrap_or(i32::MAX);
    // SAFETY: `session` is open.
    unsafe { WinHttpSetTimeouts(session.0, ms, ms, ms, ms) }
        .map_err(|e| format!("http timeouts: {e}"))?;
    let host = HSTRING::from(r.host);
    // SAFETY: `session` is open and `host` lives for the call.
    let connect = owned(
        unsafe { WinHttpConnect(session.0, &host, r.port, 0) },
        "http connect",
    )?;
    let path = HSTRING::from(r.path);
    // SAFETY: `connect` is open; default version, no referrer, no accept types.
    let req = unsafe {
        WinHttpOpenRequest(
            connect.0,
            verb,
            &path,
            PCWSTR::null(),
            PCWSTR::null(),
            std::ptr::null(),
            flags,
        )
    };
    let req = owned(req, "http request")?;
    no_redirects(&req)?;
    Ok((session, connect, req))
}

/// Makes `req` hand back a 3xx as it is instead of following it.
fn no_redirects(req: &Handle) -> Result<(), String> {
    let never = WINHTTP_OPTION_REDIRECT_POLICY_NEVER.to_le_bytes();
    // SAFETY: `req` is open; the option takes a DWORD, which `never` holds.
    unsafe { WinHttpSetOption(Some(req.0), WINHTTP_OPTION_REDIRECT_POLICY, Some(&never)) }
        .map_err(|e| format!("http redirects: {e}"))
}

/// Sends the headers and body, then waits for the answer's headers.
fn send(req: &Handle, r: &Request) -> Result<(), String> {
    let wide: Vec<u16> = headers(r).encode_utf16().collect();
    let headers = (!wide.is_empty()).then_some(wide.as_slice());
    let len = u32::try_from(r.body.len()).map_err(|_| "the clip is too large".to_string())?;
    let body = (!r.body.is_empty()).then(|| r.body.as_ptr().cast::<c_void>());
    // SAFETY: `req` is open; `headers` and `r.body` stay borrowed until after WinHttpReceiveResponse below,
    // which is as long as WinHTTP reads them in synchronous mode.
    unsafe { WinHttpSendRequest(req.0, headers, body, len, len, 0) }
        .map_err(|e| format!("http send: {e}"))?;
    // SAFETY: `req` is open and was sent.
    unsafe { WinHttpReceiveResponse(req.0, std::ptr::null_mut()) }
        .map_err(|e| format!("http answer: {e}"))
}

/// The answer's status code.
fn status(req: &Handle) -> Result<u32, String> {
    let (mut code, mut size) = (0u32, size_of::<u32>() as u32);
    let buf = Some((&mut code as *mut u32).cast::<c_void>());
    // SAFETY: `req` has an answer; `code` is a u32 of `size` bytes.
    unsafe {
        WinHttpQueryHeaders(
            req.0,
            WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
            PCWSTR::null(),
            buf,
            &mut size,
            std::ptr::null_mut(),
        )
    }
    .map_err(|e| format!("http status: {e}"))?;
    Ok(code)
}

/// The answer's body, refused past `max` bytes.
fn read(req: &Handle, max: usize) -> Result<Vec<u8>, String> {
    let (mut out, mut buf) = (Vec::new(), vec![0u8; CHUNK]);
    loop {
        let mut n = 0u32;
        // SAFETY: `buf` has `CHUNK` writable bytes; `n` receives the count.
        unsafe { WinHttpReadData(req.0, buf.as_mut_ptr().cast(), CHUNK as u32, &mut n) }
            .map_err(|e| format!("http read: {e}"))?;
        if n == 0 {
            return Ok(out);
        }
        out.extend_from_slice(&buf[..n as usize]);
        if out.len() > max {
            return Err("the answer is too large".to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// Reads one request and answers `reply`; returns what was received.
    fn serve_once(
        l: TcpListener,
        reply: impl AsRef<[u8]> + Send + 'static,
    ) -> std::thread::JoinHandle<String> {
        std::thread::spawn(move || {
            let (mut s, _) = l.accept().expect("a client");
            let mut got = Vec::new();
            let mut buf = [0u8; 4096];
            while !String::from_utf8_lossy(&got).ends_with("BODY") {
                let n = s.read(&mut buf).expect("reads");
                assert!(n > 0, "client closed");
                got.extend_from_slice(&buf[..n]);
            }
            s.write_all(reply.as_ref()).expect("answers");
            String::from_utf8_lossy(&got).into_owned()
        })
    }

    /// A listener on a free local port, and that port.
    fn listen() -> (TcpListener, u16) {
        let l = TcpListener::bind("127.0.0.1:0").expect("binds");
        let port = l.local_addr().expect("addr").port();
        (l, port)
    }

    fn request<'a>(port: u16, body: &'a [u8]) -> Request<'a> {
        Request {
            host: "127.0.0.1",
            port,
            secure: false,
            path: "/inference",
            content_type: Some("text/plain"),
            bearer: Some("k1"),
            body,
            timeout_ms: 5000,
        }
    }

    /// An answer: `head` is the status and any extra header lines, without the last CRLF.
    fn answer(head: &str, body: &str) -> String {
        let len = body.len();
        format!("HTTP/1.1 {head}\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n{body}")
    }

    #[test]
    fn a_post_sends_the_body_and_reads_the_answer() {
        let (l, port) = listen();
        let server = serve_once(l, answer("201 Created", "ok"));
        let reply = post(&request(port, b"BODY")).expect("answered");
        assert_eq!(
            (reply.status, reply.body.as_slice()),
            (201, b"ok".as_slice())
        );
        let seen = server.join().expect("server ran");
        assert!(seen.starts_with("POST /inference HTTP/1.1\r\n"), "{seen}");
        assert!(seen.contains("Authorization: Bearer k1\r\n"), "{seen}");
        assert!(seen.contains("Content-Type: text/plain\r\n"), "{seen}");
    }

    #[test]
    fn the_header_block_holds_only_the_lines_given() {
        let mut r = request(1, b"");
        let both = "Authorization: Bearer k1\r\nContent-Type: text/plain\r\n";
        assert_eq!(headers(&r), both);
        r.bearer = None;
        assert_eq!(headers(&r), "Content-Type: text/plain\r\n");
        r.content_type = None;
        assert_eq!(headers(&r), "", "a GET has no extra lines");
    }

    #[test]
    fn a_redirect_is_handed_back_and_never_followed() {
        let (other, to) = listen();
        other.set_nonblocking(true).expect("non-blocking");
        let (l, port) = listen();
        let moved = format!("302 Found\r\nLocation: http://127.0.0.1:{to}/inference");
        let server = serve_once(l, answer(&moved, ""));
        let status = post(&request(port, b"BODY")).map(|r| r.status);
        assert_eq!(status, Ok(302));
        server.join().expect("server ran");
        let followed = other.accept().map(|_| ()).map_err(|e| e.kind());
        assert_eq!(
            followed,
            Err(std::io::ErrorKind::WouldBlock),
            "no second host"
        );
    }

    #[test]
    fn an_answer_past_the_cap_is_an_error() {
        let body = "0123456789";
        let (l, port) = listen();
        let server = serve_once(l, answer("200 OK", body));
        let at_cap = exchange(&request(port, b"BODY"), POST, body.len());
        assert_eq!(at_cap.map(|r| r.body.len()), Ok(body.len()));
        server.join().expect("server ran");
        let (l, port) = listen();
        let server = serve_once(l, answer("200 OK", body));
        let past = exchange(&request(port, b"BODY"), POST, body.len() - 1);
        assert!(past.is_err_and(|e| e.contains("too large")));
        server.join().expect("server ran");
    }

    #[test]
    fn a_closed_port_is_an_error_not_a_hang() {
        let (_, port) = listen();
        assert!(post(&request(port, b"BODY")).is_err());
    }
}
