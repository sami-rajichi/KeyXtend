# Snip and screen recording — design (P12)

Decided with the owner on 2026-11-23, from the mock-up [`design/snip.html`](../../../design/snip.html): capture bar B (Keycaps), the editor with its tools on top, and videos at 30 frames a second. This file replaces spec §8.4 of [`2026-09-26-accessible-keyboard-design-modules.md`](2026-09-26-accessible-keyboard-design-modules.md), the roadmap's Phase 12 lines and ADR-0007's OCR line (ADR-0055). Every number below is a `[snip]` setting or a theme token unless it says otherwise; one design serves Windows, macOS and Linux, and only the adapters differ.

## Why

- With one finger and no keyboard, Print Screen, Win+Shift+S and Win+Alt+R are out of reach, and Windows' Snipping Tool needs a drag for a region.
- The owner wants "a Snipping Tool, not all of it": pictures of a part, a window or the screen, short videos, a light editor, and nothing kept unless asked (owner, 2026-11-23).
- References: Windows Snipping Tool (ways, auto copy, its editor's pens and shapes), Firefox and Zen screenshots (a bar of actions under the picked part), the macOS Screenshot bar (photo and video ways side by side), and P10's keycap strip.

## Goals, non-goals, anti-goals

- **Goals:**
  - one click starts a capture; every way is one more click, and a region is click, move, click;
  - a picture is copied, edited or saved, and a video played, sped up, copied or saved, from a strip of keycaps under it;
  - a small editor for pictures: crop, pen, highlighter, shapes, typed text, a cover box, numbered steps, an eraser, undo and redo;
  - videos with sound off by default, the microphone and the computer's sound each one click away;
  - the keyboard waits as its bubble during a capture and is never in the picture, unless the user asks for it.
- **Non-goals in P12 (owner, 2026-11-23):**
  - OCR, in any engine (ADR-0055); it may come back later as an idea;
  - pin on screen, scrolling capture, a timer for pictures, freeform shapes;
  - drawing on a video, trimming, pausing, GIF;
  - saving anything by itself, sharing or uploading.
- **Anti-goals (a failure even if it works):**
  - a picture or video written to a place the user did not choose, beside the Unsaved part below;
  - our capture bar, strip, record bar, edge or countdown inside a picture or video;
  - the keyboard in a picture or video while `keyboard` is not `shown`;
  - sound recorded while its switch is off;
  - anything sent to the network; picture or video content in a log;
  - the uiAccess process decoding a video, loading the system's file dialog, or loading video codecs.

## What the user sees

### Start

- One click on the side block's Snip key freezes every screen and dims it. The key stays lit until the capture ends. While the user picks, a second click on it, Close, or our keyboard's Esc ends it with nothing kept.
- The **capture bar** sits at the top centre of the screen holding the pointer: keycaps on the keyboard's plate, in the theme in front, each with an icon and a word: Region, Window, Full screen, then Video region and Video full screen (their icons drawn in the recording red), then Snip settings and Close (icons alone, named for readers).
- Region is ready at once. A hint under the bar says what to click, in the keyboard's language.

### Pictures

- **Region:** click a corner, move, click the opposite corner; the part stays clear with its size in pixels above it, the rest dimmed. Clicking the same spot twice drops the corner, and Region waits again.
- **The frame (plan 2026-11-26):** after the second corner, a dashed frame with eight handles shows round the part, for a picture or a video region.
  - Click a handle to pick it up, move, and click to set it, never a drag; each side it moves keeps at least `frame_min_px`, and a click away from the frame starts a new region there.
  - The strip hides while a handle is held and shows again beside the part once it is set. Esc drops the hold first, then ends the snip as Close below says.
- **Window:** the window under the pointer gets an outline from the window list read at the freeze; one click takes its frame, without its shadow, as it looked on screen.
- **Full screen:** one click takes the screen holding the pointer.
- The picture is cut from the frozen screen, so it is exact to the pixel (G24) and nothing moves while the user picks.

### The strip

- A strip of keycaps, like P10's selection chips: **Copy**, **Edit**, **Save**, **Close**. It sits under the part, above it when there is no room, or at the bottom centre for a full screen; it is never in the picture.
- **Copy** puts the picture on the clipboard, shows a tick and "Copied", and ends the snip.
- **Edit** opens the editor with the picture and ends the frozen screen.
- **Save** opens the system's Save dialog at the save folder (below); a saved file shows a tick and "Saved" and ends the snip; Cancel in the dialog comes back to the strip.
- **Close** asks first when the snip was neither copied nor saved (below).

### Videos

- **Video region** is picked as Region is; **Video full screen** takes the screen holding the pointer.
- The strip then shows **Record**, **Mic**, **Sound** and **Close**. Mic (the default microphone) and Sound (what the computer plays, key sounds included) start as `mic_on` and `sound_on`, both off, and each click switches one.
- **Record** lifts the frozen screen and counts down `countdown_s` seconds (3) in the part's centre; only videos count down (owner).
- While recording, a thin red edge shows what is filmed, and the **record bar** above it (or at the top centre for a full screen) shows a pulsing dot, the time, Mic, Sound, **Stop** and **Cancel**. Mic and Sound can switch at any time; off records silence.
- The Snip key or Stop ends the recording; Cancel ends it and deletes it, with no question.
- It also stops by itself at `max_min` minutes, or when the drive would drop under `min_free_mb`, with a note, keeping what was recorded.
- After Stop, or a recording that stops by itself, the strip shows the video's length, then **Play**, **Speed**, **Copy**, **Save** and **Close**; a video has no editor (ADR-0060).
- **Play** opens the video in the user's own video app; our strip and keyboard stay on top. If it cannot open, a note says so and the strip stays.
- **Speed** is one key that steps 1×, 2×, 4× and back, showing its speed after its word; the length shows at that speed (frames are dropped).
- Copy and Save give the video at the picked speed, and the strip stays with its tick, so another speed can be copied too. Copy puts the video on the clipboard as a file, so it pastes into chats, mail and folders that take files; a plain text field pastes its path instead.

### Close

- Closing a strip or the editor whose snip was neither copied nor saved since its last change asks: "This snip is not saved. Save it before closing?" with **Save**, **Don't save** and **Back** (owner). Esc means Back, as destructive questions default to safe, except over frozen screens, where Esc from our keyboard, the hand or another program means Don't save, so a second Esc always ends the snip (ADR-0059).
- Don't save deletes it for good; nothing is kept anywhere (owner).

### The editor

- Its own window, not scaled with the keyboard: a comfortable default size, smaller down to a minimum, or maximised (owner). It moves by its grip and resizes by its corner, each by click, move, click; it stays above other apps and under the keyboard, and never takes focus.
- **Top row:** Crop, Pen, Highlighter, Box, Circle, Arrow, Line, Text, Cover, Steps, Eraser; then Undo and Redo; then Copy and Save. Icons only, each named in a tip and for readers.
- **Second row:** the tool's name, six inks (red, yellow, green, blue, black, white) and the thickness, or the text size for Text, each by − / + steps.
- **Gestures, with one click each, never a drag:**
  - Pen and Highlighter: click to put the pen down, move, click to lift it;
  - Box, Circle, Arrow, Line, Cover and Crop: click the start, move, click the end;
  - Text: click where it goes; the keyboard's letters go into it as on a panel's line (ADR-0040), Arabic right to left; Enter or a click elsewhere ends it;
  - Steps: each click drops the next number;
  - Eraser: a click on a drawing removes just that drawing.
- **Crop's frame (plan 2026-11-26):** Crop's second click frames the part picked, over the picture as it shows, with the region's eight handles, set the same way; the frame stays within the part kept so far.
  - Apply (✓) cuts as one undoable step, and Cancel (✗) drops the frame; while a handle is held, the two keys hide.
  - Esc drops a held handle, then the frame, then reaches the close question. Picking Crop again keeps a waiting frame; another tool or Undo drops it.
- The picture always fits the window; no scrolling. Copy and Save give the picture at the screen's own pixels with the drawings drawn at that size; both keep the editor open, with a short "Copied" or "Saved".
- **Pictures only.** A video has no editor; its strip plays and speeds it (ADR-0060).

### The Save dialog and the folder

- Save opens the system's own Save dialog, as every app does (owner), at `save_dir`; empty means the Snips folder in KeyXtend's data folder. The name proposed is the snip's date and time, PNG for a picture and MP4 for a video.
- A recording is written while it records into the Snips folder's `unsaved` part. Don't save, Cancel and Close delete it; a copied recording stays there `copied_days` (1) so a paste from the clipboard history still works that day, then goes at the next start.
- A copied picture is kept by the clipboard history as any copy is, and a copied video as a file reference, as history keeps media (spec §7).

### The keyboard during a capture

- `keyboard` (owner):
  - **Bubble** (default): the keyboard turns into its bubble while the user picks and records, and comes back when the capture ends or the editor opens. A click on the bubble brings it back as usual, to scroll or type, and it still stays out of the picture.
  - **Hidden:** neither the keyboard nor its bubble shows until the capture ends; the bar, strip and record bar still show.
  - **Shown:** the keyboard stays where it is and is in the picture or video.
- Our capture bar, strip, hint, edge, countdown and record bar are never in a picture or video, in every case.

### The Snip panel

- Snip's settings live in its own panel, as Voice's do (owner), opened from the capture bar's settings key and from the tray menu. It opens above the keyboard and pages like every panel.
- **Capture page:** Keyboard during a capture (Bubble, Hidden, Shown, with one line saying what it does); the folder the Save dialog opens at, with Change (the system's folder dialog) and Open folder.
- **Video page:** countdown (− / +, 0 to 10 s), frames per second (15 or 30), longest video (− / +, 1 to 120 min), microphone at start and computer sound at start (Off or On).

## Every platform

- The capture flow, the strip, the editor, the panel and every rule above are shared code. Each OS gives four adapters behind neutral ports: freeze the screens and list the windows; keep our windows out of captures; record a part with sound into an MP4; and its own Save and folder dialogs.
- Windows now; macOS and Linux in P15. On Wayland the system asks the user through its portal before any capture; where an OS cannot keep a window out of a capture, our windows step aside for that moment instead.
- Videos are made by the system's own encoder (Media Foundation on Windows), so Snip adds nothing to the download (owner: no size growth). Windows "N" editions without the Media Feature Pack dim both video keys, with a tip saying why.

## Settings `[snip]`

- `keyboard` "bubble" ("bubble", "hidden", "shown"); `save_dir` "" (empty: the Snips folder); `countdown_s` 3 (0–10); `fps` 30 (15 or 30); `max_min` 30 (1–120).
- `mic_on` false; `sound_on` false; `copied_days` 1 (1–7); `min_free_mb` 2048 (512–20480); `undo_max` 50 (10–200); `shot_cap_mb` 256 (64–1024, the largest frozen copy, from the spike); `record_mb` 512 (256–2048, the recording job's memory cap).
- The editor's last size and place, and whether it was maximised, are remembered, as the keyboard's are.
  - A maximised editor whose screen is gone opens at the theme's default size, centred on the main screen.
  - An unmaximised editor whose screen is gone opens at that default size on the pointer's screen.

## Security and privacy

- Pictures and videos can hold anything on screen. They live in memory, the clipboard and files the user chose; only a recording writes to the `unsaved` part, and it goes as above. None is ever logged.
- The uiAccess app freezes the screen as raw pixels and draws them; it decodes no file. PNG files are made in the worker (ADR-0034), and recording, sound and the MP4 encoder run in a worker job with normal rights. The Save and folder dialogs run outside the uiAccess process, so no shell extension loads into it (ADR-0056).
- No network is touched. Sound is heard only while its switch is on.

## Accessibility

- Every action is short left clicks: no drag, no scroll, no right click (ADR-0008). Every control has a role, a name and a tip in the keyboard's language; Arabic flips text only (ADR-0020).
- Every surface follows the theme in front, light and dark and high contrast, and passes the theme's contrast check; motion follows reduced motion, and the recording dot stops pulsing there.

## As built (P12, 2026-11-25)

Where the build settles or departs from the text above, one or two sentences each; the P12 plan's rulings and task reports are the record. The owner's hands-on check confirms each look; the owner accepted ADR-0057 and ADR-0058 on 2026-11-25.

### Start and picking

- **The freeze waits for the step aside.** With the keyboard aside, or in every mode for a snip from the editor or its question (`SnipAside::Stay` when Shown), `start` tells the ui and freezes once it answers that our windows are shielded or gone. If no answer comes within `aside_ms`, the freeze goes anyway.
- **Every other window of ours steps aside.** In Bubble and Hidden every window of ours on the aside list (`WINDOWS` and the tray menu) is shielded during the aside, not only the keyboard and its bubble; one Windows refuses turns the aside Hidden.
- **Snip's own windows stay off that list (`SNIP_WINDOWS`).** The six capture views shield themselves, while the overlay, the editor and the close question are not shielded. None of these three shows while a freeze or a recording runs: the question shows only over the done strip or the editor, and Record hides it; a snip started from the editor or its question freezes only once the ui says they are gone, or at `aside_ms`.
- **The Snip key and Esc per step.** The key ends a snip while picking or counting down, stops a recording, and closes the strip or editor, asking first when nothing was kept; from the editor a new snip then starts. Esc does the same but never starts a snip, so while recording it means Stop, never Cancel.
- **The key's light goes off when the editor opens,** as the frozen screen ends there and the keyboard comes back.
- **Keys only after a press (ADR-0057).** Snip's windows take the keyboard's letters only after the user's press on them or on the Snip key, and may show beside one other panel. A strip that shows by itself never takes the keys from the app in front.
- **A hidden run (ADR-0037)** makes no Snips folder, so Video, Record and Save dim with a tip, and Copy still works.
- **The frozen screens never trap (ADR-0059).** While they show they lie under Snip's other windows and the keyboard, at each stage and after a move or a new scale, and an Esc from the hand or another program, such as Voice Access, ends a pick or closes the strip as our keyboard's Esc does, still reaching the app. No timer ends a snip.

### Videos

- **A video region stays on one screen.** Its moving corner stops at the edge of the first corner's screen, and the second click cuts each side to an even length of at least 2 pixels, so the size tag may show one pixel more than the video gets; a picture's region may still cross screens (edge 2).
- **The countdown has a Cancel key** under its number, so one click ends it; the mock-up's countdown has none.
- **Bare dim and note.** When Windows will not keep a capture window out of captures, both video ways and Record dim (`Dim::Bare`). A countdown then ends, or a recording stops, with a note (`SnipNote::Bare`), so no video runs with a hidden record bar.
- **The record bar's place.** It and a recorded video's strip use each screen's work area, so the taskbar never hides them; a part touching its screen's top puts the bar at the top, as the mock-up does.
- **Play opens the video at the speed the Speed key shows.** At 2× or 4× it opens the re-timed copy that Copy and Save use, made once per speed; the strip's keys wait while it is made, and a failed copy shows its note.
- **A key waiting for that copy shows it.** While the copy is written (after a Save's dialog), Play, Copy or Save, whichever asked, slides part of a line along its foot (the voice panel's writing line, shared, in the key's ring colour) and says why to readers and in its tip; reduced motion draws the line still.
- **The countdown after a Cancel.** Its hide lets go of the Cancel's press, its pop rests on time and it draws again then and at each later second, and the frozen screens hide in the turn Record drops them. Each second logs at debug, numbers only, as its view comes and as it shows.
- **`[ui] frame_latency` is 0.** Qt's own frame-latency wait stalled a Snip window shown again for about 1 s a frame, which froze the countdown after a Cancel; 0 skips that wait, and Direct3D's own queue then bounds Present (ADR-0061; owner, 2026-11-27, live logs).
- **Speed copies.** A copy at one of the strip's speeds (`SNIP_SPEEDS`) speeds up its sound too, which plays higher as fast-forward does. It decodes and encodes on the processor; only live recording keeps frames on the graphics card (ADR-0056 decision 4).
- **The encoder's values** (bit rates, waits, buffers, the sound's queue, cushion, drift and downmix) live in the adapter table `record.toml`, not in `[snip]`.
- **"Kept" means copied or saved since the last change,** and a speed change counts as one. A recording that stops by itself goes to the strip with its note and is not kept, so Close still asks.
- **G24 clicks a video's Speed, Copy and Close, not its Save,** though ADR-0060 names Save; G24's Save clicks are on picture strips, and a video's Save is checked by hand.

### Copy, Save and the folder

- **Copied videos are marked.** Copy writes a small `.copied` mark beside the video, and each start and each snip's start deletes every PNG and MP4 of `unsaved/`, with its mark, but a video marked within `copied_days` or a file a job still uses, then marks whose video is gone, so "copied" survives a restart.
- **Windows' history keeps pictures only.** Copy writes without our marker (`write_kept`), so KeyXtend's history keeps both kinds, but Windows' own history never keeps a file, so it misses a copied video; ADR-0056 decision 2 says both histories.
- **Copy and its notes.** A busy clipboard is tried again `copy_again` times, and a refused Copy and a failed speed change have notes of their own (`CopyFailed`, `SpeedFailed`).
- **Save writes whole or nothing:** a `.kxpart` file beside the target, renamed over it once whole.
- **Names.** A recording in `unsaved/` is named `<unix seconds>.mp4`, which a pasted copy carries; the Save dialog proposes the snip's local date and time.
- **The dialog in front.** While a Save or folder dialog shows, Snip's windows leave the topmost band, the keyboard stays above, and the app grants the worker the foreground (ADR-0056's open point, checked on a dev-install at the hands-on check).
- **A mapped network drive** passes the dialog's check, which is lexical, as the user chose that place.

### Settings

- **New `[snip]` keys:** the copy's `copy_ms`, `copy_poll_ms`, `copy_again` and `copy_again_ms`; `note_s`, `shown_ms`, `aside_ms`; `editor`, the editor's last size and place, and `editor_full`, the work area it filled when last maximised; `frame_min_px` (8), the least side a region's or a crop's frame keeps; the panel's steps `countdown_step_s` and `max_min_step`; `stroke_max`; the recorder's waits `record_start_ms`, `record_tick_ms`, `record_end_ms`, `switch_ms`, `speed_ms` and `speed_mb_ms`; the dialog job's `pick_mb`, `dialog_min`, `pick_idle_s`, `pick_restart_ms` and `pick_restart_max_ms`; and the PNG job's `encode_ms`, `encode_mb_ms`, `encode_idle_s` and `encode_restart_ms`. Their defaults are in `crates/kx-mod-snip/defaults.toml` and their ranges in its `src/settings.rs` and `src/settings/jobs.rs`; the Snip panel leaves them out, and P13's Settings window shows them (`docs/items/P13.md`).

### The editor

- **Its least size** is the theme's `editor_min_px` (`crates/kx-theme/tokens/snip.toml`), so the French tool row fits.
- **Gestures where the spec is silent:** a drawing joins at its end click, a second click on the same spot drops the open gesture, Undo cancels an open gesture first, the Eraser takes the top-most drawing under the click, Steps go on from the highest number shown, and Crop's Apply is one undoable step.
- **Copy and Save flatten the picture** without the typing outline or caret.
- **Its windows are made once at start (plan 2026-11-26),** the shadow before the editor, so Edit shows them at once instead of making them then.

### Look

- **`rec_light` and `ring_act`.** The video ways' red light and Dolch light's picked-way ring use new theme tokens chosen to reach 3:1 on their keys (WCAG 1.4.11); a theme without them falls back to `rec` and `ring`.
- **Restore has its own icon,** Lucide minimize-2 (owner, 2026-11-25); the mock-up's key shows maximize-2 in both states.
- **The strip's Record has no red light:** it wears Enter's colours, on which no red reaches 3:1.
- **Speed's icon is a stand-in,** Lucide chevron-right, until a fast-forward mark ships after its licence check (ADR-0060).
- **Each shadow shows with its window (plan 2026-11-26),** never before it or above it, so no dark flash comes first.

### Security and the plan

- **A `Speech` capability (ADR-0058).** Speech gets its own gate, so `Microphone` gates only Snip's microphone pass, and the voice module no longer holds it.
- **The `Snip` event gate.** Every snip event needs `Snip`, held by the module, the keyboard and the ui (ADR-0057 decision 5); ADR-0056 decision 10's list leaves it out.
- **`tool_box` went with the spike's tools row** (task 19), though the plan kept it for G7 and G25; neither gate called it.
