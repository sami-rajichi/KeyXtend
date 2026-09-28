# Accessible on-screen keyboard — design spec

- **Date:** 2026-09-26
- **Status:** The product owner approved the features and UX in question rounds 1–6 and mock-up v2. The architecture (§11) is proposed and awaits the owner's review. Open decisions are listed in §14.
- **Product owner:** see `.github/CODEOWNERS`
- **Name:** KeyXtend (the eXtended keyboard); code prefix `kx-`
- **Interactive mock-up:** https://claude.ai/artifact/3mftKZSPBvVqiqf33bvaJB (file: `design/keyboard-style-lab.html`)

---

## 1. Why this exists

KeyXtend is for people who use a computer with **a single pointer button**, for example a finger on a mouse or touchpad, a head or eye pointer with a dwell click, or a mouth stick. They cannot use:

- a right-click;
- a scroll wheel;
- a physical keyboard;
- dragging while holding the button.

The first languages are **English, French and Arabic**; any other language installed in the OS is detected too (§4.3).

Windows' On-Screen Keyboard (osk.exe) is the only tool that stays on top of everything, but it:

- looks dated and has no themes;
- renders keys tiny in some setups;
- has no clipboard history, voice typing or mouse helpers.

The Windows 11 touch keyboard is built for touch screens. macOS Dwell and OptiKey solve parts of the mouse problem, but neither covers Arabic or one-button use on Windows.

**Goal:** a small, fast, open-source, cross-platform on-screen keyboard that lets a one-button pointer user do *everything* a keyboard-and-mouse user can do.

**Success criteria**

1. A user can type EN/FR/AR into any app, including admin windows, without losing characters or focus.
2. A user can right-click, drag and drop, select text and scroll anywhere, using only short left clicks and 1.5-second holds.
3. No part of the product needs a scroll wheel, a scrollbar drag or a right-click to operate (see §3.3).
4. Idle cost is small enough to leave it running all day (budgets in §10).
5. The product is safe to publish on GitHub: licences are clean, releases are signed and verifiable, and no user data leaves the PC unless the user opts in.

## 2. Scope

**Platforms**

- **Windows 10/11 first-class.**
- macOS and Linux must work later from the same codebase. Platform code lives behind ports (see ARCHITECTURE.md).

**In scope for v1.0 (Windows)**

- Keyboard core, languages and layouts, modifiers, shortcuts layer.
- Themes and window behaviour.
- Mouse assist: Right-click hold, Grab for drag-drop and text selection.
- Scroll pad.
- Word prediction, including emails.
- Clipboard history.
- Quick-fill vault.
- Selection helper.
- Voice typing.
- Snip with annotation and OCR.
- Power key.
- Settings window.

**Later:** macOS port, Linux port, and the future features in §13.

**Out of scope:**

- a touch-first layout;
- docking the keyboard to a screen edge (the keyboard is floating only);
- cloud sync;
- any telemetry.

---

## 3. Global UX rules (apply to every module)

### 3.1 The window

- **Floating only.** The keyboard is **always on top of every window**, including the Start menu, Search and Task Manager. This is a mandatory priority and needs Windows uiAccess (ADR-0003).
- **Never takes focus.** Clicking a key never activates our window; the target app keeps the caret. Windows: `WS_EX_NOACTIVATE | WS_EX_TOPMOST | WS_EX_TOOLWINDOW`, with `WM_MOUSEACTIVATE` returning `MA_NOACTIVATE`. macOS: non-activating `NSPanel`.
- **Clicks pass around it.** The shadow and any empty space around the keyboard never block a click on the app behind.
- **Size:** small by default. Presets S = 1.00×, M = 1.25×, L = 1.50×, plus fine steps of 0.05× between 0.80× and 1.80×. Icon keys never show labels under the icon; at every size, hovering any icon button shows its name in a small tooltip.
- **Move** without dragging: click the grip, move the pointer, click to place.
- **Resize** the same way with the corner handle. There is no Esc to undo: Esc belongs to the app, so Settings has "Reset size" (owner, 2026-09-28).
- **Minimise** puts a bubble in a bottom corner of the keyboard's screen, above the taskbar, wherever the keyboard was: right by default, left as a setting, never the top (owner, 2026-09-28).
- **Remembers its state:** size, position, theme, mode and every setting survive a restart or shutdown.
- **Top bar:** grip, suggestion chips, light/dark toggle (sun/moon), size buttons (lens −/+ and S/M/L), minimise-to-bubble (−), close-to-tray.
- **Fade when idle** was dropped (owner, 2026-09-28): minimise-to-bubble does the job better.

### 3.2 Feedback

- **Wave ring:** during a hold, a ring appears after 150 ms and shrinks into the pointer over the rest of the hold time. When the hold fires, a burst of 6 dots appears.
- **Hover:** a key under the pointer turns a little darker in light mode and lighter in dark mode.
- **Mode badge** next to the pointer: Right-click on, Grab/Holding/Selecting, scroll direction and speed, recording.
- **Sounds**, soft and never harsh:
  - a bubble "pop" when a hold fires;
  - a soft tick on key press;
  - a drop sound;
  - distinct start and stop chimes for the mic.
- Each feedback type can be switched off. Sound files are generated by us and released under CC0.

### 3.3 No-scroll, no-drag UI (hard rule)

- **Lists** (clipboard, quick-fill, settings lists) show exactly **5 rows per page**.
- Every list has large **▲/▼ page buttons** and a "page n of m" indicator. A short last page is padded so paging lands on whole pages.
- While a panel is open, the keyboard's own keys drive it: **↑/↓** move the highlight, **←/→** switch tabs, **Enter** activates, **Space** ticks, **Esc** closes.
- The scroll pad can also scroll any list.
- No sliders. Every numeric setting uses **−/+ steppers**.

### 3.4 Scaling and placement

- All panels scale with the keyboard size (one `ks` scale factor).
- A panel opens **above** the keyboard, or **below** it if there is no room. If it still does not fit, it shrinks to fit, but never below 0.80×. It never covers the keys unless the screen is physically too small.
- The voice caption bar sits at the bottom centre of the screen and moves up to stay clear of the keyboard and any open panel.

### 3.5 Themes

Three themes. Each has a **light** version and a **soft dark** version:

- no pure black: dark greys around #2A2C2E–#3B3E43;
- off-white legends;
- contrast of at least 4.5:1 for legends (target 7:1).

References: Material dark-theme guidance, GitHub "Dark dimmed".

| Theme | Idea | Fonts (all OFL) |
|---|---|---|
| **Native Adaptive (default, light)** | Looks like part of the OS: Fluent geometry (4 px keys in an 8 px window), Mica-like plate, its own blue (system accent as an option) | System UI font + Noto Sans Arabic |
| ET66 | Braun ET66: colour marks function. Brown = modifiers, yellow = Enter, a green LED = on/locked | IBM Plex Sans + IBM Plex Sans Arabic |
| Modern Dolch | GMK Modern Dolch keycaps: grey letters, dark modifiers, teal Enter, rose Esc/Backspace, pressable inset | Rubik |

- Light, dark, or **follow system** can be chosen in Settings or from the keyboard's sun/moon button.
- Icons: **Lucide** (ISC). The icons are platform-specific: the Windows logo on Windows, ⌘⌥⌃ on macOS, "Super" on Linux.
- Exact colour tokens live in the mock-up file (`design/keyboard-style-lab.html`, CSS `[data-style][data-mode]` blocks) and move to `ui/theme.slint`.

### 3.6 Languages of the UI

- The keyboard's own UI text (settings, tooltips) is translatable from day one (Slint `@tr`).
- v1 ships EN, FR and AR, including right-to-left layout of the settings window for Arabic.
- Slint has no automatic RTL mirroring (#2294), so we mirror by hand with a `Dir.rtl` global.
- Slint's in-app Arabic text *editing* has a known caret bug (#7841). Our UI therefore prefers pickers and steppers over text fields. Typing into *other* apps is unaffected.

---

## 4. Keyboard core

### 4.1 Layout (main block, 16 units wide on a 64-column grid)

| Row | Keys (width in units) |
|---|---|
| 1 | Esc 1 · 13 character keys · Backspace 2 |
| 2 | Tab 1.5 · 13 character keys · Del 1.5 |
| 3 | Caps 1.75 · 11 character keys · Enter 3.25 |
| 4 | Shift 2.25 · 10 character keys · Shift 1.75 · ↑ 1 · Menu 1 |
| 5 | Ctrl 1.25 · Win/⌥/Super 1 · Alt/⌘ 1 · Language 1.75 · Space 7 · AltGr/Alt/⌘ 1 · ← ↓ → |

- **Removed on purpose:** Home, End, Insert, PgUp, PgDn, the nav block, and F-keys. F-keys can come later in the shortcuts layer.
- **Replaced:** PrtScn becomes **Snip**; Help becomes **Settings**.

### 4.2 Side block (3 columns × 5 rows)

| | Col 1 | Col 2 | Col 3 |
|---|---|---|---|
| Row 1 | Right-click (toggle) | Grab (toggle) | Shortcuts layer |
| Rows 2–4 | Scroll pad (round, 4 directions + centre Stop), spanning cols 1–2 | | Mic · Clipboard · Snip |
| Row 5 | Quick-fill | Settings | Power |

The **shortcuts layer** replaces the side block with 15 one-click shortcuts:

- Select all, Copy, Cut, Paste, Undo, Redo;
- Find, Save, Switch app, Show desktop;
- New tab, Close tab, Emoji, Print;
- Back.

Each shortcut maps to the platform's combination, for example Ctrl+C on Windows and ⌘C on macOS.

### 4.3 Languages and layouts

- Detect every input language installed in the OS. Windows: `GetKeyboardLayoutList`. Labels are read per layout with `ToUnicodeEx`, flag `0x4` (does not change kernel keyboard state).
- v1 must render and type **English (US QWERTY)**, **French (AZERTY)** and **Arabic (101)** correctly:
  - AltGr level on AZERTY (e.g. `€ ~ # { [ | \` \\ ^ @ ] }`);
  - Arabic shifted level with harakat shown on a dotted circle `◌`;
  - `لا` as one key that types two code points.
- Keys show **only the active language**.
- **Language key** — three designs are prototyped; the owner's pick is saved from the mock-up:
  - **Carousel (default):** shows previous · current · next. Click the left third = previous, anywhere else = next. It swipes with animation.
  - **Cycle:** globe icon plus the current language and dots.
  - **List:** a popover listing all languages.
- Changing language switches our labels **and** asks the target window to switch too (`WM_INPUTLANGCHANGEREQUEST`), so the OS indicator and IMEs stay in sync.

### 4.4 Modifiers (osk.exe behaviour, decided in round 6)

- Shift, Ctrl, Alt, Win and AltGr: **one click = held**, shown filled with the LED on. **Click again = released.**
- **Release after next key** (default): a held modifier releases automatically after the next non-modifier key. Setting **"Stay held"**: it stays down until clicked again.
- There is no third "lock" state; the owner found it confusing.
- **Caps Lock** is a simple toggle, labelled "Caps" as in osk.exe so it never looks like Shift.
- **Modifier + mouse:** while a modifier is held, the next mouse click is sent with that modifier down, so Shift+click extends a selection and Ctrl+click multi-selects. This is essential for one-button users.
- Keys pressed while one of our panels is open go to that panel (§3.3), not to the target app.

### 4.5 Sending input (Windows)

- **Characters:** `SendInput` with `KEYEVENTF_UNICODE`. The result does not depend on the target's layout, so AZERTY and Arabic are never garbled.
- **Shortcuts, navigation keys and modifiers:** virtual-key/scan-code events, for example Ctrl+C = `VK_CONTROL` + `VK_C`, because many apps ignore Unicode events when combined with Ctrl.
- **Per-app fallback:** some apps drop `VK_PACKET` events. For these, map the character back to a scan code in the target's layout (`VkKeyScanEx`).
- Every injected event carries our signature in `dwExtraInfo`, so our own hooks can ignore our own injections.

---

## 5. Mouse assist

### 5.1 The hold engine (shared by Right-click and Grab)

A low-level mouse hook (`WH_MOUSE_LL`) watches the physical left button.

**Hook discipline**

- The hook callback does **no work beyond a queue push** and returns within about 1 ms.
- Windows silently removes hooks that exceed `LowLevelHooksTimeout`. A health monitor detects a removed hook and reinstalls it (§11).

**State machine** (pure Rust, clock injected, fully unit-tested)

```
Idle ──left down (mode on)──▶ Pending(t0, p0)           [press is held back]
Pending ──up before T──▶ replay click at p0 ──▶ Idle     (normal click, instant)
Pending ──moved > 7 px──▶ replay down at p0, then pass moves through ──▶ Idle (normal drag)
Pending ──T elapsed, still──▶ Fire
  Right-click mode: inject right down+up at p0; the swallowed left press is never sent
  Grab mode:        inject left down at p0 (latched) ──▶ Carrying
Carrying ──next click──▶ swallow it, inject left up there ──▶ Idle; Grab turns off
```

- **T = 1.5 s** by default. It is adjustable from 0.5 s to 3.0 s in 0.1 s steps.
- The ring shows from 150 ms. The wave shrinks over T − 150 ms.

**Rules**

- **Right-click mode never blocks normal clicks.** Short clicks and drags behave exactly as without the mode; only a *still* hold becomes a right-click. The existing selection is preserved because the left press was never delivered.
- Right-click mode is a toggle key and stays on until toggled off.
- **Grab** turns Right-click off while it is active. After a drop, Grab turns itself off and **Right-click comes back** if it was on before.
- **Drop = one single click** on the target. Esc cancels a carry.
- Holds that start on our own keyboard, panels or overlays are never intercepted.
- Grab covers **drag-and-drop** (files, windows, images) and **text selection**: hold at the start, move, click at the end. A selection made this way is then offered to the Selection helper (§8.3).

### 5.2 Scroll pad

- A round pad with ▲ ▼ ◀ ▶ (PlayStation-style) and a centre **Stop** button that appears only while scrolling.
- **Tap a direction = slow. Tap it again = fast. Third tap = stop** (decided in round 6). There are **two speeds only**, and two level dots show the current speed.
- Tapping another direction switches direction at slow speed.
- **Target:** the scrollable thing under the pointer, including long text fields and our own panel lists. While the pointer is on the keyboard, the target is the **last point the pointer was at outside the keyboard**. The target gets a subtle outline.
- **Windows mechanism, in priority order** (to be proven in the spike):
  1. UI Automation `ScrollPattern` on the element at the target point.
  2. `WM_MOUSEWHEEL`/`WM_MOUSEHWHEEL` posted to the window at the target point.
  3. `SendInput` wheel events, when the pointer itself is on the target.

### 5.3 Overlays

- The ring, burst and mode badge are drawn in **one native click-through overlay window**: `WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOPMOST`.
- It is painted with tiny-skia and `UpdateLayeredWindow` by the platform adapter, not by Slint, whose transparent and click-through windows have open bugs (ADR-0002).
- It is updated only while something is animating.

---

## 6. Word prediction

- Up to 4 chips in the top bar. Tapping a chip replaces the current word and adds a space.
- **Engine:** our own small Rust engine.
  - A prefix trie with unigram and bigram counts per language.
  - Seeded from **wordfreq** lists (data licence CC-BY-SA-4.0, shipped as separate data files with attribution).
  - It learns from what the user types.
  - No neural model in v1.
- **Emails:**
  - Typing `@` suggests domains (gmail.com, outlook.com, hotmail.com, yahoo.com, icloud.com).
  - Typing the start of a saved email suggests it. Saved emails come from the Quick-fill vault; the chip shows a key icon.
- **Hold-to-fill** (round 5 idea):
  - holding **@** for 1.5 s lists saved emails above the field;
  - holding **P** for 1.5 s lists saved passwords (vault must be unlocked).
- **Privacy:**
  - Nothing is learned in password fields. They are detected through UI Automation `IsPassword`.
  - The user can remove a learned word.
  - The learned data is stored locally and encrypted (§9.1).
- **Optional:** the Windows `TextPredictionGenerator` API, but only where testing shows it works for the language. Its Arabic and French support is unverified.

---

## 7. Clipboard history

- Watches the clipboard. Windows: `AddClipboardFormatListener`.
- **Respects the exclusion formats** that password managers set: `ExcludeClipboardContentFromMonitorProcessing`, `CanIncludeInClipboardHistory = 0`, `Clipboard Viewer Ignore`.
- **Stores:** text, links, images, files (as **references**, never copies of big files), and video/audio (as references).
- **Retention:** 1 week by default, configurable. Video and audio: 1 day. Pinned items never expire.
- **Panel**
  - One vertical list, **newest first**, 5 per page (§3.3).
  - Each row: tick box, thumbnail or type icon, title, "source app · time · size", an expiry chip, and a pin marker.
  - Filters: All, Text, Images, Files, Media.
  - Click a row to paste it. Tick several rows (the tick order is shown as numbers), then **Paste together**.
  - Keys: ↑↓ move, Enter pastes, Space ticks, ▲▼ page, Esc closes.
- **Paste:** text is pasted as text. Images and files go into the target the same way Explorer or a browser would receive them: we set clipboard formats, then send Ctrl+V.
- **Win+V** from our keyboard opens our history instead of the Windows one (Windows only; setting).
- **Security**
  - The database is encrypted at rest (§9.1).
  - Thumbnails of untrusted images are decoded in a **worker process** (§11), never in the uiAccess process.

---

## 8. Other modules

### 8.1 Quick-fill vault

- **Tabs:** Emails (with one-click domain chips), Passwords, Names. Each is a paged list.
- Password rows have two actions: **User** fills the username, **Password** fills the password.
- **Passwords are locked** until unlocked with **Windows Hello / PIN** (`UserConsentVerifier`), Touch ID on macOS, or the system password on Linux. The vault locks again after an idle timeout.
- **Storage:** one encrypted file (§9.1). Filled text never goes to the clipboard, the prediction learner or the logs.
- **Export:**
  - an encrypted `age` file (open format);
  - optionally a plain Bitwarden-compatible CSV, behind an explicit warning.
- **Import:** the same two formats.
- The settings view gets a nicer "profile" visual in a later round. The v1 layout matches mock-up v2.

### 8.2 Voice typing

- **Mic key:** click to start, click to stop. There is no voice-activity auto-stop, so users who speak slowly or pause are never cut off.
- Distinct start and stop sounds, and a live waveform in a **caption bar at the bottom centre**, never over the keyboard.
- **Flow:** Listening, then Transcribing (shimmer), then the text is inserted into the focused field and shown in the caption with **Copy** and **Undo insert**.
- **Forced language setting:** Auto, English, French or Arabic. Auto can mis-detect Arabic, so forcing is recommended for Arabic.
- **Engines:**
  - Default: local **whisper.cpp** via `whisper-rs` (MIT).
    - Model `large-v3-turbo` q5_0 (574 MB), or `small` q5_1 (190 MB) for weak PCs.
    - Models are downloaded on first use and verified by SHA-256; they are not bundled.
    - A **benchmark button** tells the user which model their PC can handle.
  - Optional cloud, **bring your own free key**, off by default:
    - Groq (free tier: 8 audio hours/day);
    - Cloudflare Workers AI;
    - Azure Speech F0 (5 h/month; 18 Arabic locales, including Maghrebi dialects).
    - Gemini's free tier is offered only after a warning that the data may be used for training.
- Silence trimming before sending, to reduce Whisper's hallucinations on silence.
- **Transcript history:** kept locally and encrypted, 1 day by default. It can be exported later to build a personal dataset; this is opt-in.
- Runs in the **voice worker process**, the only process that may use the microphone and the network (§11).

### 8.3 Selection helper

- Whenever text is selected anywhere, a small pill with **Copy** (and **Search**) appears next to the selection, like PopClip or SnipDo.
- **Detection:**
  - Primary: UI Automation `TextPattern.GetSelection` on the focused element.
  - Fallback: our mouse hook sees a drag-select or double-click gesture.
- It **never probes** by sending Ctrl+C. Copy sends Ctrl+C only when the pill is clicked.
- The pill is non-activating and disappears on the next click elsewhere.

### 8.4 Snip

- Capture modes: **Region** (click, move, click; no dragging), **Window**, **Full screen**, **Scrolling**, **3-second timer**.
- **Editor:**
  - compact icon-only toolbars (labels appear at size L);
  - tools: arrow, box, text, highlighter, pen, numbered steps, crop, undo/redo;
  - actions: **copy text (OCR)**, copy image, save, pin on screen.
- **OCR** (in a worker): Windows.Media.Ocr first, then Tesseract `tessdata_fast` (ara/fra/eng), then PaddleOCR via ONNX for hard Arabic.
- **Capture API:** Windows.Graphics.Capture or DXGI duplication. The overlay is a full-screen topmost window in the main process.

### 8.5 Power key

Opens the **system's own** power dialog; it never shuts down directly.

| OS | Method |
|---|---|
| Windows | `Shell.Application.ShutdownWindows()`, which shows the "Shut Down Windows" dialog |
| macOS | The loginwindow "restart/shut down" prompt |
| Linux | `gnome-session-quit --power-off` or the KDE logout prompt, through the adapter |

### 8.6 Settings window

- **Quick settings first** (owner's idea, 2026-09-28):
  - the Settings key opens a small strip under the keyboard with theme, mode and size, like the P1 test strip;
  - the strip never takes focus; its "All settings" button opens the full window below;
  - it shows only when asked, never all the time.
- A normal window, not on top of everything, and never needing scrolling: tabbed pages with paged lists and −/+ steppers.
- **Contains:**
  - theme and mode, size and "Reset size", bubble corner;
  - language key design, modifier behaviour;
  - hold time, sounds;
  - clipboard retention;
  - voice engine, model, forced language and keys;
  - vault export and import;
  - privacy and data deletion;
  - about and licences.
- Settings are versioned TOML with migrations (ARCHITECTURE.md).

---

## 9. Security and privacy (applies to all modules)

### 9.1 Data at rest

- Clipboard DB, vault, learned words and transcripts are encrypted with **XChaCha20-Poly1305** (RustCrypto crates).
- The data key is protected by the OS: **DPAPI** (Windows), **Keychain** (macOS), **Secret Service** (Linux).
- The vault adds user-presence verification before it decrypts.
- We never write our own cryptographic primitives.

### 9.2 Network

- **No telemetry, no analytics, no crash upload.**
- The only network traffic is:
  - opt-in cloud voice, with the user's own key;
  - voice-model downloads over HTTPS with hash verification;
  - an optional update check, which is a notification only.
- All of it happens **outside** the uiAccess process.

### 9.3 Logs

- `tracing` with a redaction layer. Typed text, clipboard content, passwords and transcripts are **never** logged; a `Redacted<T>` type makes accidental logging print `‹redacted›`.

### 9.4 Least privilege

- Every module declares its **capabilities**, such as InjectInput, ReadClipboard, Microphone, Network, ScreenCapture or SecretStore.
- The kernel grants only what is declared and allowed by policy.
- Untrusted data (images, files, network responses, audio) is parsed only in worker processes.

### 9.5 uiAccess honesty

uiAccess is used only for genuine accessibility features, as Microsoft's rules require.

---

## 10. Performance budgets (targets; measured in the Phase 1 test round, then enforced in CI where possible)

Measured on the development PC and in weak mode (2 cores at a low CPU rate, 4 GB, software rendering). Weak mode must meet every target except cold start, which may take up to 1 s there.

| Metric | Target |
|---|---|
| Download size, without voice models | ≤ 80 MB (owner's limit, 2026-09-26) |
| Main process memory, idle | ≤ 80 MB |
| Memory after 8 hours of use | No growth |
| Cold start to visible keyboard | ≤ 500 ms |
| Key click to key highlight | Next screen frame |
| Key click to character in target app | ≤ 30 ms |
| Animations | No dropped frames (PresentMon) |
| Mouse hook callback | ≤ 1 ms (p99) |
| CPU while idle | ≈ 0 % (no timers or animation when nothing changes) |

---

## 11. Architecture summary

The full design is in `ARCHITECTURE.md`; the reasons are in `docs/adr/`.

- **Rust** core. The UI toolkit is chosen by the Phase 1 test round: Qt 6 Quick (recommended) or Slint (ADR-0011; ADR-0002 applies until then).
- **Main process `keyxtend` (uiAccess, signed, installed in Program Files):**
  - Slint UI, kernel and trusted modules: keyboard, layouts, input, mouse assist, scroll, prediction, overlays, vault UI.
  - **No network. No parsing of untrusted media.** (ADR-0003)
- **Worker processes (normal rights):** voice (mic + network), OCR, media thumbnails, and the update check. They talk to the main process over an authenticated local pipe with a versioned schema (ADR-0004).
- **Micro-kernel with "Lego" modules** (ADR-0005), inspired by DeepSeek Harness/Cordis:
  - each module declares a manifest (id, requires/provides services, capabilities, settings version);
  - the kernel starts modules in dependency order and contains failures;
  - modules never call the OS directly, only ports (hexagonal).
- **Health monitor:** watches the mouse hook (reinstalls it if Windows drops it), workers (restarts them with backoff) and the overlay.

## 12. Testing strategy (summary; details in CONTRIBUTING.md)

- **Pure logic:** unit tests plus property tests (proptest) for the keyboard state machine, hold engine, scroll controller, prediction and retention policy. A fake clock is injected for all timers.
- **UI:** the Slint testing backend (finds elements by accessible label; pin its version exactly).
- **Platform:** a fake adapter for integration tests. On Windows CI, a tiny **test target window** records every character it receives, to prove no lost or garbled characters in EN/FR/AR.
- **Fuzzing (cargo-fuzz):** IPC message decoding, vault file parsing, clipboard format parsing.
- **Manual:** a Windows checklist for things CI cannot do (uiAccess z-order, admin windows, real apps), recorded in `docs/test-reports/`.

## 13. Future features (not v1)

F1–F12 were chosen on 2026-09-26 and F13–F16 on 2026-09-27. They are numbered in `docs/FEATURES.md` and come after v1.0.


- **Camera control:** head-tracking pointer and facial-gesture click. Webcam through `nokhwa`, face landmarks as ONNX models through `ort`. Not Google MediaPipe Tasks, which sends metrics to Google; Project Gameface was archived in September 2025.
- **More pointer actions:** zoom-to-click magnifier for tiny targets; a mouse grid; double-click and middle-click hold actions; "repeat last action".
- **Text snippets and macros** (signatures, addresses); **per-app profiles** (auto language/mode per app).
- **Read-aloud** (TTS) and **translate selection**. **Voice commands.**
- **Emoji, symbols and Arabic diacritics panel.**
- **Quick system panel:** volume, brightness, night light, Wi-Fi; window snap, switch and minimise.
- **Third-party plugins** (WASM sandbox, Zed-style capability grants). v1 has built-in modules only.
- **More ways to type:** hover-to-type, switch scanning, a number pad and navigation panel, and an IME mode for Chinese, Japanese and Korean.

## 14. Open decisions (recommended default applied until the owner decides)

| # | Decision | Recommended default |
|---|---|---|
| D1 | Product name and crate prefix | **Decided:** KeyXtend, crates `kx-*`, binaries `keyxtend.exe` / `keyxtend-worker.exe` |
| D2 | Licence | **Decided for now:** open source, GPL-3.0-or-later; revisit before v1.0 (ADR-0006) |
| D3 | Type into admin/elevated windows | Yes, like osk.exe. The spike verifies that uiAccess allows it |
| D4 | Vault unlock | OS verification (Windows Hello/PIN), with an optional master password |
| D5 | Updates | Notify only, never auto-install; the user downloads a signed release |
| D6 | Plugins | Built-in modules only in v1 |
| D7 | Which §13 features to schedule, and in what order | **Decided:** all, after v1.0, one by one from `docs/FEATURES.md` |
| D8 | Installer technology | **Decided:** Inno Setup 7 (ADR-0012) |
| D9 | UI toolkit | The Phase 1 test round decides (ADR-0011). Qt 6 Quick recommended; Tauri rejected on evidence |
| D10 | Native Adaptive accent colour | **Decided (changed 2026-09-28):** the theme's own blue; following the system accent is a setting, off by default |
| D11 | Linux Wayland limits | **Decided:** desktop built-ins first; the GNOME extension and the mouse helper are opt-in |
