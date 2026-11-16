# Security note: kx-platform-net

- **Owner:** product owner
- **Last review:** 2026-11-16 (P11 task 29 `kx-security-check`: a first address needs a host listed by name, while a redirect may reach the hosts under a listed `.` domain, for Hugging Face's regional CDN hosts; the task 13 follow-ups are done; final); 2026-11-14 (P11 task 13: the crate's first version, then a hard review against the code)
- **Status:** Reviewed
- **Scope:** `crates/kx-platform-net`, used only by `keyxtend-worker`'s `fetch` job (ADR-0004, ADR-0043 decision 8). It holds the `Http` port with its address, redirect and range rules, the WinHTTP adapter (`WinHttp`) and a scripted fake behind the `fake` feature. The `keyxtend` app never depends on it; P11 task 22 adds it to tidy's `banned_network` list.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None held | A library: the worker's `fetch` job calls it, and the module that asks for a download holds `Network` behind the `ModelFiles` port (ADR-0042). | N/A |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Download addresses and hosts, from the models table | Public | Memory, for one request | N/A | One download |
| A redirect's address, which a CDN may sign with a short-lived query | Public (a signed link to a public file) | Memory; `Answer::url` hands the last one to the caller | N/A | One download |
| A redirect's `Location`, `Content-Length` and `Content-Range` | Public (server headers) | Memory, read into a buffer of at most `MAX_URL_BYTES + 1` UTF-16 units | N/A | One request |
| Model file bytes | Public (untrusted data) | Memory, as the caller's buffer; the fetch job writes them to its `.part` file | TLS in transit | One read |
| The user agent `AGENT` (`KeyXtend`) | Public | Sent with each request | TLS | N/A |

No personal data is sent: no cookies (`WINHTTP_DISABLE_COOKIES`), no credentials (the crate never calls `WinHttpSetCredentials`), no `Accept-Encoding`, and a user agent that is the product name only. `NetError` names a fixed reason and at most a status code, never an address or a server's words.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| The address a `Get` names (from a checked `FetchAsk`) | `Target::parse`: visible ASCII, `https://` in any case, no user name, no IPv6 literal, a host of lower-case letters, digits, dots and hyphens (`kx_ipc::rules::host`), a port from 1 to 65535; the fragment is dropped and `.` and `..` folders are resolved (RFC 3986 §5.2.4). A plain entry of the download's list must name the host (`Target::named`, `kx_ipc::rules::names`) before any request; a `.` entry alone never does | `MAX_URL_BYTES` for the text and for the address written out, `MAX_HOST_BYTES` | Table tests (`tests/target.rs`) and a sweep of every text of up to 4 tricky characters (`tests/sweep.rs`) |
| A redirect's `Location` | `Target::join` reads it against the current address as RFC 3986 §5.2 says, then the same rules; a scheme other than HTTPS is refused, a relative one keeps the host and port, and the new host must be listed, before it is asked | `MAX_HOPS` (5) redirects; the header and the joined address within `MAX_URL_BYTES` | Table tests and the sweep |
| The status, `Content-Length` and `Content-Range` | Only 200, 206 and the five redirect statuses go on. A length is digits only within 64 bits; a range must be `bytes first-last/total` with first ≤ last < total, and a 206 must start at the byte asked; a 200 to a range request restarts the part from 0 | 64-bit numbers, no arithmetic on them | Table tests (`tests/range.rs`) |
| The body | Read into the caller's buffer, never parsed or decompressed here; the fetch job checks the size and SHA-256 before the file gets its name | Each read waits at most `wait_ms(stall_s)` | N/A |

The host and the path go to WinHTTP apart (`WinHttpConnect`, `WinHttpOpenRequest`), so no other reading of the raw address can reach another host.

## Threats (STRIDE-lite)
- **Spoofing:** a server posing as a listed host is stopped by TLS, which Windows checks against its trusted roots (`WINHTTP_FLAG_SECURE`, TLS 1.2 or 1.3, no certificate-error flags). A swapped file still fails the fetch job's SHA-256 check.
- **Tampering:** a redirect to an unlisted host, or to `http://`, is refused before it is asked, at every hop of a chain (edge 22). A listed host takes only itself; a `.` entry such as `.hf.co` takes a redirect to the hosts under that domain, never the domain, a look-alike or a bare top-level domain (`kx_ipc::rules::covers`), and never a first address (`a_first_address_under_an_under_entry_alone_is_never_asked`). WinHTTP's own redirects are off (`WINHTTP_OPTION_REDIRECT_POLICY_NEVER`). A 206 that answers another range is refused, so no wrong bytes join the part.
- **Logging / repudiation:** the crate logs nothing; errors carry fixed reasons.
- **Information disclosure:** no cookies, no credentials, no telemetry, no personal user agent. The loopback test checks that the first byte sent behind an HTTPS address is TLS's, never a plain GET.
- **Denial of service:** every wait is capped at `wait_ms(stall_s)`: name, connection, send, head and each read. `wait_ms` holds `stall_s` within `kx_ipc::STALL_S`, so a 0, which WinHTTP reads as no limit, never reaches it. Redirects stop at `MAX_HOPS`; header reads stop at `MAX_URL_BYTES`; a relative redirect cannot grow the address past `MAX_URL_BYTES`. The body's total size is the fetch job's to cap.
- **Elevation:** the crate runs inside the restricted worker, never in the uiAccess app. Its `unsafe` blocks are WinHTTP calls on handles it owns, each with a `// SAFETY:` comment. A `Handle` is made only by `owned`, which refuses null, and is closed once in `Drop`; it is `Send` because a WinHTTP handle is not tied to a thread and its one owner makes one call at a time.

## Findings
### Must fix (blocks Definition of Done)
- None open.
- Done (task 13 review): `wait_ms` passed `stall_s` straight through, so a 0 from a caller became an endless wait in `WinHttpSetTimeouts`. It now holds `stall_s` within `kx_ipc::STALL_S` (`the_stall_time_is_held_within_its_range_in_milliseconds`).
- Done (task 13 review): `join` never checked the joined length, so each relative redirect could grow the address past `MAX_URL_BYTES`. Every `Target` is now held within it written out (`a_location_that_joins_past_the_cap_is_refused`, `a_location_that_joins_past_the_cap_is_refused_before_it_is_asked`).
- Done (task 13 review): `join` read a lone `#frag` as the folder, dropping the file and query, and kept `.` and `..` folders. Both now follow RFC 3986 §5.2 (`dot_folders_and_a_lone_fragment_in_a_location_are_resolved`).
- Done (task 29 code review): the first hop took an address whose host only a `.` entry covered; `get` now asks `Target::named` first, and only redirects use `Target::listed` (`only_a_plain_entry_names_a_first_address`).
### Should fix (tracked issue)
- Done (task 22): `kx-platform-net` is in tidy's `banned_network` list for `keyxtend`, and the worker takes its `fake` feature only as a dev-dependency.
- No `cargo fuzz` or `proptest` target: the sweep stands in, as `proptest` would be a new dev-dependency here and `cargo fuzz` needs the nightly toolchain, as in `kx-ipc.md`.
- Done (task 13): the fetch job stops reading at the row's size (`reads_stop_at_the_rows_size`) and never logs an address, `Answer::url` or its `Debug` (`keyxtend-worker.md`).
### Accepted (with reason)
- Certificate revocation is not checked, as that would contact the CA's servers, which are no listed host; the SHA-256 fingerprint protects the file itself.
- `WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY` follows the system's proxy settings, which may discover a proxy on the local network under Windows' own time limits; that is the user's own setting, and the request itself still goes only to listed hosts.
- WinHTTP's automatic sign-in policy is left as Windows sets it, which offers Windows credentials only to intranet hosts; the table lists only public hosts.
- A pause or cancel that lands during a stalled read waits up to `stall_s` for that read to return: the body owns its handles and is read on one thread, so nothing closes them mid-read.
- `Unlisted` reads as `FetchFault::Redirect` even for the table's first address; the table names its own hosts, so that only happens on a table mistake.
- A `.hf.co` entry trusts every host under Hugging Face's hf.co for a redirect, as its firewall guide advises, since the CDN host differs by region and changes; TLS and the SHA-256 still guard the file. The host rule knows no public-suffix list, so the models table's test pins its `.` entries to `.hf.co` alone.
- `get` is a provided method of `Http`, so an adapter could replace the rules; only this crate's `WinHttp` and `FakeHttp` implement it, and neither does.
