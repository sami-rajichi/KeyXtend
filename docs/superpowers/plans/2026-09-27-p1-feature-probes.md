# P1 stage 1b: feature probes (G19–G25)

## Goal
Prove on this PC the risky part of the selection helper, clipboard history, password detection, voice, quick-fill, snip and the language key (spec §7, §8.1–8.4, §4.3).
This is the detail of tasks 14–16 of `2026-09-27-phase-1-toolkit-test-round.md`; the pass marks are in `docs/roadmap.md` (Phase 1).

## Scope
- **In:** three parts, each ending with a checkpoint pop-up.
  - **A, harness only, no installs:** G20 clipboard, G21 password, G25 language and shortcuts, G19 detection.
  - **B, both faces:** a small tools row and three windows: the selection pill (G19), Hello quick-fill (G23), the snip overlay (G24).
  - **C, voice:** a worker process records and runs Whisper base; each face shows a caption bar (G22).
- **Out:** real panels and lists, OCR, snip editor, cloud voice, the vault store, Mac and Linux (stage 4).
- **Word** runs only if the owner opens it first (it shows a subscription dialog in every fresh Word process).
- A failed core part is a design finding. A failed window part (pill, Hello from the no-focus face, overlay, caption bar) stops that toolkit.

## Design notes
- **Spike only, light rules:** throwaway code in `spike/`; what we learn feeds P6–P11.
- **Core, toolkit-free (`spike/core/src/`):**
  - `uia.rs`: moved from the harness. It adds the focused element, its selection text and boxes, and its password flag.
  - `hello.rs`: Windows Hello consent for one of our windows. The answer is Verified, Canceled or Unavailable.
  - `capture.rs`: a frozen copy of the whole screen, and a region of it, in physical pixels.
  - `inject.rs`: gains text typing (Unicode input), for the quick-fill values and the transcripts.
  - `weak.rs`: runs a child process in a Windows job capped at 2 cores and 30 % CPU. G22 needs it now; stage 2 reuses it.
- **Faces:**
  - Each face gets a tools row: Fill User, Fill Password, Snip, Mic.
  - A selection watcher thread polls UIA and hands the result to the UI thread.
  - Every new window is non-activating through `window::guard`. The overlay still takes clicks.
- **Worker (`spike/worker`), the only process that opens the mic (spec §8.2):**
  - It records into memory and transcribes with whisper-rs and the `base` model.
  - It talks to the face in JSON lines over stdin and stdout.
- **Settings:**
  - `spike.toml [tools]`: window titles for pill, caption and overlay; `selection_poll_ms = 300`; `pill_offset_px = 8`.
    - Caption size and margin; `hello_message`.
    - `test_user` and `test_password`: fake values only.
    - The worker path, the model path, `record_max_s = 30` and the forced language.
  - `harness.toml [g19]`–`[g25]`: texts, counts and waits.
    - `[g20]`: `copies = 20` and `excluded = 5`.
    - `[g22]`: `weak_cores = 2`, `weak_rate_pct = 30` and `weak_mem_mb = 4096`.
- **Model:** `ggml-base.bin` (about 142 MB), downloaded after a yes and checked by SHA-256.
  - It is stored in `D:\dev\models` and never bundled.
  - Build with Visual Studio's CMake and whisper-rs's pre-generated bindings, so no LLVM install.

## Tasks
**Part A — harness only**
1. **UIA in core.** *Files:* `core/src/uia.rs` (moved), harness imports.
   - *Test first:* a selection box turns into a pill anchor that stays on screen. The password flag reads UIA first, then `ES_PASSWORD`.
   - *Implement:* focused element, `GetSelection` text and boxes, `IsPassword`.
   - *Verify:* `cargo test --workspace` green; G17, G18 and G6 still pass.
2. **G20 clipboard.** *Files:* `harness/src/g20.rs`, `cliplisten.rs`.
   - *Test first:* which formats mean "skip this copy".
   - *Implement:* a listener thread using `AddClipboardFormatListener`, driven through these steps:
     - the harness makes 20 copies, as its own writes and as Ctrl+C in Notepad;
     - it makes 5 copies carrying each exclusion format;
     - it pastes an older entry into Notepad with one click (set the clipboard, then Ctrl+V) and reads the file back.
     - The owner's text clipboard is kept.
   - *Verify:* `harness g20 notepad`: 20 seen, 5 skipped, the paste matches.
3. **G21 password.** *Files:* `target-window` (a `--password` flag makes its box a password box), `harness/src/g21.rs`.
   - *Test first:* the flag logic from UIA and Win32 values.
   - *Implement:* click each field (Chrome's password and text inputs, target-window with and without the flag) and read the flag.
   - *Verify:* `harness g21 chrome` and `g21 win32`: password boxes flagged, the others not.
4. **G25 language and shortcuts.** *Files:* `harness/src/g25.rs`.
   - *Test first:* the next-layout order wraps from the last installed layout to the first.
   - *Implement:* the language key asks the window in front for the next layout, EN → FR → AR → EN, and each change is read back.
     - Ctrl+C, Ctrl+V and Ctrl+Z change Notepad's text as expected.
     - Win+V opens a Windows clipboard window, and Esc closes it.
     - Alt+Tab moves the front window away and back, and nothing else is clicked.
   - *Verify:* `harness g25 notepad` passes.
5. **G19 detection.** *Files:* `harness/src/g19.rs`.
   - *Implement:* select text in Notepad and Chrome with Shift+arrows, then read it through UIA.
   - *Verify:* `harness g19 core`: the text matches, and the clipboard sequence number never changes.
   - **Checkpoint A** (pop-up), then commit.

**Part B — both faces**
6. **Tools row, selection watcher and pill.** *Files:* both faces, `spike.toml`.
   - The pill shows next to a selection without taking focus, and hides on the next click elsewhere.
   - Its Copy button sends Ctrl+C to the app in front.
   - *Verify:* both build, `dev-install` and `check-uiaccess` pass.
7. **G19 pill.** `harness g19 <slint|qt>`: the pill appears within 1 s next to the selection and the front window is unchanged; clicking Copy puts the selection on the clipboard.
8. **G23 Hello.** *Files:* `core/src/hello.rs`, both faces, `harness/src/g23.rs`.
   - *Test first:* each Hello answer maps to "type" or "type nothing".
   - *Implement:* Fill User or Fill Password asks Hello for our window. After Verified it types the test value into the field in front; after Canceled or Unavailable it types nothing.
   - *Verify:* `harness g23 <face>` on a local Chrome login page, with the owner confirming once and cancelling once. The field values are read through UIA.
9. **G24 snip.** *Files:* `core/src/capture.rs`, both faces, `harness/src/g24.rs`.
   - *Test first:* a region picked in any corner order gives the same box.
   - *Implement:* Snip freezes the screen and shows a full-screen overlay above every window, including our keyboard and the taskbar. A region is picked by click, move, click, and the image is saved in `target/spike-logs`.
   - *Verify:* `harness g24 <face>` snips a known pattern in target-window, and the saved image equals the harness's own capture at 125 %.
   - **Checkpoint B**, then commit.

**Part C — voice**
10. **Installs**, each with a yes, `kx-licence-check`, and the C: and D: space noted in `D:\dev\INSTALLED.md`:
    - crates `cpal` and `whisper-rs`;
    - the `base` model.
11. **Worker.** *Files:* `spike/worker`.
    - *Test first:* the WAV header, resampling to 16 kHz mono, and the JSON line format.
    - *Implement:* `record` runs until stop or `record_max_s`; `transcribe <lang>` follows.
12. **Mic and caption bar** in both faces.
    - Mic starts and stops the worker.
    - The caption bar sits at the bottom centre, never takes focus, and shows Listening, Transcribing, then the text.
    - The text is typed into the field in front.
13. **G22.** `harness g22 <face>` works with the owner, who speaks one sentence each in English, French and Arabic into Notepad.
    - Checks: the caption bar never took focus, and the time to text.
    - `harness g22 bench` re-runs the saved clips in normal and weak mode and reports speed against real time.
    - The clips are deleted after.
    - **Checkpoint C**, then commit and record the numbers.

## Edge cases (each maps to a test)
1. A selection box partly off screen: the pill anchor is clamped onto the screen.
2. An empty selection or a bare caret: no pill.
3. A copy with any exclusion format is skipped, even when it also holds text.
4. Two clipboard updates in quick succession are both counted.
5. The password flag comes from UIA, falls back to `ES_PASSWORD`, and is false when neither says so.
6. The next layout wraps from last to first; a PC with one layout reports that it cannot switch.
7. Hello unavailable (no PIN set up): Fill says so and types nothing.
8. A snip region picked right-to-left or bottom-to-top gives the same box.
9. A recording longer than `record_max_s` stops by itself; a missing model gives a clear error.
10. A failed weak-mode job setup is reported, never ignored.

## Security
- **Capabilities:**
  - UIA read of the focused element (selection, password flag);
  - the clipboard, where the owner's text is kept and non-text refused;
  - the mic, in the worker only;
  - network only for the one model download, after a yes.
- **Data:**
  - selections and clipboard entries are test texts;
  - voice clips are Sensitive: they are kept only for the bench run, then deleted;
  - the fill values are fake.
- **Threats to test:**
  - The pill, overlay and caption bar never take focus.
  - Fill types only after Verified, and only into the window in front.
  - The Win+V and Alt+Tab steps never click the owner's windows.

## Manual Windows check (owner)
- G23: confirm Windows Hello with your own PIN or face once per face, and press Cancel once.
- G22: speak three short sentences (English, French, Arabic) per face.

## Done when
G19–G25 print their numbers and checkpoints A, B and C are answered. The results are in `docs/NEXT-SESSION.md`, ready for ADR-0013, and every test file and voice clip is deleted.
