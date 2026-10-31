# Security note: keyxtend-worker

- **Owner:** product owner
- **Last review:** 2026-10-31 (P8 security check: the restricted token and what a compromised worker can reach; P8 task 8: the `media` job; P8 final fix: the DIB ask)
- **Status:** Reviewed
- **Scope:** `apps/keyxtend-worker`, the program `keyxtend-worker media --max-pixels <n>`. It decodes copied PNG and DIB images into small RGBA thumbnails, and makes a `CF_DIB` from an inserted PNG, outside the uiAccess keyboard (ADR-0004, ADR-0034 decision 5). The host in `kx_platform::pictures` starts it on demand, through the adapter's spawner with lowered rights (`kx-platform.md`); the voice, OCR and update jobs come in later phases.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| None | It reads asks on the stdin and writes replies on the stdout it is given, and opens no file, socket, pipe or window. The `Pictures` port it serves needs no capability (ADR-0034 decision 3). | N/A |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Image bytes in a `Draw` or a `Dib` ask (copied by the user) | Sensitive | Memory only | No; only the host and the worker hold the pipe | Dropped before the reply is written |
| Pixels in a `Pixels` reply, a DIB in a `Dib` reply | Sensitive | Memory only | No | Until the reply is written |

The worker never logs and writes nothing to disk. Its replies give a fixed reason (`Why`), never decoder text.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Asks from the host on stdin | `kx-ipc`: magic, version, kind, length cap, codec, side 1 to 4096, a DIB's cap 1 to `MAX_IMAGE_BYTES` (`docs/security/kx-ipc.md`). A frame that fails ends the worker with exit code 1. | `MAX_ASK_LEN` (64 MiB image) | Property tests in `kx-ipc` |
| PNG bytes from any app | The `png` crate checks the signature, chunk checksums and zlib stream. The header is read first: an empty image or one past `--max-pixels` is refused before any row is decoded, and an interlaced one is refused. Text and colour-profile chunks are skipped. Every row must hold exactly the width, and every row must arrive. A `Dib` ask whose DIB (width × height × 4 + 40) would pass its cap is `TooBig` from the header alone. | Decoder allocations ≤ `DECODE_BYTES` (16 MiB, in `kx-ipc`), so rows over 4 Mi pixels wide are `TooBig`; output ≤ side² × 4, the side clamped to `MAX_SIDE` | Property tests: random bytes, a changed byte, every cut, any declared size |
| DIB bytes (`CF_DIB`, `CF_DIBV5`) from any app | Our reader: header size 40, 108 or 124 within the buffer; width > 0, height ≠ 0 either way up, one plane; 24-bit `BI_RGB`, or 32-bit `BI_RGB` or `BI_BITFIELDS` with distinct whole-byte masks; pixel count ≤ `--max-pixels`; masks, colour table and padded rows inside the buffer, by checked sums. All of it before any pixel is read. | Rows are read in place: nothing is allocated but the output and one row of sums | Property tests: random bytes, a header with each field good, odd or extreme |
| The command line | Only `media --max-pixels <n>` with n above 0; anything else exits with code 1 and one line on stderr. The host passes it. | N/A | Unit tests and a test of the program itself |

## Threats (STRIDE-lite)
- **Spoofing:** the worker trusts only its own stdin, an anonymous pipe the host hands to its child (task 9). Each reply repeats its ask's id, so the host can refuse a stray answer (`serve` tests).
- **Tampering:** a crafted image targets the decoder.
  - PNG goes through the `png` crate and its inflater `fdeflate`, both `#![forbid(unsafe_code)]`; only the checksum crates (`crc32fast`, `simd-adler32`) use SIMD code.
  - The DIB reader is ours, safe Rust, with every field, offset and size checked: `other_header_sizes_are_refused`, `rows_past_the_buffer_are_refused`, `huge_fields_never_overflow`, `masks_that_are_not_distinct_whole_bytes_are_refused`.
  - Each ask runs under `catch_unwind`, so a panic is a `Failed` reply with `Internal`, and the loop goes on (`a_panic_is_an_internal_failure_and_the_loop_goes_on`, `a_dib_ask_gets_its_dib_and_a_refused_or_panicking_one_a_failure`).
  - The DIB it makes is 32-bit `BI_RGB` after a `BITMAPINFOHEADER`, rows bottom-up, BGRA with alpha kept, never past the ask's cap (`a_small_png_becomes_its_exact_dib`, `a_dib_past_the_cap_is_too_big_and_one_at_it_is_made`, `the_cap_is_checked_before_any_row_is_decoded`). The keyboard checks only its length and sets it on the clipboard as it is, never parsing it.
- **Logging / repudiation:** no logging. Stderr gets one line for a refused command line or a broken frame, naming only the kind of error. The media job installs a silent panic hook, so a caught panic writes nothing: no message, place or backtrace (`a_hushed_panic_writes_nothing_on_stderr`).
- **Information disclosure:** content stays in memory and on the two pipes. The worker opens nothing else and has no network crate; ADR-0004 allows network access only in the voice and update jobs.
- **Denial of service:**
  - A decompression bomb is refused from its header: `too_many_pixels_are_refused_from_the_header_alone`, `a_declared_size_past_the_limit_is_always_too_big`.
  - No full-size copy of an image is ever held, so a large image stays within the worker's memory: PNG rows and DIB rows go one at a time into a box filter that keeps the output and one row of sums (`a_large_image_scales_holding_only_the_output_and_one_row_of_sums`, `a_large_png_is_read_row_by_row_within_the_decoder_budget`, 8000 × 6000).
  - Peak memory is the ask (kx-ipc's read holds at most `READ_TIMES`, 2, times it while its buffer grows, up to 64 MiB of image), the decoder's rows (≤ `DECODE_BYTES` counted, plus a few raw rows each at most twice that row), the output (pixels ≤ 64 MiB, or a DIB ≤ its cap, `max_entry_mb`) and the sums (≤ 128 KiB). The settings check refuses a `worker_mb` below `READ_TIMES` times `max_entry_mb`, plus the larger of a preview's RGBA and that DIB, plus `WORKER_BASE_MB`: `DECODE_BYTES` and 16 MB for the worker itself. Both constants come from `kx-ipc`, so the check follows the worker; the job object's cap ends a worker past `worker_mb` (512 MB), and the host restarts it (task 9).
  - Time grows with the pixels, at most `max_pixels`; the host's deadline (`media_ms`) stops a slow draw or DIB (task 9). The worker and its decoder run optimised in dev builds too, so a debug build meets that deadline.
- **Elevation:** the worker runs in its own process from a restricted token: no privilege but change-notify, Administrators deny-only, no uiAccess, Medium integrity (task 9). `#![forbid(unsafe_code)]`; its only dependencies are `kx-ipc`, `png` and `thiserror`: no `kx-platform*`, no `kx-mod-*`, no network crate. Tidy bans `png` and `fdeflate` from the uiAccess app's tree. A compromised worker still runs as the user, with the user's SID, minus the job's limits and the restricted token, so it could reach the user's own files; the host checks only the replies it reads (`kx-ipc`, then `Pixels::new` or a DIB's length).
  - Its job object caps its memory at `[clipboard] worker_mb`, takes no second process, bars it from shared UI such as other programs' windows and the clipboard, and ends it with the keyboard (`kx-platform-windows.md`, Elevation (worker)).
  - The stage ships it beside the program and refuses a worker that is missing or whose manifest asks for uiAccess (`dev-tools.md`).

## Findings
### Must fix (blocks Definition of Done)
- None.
### Should fix (tracked issue)
- Done for Paint, a browser and Snip (P8 hands-on check, 2026-10-31): their copies showed real thumbnails and previews, and a Snip pasted whole into Paint. A copy that offers only a V4 or V5 DIB with bit masks is still to be seen, in P14's check (`docs/items/P14.md`).
- Done (task 9): the host and the worker read the command words and exit codes from one copy in `kx-ipc` (`MEDIA_JOB`, `MAX_PIXELS_FLAG`, `CLEAN_EXIT`, `REFUSED_EXIT`); `the_real_worker_draws_through_the_host` and `the_workers_words_and_exit_codes_match_the_host` in `kx-platform-windows` start the real worker.
### Accepted (with reason)
- Interlaced PNGs are refused: their seven passes cover the whole image, so drawing one needs a full-size buffer, which the row-by-row decoders never hold. Apps rarely copy interlaced PNGs; the history row keeps its type icon.
- Colour profiles, gamma and DIB colour spaces are ignored; pixels are taken as sRGB, which suits a thumbnail.
- 32-bit DIB alpha is taken as straight, not premultiplied; an all-zero alpha means opaque.
- An animated PNG shows its default image only.
- The `png` crate's memory limit is best effort: its raw-row buffers are not counted. They are bounded by the counted row, and the job object caps the whole process.
- Image bytes and pixels are not wiped, as in `kx-ipc`: the worker runs as the user, who can read their own memory anyway.
- The worker keeps the user's SID and logon session, so at Medium it could still read the user's own files, the history folder among them. ADR-0004 asks for less privilege, not a sandbox; an AppContainer or a separate low-rights account is a later hardening item (`docs/items/P14.md`).
- No `cargo fuzz` target: property tests stand in until the owner allows the nightly toolchain, as in `kx-kernel.md`.
- A compromised worker could hand back any DIB within the cap for an insert. The target app reads it as it reads any app's copy, and the keyboard never parses it.
- A DIB ask can wait behind a thumbnail or preview draw and miss `media_ms`; the insert then sets the PNG alone, which most apps read.
