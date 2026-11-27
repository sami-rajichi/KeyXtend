# kx-mod-snip

Snip and screen recording (P12, ADR-0055, ADR-0056): one click on the Snip key freezes the screens, and the user takes a picture of a region, a window or the screen, or records a part or the screen. A strip of keycaps then copies, edits or saves it, and nothing is kept unless the user asks. The design is `docs/superpowers/specs/2026-11-23-snip-design.md`.

Today the crate holds the manifest, the `[snip]` settings, the capture flow (`src/flow.rs`, a pure state machine on an injected clock that names the steps the actor will carry out), the picking rules (`src/part.rs`: what a click takes for each way, and the screen holding a part), where the strip and the record bar go (`src/strip.rs`), the picture editor's model (`src/edit.rs`: its tools, inks, sizes, Crop, Undo and Redo, and the text being typed), and the actor (`src/actor.rs` on its `kx-snip` thread), which carries the flow's steps out on the services below. Blocking calls run in short jobs off that thread, so it never waits on the worker or a dialog. Copy of a video marks it copied with a small file beside it; at each start and at each snip's start the sweep keeps a video marked within `copied_days` and deletes every other PNG and MP4 of the `unsaved` part that no job uses, with its mark, then marks whose video is gone. A recorded video has no editor: its strip steps its speed through `SNIP_SPEEDS`, plays it at that speed, and stays up after a Copy or Save at that speed; each speed's copy is made once and reused by Play, Copy and Save (ADR-0060). The ui tells each screen's work area (`SnipClick::Screens`), where the record bar and a recorded video's strip go. With the keyboard aside, or in every mode for a snip started from the editor or its question (`SnipAside::Stay` when Shown), the freeze waits for the ui's `SnipClick::Aside` until `aside_ms`. The ui's `SnipClick::Bare` says a capture window could not be kept out of captures: video dims (`Dim::Bare`) until the module stops, and a countdown ends or a recording stops with `SnipNote::Bare` saying why. A microphone or sound device that fails at a recording's start brings the strip back with its switch off, and one that fails during it turns off, each with its note (`SnipNote::MicFailed`, `SnipNote::SoundFailed`). The view's `dialog` is true while a Save or folder dialog's job is out.

The handfeel round (plan 2026-11-26) added these:

- **The frame** (`src/frame.rs`): a region's and Crop's eight handles, picked up and set by clicks, never a drag. A set keeps the frame within its box and each moved side at least `frame_min_px`.
- **A region's frame on the frozen screens** (`src/flow/frame.rs`): while a handle is held the strip hides and its keys wait, and Esc drops the hold first. A click away from the frame starts a new region there.
- **Crop's waiting frame** (`src/edit/cut.rs`): Crop's second click holds a frame over the picture. Apply (✓) cuts it as one undoable step, Cancel (✗) drops it, and Esc drops a held handle, then the frame.
- **The editor's place:** `editor` keeps its last box, and `editor_full` the work area it filled when maximised, so it opens maximised there while that screen is still there.
- **Videos have no editor** (ADR-0060), and only a picture's strip holds frozen pixels (`Held::Picture`), so a video's countdown, recording and strip hold none.
- **Timing lines:** at debug, the actor logs how long the freeze answered after the Snip key, in milliseconds only (`src/actor/outs.rs`).

## Services

| Service | Direction | Use |
|---|---|---|
| `ScreenFreeze` (`ScreenFreezeKey`) | needs | Freezes every screen as raw pixels with the windows' frames, refused past `shot_cap_mb`. |
| `Recorder` (`RecorderKey`) | needs | Records a part with its sound switches in the worker's `record` job, and writes a video's re-timed copy. |
| `MicPasses` (`MicPassKey`) | needs | The app's pass that lets the recorder hear the microphone. |
| `SnipFolder` (`SnipFolderKey`) | needs | The Snips folder, the videos of its `unsaved` part, their copy marks and removal, and the drive's free room. |
| `FileAsk` (`FileAskKey`) | needs | The system's Save and folder dialogs, in the worker's `pick` job; Save's answer is where the snip is written. |
| `MediaOpen` (`MediaOpenKey`) | needs | Play: opens a video of the Snips folder in the user's own player. |
| `ClipboardWriter` (`ClipboardWriterKey`) | needs | Copy: a picture's PNG and `CF_DIB`, or a video's file, written with `write_kept`, so KeyXtend's history keeps it and Windows' history a picture only. |
| `Pictures` (`PicturesKey`) | needs, ungated | A picture's PNG, made in the worker's media job with a memory cap and a deadline sized from its pixels. |
| `FolderReveal` (`FolderRevealKey`) | needs | Open folder on the Snip panel. |
| `Calendar` (`CalendarKey`) | needs, ungated | The snip's local date and time, which names it in the Save dialog. |

It provides no service; it talks to the keyboard and the ui over the bus (`kx_module_api::ports` snip events). The ui tells it an Esc from the hand or another program (`SnipClick::Esc`, ADR-0059) only while frozen screens show with no system dialog in front, and the module does what our keyboard's Esc does, which over frozen screens means Don't save at the close question.

## Capabilities

| Capability | Why |
|---|---|
| `Snip` | It gates the snip events: the key, its light, the views, their clicks, the keyboard's step aside and the editor's line (ADR-0057 decision 5). |
| `ScreenCapture` | It gates the freeze, the recorder and the Snips folder. |
| `Microphone` | It gates the microphone's pass for Mic; only this module holds it (ADR-0058). |
| `WriteClipboard` | It gates Copy. |
| `OpenFolder` | It gates Open folder and Play. |
| `SaveFile` | It gates the system's Save and folder dialogs; only this module holds it. |

It holds no `InjectInput`, `Network` or `ReadClipboard`.

## Settings (`[snip]`, version 1)

The defaults live only in `defaults.toml`, each with its one-line meaning, and no table here copies one. The ranges the worker's frames share are kx-ipc's (`crates/kx-ipc/src/limits/snip.rs`), which `kx-module-api` passes on as `RecordAsk::FPS`, `MAX_MIN`, `MIN_FREE_MB` and `JOB_MB`, and `SHOT_CAP_MB` with its checked `ShotCap`; the jobs' waits and the dialogs' limits are the ports' `RecordWaits` and `PickLimits` ranges, and the module's own are named below (`src/settings.rs`, `src/settings/jobs.rs`). A bad value goes back to its default with the kernel's settings notice.

| Key | Range | Meaning |
|---|---|---|
| `keyboard` | `SnipKeyboard` | What the keyboard does during a capture. |
| `save_dir` | | The folder the Save dialog opens at, kept as typed and checked when used; empty means the Snips folder. |
| `countdown_s`, `countdown_step_s` | `COUNTDOWN_S`, `COUNTDOWN_STEP_S` | The seconds a video counts down before it records, and the Snip panel's step for it. |
| `fps` | kx-ipc's `SNIP_FPS` | A video's frames a second. |
| `max_min`, `max_min_step` | kx-ipc's `SNIP_MAX_MIN`; the module's `MAX_MIN_STEP` | The longest video in minutes, and the Snip panel's step for it, whose minus and plus stop on its multiples. |
| `mic_on`, `sound_on` | | The microphone, and what the computer plays, record from a video's start. |
| `copied_days` | `COPIED_DAYS` | Days a copied video stays in the unsaved part, counted from its copy mark. |
| `min_free_mb` | kx-ipc's `SNIP_MIN_FREE_MB` | The room a video's drive keeps free. |
| `undo_max`, `stroke_max` | `UNDO_MAX`, `STROKE_MAX` | The most steps the editor's Undo takes back, and the most points one stroke keeps. |
| `shot_cap_mb` | kx-ipc's `SNIP_SHOT_MB` | The largest frozen copy of the screens. |
| `record_mb` | kx-ipc's `SNIP_RECORD_MB` | The recording job's memory cap. |
| `record_start_ms`, `record_tick_ms`, `record_end_ms` | `RecordWaits::START_MS`, `TICK_MS`, `END_MS` | How long a recording's job may take to start, to tell its time, and to end after Stop or quit. |
| `switch_ms` | `RecordWaits::SWITCH_MS` | How long Mic or Sound waits for the recording's answer. |
| `speed_ms`, `speed_mb_ms` | `RecordWaits::SPEED_MS`, `SPEED_MB_MS` | A speed copy's least deadline, and its more time for each MB of its video. |
| `pick_mb` | `PickLimits::JOB_MB` | The Save and folder dialogs' job memory cap. |
| `dialog_min`, `pick_idle_s` | `PickLimits::DIALOG_MIN`, `IDLE_S` | How long a dialog may stay open, in minutes, and how long its job waits for the next, in seconds. |
| `pick_restart_ms`, `pick_restart_max_ms` | `PickLimits::RESTART_MS`, `RESTART_MAX_MS` | The first and the longest wait before a failed dialogs' job starts again; the first never passes the longest. |
| `encode_ms`, `encode_mb_ms` | `ENCODE_MS`, `ENCODE_MB_MS` | A picture's PNG job's least deadline, and its more time for each MB of pixels. |
| `encode_idle_s`, `encode_restart_ms` | `ENCODE_IDLE_S`, `ENCODE_RESTART_MS` | How long the media job idles after a PNG, and the wait before a failed one starts again. |
| `copy_ms`, `copy_poll_ms` | `COPY_MS`, `COPY_POLL_MS` | How long Copy waits for another app to let go of the clipboard, and how often it looks; the poll never passes the wait. |
| `copy_again`, `copy_again_ms` | `COPY_AGAIN`, `COPY_AGAIN_MS` | How many times Copy tries a busy clipboard again, and the pause before each try (edge 5). |
| `note_s`, `shown_ms` | `NOTE_S`, `SHOWN_MS` | How long a note shows, in seconds, and a Copy's or Save's tick, in milliseconds. |
| `aside_ms` | `ASIDE_MS` | How long the freeze waits for the keyboard to step aside, in milliseconds. |
| `frame_min_px` | `FRAME_MIN_PX` | The least side, in physical pixels, a region's or a crop's frame keeps once a handle moves; a video region takes the even number at or above it. |
| `editor` | empty, or `x`, `y`, `w` and `h` with a width and a height | The editor's last size and place, in physical pixels. |
| `editor_full` | empty, or a rectangle as `editor`, only with an `editor` | The work area the editor filled when last maximised, which names its screen; it opens maximised there over `editor`, or at its default size, centred on the main screen, once that screen is gone. |

## Public API

`SnipModule`, `ID`, `MANIFEST`, `SnipSettings` (with `from_defaults`, `shot_cap`, `copy_wait`, `record_waits` and `pick_limits`), its `SPEC`, and the module's own ranges above. The modules `edit` (`Canvas` and `Did`), `flow` (`Flow` with its `In`, `Out` and `Stage`), `frame` (`spot`, `hit` and `moved`), `part` (`read`, `window`, `screen`, `home`, `Pick` and `Home`) and `strip` (`place`, `bar` and `Size`) are public for the actor and the view.

## Tests

`cargo test -p kx-mod-snip`: the manifest's gates, every setting's defaults, ends and names, with property tests over the ranges and frame rates, in `src/settings/tests.rs`; the capture flow's every path on a test clock, with the property that no path keeps a file the user did not choose, in `src/flow/tests.rs` and `src/flow/tests/`, where `reel.rs` drives a recorded video's strip, `frame.rs` a region's frame, and `canvas.rs` joins the editor's model to it; the editor's model (each tool, Crop's waiting frame, Undo and Redo, the Eraser's reach at each scale, Text in Arabic too), with property tests, in `src/edit/tests.rs` and `src/edit/tests/`; the frame rules, with property tests, in `src/frame/tests.rs` and `src/frame/tests/`; the picking rules and where the strip goes, with property tests, in `src/part/tests.rs` and `src/strip/tests.rs`; the actor over every port's fake, without a thread or a sleep, in `src/actor/tests.rs` and `src/actor/tests/` (Copy, Save, recordings, a recorded video's strip, the editor, every end and Quit, the sweep, the panel and the keyboard's keys), and the `CF_DIB` in `src/actor/dib/tests.rs`; the module's start, Snip key and stop on the fake host in `tests/snip/module.rs`; and the kernel's repair of a bad `[snip]` value in `tests/snip/settings.rs`.

## Security

See `docs/security/kx-mod-snip.md`.
