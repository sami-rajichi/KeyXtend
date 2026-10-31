# Accessible on-screen keyboard — design spec, modules

- **Date:** 2026-09-26. Part of the design spec ([`2026-09-26-accessible-keyboard-design.md`](2026-09-26-accessible-keyboard-design.md)), split out to keep each file short; the section numbers are the spec's own.

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

Decided with the owner on 2026-10-24, from the research report on keyboard prediction engines (ADR-0032). Every number below is a `[predict]` setting.

- **Chips:** the word chips sit in the top bar, between the grip and the buttons, as in Windows' touch keyboard; there is no extra row.
  - As many chips as fit, up to `chips` (5 by default, 3–7); the best guess comes first. Hiding top-bar buttons later (P13) leaves room for more.
  - Tapping a chip replaces the word being typed with the chip's word, in its correct spelling, and adds a space. Before any letter, the chips guess the next word.
- **Engine:** our own Rust engine, of the kind the big keyboards still run underneath (Gboard's decoder in 2024).
  - Per language, a word list finishes the word, and counts of word chains (n-grams) guess the next one from the last N words (`context_words`, 2 by default, 1–4). A context it has never seen falls back to fewer words.
  - Seeded from wordfreq (word frequencies) and from Wikipedia and Tatoeba (word chains), shipped as separate data files with their credits (CC BY-SA 4.0, CC BY 2.0 FR).
  - Full packs for English, French and Arabic; word lists only for wordfreq's other languages; any other layout language starts empty and learns.
  - Seed words are kept only where an open spelling dictionary (the Hunspell lists of LibreOffice and Firefox, licence-checked) knows them, and they take its capitals: `london` becomes `London`, `i` becomes `I`. Languages without such a dictionary keep wordfreq's words.
  - The packs are built once from capped downloads (about 6.5 GB on the development PC) and kept in the repository.
  - The packs are read straight from disk (memory-mapped): about 20 MB of the download. Only the language in front is loaded, about 2 MB of private memory, measured by the memory gate (G8) before it ships.
  - No neural model and no Windows prediction API in v1; Microsoft documents that API as Japanese-only. A small neural re-ranker may come later, where our own measurements show the word chains fall short.
- **Language:** prediction follows the layout in front and suggests only that language's words.
- **Matching:** letters match loosely, but the correct spelling is always typed.
  - French ignores accents: `ecole` finds `école`.
  - Arabic ignores harakat and tatweel, and folds أ إ آ ٱ to ا and ى to ي: `ان` finds `أن`.
  - Capitals follow what was typed, except a word the dictionary always capitalises.
- **Knowing the word:** the keyboard follows its own key clicks.
  - A space, Enter, Tab, an arrow, a shortcut, a click in the app or a new text box ends the word.
  - After a click or a new box, it reads only the N words before the cursor through UI Automation, where the app allows it: never more, never in a password box, never logged.
- **Learning:**
  - A word, and the chain it ends, counts when the word ends or a chip is picked.
  - A new word is suggested only from its second use, so a one-off typo never comes back.
  - A word typed in an informal spelling of a known word counts toward the correct form: typing `انا` teaches `أنا`.
  - Per language it keeps at most 10,000 words and 30,000 chains of each length. When full, the least recently used go first, and entries unused for 300 days are dropped.
  - Learned words weigh more than the seed as the user types more.
  - Stored encrypted in the data folder, one file per language (§9.1).
- **Never learned:** anything in a password box or in a box whose kind Windows cannot tell (games, remote desktops; the chips still help there), words with digits, and web addresses; emails go only to the email list below.
- **Punctuation and numbers are left to the user** (owner, 2026-10-24): no chips for them, and a chip's space stays before any punctuation.
- **Emails:**
  - After one `@`, the chips show the domains in `domains` (gmail.com, outlook.com, hotmail.com, yahoo.com, icloud.com).
  - Typing the start of a known email shows it as a chip with an @ icon.
  - Typing `@@` with the keyboard shows the known emails as chips, and picking one replaces the `@@`. Only an `@@` the keyboard has just typed counts. With more emails than fit, a "More" chip opens a paged list (5 rows per page).
  - Known emails are the ones typed with the keyboard, encrypted and removable; the vault (P9) adds its saved emails with a key icon.
  - An email is learned when the user leaves it: a space, Enter or Tab after it, a domain chip that completes it, a click outside the keyboard, or focus on another box (owner, 2026-10-26).
  - An email chip adds no space.
- **Password boxes** (UI Automation `IsPassword`, with the Win32 password style as a fallback): no chips and no learning; the chip lane shows a lock and a short note.
  - Filling passwords belongs to the vault (P9).
  - Holding a key no longer fills anything: @ and P repeat like every letter (§4.6, owner, 2026-10-24).
- **Controls:** a small menu button at the end of the chip lane opens a panel with three rows.
  - **Forget a word:** tap it, then tap a chip; the word is forgotten and an Undo chip takes its place until the next key. A forgotten built-in word stays hidden until the user types it twice again.
  - **Pause learning** and **Prediction off**, each a switch that stays as set. With prediction off, the lane is empty and the menu button stays.
  - The Settings window (P13) adds a paged list of learned words.
- **Target:** a meter types held-back test sentences and counts the key presses the chips save. With 5 chips, English and French must save at least 40% (studies report 42–60%). Arabic is measured and reported but does not block, since no study has a number for it. The owner's hands-on check has the last word.

---

## 7. Clipboard history

Decided with the owner on 2026-10-30 (P8), from the mock-up's panel. Every number below is a `[clipboard]` setting, shown later in the Settings window (P13). One design serves Windows, macOS and Linux; only the adapters differ.

- **Watching:** the history hears every copy through the platform's clipboard port. Windows: `AddClipboardFormatListener`. macOS polls `NSPasteboard`'s change count, and Linux uses XFixes on X11 and the data-control protocol on Wayland where the compositor has it (P15).
- **Skipped copies:** a copy carrying a "leave me out of history" marker is never read. The markers live in each adapter's table:
  - Windows: `ExcludeClipboardContentFromMonitorProcessing`, `CanIncludeInClipboardHistory = 0`, `Clipboard Viewer Ignore`;
  - macOS: `org.nspasteboard.ConcealedType` and `org.nspasteboard.TransientType`; KDE: `x-kde-passwordManagerHint = secret`.
  - Our own clipboard writes carry the marker too, so an insert never adds a row.
  - **Follow the clear (owner):** when the app that made a copy clears or blanks the clipboard within `clear_follow_s`, the entry is deleted for good, as password managers intend.
  - **Skipped apps (owner):** `skip_apps` lists apps whose copies are never kept. It starts empty and is edited in the Settings window (P13).
- **What is kept:** text with its formatting (HTML and RTF beside the plain text), links, images, and files as references (paths, never copies). Video and audio are files whose extension the module's media table lists.
- **Limits (owner):**
  - entries are kept `keep_days` days (7 by default, 1–7), video and audio `media_days` (1 by default), and pinned entries never expire;
  - at most `max_entries` entries (1,000 by default, 1–5,000), and the whole history under `max_mb` (1 GB by default); past either, the oldest unpinned entry goes first;
  - nothing new is kept while the history's drive has less than `min_free_mb` free (2 GB by default), and an entry over `max_entry_mb` (32 MB by default) is not kept;
  - each such skip shows a short note in the panel.
  - Days are calendar days (owner): 7 means today and the 6 days before, and 1 means today only.
  - The limits count what the history stores; a file row shows the size of the file it points to, so a large file is still kept as a reference (builder's call).
  - Pinned entries count toward the limits. When they alone fill one, new copies are not kept, and the panel says so (builder's call).
- **Order (owner):** pinned entries first, then the rest, each group newest first. Copying the same content again, or inserting an entry, moves it to the top with the new time. An unpinned entry counts as copied now, so it never vanishes at the next sweep (builder's call).
- **Store (§9.1):**
  - Each entry is sealed under the OS-protected key in a folder per day, `clipboard/<yyyy-mm-dd>/`, and pinned entries in `clipboard/pinned/`. So at most 7 day folders exist: a new day removes the folders past `keep_days` whole.
  - The list of entries (titles, times, sizes) is sealed too. Outside the panel the files are unreadable, by design.
  - Deleting is permanent: the files leave the disk at once, with no recycle bin. A day folder deleted by hand is safe; its entries leave the list.
- **Panel** (the mock-up's layout plus a preview page, owner):
  - The side block's clipboard key opens and closes it with one click, on every platform; the history shortcut below is a second way. It opens above the keyboard (§3.4).
  - Header: icon, title, "kept N days", Delete all and close.
  - **Search line:** while the panel is open, letters clicked on our keyboard go into it instead of the app. It matches the first `search_chars` characters of each text (1,000; builder's call), file names and app names, ignoring case, accents, harakat, tatweel and the forms of alif (as §6 matches). Backspace erases, and Esc clears it first, then closes the panel. No AI.
  - Filters: All, Text (with links), Images, Files, Media.
  - One list, 5 rows per page (§3.3), with ▲/▼ and "n of m".
  - Each row: a tick box showing its tick number, a thumbnail or type icon, the first two lines of text (… when longer), and "app · time · size". Time reads "Today 16:32", "Yesterday 18:21", "Tue 14:12", then a date. Size is always in MB with two decimals, and "< 0.01 MB" below that. A pin marks pinned rows, and a chip marks media that expire today.
  - **Tick, then act (owner):** a click on a row only ticks or unticks it. The footer acts on the ticked rows: Preview, Pin (Unpin when all are pinned), Delete and **Insert (n)**, n being the ticked count. Delete acts at once; Delete all asks once, with Yes and No, and keeps pinned entries.
  - **Ticks and views (owner):** ticks survive a new search or filter, so the user can search, tick, search again, tick, then Insert (2). Delete all always means every unpinned entry, whatever the view, and its question names the count.
  - **Preview page:** the whole entry: long text paged, never scrolled, with its character count; an image large; files with their paths. It has Back, Pin, Delete and Insert.
  - A storage line, "History 182 MB of 1 GB · 412 GB free on D:", and Open folder, which shows the history's folder in the system's file manager.
  - An empty history asks the user to copy something.
  - Keys, from a key list per panel in the keyboard (§3.3): ↑↓ move the highlight, ←→ switch filters, Space ticks, Enter inserts the ticked rows (the highlighted one when none is ticked), Esc as above. The scroll pad turns pages.
  - Its look follows the keyboard's theme in light, dark and high contrast, and every text passes the theme's contrast check (§3.5).
- **Insert:**
  - Text goes in as copied, with its formatting. Images and files go in as Explorer or a browser would hand them: the module sets the clipboard formats, then the keyboard sends the platform's paste shortcut. Files are always pasted as a copy, even when they were cut.
  - Several ticked entries go in tick order. Between two texts go `join_breaks` line breaks (2 by default, a blank line; 1–3) and an optional separator line, `join_rule` (empty by default). Images and files get no separator.
  - The breaks are pasted, never typed: Enter sends the message in chat apps. Each step waits `paste_gap_ms`, so slow apps keep up.
  - Insert goes to the app the keyboard types into at that moment. The panel then closes, and the last inserted entry stays on the clipboard.
  - A file that is gone leaves its entry; an entry whose files are all gone leaves the list by itself (owner).
- **History shortcut (owner, 2026-10-23 and 2026-10-30):** with `history_key` on (the default), the platform's clipboard-history shortcut clicked on our keyboard opens our panel instead of the system's: Win+V on Windows and Super+V on Linux. macOS has none of its own, so P15 picks its keys. The keys come from the adapter's shortcut table (ADR-0031). A physical keyboard's shortcut still opens the system's history, and with the module off ours goes to the system too.
- **Security**
  - The store is encrypted at rest (§9.1), and clipboard content is never logged (`Redacted<T>`).
  - Image thumbnails and previews are decoded in the `media` worker (§11, ADR-0004): a process with normal rights and no uiAccess, which gets the bytes over a size-limited pipe and returns small plain pixels the panel draws. The uiAccess process never decodes an image: it only stores the bytes it read and sets them back.
- **As built (P8, 2026-10-31):** where the build settles or differs from the text above.
  - An Arabic row flips its text only; the tick box, thumbnail, pin and chip keep their places (ADR-0020, Ruling 49).
  - Delete all stays in the header, as above (Ruling 50). It dims while no entry is unpinned (Ruling 34), and in its question Esc is No and only a click on Yes deletes (Ruling 33).
  - The panel stays open while Insert runs and closes when it ends; a failed step stops the insert and keeps the panel open with a note (Ruling 41).
  - A file found gone is left out of the paste, and the entry goes only when every file is gone (Ruling 39). A file whose drive or share does not answer keeps its row and is skipped (Ruling 25).
  - Follow-the-clear spares pinned entries (Ruling 22).
  - A clipboard watch the system refuses fails the module's start, with Try again (Ruling 38).
  - Sizes use each language's thousands grouping and decimal mark (Ruling 37).
  - The worker decodes PNG and DIB only, and an interlaced PNG keeps its type icon (Ruling 18).
  - A copy with text keeps no picture beside it, as Office copies carry one (Ruling 20).
  - A copy is refused when keeping it would leave less than `min_free_mb` free (Ruling 23).
  - The panel opens above the keyboard, else below, else shrinks to fit, never below 0.80 of its size (Ruling 53).
  - The history's folder is made at boot, so Open folder always opens (Ruling 47), and a note takes only the storage line's text, with Open folder beside it (Ruling 58).
  - The scroll pad turns the list's pages; the preview's long text pages by its own buttons only (Ruling 52, `docs/items/P8.md`).

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

