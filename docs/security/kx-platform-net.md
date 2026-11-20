# Security note: kx-platform-net

- **Owner:** product owner
- **Last review:** 2026-11-20 (P11 part 2 task 22: the `speech-net` job's POST added to this note from the code, the crate's README and the task 8 report; nothing new found); 2026-11-16 (P11 task 29 `kx-security-check`: a first address needs a host listed by name, while a redirect may reach the hosts under a listed `.` domain, for Hugging Face's regional CDN hosts; the task 13 follow-ups are done; final); 2026-11-14 (P11 task 13: the crate's first version, then a hard review against the code)
- **Status:** Reviewed
- **Scope:** `crates/kx-platform-net`, used only by `keyxtend-worker`'s `fetch` and `speech-net` jobs (ADR-0004, ADR-0043 decision 8, ADR-0047 decision 6). It holds the `Http` port with its address, redirect and range rules, the one-request `Call` (GET or POST, `Field`, `Verb`, `SEND_CHUNK`) the speech job sends, the WinHTTP adapter (`WinHttp`) and a scripted fake behind the `fake` feature. The `keyxtend` app never depends on it; P11 task 22 adds it to tidy's `banned_network` list.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None held | A library: the worker's `fetch` and `speech-net` jobs call it, and the module that asks for a download or a cloud dictation holds `Network` behind the `ModelFiles` and `CloudSpeech` ports (ADR-0042, ADR-0047). | N/A |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Download addresses and hosts, from the models table | Public | Memory, for one request | N/A | One download |
| A redirect's address, which a CDN may sign with a short-lived query | Public (a signed link to a public file) | Memory; `Answer::url` hands the last one to the caller | N/A | One download |
| A redirect's `Location`, `Content-Length` and `Content-Range` | Public (server headers) | Memory, read into a buffer of at most `MAX_URL_BYTES + 1` UTF-16 units | N/A | One request |
| Model file bytes | Public (untrusted data) | Memory, as the caller's buffer; the fetch job writes them to its `.part` file | TLS in transit | One read |
| The user agent `AGENT` (`KeyXtend`) | Public | Sent with each request | TLS | N/A |
| A call's header fields, a cloud key among them (speech-net job) | Secret | Memory: the caller's wiped buffer, then the head written as one block in a wiped buffer sized once (`win/post.rs`); `Call`'s `Debug` shows names and sizes only | TLS in transit | One call |
| A call's body, a piece of sound (speech-net job) | Secret | The caller's wiped buffer, written in `SEND_CHUNK` pieces and never copied here | TLS in transit | One call |
| A call's kept answer headers, such as a 429's `Retry-After` | Public (server headers) | Memory, read into a buffer no longer than the longest address | N/A | One call |

No personal data is sent on a download: no cookies (`WINHTTP_DISABLE_COOKIES`), no credentials (the crate never calls `WinHttpSetCredentials`), no `Accept-Encoding`, and a user agent that is the product name only. A cloud call sends only the sound and key its caller hands it, to the one address its caller checked. `NetError` names a fixed reason and at most a status code, never an address or a server's words.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| The address a `Get` names (from a checked `FetchAsk`) | `Target::parse`: visible ASCII, `https://` in any case, no user name, no IPv6 literal, a host of lower-case letters, digits, dots and hyphens (`kx_ipc::rules::host`), a port from 1 to 65535; the fragment is dropped and `.` and `..` folders are resolved (RFC 3986 §5.2.4). A plain entry of the download's list must name the host (`Target::named`, `kx_ipc::rules::names`) before any request; a `.` entry alone never does | `MAX_URL_BYTES` for the text and for the address written out, `MAX_HOST_BYTES` | Table tests (`tests/target.rs`) and a sweep of every text of up to 4 tricky characters (`tests/sweep.rs`) |
| A redirect's `Location` | `Target::join` reads it against the current address as RFC 3986 §5.2 says, then the same rules; a scheme other than HTTPS is refused, a relative one keeps the host and port, and the new host must be listed, before it is asked | `MAX_HOPS` (5) redirects; the header and the joined address within `MAX_URL_BYTES` | Table tests and the sweep |
| The status, `Content-Length` and `Content-Range` | Only 200, 206 and the five redirect statuses go on. A length is digits only within 64 bits; a range must be `bytes first-last/total` with first ≤ last < total, and a 206 must start at the byte asked; a 200 to a range request restarts the part from 0 | 64-bit numbers, no arithmetic on them | Table tests (`tests/range.rs`) |
| The body | Read into the caller's buffer, never parsed or decompressed here; the fetch job checks the size and SHA-256 before the file gets its name | Each read waits at most `wait_ms(stall_s)` | N/A |
| A `Call` from the speech-net job | `call` runs `Call::check` before anything is sent: each header name a token and each value visible ASCII or a space, so no value can add a line, and no body on a GET. `deliver` sends to the one checked `Target` and never follows a redirect: a 3xx comes back as its status, which the job reads as a strange reply | `stall_s` for each wait; the body in `SEND_CHUNK` (64 KiB) pieces, the stop flag looked at before each one and after the last | Unit tests over `FakeHttp` and the loopback (`win/tests.rs`: the first byte sent is TLS's, never a POST's key) |
| A cloud call's answer | Its status and kept headers come back as they are; the job reads the body within its own cap (`keyxtend-worker.md`). Each `NetError` maps to one `NetFault` in one table (`error.rs`): a stall is `Timeout`, a failed name, connection or TLS is `Network`, a stop is `Gone` | `stall_s` | Table tests |

The host and the path go to WinHTTP apart (`WinHttpConnect`, `WinHttpOpenRequest`), so no other reading of the raw address can reach another host.

## Threats (STRIDE-lite)
- **Spoofing:** a server posing as a listed host is stopped by TLS, which Windows checks against its trusted roots (`WINHTTP_FLAG_SECURE`, TLS 1.2 or 1.3, no certificate-error flags). A swapped file still fails the fetch job's SHA-256 check.
- **Tampering:** a redirect to an unlisted host, or to `http://`, is refused before it is asked, at every hop of a chain (edge 22). A listed host takes only itself; a `.` entry such as `.hf.co` takes a redirect to the hosts under that domain, never the domain, a look-alike or a bare top-level domain (`kx_ipc::rules::covers`), and never a first address (`a_first_address_under_an_under_entry_alone_is_never_asked`). WinHTTP's own redirects are off (`WINHTTP_OPTION_REDIRECT_POLICY_NEVER`). A 206 that answers another range is refused, so no wrong bytes join the part. A cloud call follows no redirect at all, so a sign-in page or a look-alike host never gets the key or the sound.
- **Logging / repudiation:** the crate logs nothing; errors carry fixed reasons.
- **Information disclosure:** no cookies, no credentials, no telemetry, no personal user agent. A call's key sits only in its header block, which is wiped, and `Debug` never shows a value or the body. The loopback test checks that the first byte sent behind an HTTPS address is TLS's, never a plain GET.
- **Denial of service:** every wait is capped at `wait_ms(stall_s)`: name, connection, send, head and each read. `wait_ms` holds `stall_s` within `kx_ipc::STALL_S`, so a 0, which WinHTTP reads as no limit, never reaches it. Redirects stop at `MAX_HOPS`; header reads stop at `MAX_URL_BYTES`; a relative redirect cannot grow the address past `MAX_URL_BYTES`. The body's total size is the fetch job's to cap, and a cloud answer's the speech job's. A cloud upload's chunks each wait at most `wait_ms(stall_s)`, and a set stop flag ends it between two.
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
- A stop cannot cut a WinHTTP wait already begun (the connection, a chunk's write or the answer's head); the call ends at its next step, within `stall_s` (P11 speech-net).
- On a slow uplink each 64 KiB chunk must go within `stall_s`; a link slower than that reads as a stall, which the job tries again up to `net_tries`.
- `get` is a provided method of `Http`, so an adapter could replace the rules; only this crate's `WinHttp` and `FakeHttp` implement it, and neither does.
