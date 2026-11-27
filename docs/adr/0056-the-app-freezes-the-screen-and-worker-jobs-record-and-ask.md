# 56. The app freezes the screen, and worker jobs record and ask where to save

- **Date:** 2026-11-23
- **Status:** Accepted. Adds the `record` and `pick` jobs to ADR-0004's worker, `Encode` to ADR-0034 decision 5's `media` job and `write_kept` to its decision 3's writer, `shield` to ADR-0013's window guard, and Snip as a second holder of ADR-0046's `OpenFolder`. The Save dialog's place in front of the editor and its foreground grant are still open (below).

## Context

- P12 takes pictures and records videos ([`2026-11-23-snip-design.md`](../superpowers/specs/2026-11-23-snip-design.md), ADR-0055).
- The uiAccess app must not decode media, load codecs or load the shell's extensions, since a bug there would give an attacker UIPI bypass (ADR-0004).
- Our capture bar, strip, record bar and, unless asked, the keyboard must never be in a picture or video.
- The owner wants the system's own Save dialog, as every app has, and no growth of the download.
- Task 1 probed the risky parts on 2026-11-23 (`docs/items/P12.md`, live tests behind `KX_LIVE_SNIP=1`).

## Decision

Decided with the owner's approval of the P12 design and plan on 2026-11-23, and Task 1's evidence below.

1. **Freeze in the app.** One click grabs every screen as raw pixels (GDI, as the spike's G24 did) with the window outlines read at that moment, refused past `shot_cap_mb`. The app draws those pixels and cuts pictures from them; it never decodes a file.
2. **Pictures leave through the worker, and copies are kept.** Copy puts on the clipboard a `CF_DIB` the app builds from its own pixels and a PNG; every PNG comes from the `media` job's new `Encode`, raw pixels in and PNG out, so the app links no image crate. Copy, of a picture or a video, writes with `ClipboardWriter::write_kept`, without our "leave out of history" marker, so KeyXtend's and Windows' clipboard histories both keep the snip, as the owner chose and as Snipping Tool does.
3. **A `record` job.** The screen recorder, the microphone, the computer's sound, the MP4 encoder and the speed change run in a new worker job with normal rights, capped at `record_mb`. It records into the Snips folder's `unsaved` part and tells the app the time, the end and faults over the streaming session (ADR-0042).
4. **Windows Graphics Capture, frames on the card.** The `record` job films a part with Windows Graphics Capture, pointer on and border off. Each frame goes to Media Foundation on the graphics card through its DXGI device manager, never through the processor.
5. **The system's encoders.** Media Foundation's Sink Writer makes H.264 and AAC into MP4, and the microphone and the computer's sound (WASAPI through `cpal`) mix into one track. We ship no codec; a Windows N edition without the Media Feature Pack dims both video keys.
6. **A `pick` job.** The system's Save and folder dialogs run in their own job, owned by a hidden topmost window of the worker, so no shell extension loads into the app. It answers a path or nothing, with `FOS_DONTADDTORECENT`, and the answer is checked (a drive path of the asked type) before anything is written there.
7. **Every UI limit kept.** Both jobs run under the worker's restricted token and the full `job.rs` `UI` limits; neither lifts one. Neither has network, and tidy keeps `kx-platform-record`, which holds them, out of the app (D4).
8. **Shielded windows.** `WindowGuard::shield` keeps one of our windows out of every capture (`WDA_EXCLUDEFROMCAPTURE` on Windows). The capture bar, strip, hint, edge, countdown and record bar are shielded before they first show; the keyboard and its bubble are shielded during a capture unless `keyboard` is `shown`.
9. **Focus.** No Snip window of the app takes focus, the editor included (ADR-0013). The system's dialogs take focus, as Settings does, since a file name is typed there.
10. **Gates.** `kx-mod-snip` holds `ScreenCapture`, `Microphone`, `WriteClipboard`, `OpenFolder` (Open folder, and Play through `MediaOpen`) and a new `SaveFile` for the dialogs, held by Snip alone. `MediaOpen` opens only an MP4 of the Snips folder, through the running Explorer as `FolderReveal` does, so nothing starts as our child.

## Evidence

Task 1, on 2026-11-23, Windows 11 at 1920×1080:

- Graphics Capture ran in the worker under its restricted token and job and left a shielded window out. Desktop Duplication was not needed, and it would have made us draw the pointer.
- Pointer on and border off were accepted as the session reads them back; the owner's hands-on check is the visual proof.
- Both jobs worked under the full `UI` limits. A recording peaked at 95–97 MB, under the planned `record_mb` of 512; the dialog's worker at 11 MB.
- The whole work area recorded at 30 fps with no late frame, also in weak mode (2 cores at 30 %). Copying frames through the processor made 12, then 60, of 90 frames late there; the card path fixed that and cut peak memory from 142–204 MB to 95 MB.
- A hardware H.264 encoder was picked. Each 3 s file read back as 3008 ms of H.264 at 30 fps with 90 frames, and AAC at 48 kHz stereo.
- A window shielded before it first shows was in no GDI grab. A shown window left at the next grab after the call, 5–8 ms, in 25 of 25 rounds.
- The Save dialog opened from the worker in front of a topmost stand-in editor and returned the exact path. It left one registry value, its remembered size for `keyxtend-worker.exe`.

## Still open

- **To confirm on a dev-install (Task 7):** the real editor and keyboard sit in the uiAccess band, above any worker dialog. The editor drops topmost while the dialog shows, the keyboard stays above so the user can type, and the app grants the worker the foreground (`AllowSetForegroundWindow`).
- **Task 6:** the worker has no DPI-aware manifest, so the `record` and `pick` jobs settle how they read physical pixels.
- **Task 6:** Task 1's worker imports `mfplat.dll` and `mfreadwrite.dll` at start, so on an N edition without them no job would start. Media Foundation must load only when a video job needs it.
- Task 1 ran on Windows 11 only; Windows 10's capture border and pointer switches are untested.

## Alternatives

- **Desktop Duplication:** it leaves the pointer out, so we would draw it ourselves; Graphics Capture draws it.
- **Recording or dialogs in the app:** codecs, sound stacks and shell extensions would load into the uiAccess process.
- **Shipping an encoder** such as FFmpeg: megabytes more in the download, and codec licences of its own.
- **Hiding our windows for each grab:** a flash and a wait per capture; it stays the fallback where an OS has no shield.

## Other platforms

- macOS and Linux (P15) fill the same four ports: freeze and list windows, shield, record into MP4 with the system's encoder, and the system's dialogs.
- On Wayland the system asks the user through its portal before any capture. Where an OS cannot keep a window out of a capture, our windows step aside for that moment.

## Consequences

- A crafted file, codec or shell extension meets only the worker; the app holds raw pixels and checked paths, and never parses a media file.
- Snip adds no codec, library or other file to the download: the `windows` features add bindings only, and Graphics Capture and Media Foundation are part of Windows.
- A copied snip can reach Windows' clipboard history and, if the user turned it on, its cloud sync, as any copy does; only the user's Copy puts it there.
- The microphone is now heard in two jobs, `voice` and `record`; neither has network (ADR-0042).
- While shielded, our windows are also missing from other apps' screenshots and screen shares; the keyboard is shielded only during a capture.
- The system's dialogs are Windows' own, not paged (ADR-0008); the user types a name with our keyboard and clicks Save.
- Each new IPC frame needs its property tests and a line in `docs/security/kx-ipc.md`.
