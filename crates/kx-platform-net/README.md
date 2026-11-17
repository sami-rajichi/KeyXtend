# kx-platform-net

Worker-only HTTPS requests (ADR-0004, ADR-0043 decision 8, ADR-0047). The worker's `fetch` job uses it to download a model file, and its `speech-net` job to send a piece of sound to a cloud service; the uiAccess `keyxtend` app must never depend on it (ARCHITECTURE.md invariant 4).

The crate builds on every system. The rules (addresses, redirects, ranges, headers, the fault maps) and the scripted fake are pure and tested everywhere; only `WinHttp` calls Windows.

## What it provides

| Part | What it does |
|---|---|
| `Http` | The port. `send` makes one GET and hands back the answer as it came, a redirect included. `get`, a provided method, runs the rules over `send`: HTTPS only, each address checked against the download's hosts before it is asked (the first by a plain entry's name, a redirect also under a `.` entry's domain), at most `MAX_HOPS` redirects followed by hand, and the range check. `deliver` makes one `Call` and never follows a redirect; `call`, a provided method, checks the call's headers first. |
| `Call`, `Field`, `Verb` | One request to one checked address: GET or POST, its header fields, its body (POST only), the answer headers to keep and `stall_s`. A field's name must be a token and its value visible ASCII, so no value can add a line; `Debug` shows names and sizes only, never a key or the body. |
| `Get`, `Answer` | One download's request (address, hosts, first byte, `stall_s`) and its answer: the address that answered, where the body starts (`from`, or 0 when the server sent the whole file, so the part restarts), the file's size when known, and the body. |
| `Hop`, `Reply`, `Head`, `Body` | One GET to one checked address, its answer, the status and the headers the rules read (`Location`, `Content-Length`, `Content-Range`, plus a call's kept ones in `extra`, read by `header`), and the body, read in order on any thread. |
| `Target` | A checked HTTPS address: a plain lower-case host, a port and the path, its `.` and `..` folders resolved, at most `MAX_URL_BYTES` written out. A redirect's `Location` is read against it as RFC 3986 §5.2 says (`join`); `named` refuses a first address no plain entry names, and `listed` a redirect's host the download does not list. |
| `Span` | A partial answer's `Content-Range`: its first and last bytes and the total. |
| `NetError` | Why a request failed, in fixed words; `From<NetError>` gives `kx_ipc::FetchFault` for a download and `kx_ipc::NetFault` for a cloud call. |
| `wait_ms`, `AGENT`, `MAX_HOPS` | The stall time in ms, held within `kx_ipc::STALL_S` (WinHTTP reads 0 as no limit), the user agent (the product name only) and the redirect limit, each named once. |
| `WinHttp` (Windows) | The adapter, moved from the spike's `http.rs`: one WinHTTP session per request with the system's proxy, TLS 1.2 or 1.3 from Windows, automatic redirects off, no cookies, and every wait (name, connection, send, head, each read) capped at `wait_ms(stall_s)`. A POST writes its body in chunks, and its header lines live in a wiped buffer. Its error codes map to `NetError` through one table (`win/codes.rs`). |
| `fake::FakeHttp` (feature `fake`, and the crate's tests) | A scripted server: each request takes the next act and is kept in order, a call with its fields and body (`called`). A body step can give bytes, fail, or stay quiet for some ms; a quiet at or past `wait_ms` reads as a stall, so no test sleeps. |

## Faults

| `NetError` | `FetchFault` | `NetFault` |
|---|---|---|
| `NotHttps`: an address or a redirect is not a plain HTTPS address | `NotHttps` | `Range` |
| `Header`: a header field would break the request | `Internal` | `Range` |
| `Unlisted`: an address or a redirect names a host the download does not list | `Redirect` | `Strange` |
| `Hops`, `Status(code)`, `Range` (a partial answer that starts elsewhere), `Server` (a head that cannot be read) | `Server` | `Strange` |
| `Stall` | `Stall` | `Timeout` |
| `Net`: the name, the connection or TLS failed | `Net` | `Network` |
| `Internal` | `Internal` | `Gone` |

## Dependencies

`kx-ipc` (the fetch limits, the host rule, `FetchFault` and `NetFault`), `thiserror`, `zeroize` (a call's header lines), and on Windows the workspace's `windows` crate with `Win32_Foundation` and `Win32_Networking_WinHttp`. No TLS or HTTP crate: WinHTTP does both.
