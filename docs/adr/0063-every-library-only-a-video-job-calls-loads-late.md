# 63. Every library only a video job calls loads late

- **Date:** 2026-11-27
- **Status:** Accepted (2026-11-27), from the owner's yes. Widens ADR-0056's edge 14 from Media Foundation's libraries to every library only a video job calls.

## Context

- ADR-0056's edge 14 made the worker delay-load `mfplat.dll` and `mfreadwrite.dll`, so a system without Media Foundation still starts every other job.
- The recorder also made the worker import `d3d11.dll` (the capture's device) and `propsys.dll` (a file's length) at start.
- PR #28's Windows CI could not start the worker at all: every worker command-line test ended with `0xC0000135`, a missing library. Qt's own libraries import `d3d11.dll` there, and main's worker already imported `mmdevapi.dll`, which leaves `propsys.dll` as the likely gap.
- One missing library at start stops every job: images, the vault, voice and downloads, not only video.

## Decision

1. **One list.** `media_dlls` in `kx-platform-record/src/record.toml` names every library only a video job calls: Media Foundation's, `d3d11.dll` and `propsys.dll`.
2. **The same path as Media Foundation.** The worker's build delay-loads each, and a video or copy job loads each from System32 before its first call, answering `NoVideo` when one is missing.
3. **The keyboard's check follows.** The kit offers video only when every listed library is in the system folder, as it did for Media Foundation.
4. **Shared libraries stay.** `mmdevapi.dll`, which the voice job's microphone also uses, and the WinRT API sets, which ship with every supported Windows, still load at start.

## Consequences

- A system missing one of these libraries, such as a Windows N edition or a bare server, still runs every non-video job and dims video.
- The worker's `tests/imports.rs` reads the same list, so a library added later is checked without a new test.
- If Windows CI still cannot start the worker, `mmdevapi.dll` is the next to check.
