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

**Goal:** a small, fast, source-available, cross-platform on-screen keyboard that lets a one-button pointer user do *everything* a keyboard-and-mouse user can do.

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
- **Size:** small by default. Presets S, M and L, plus fine steps between a smallest and a largest size (`[keyboard]` `presets`, `size_step`, `size_min`, `size_max`). Icon keys never show labels under the icon; at every size, hovering any icon button shows its name in a small tooltip.
- **Move** without dragging: click the grip, move the pointer, click to place. The keyboard stays wholly on one screen, so a move past the edge stops there, and its size follows that screen's DPI.
- **Resize** the same way with the corner handle. There is no Esc to undo: Esc belongs to the app, so Settings has "Reset size" (owner, 2026-09-28).
- **Minimise** puts a bubble in a bottom corner of the keyboard's screen, above the taskbar, wherever the keyboard was: right by default, left as a setting (`[keyboard] bubble_corner`), never the top (owner, 2026-09-28).
- **Remembers its state:** size, position, theme, mode and every setting survive a restart or shutdown. A saved place on a screen that is gone brings the keyboard back to the bottom centre of the main screen.
- **Top bar:** grip, word chips and their menu button (P7, §6), light/dark toggle (sun/moon), a theme button (a trial; P13 decides), size buttons (lens −/+ and S/M/L), minimise-to-bubble (−), close-to-tray (×).
- **Tray and one copy:** × hides the keyboard; the tray icon's menu shows it again or quits. One copy runs per user session: starting it again shows the keyboard, and `keyxtend --quit` ends it.
- **Fade when idle** was dropped (owner, 2026-09-28): minimise-to-bubble does the job better.

### 3.2 Feedback

- **Wave ring:** during a hold, a ring in the theme's ring colour appears after 150 ms and shrinks into the pointer over the rest of the hold time, whatever hold time the user set. When the hold fires, a burst of 6 dots appears.
- **Hover:** a key under the pointer turns a little darker in light mode and lighter in dark mode.
- **Mode badge** beside the pointer, clear of the pointer's own size, while a mode is on or, with mouse assist on, while the pad scrolls: Right-click, Grab, Holding (a carry), the scroll direction with "slow" or "fast" beside it (§5.2), and later recording (P11). It hides while the pointer is over one of our windows.
- **No banner** when Right-click or Grab turns on: the key's light, its tip and the badge show the mode (owner, 2026-10-13).
- **Sounds**, realistic, soft and never harsh (owner, 2026-10-13):
  - a key-press sound like a real keyboard's on every key of the keyboard, not on the top-bar buttons; a held, repeating key sounds once, at the press;
  - a soft mouse-button click when a hold right-clicks;
  - a pick-up sound when Grab grabs, and a placing sound when it drops;
  - distinct start and stop chimes for the mic (P11).
- One sound set serves every theme. Each sound is made by us or is a CC0 recording, picked by the owner by ear, and plays at our own soft level (`[ui] sound_volume`) under the Windows volume.
- Each feedback type can be switched off.

### 3.3 No-scroll, no-drag UI (hard rule)

- **Lists** (clipboard, quick-fill, settings lists) show exactly **5 rows per page**.
- Every list has large **▲/▼ page buttons** and a "page n of m" indicator. A short last page is padded so paging lands on whole pages.
- While a panel is open, the keyboard's own keys drive it: **↑/↓** move the highlight, **←/→** switch tabs, **Enter** activates, **Space** ticks, **Esc** closes.
- The scroll pad turns one page of any of our lists per tap (§5.2).
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

| Theme | Idea | Fonts (the bundled ones OFL) |
|---|---|---|
| **Native Adaptive (default, light)** | Looks like part of the OS: Fluent geometry (4 px keys in an 8 px window), a solid plate or Mica or Acrylic as an option, its own blue (system accent as an option, `[ui] follow_accent`) | System UI font (Segoe UI) + Noto Sans Arabic |
| ET66 | Braun ET66: colour marks function. Brown = modifiers, yellow = Enter, a green LED = on/locked | IBM Plex Sans + IBM Plex Sans Arabic |
| Modern Dolch | GMK Modern Dolch keycaps: grey letters, dark modifiers, teal Enter, rose Esc/Backspace, pressable inset | Rubik + Noto Sans Arabic |

- Light, dark, or **follow system** (the default) is chosen in Settings (`[ui] mode`); the keyboard's sun/moon button switches between light and dark, which stops following the system.
- **Behind the plate** (`[ui] backdrop`): a solid plate by default, or Mica or Acrylic through a see-through plate where Windows draws them.
- **High contrast** on: system colours only, with a hover outline in the system's highlight colour. Animations, except the hold ring, stop when Windows turns them off or `[ui] reduced_motion` is on.
- Icons: **Lucide** (ISC). The icons are platform-specific: the Windows logo on Windows, ⌘⌥⌃ on macOS, "Super" on Linux.
- Exact colour tokens come from the mock-up file (`design/keyboard-style-lab.html`, CSS `[data-style][data-mode]` blocks) and live in `kx-theme`'s `themes.toml`, with shared sizes in `shape.toml`, motion in `motion.toml` and icon names in `icons.toml`, all compiled in (ADR-0013, ADR-0016).

### 3.6 Languages of the UI

- The keyboard's own UI text (settings, tooltips) is translatable from day one, from one Fluent file per language, compiled in (ADR-0017).
- v1 ships EN, FR and AR. The keyboard's text, and later the settings window's, follows the language of the layout in front; the tray follows the system's (ADR-0020). `[ui] language` can pin EN, FR or AR, and a language without texts falls back to English.
- Key labels read as real keyboards print them: French shows Échap, Suppr, Verr. Maj and Alt Gr, while English and Arabic keep Esc, Back, Caps and AltGr. Tips and screen-reader names are fully translated (owner, 2026-09-30).
- Arabic flips text only: windows and controls keep their places, Arabic text reads right to left, and Qt edits it with a correct caret (gate G7, ADR-0013, ADR-0020).
- Our UI still prefers pickers and steppers over text fields, since they need fewer clicks.

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

The **shortcuts layer** (owner, 2026-10-23):

- The Shortcuts key swaps the rest of the side block for one-click shortcuts. It stays in its place, lit like a latched modifier, and a second click brings the usual side keys back; there is no Back key.
- The layer stays up after a shortcut, so Select all and then Copy need no trip back.
- Shortcut keys show an icon. Their tip gives the name and the keys sent, such as "Copy all (Ctrl+A, Ctrl+C)", in the keyboard's language.
- F1–F12 show as text, like real keys.
- A shortcut can be a short chain of steps: Copy all is Ctrl+A, then Ctrl+C.
- **Pages:** the user arranges the shortcuts on pages, like a phone's home screen (`[keyboard] shortcut_pages`, edited in the Settings window from P13).
  - With more than one page, the bottom-right cell is a Next key with one dot per page, the current one lit, as the language key's Cycle design. After the last page it goes back to the first.
  - A page holds 13 shortcuts, or 14 when it is the only page, and there are at most 6 pages, as many dots as the Next key fits.
- **Starting set:**
  - page 1: Copy, Paste, Cut, Select all, Undo, Redo, Copy all, Cut all, Delete all, Find, Save, Print, Emoji;
  - page 2: Switch app, Show desktop, File Explorer, Lock PC, Task Manager, Close window, New tab, Close tab, Reopen tab, Refresh, Zoom in, Zoom out;
  - page 3: F1–F12.
- Users show, hide and order these; they cannot make new ones. New shortcuts come from the owner in updates.
- Every latched modifier, AltGr too, joins each step, as on a real keyboard (Shift + New tab reopens a closed tab), then lets go as after any key; Lock PC goes alone. Shortcut keys never repeat while held.
- **Switch app** opens the system's switcher and keeps it open, so one click shows every window and the user clicks one: Ctrl+Alt+Tab on Windows, Mission Control on macOS, the Activities view on GNOME.
- Opening the layer stops the scroll pad, since its Stop leaves the screen. Right-click and Grab keep their mode, which the badge shows.
- **Every platform** (ADR-0031): the keyboard knows a shortcut only by what it does. Each platform adapter's table gives its keys, such as Ctrl+C on Windows and Linux and ⌘C on macOS, or an OS call where keys cannot do it; a shortcut a platform lacks is hidden there.

### 4.3 Languages and layouts

- Detect every input language installed in the OS. Windows: `GetKeyboardLayoutList`. Labels are read per layout with `ToUnicodeEx`, flag `0x4` (does not change kernel keyboard state).
- The labels follow the layout of the app in front, checked every `[keyboard] relabel_ms`, so a layout added or removed while running shows at the next check. A key that must go as a scan code while the labels lag the app's layout sends nothing; the keyboard redraws, and the user clicks again.
- v1 must render and type **English (US QWERTY)**, **French (AZERTY)** and **Arabic (101)** correctly:
  - AltGr level on AZERTY (e.g. `€ ~ # { [ | \` \\ ^ @ ] }`);
  - Arabic shifted level with harakat shown on a dotted circle `◌` and typed bare;
  - `لا` as one key that types two code points.
- P3 was tested on US, French (Legacy, AZERTY) and Arabic (102) AZERTY; Arabic (101) is type-tested before v1 (P14).
- On a layout without AltGr characters, AltGr acts as Right Alt.
- Keys show **only the active language**.
- Apps listed in `[keyboard] key_apps` get text as key presses in their own layout instead of Unicode (§4.5).
- **Language key** — three designs are prototyped; the owner's pick is saved from the mock-up:
  - **Carousel (default):** shows previous · current · next. Click the left third = previous, anywhere else = next. It swipes with animation.
  - **Cycle:** globe icon plus the current language and dots.
  - **List:** a popover listing all languages, one page of `[ui] page_rows` rows, driven by the keyboard's arrows, Enter and Esc.
  - With one layout installed, the key shows only its name, a click changes nothing, and its tip and screen-reader text say so.
- Changing language switches our labels **and** asks the target window to switch too (`WM_INPUTLANGCHANGEREQUEST`), so the OS indicator and IMEs stay in sync.

### 4.4 Modifiers (osk.exe behaviour, decided in round 6)

- Shift, Ctrl, Alt, Win and AltGr: **one click = held**, shown filled with the LED on. **Click again = released.**
- **Release after next key** (default): a held modifier releases automatically after the next non-modifier key. Setting **"Stay held"**: it stays down until clicked again.
- There is no third "lock" state; the owner found it confusing.
- **Win alone** (owner, 2026-10-17): clicking Win again with nothing used since it latched (no key, Caps Lock, modifier + click, or a latch latched after it) lets go and sends a lone Win press, so Start opens as with a real key; otherwise it only lets go. The lone press goes out without the other latches, which then let go or stay as after any key. A latch key opts in with `lone` in `board.toml`; never Alt, whose lone press opens the app's menu bar.
- **Caps Lock** is a simple toggle, labelled "Caps" as in osk.exe so it never looks like Shift ("Verr. Maj" in French, §3.6).
- **Modifier + mouse:** while a modifier is held, the next mouse click is sent with that modifier down, so Shift+click extends a selection and Ctrl+click multi-selects. This is essential for one-button users.
- Keys pressed while one of our panels is open go to that panel (§3.3), not to the target app.

### 4.5 Sending input (Windows)

- **Characters:** `SendInput` with `KEYEVENTF_UNICODE`. The result does not depend on the target's layout, so AZERTY and Arabic are never garbled.
- **Shortcuts, navigation keys and modifiers:** virtual-key/scan-code events, for example Ctrl+C = `VK_CONTROL` + `VK_C`, because many apps ignore Unicode events when combined with Ctrl.
- **Per-app fallback:** some apps drop `VK_PACKET` events. For these, map the character back to a scan code in the target's layout (`VkKeyScanEx`).
- Every injected event carries our signature in `dwExtraInfo`, so our own hooks can ignore our own injections.

### 4.6 Key repeat (owner, 2026-10-11)

- Holding a typing key types it again and again until the pointer lets go, as a physical keyboard does. A short click types once, at the press, so a slip of the hand never cancels it (ADR-0024).
- The calm defaults suit a pointer: the first repeat after 0.8 s, then 10 a second (`[keyboard] repeat_delay_ms`, `repeat_every_ms`). Each user can change both, or turn repeat off (`repeat`); a held key then types once, at the press.
- **Repeating keys:** characters (letters, digits, symbols), which always repeat, and the named keys marked `repeat` in `board.toml`: Space, Enter, Backspace, Delete and the four arrows.
- **Never repeating:** Shift, Ctrl, Alt, AltGr, Win, Caps Lock, Tab, Esc, Menu, the language key, accent (dead) keys, dimmed keys and the top-bar buttons.
- Latched modifiers stay down for the whole burst and let go when it ends, so Shift with a held "a" types "AAAA". With Ctrl, Alt or Win latched (or AltGr acting as Right Alt), a held key types once, so a shortcut such as Ctrl+W never fires many times.
- The burst stops when the pointer lets go, the keyboard hides, one of our panels opens, or a key does not reach the app; a slip anywhere while held changes nothing, and a key slipped onto never acts. While a panel is open, keys go to the panel and do not repeat.

---

## 5–8. Modules

Mouse assist (§5), word prediction (§6), clipboard history (§7) and the other modules (§8) are in [`2026-09-26-accessible-keyboard-design-modules.md`](2026-09-26-accessible-keyboard-design-modules.md), under the same section numbers.

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
- The log files are `logs/keyxtend.log` (this run) and `logs/keyxtend.previous.log` (the run before), each capped in size. They stay on the PC and are never sent anywhere (ADR-0015).

### 9.4 Least privilege

- Every module declares its **capabilities**, such as InjectInput, ReadClipboard, Microphone, Network, ScreenCapture or SecretStore.
- The kernel grants only what is declared and allowed by policy.
- Untrusted data (images, files, network responses, audio) is parsed only in worker processes.

### 9.5 uiAccess honesty

uiAccess is used only for genuine accessibility features, as Microsoft's rules require.

---

## 10. Performance budgets (targets; measured from P3 on as gate G8, then enforced in CI where possible)

Measured on the development PC and in weak mode (2 cores at a low CPU rate, 4 GB, software rendering). Weak mode must meet every target except cold start, which may take up to 1 s there.

| Metric | Target |
|---|---|
| Download size, without voice models | ≤ 80 MB (owner's limit, 2026-09-26) |
| Main process memory, idle | ≤ 100 MB private working set and ≤ 140 MB private commit (ADR-0021, owner, 2026-10-10) |
| Memory after 8 hours of use | No growth |
| Cold start to visible keyboard | ≤ 500 ms |
| Key click to key highlight | Median within the next frame (≤ 2 frames) and slowest one refresh later (≤ 3 frames), as Windows' On-Screen Keyboard (ADR-0025, owner, 2026-10-12) |
| Key click to character in target app | ≤ 30 ms |
| Animations | No dropped frames (PresentMon) |
| Mouse hook callback | ≤ 1 ms (p99) |
| CPU while idle | ≈ 0 % (no timers or animation when nothing changes) |

---

## 11. Architecture summary

The full design is in `ARCHITECTURE.md`; the reasons are in `docs/adr/`.

- **Rust** core; every window is drawn with **Qt 6 Quick** (QML) through cxx-qt (ADR-0013).
- **Main process `keyxtend` (uiAccess, signed, installed in Program Files):**
  - Qt UI, kernel and trusted modules: keyboard, layouts, input, mouse assist, scroll, prediction, overlays, vault UI.
  - **No network. No parsing of untrusted media.** (ADR-0003)
- **Worker processes (normal rights):** voice (mic + network), OCR, media thumbnails, and the update check. They talk to the main process over an authenticated local pipe with a versioned schema (ADR-0004).
- **Micro-kernel with "Lego" modules** (ADR-0005), inspired by DeepSeek Harness/Cordis:
  - each module declares a manifest (id, requires/provides services, capabilities, settings version);
  - the kernel starts modules in dependency order and contains failures;
  - modules never call the OS directly, only ports (hexagonal).
- **Health monitor:** watches the mouse hook (reinstalls it if Windows drops it), workers (restarts them with backoff) and the overlay.

## 12. Testing strategy (summary; details in CONTRIBUTING.md)

- **Pure logic:** unit tests plus property tests (proptest) for the keyboard state machine, hold engine, scroll controller, prediction and retention policy. A fake clock is injected for all timers.
- **UI:** Qt Quick Test (`qmltestrunner`), finding controls by accessible name; the Windows gate runner (`tools/kx-gates`) drives the real app.
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
| D2 | Licence | **Decided:** view only, all rights reserved; the app is sold under its own user licence (ADR-0030, superseding ADR-0006) |
| D3 | Type into admin/elevated windows | **Decided:** yes, like osk.exe; gate G4 showed uiAccess is enough (ADR-0013) |
| D4 | Vault unlock | OS verification (Windows Hello/PIN), with an optional master password |
| D5 | Updates | Notify only, never auto-install; the user downloads a signed release |
| D6 | Plugins | Built-in modules only in v1 |
| D7 | Which §13 features to schedule, and in what order | **Decided:** all, after v1.0, one by one from `docs/FEATURES.md` |
| D8 | Installer technology | **Decided:** Inno Setup 7 (ADR-0012) |
| D9 | UI toolkit | **Decided:** Qt 6 Quick with a Rust core (ADR-0013); Slint dropped, Tauri rejected on evidence |
| D10 | Native Adaptive accent colour | **Decided (changed 2026-09-28):** the theme's own blue; following the system accent is a setting, off by default |
| D11 | Linux Wayland limits | **Decided:** desktop built-ins first; the GNOME extension and the mouse helper are opt-in |
