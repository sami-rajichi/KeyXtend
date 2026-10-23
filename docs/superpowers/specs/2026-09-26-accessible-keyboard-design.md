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
- **Top bar:** grip, suggestion chips (P7), light/dark toggle (sun/moon), a theme button (a trial; P13 decides), size buttons (lens −/+ and S/M/L), minimise-to-bubble (−), close-to-tray (×).
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

## 5. Mouse assist

### 5.1 The hold engine (shared by Right-click and Grab)

A low-level mouse hook (`WH_MOUSE_LL`) watches the physical left button (ADR-0027).

**Hook discipline**

- The hook runs only while it is needed: a mode is on, a modifier is latched, or a carry is in progress.
- The callback runs only the engine's pure step: no waiting on locks, no I/O, no logging and no allocation. It returns within 1 ms at p99 (§10, gate G8).
- Our clicks go out after the callback returns, on the hook's own thread.
- Only physical input counts, plus our own keyboard's Esc, which can only cancel a carry. Injected input, ours or another program's, passes untouched, so no program can borrow our rights through the hook.
- Windows silently removes a hook that exceeds `LowLevelHooksTimeout`, so every `[mouse] health_ms` (one second) a health check compares Windows' last-input time with the last time a hook callback ran or our own input went out (§11). It reinstalls the hooks only after two lagging checks in a row, and skips while another desktop, such as the lock screen, is in front.

**State machine** (pure Rust, clock injected, fully unit-tested)

```
Idle ──left down (Right-click on)──▶ Pending(t0, p0)    [press is held back]
Pending ──up before T──▶ replay click at p0 ──▶ Idle     (normal click)
Pending ──moved > 7 px──▶ replay down at p0, then pass moves through ──▶ Idle (normal drag)
Pending ──T elapsed, still──▶ inject right down+up at p0; the held left press is never sent
Idle ──left down (Grab on)──▶ the press goes through at once ──▶ Carrying   [its release is dropped]
Carrying ──next click──▶ swallow it, inject left up there ──▶ Idle; Grab turns off
```

- **T = 1.5 s** by default (`[mouse] hold_ms`). It is adjustable from 0.5 s to 3.0 s in 0.1 s steps.
- The ring shows from 150 ms. The wave shrinks over T − 150 ms, so it always ends when the hold fires.

**Rules**

- **Right-click mode never blocks normal clicks.** Short clicks and drags behave exactly as without the mode; only a *still* hold becomes a right-click. The existing selection is preserved because the left press was never delivered.
- Right-click mode is a toggle key and stays on until toggled off. It is remembered across restarts; Grab never is (owner, 2026-10-13).
- **Grab grabs at the next press, with no wait** (owner, 2026-10-13), as the drag tools of Dwell Clicker 2, Grid 3 and Windows Eye Control do.
- Grab turns Right-click off while it is active. After a drop, Grab turns itself off and **Right-click comes back** if it was on before.
- **Drop = one single click** on the target. The Grab key again, a click on any of our windows, or Esc cancels a carry: Esc goes to the app, then the button is let go.
- A let-go during a carry (a stop, a hook reinstall or the panic path) also sends Esc before the Up, so nothing drops where the pointer happens to be.
- Holds that start on our own keyboard, panels or overlays are never intercepted, so a held key repeats (§4.6).
- Grab covers **drag-and-drop** (files, windows, images) and **text selection**: press at the start, move, click at the end. A selection made this way is then offered to the Selection helper (§8.3, P10).
- **Modifier + click** (§4.4): a latched modifier goes down with the next click outside our windows, in any mode, and lets go after its release. With "Release after next key" it then unlatches, as after a key.
- To copy with Grab, latch Ctrl first; it stays down until the drop.
- If the app ends during a carry, its panic path lets the button go. A forced kill cannot, and the next physical click releases it.

### 5.2 Scroll pad

- A round pad with ▲ ▼ ◀ ▶ and a centre **Stop** button that appears only while scrolling. Each arrow sits in the middle of its quarter of the ring, like the iPod click wheel's labels (owner, 2026-10-16).
- **Tap a direction = slow. Tap it again = fast. Third tap = stop** (decided in round 6). There are **two speeds only**, and two dots under Stop's square show the current speed.
- Tapping another direction switches direction at slow speed.
- The page **glides**: many small steps a second, slow 3 and fast 10 lines a second by default (`[scroll] slow`, `fast`; owner, 2026-10-16).
- **It also stops** at the end of the content, at any press outside our windows (with mouse assist on, whose hook hears the press), `[scroll] limit_s` (2 minutes) after the last tap on the pad, when the target closes, and when the keyboard hides (Close or Minimise), which takes Stop off the screen; each of the first two can be switched off.
- A tap with nothing to scroll there, or already at the end, stops at once with a short notice; with `stop_at_end` off, a tap at the end scrolls on with no notice, as a feed may load more.
- **Target:** the scrollable thing under the pointer, including long text fields. While the pointer is on the keyboard, the target is the **last point the pointer was at outside the keyboard**; moving the pointer onto something else while it scrolls makes that the target.
- **Outline:** a thin outline in the theme's ring colour (Highlight in high contrast) marks the target while the pointer is on the pad and all the time it scrolls.
- **Our own lists:** while one of our paged lists shows, it is the target, whatever point was kept, and each tap turns one page, with no speeds and no Stop. A glide already running when the list opens keeps its target.
- **Badge:** with mouse assist on, the pointer badge shows the direction's arrow and "slow" or "fast", beside a Right-click or Grab icon (§3.2).
- **Every platform** fills one OS-neutral port (`ScrollTarget`); the rules live in `kx-mod-scroll` (ADR-0029).
- **Windows mechanism** (ADR-0029, which reorders P1's list proven by gate G6, ADR-0013):
  1. Small `WM_MOUSEWHEEL`/`WM_MOUSEHWHEEL` deltas posted to the window at the target point, or `SendInput` wheel events while the pointer itself is on the target.
  2. UI Automation finds the scrollable element, its box and its position, and moves it by `ScrollPattern` steps when the window ignores wheel messages.
  3. The point before the keyboard comes from Windows' pointer history when the pointer enters the keyboard.
- **As built in P5** (refines ADR-0029's fallback and end rule; the names are `kx-platform-windows`' `scroll` constants):
  - A target falls back to `ScrollPattern` steps once its percent has stayed still, short of the end, for `[scroll] check_ms` **and** at least one whole notch of wheel delta went out since it last moved.
  - Under `ScrollPattern` steps, a percent that stays still for `check_ms` after at least one step reads as the end. This also covers wheel settings of 0 lines or one page, which step by UI Automation from the start.
  - When those steps read the end at the very percent the wheel stopped at, as Explorer's bottom at 99.991, that was the end, not a window deaf to the wheel: the fallback clears, and the next scroll of that window uses the wheel again.
  - Both rules count only steps sent before the previous read, so a slow app has a whole read to show them; each therefore comes about 2 × `check_ms` after the percent stops.
  - A percent within a rounding (`END_SLACK`) of 0 or 100 is at its end, and any percent above 100, as Notepad's 100.005 after Ctrl+End, reads as 100. Only NaN, infinities and values below 0 past a rounding (UI Automation's -1 for "cannot scroll") are refused.
  - A gap between reads longer than `PAUSE_CHECKS` × `check_ms` + `read_ms` starts the wait afresh.
  - A busy target, one that left a `WM_NULL` ping unanswered or that Windows calls hung, gets no steps.
  - A `SendInput` wheel goes out only while Windows gives wheels to the window under the pointer (mouse-position routing) and the pointer is still on the target; otherwise the step is posted.
  - Our windows that do not page, such as the keyboard and its banner, are passed over, so the kept point is the last one outside them. The UI marks the open list as paging as it opens, and while it shows the pad pages it, even when the pointer crossed the app in the gap above the keyboard; a peek outlines the list window.
  - A kept point that one of our non-paging windows now covers, as when the banner shows over it, aims at the first other app's window behind ours there. With none, the tap asks the user to point first; it never turns a page silently. UI Automation then reads and steps the element at that point inside the app's own window, never ours.
  - With the pointer resting on our banner or tray menu, as through Voice Access, there is no target: a check mid-scroll keeps the last target, and a tap asks the user to point first.
  - A target that closes stops the scroll, even when another window now lies at its point.
  - UI Automation takes the box and percent from the first element that scrolls along the tapped way's axis, so ▼ over a wide table reads the page around it.

### 5.3 Overlays

- The ring, burst and mode badge are drawn in **one click-through Qt window** that never takes focus (`Qt.WindowTransparentForInput`, guarded by the Windows adapter).
- Gate G12 proved it: 200 of 200 clicks under the ring reached the app, at the screen's full rate (ADR-0013).
- It is made while a mode is on or, with mouse assist on, while the pad scrolls, and freed when neither holds. It is updated only while something is animating, and it follows the pointer only when the pointer moves, with no timer, so a still pointer costs nothing.

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
- **Win+V** from our keyboard opens our history instead of the Windows one (Windows only; setting). It is built with the history in P8 (owner, 2026-10-23).
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

- One neutral port opens the dialog; `kx-mod-power` asks it when the key is clicked. While that module is off, the key is dimmed with a tip.
- The French name is «Marche/Arrêt», Windows' own word for its power button; «Arrêt» alone read as "shut down" (owner, 2026-10-23).
- Tests open the dialog, check that it showed, and close it with Cancel. They never confirm it.

### 8.6 Settings window

- **Quick settings first** (owner's idea, 2026-09-28):
  - the Settings key opens a small strip under the keyboard with theme, mode and size, like the P1 test strip;
  - the strip never takes focus; its "All settings" button opens the full window below;
  - it shows only when asked, never all the time.
- A normal window, not on top of everything, and never needing scrolling: tabbed pages with paged lists and −/+ steppers.
- It has its own icon, speaks the keyboard's language, and carries each theme's character (colours, shadows, motion) as strongly as the keyboard does (owner, 2026-09-29).
- **Contains:**
  - theme and mode, size and "Reset size", bubble corner;
  - language key design, modifier behaviour;
  - hold time; each sound on or off, and their volume;
  - key repeat: on or off, its delay and its speed, each explained in plain words (§4.6);
  - clipboard retention;
  - voice engine, model, forced language and keys;
  - vault export and import;
  - privacy and data deletion;
  - about and licences.
- Settings are versioned TOML that holds only the user's changes, in the data folder (ADR-0015), with migrations (ARCHITECTURE.md). An "Open my settings folder" button shows where the file is.

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
