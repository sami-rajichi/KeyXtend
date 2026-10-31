# Security note: kx-ipc

- **Owner:** product owner
- **Last review:** 2026-10-31 (P8 security check: a dripped reply's buffer and the worker's command words; P8 task 6: the frame format and its limits)
- **Status:** Reviewed
- **Scope:** `crates/kx-ipc`, a library with no module of its own. The worker host (P8 task 9) writes asks and reads replies; `keyxtend-worker media` (task 8) reads asks and writes replies (ADR-0004, ADR-0034 decision 5). It also holds the worker's command words and exit codes (`MEDIA_JOB`, `MAX_PIXELS_FLAG`, `CLEAN_EXIT`, `REFUSED_EXIT`), so the host and the worker read one copy.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None of its own | It reads and writes frames on the streams it is given and opens no pipe, file or socket. The `Pictures` port it serves needs no capability (ADR-0034 decision 3). | N/A |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Image bytes in a `Draw` (copied by the user) | Sensitive | Memory, and the pipe between the host and its worker | No; only the two processes hold the pipe | Until the frame is dropped |
| Pixels in a `Pixels` reply | Sensitive | Memory, and the same pipe | No | Until the frame is dropped |

`Debug` of `Ask` and `Reply` prints lengths, never bytes. Errors name only the kind of refusal.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Replies the host reads, from a worker that decodes hostile images and may be compromised | Magic, version, then the kind (`Pixels` or `Failed` only) and the length within that kind's range, all before any payload byte is read. Width and height from 1 to `MAX_SIDE`; the length must equal the fields plus width × height × 4 before the pixels are read; the reason comes from a fixed list | `MAX_REPLY_LEN`: 8 + 4096 × 4096 × 4 bytes (64 MiB + 8). The buffer starts at `READ_CHUNK` (64 KiB) and doubles only as bytes arrive | Property tests (proptest): random bytes; a good header with any kind and length; a changed byte in a good frame; every cut |
| Asks the worker reads, from the host | The same header checks (`Draw` or `Bye` only). Codec from a fixed list, side from 1 to `MAX_SIDE`, an image of at least one byte; `Bye` has id 0 and no payload | `MAX_ASK_LEN`: 5 + 64 MiB (`MAX_IMAGE_BYTES`, the largest entry `max_entry_mb` allows) | The same properties |
| Frames either side writes | The writer runs the same checks and refuses before writing a byte, so neither side sends what the other refuses | The same caps | Round-trip properties |

The crate sets no time limit: the host's deadline (`media_ms`) and kill on timeout belong to the worker host (task 9).

## Threats (STRIDE-lite)
- **Spoofing:** frames carry no authentication; the pipe is the boundary. The host hands only inherited anonymous pipe handles to its own child (ADR-0004), and the reply's `id` lets the host refuse an answer to another request (task 9).
- **Tampering:** a compromised worker can send any bytes, so every field is checked before use and a length that disagrees with its fields is `Inconsistent`. The host then checks the size it asked for (`Pixels::new`). Tests: `a_field_out_of_range_is_refused`, `pixels_that_are_not_width_by_height_are_inconsistent`, `a_changed_reply_never_panics_and_what_reads_writes_back_the_same`.
- **Version confusion:** the magic `KXIP` and the version byte come first, so another version or stray output on the pipe is `BadMagic` or `BadVersion`. Each side reads only the other side's kinds (`BadKind`), so an echoed ask is never taken as a reply. Tests: `a_wrong_magic_is_refused`, `another_version_is_refused`, `an_unknown_kind_is_refused`, `each_side_reads_only_the_other_sides_kinds`.
- **Logging / repudiation:** the crate never logs. `Debug` shows lengths only (`debug_shows_lengths_never_content`), and `Why` is a fixed enum, so a worker cannot push free text into the host's logs.
- **Information disclosure:** content stays in memory and in a pipe only the two processes hold. Buffers are not wiped (Accepted).
- **Denial of service:**
  - Oversized frames: the length is checked against its kind's cap right after the header, before any payload byte is read (`a_length_past_its_kind_is_refused_before_the_payload`, `a_good_header_is_never_read_past_its_frame_or_cap`).
  - A lying length within the cap costs only what arrives, and never more than the cap (`a_read_buffer_grows_past_neither_its_length_nor_twice_what_arrived`, `a_lying_pixel_length_is_refused_before_the_pixels`).
  - A reader never reads past its frame, so a refused frame cannot swallow the next one (`*_round_trip_and_end_at_their_frame`).
  - Random and cut input never panics (`random_bytes_never_panic`, `a_cut_frame_fails_cleanly`). A worker that stops mid-frame blocks the read until the host's deadline ends it (task 9).
- **Elevation:** `#![forbid(unsafe_code)]`, no dependency beyond `thiserror`, and no media parsing. Decoding stays in the worker, which runs from a restricted token without uiAccess at Medium integrity (ADR-0034 decision 5, `kx-platform-windows.md`).

## Findings
### Must fix (blocks Definition of Done)
- None.
### Should fix (tracked issue)
- Done (task 16): the settings stay inside these limits, `max_entry_mb` at most 64 and `preview_px` and `thumb_px` at most `MAX_SIDE`; the app's agree test pins both (`apps/keyxtend/tests/agree/clip.rs`).
### Accepted (with reason)
- The host may hold up to `MAX_REPLY_LEN` for one reply: a worker that declares a near-cap `Pixels` length and drips its bytes makes `read_tail` double the buffer (`grow`) to about 64 MiB. The host sends one draw at a time, which bounds it.
- No authentication or encryption on frames: the pipe is anonymous and inherited only by our child. A named pipe would need the checks in ARCHITECTURE.md's IPC section.
- Image bytes and pixels are not wiped on drop; the clipboard module holds the same content unwiped in `Redacted`.
- No rate limit in the crate: one draw at a time within the host's deadline bounds the rate ADR-0004 asks for (task 9).
- After any error the stream is out of step; the host restarts the worker with backoff, and the worker exits.
- No `cargo fuzz` target: property tests stand in until the owner allows the nightly toolchain, as in `kx-kernel.md`.
