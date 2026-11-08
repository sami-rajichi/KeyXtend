# Selection chips — design (P10)

Decided with the owner on 2026-11-07, from five mock-ups shown in the session; the owner chose design 4, Keycaps. The mock-up is [`design/selection-chips.html`](../../../design/selection-chips.html): the strip, its sizes, legends, plate, tick and fade follow it. This file replaces spec §8.3 of [`2026-09-26-accessible-keyboard-design-modules.md`](2026-09-26-accessible-keyboard-design-modules.md) and the roadmap's Phase 10 line. Every number below is a `[selection]` setting unless it says otherwise; one design serves Windows, macOS and Linux, and only the adapters differ.

## Why

- Copy, Cut and Paste need Ctrl and a letter, or a trip to the Shortcuts layer. With one finger, that is three or four clicks for the most common edit.
- Phones solved this with a small edit bar on selected text and a Paste offer at the caret ([Android's text selection toolbar](https://material.io/design/platform-guidance/android-text-selection-toolbar); [PopClip](https://www.popclip.app/guide/basics) on the Mac, SnipDo on Windows).
- The owner's idea (P9, 2026-11-01): selected text shows Copy, Cut and Paste, a caret shows Paste, and the user picks the action.

## Goals, non-goals, anti-goals

- **Goals:**
  - one click copies, cuts or pastes at the selection, in any app that tells its selection;
  - one click pastes at a caret just clicked into;
  - one click opens a translation of the selection;
  - every part can be switched off; all of it is on by default.
- **Non-goals in P10:**
  - a translation shown inside KeyXtend (F8 adds it, on this PC, after P11; see Translate);
  - other translate sites than Google (the sites live in a table, added later once tested);
  - Search, read aloud, or any chip beyond the four;
  - chips in apps that do not tell their selection (no gesture guess: owner).
- **Anti-goals (a failure even if it works):**
  - reading, keeping or logging selected text when no Translate chip was clicked;
  - sending Ctrl+C to probe for a selection;
  - a chip that takes focus from the app;
  - Ctrl+C reaching a terminal that no longer has a selection (it would stop the running program);
  - chips in a password box;
  - a chip under the keyboard or off the screen.

## What the user sees

### The keycaps (owner: design 4)

- One strip of real keycaps on the keyboard's plate, in the theme in front: flat caps in Native, convex caps in ET66, sculpted caps in Dolch, light or dark as the keyboard is.
- Each keycap shows an icon and a word: Copy (`copy`), Cut (`scissors`), Paste (`clipboard-paste`) and Translate (Lucide `languages`, added after `kx-licence-check`). One setting, `look`, shows both (the default), the icons alone or the words alone, so a keycap never shows nothing.
- Words are in the keyboard's language (EN, FR, AR) and come from the translations: Copy, Cut, Paste and Translate; Copier, Couper, Coller et Traduire; نسخ، قص، لصق، ترجمة. Arabic flips text only: the order stays Copy, Cut, Paste, Translate from left to right (ADR-0020).
- **Size (owner):** each keycap is one key row tall at the keyboard's current size (31, 39 or 47 px at the presets 1.0, 1.25, 1.5) and grows with it. Paddings, the word and icon sizes and the gap from the text come from `shape.toml [chips]`, which replaced the old `[pill]` tokens; the keycaps' gap, legend weight and plate corner are each theme's own, as in the mock-up.
- Each keycap is as wide as its label needs; the strip's place is worked out from its real size, never from a fixed width (P10 item note).
- **Motion:** the strip pops in as the mock-up's `.selpill` did (`motion.toml` `chips`). A clicked keycap presses like a key, shows a tick and a past-tense word (Copied, Cut, Pasted, Opening; Copié, Coupé, Collé, Ouverture; تم النسخ، تم القص، تم اللصق، جارٍ الفتح), then the strip fades. Reduced motion skips the pop and the fade but keeps the tick's time. Timings are motion tokens: `chips`, `chips_fade` and `[shows] chips_done`.
- **Sound:** a click that acts plays the key click sound when key sounds are on; a late click is silent.
- **Readers:** each keycap is a button named by its word, and a ticked keycap by its done word (Copied, Copié, تم النسخ); the strip never takes focus.

### Where it sits

- Just under the selection's last line, centred on that line (the spike's rule, which G19 placed within 1 px), the theme's gap below it, which grows with the keyboard's size as the mock-up's does.
- Above the first line when there is no room below, or when the strip would cover the keyboard or one of our panels and there is room above; covering the keyboard's edge beats covering the text.
- Always inside the work area of the screen that holds the selection, sized for that screen's DPI.
- It moves when the selection moves (a scroll, a typed key that changes the line) at the next look.

## When the chips show

### On a selection

- A text box or document in front holds a selection, and its app tells it through the platform's accessibility API.
- What shows depends on the box (adapter tables say which control is which):
  - an editable box: Copy, Cut, Paste, Translate;
  - read-only text, such as a web page or a document viewer: Copy, Translate;
  - a terminal: Copy, Paste, Translate; never Cut;
  - a password box, a box of unknown kind, or our own windows: nothing.
- A selection made with our keyboard (Shift with the arrows, Select all from the Shortcuts layer) shows the chips too.
- A selection Grab made (spec §5.1) shows them too. As built, the module follows Grab's state (`AssistChanged`), not `AssistNote`, which needs `KeyboardUi`: a release is no look while Grab carries, and after the drop the next release or the app's own selection event looks.

### At a caret (owner)

- After a press and release inside an editable box that leaves a caret and no selection, a single Paste keycap shows by the caret, if the clipboard holds anything. Only the clipboard's formats are checked; nothing is read.
- It goes at the first key typed, the next click anywhere, a caret move, or after `caret_s` seconds (5).

### Keyboard states

- Minimised to its bubble: the chips still show (owner, `minimised` = on).
- Closed to the tray, or the module switched off: no chips.
- While one of our panels' lines is aimed at (ADR-0040), no chips: the letters go to the panel, not the app.
- As built: the ui only tells the keyboard's place (`BoardPlace`: open, minimised or tray), and the selection module applies these rules and puts the keycaps' legend in each view, so the ui reads none of its settings.

## When the chips hide

- The selection empties or changes to none; focus moves to another box or window.
- A press outside the strip (a later look may show it again for a new selection).
- Esc, sent by our keyboard or the user's.
- After any keycap's click, once its tick has shown.
- A shown strip whose selection is gone by the next look never stays.

## What each keycap does

- **Copy, Cut, Paste:** exactly Ctrl+C, Ctrl+X or Ctrl+V on Windows and Linux, ⌘ on macOS, from the platform's shortcut table, through the keyboard (the only typist).
- **In a terminal (builder's call, from the design check):** Copy and Paste use the table's terminal ids, keys that copy or paste but can never stop a program: on Windows, Ctrl+Insert and Shift+Insert, or Ctrl+Shift+C and Ctrl+Shift+V, whichever G19 shows both Windows Terminal and the classic console take on this PC ([terminal#5790](https://github.com/microsoft/terminal/issues/5790)); on Linux, Ctrl+Shift+C and Ctrl+Shift+V (P15). Latched modifiers are not added to them and stay as they were, so Chrome never gets Ctrl+Shift+C. Prediction hears one break, as after a Shortcuts-layer key.
- **Before Copy or Cut is sent,** the focused box is checked again: it must be the same box and still hold a selection. If not, nothing is sent and the strip hides. The terminal keys above close the gap between that check and the key's arrival.
- **Paste** pastes what the clipboard holds now, as Ctrl+V does. Clipboard history stays on its own key.
- The app keeps focus throughout; the strip is a non-activating window (ADR-0013).

## Translate (owner: the browser page now, the box in F8)

- **Why the browser page:** extensions translate for free through private addresses of Google or Microsoft that have no published terms for other apps. Free official services need an account and a key for each user. Zen and Firefox translate on the PC with small local models ([Zen](https://docs.zen-browser.app/user-manual/translate)), which needs an engine and model downloads that P11 builds for voice. The owner chose the page now and the in-app box later.
- **The click:** the selected text is read once, at most `translate_chars` characters (1,000, cut at a whole word), and goes into the chosen site's link, which opens in the user's default browser. The link was tested on 2026-11-07: Google detected English and gave correct Arabic.
- **Consent:** the first click asks once, in a banner, because the text leaves the PC: "Translate opens Google Translate in your browser and sends it the selected text." Yes saves `translate_ok` and opens the page, but only while the strip it asked about still shows: a selection changed during the wait was not the one the user chose, so it opens nothing. Not now opens nothing and asks again next time (builder's call). Settings can turn it off again.
- **Target:** `translate_to`, empty by default, which means Windows' display language (the OS's on other platforms); otherwise a language code. The page itself lets the user change it.
- **Sites:** a table in the module (`sites.toml`): an id, a name, a link pattern with the text and target, its own length limit, and the most bytes of the whole encoded link (`link_max`, provisional until G19 measures it). P10 ships `google`; `translate_site` picks one.
- **Never:** the text in a log, the link in a log, a read before the click, a read in a password box, or a link opened by our own process with its rights: the platform hands the link to the desktop shell, as the Reveal port opens folders (P8).
- **Later (F8):** a `translate_with` choice of "browser page" or "on this PC"; the same keycap then shows the translation in a box, with Mozilla's models (about 24 MB per direction, Arabic both ways, MPL-2.0; `kx-licence-check` first).

## Settings (`[selection]`, all shown by P13's Settings window)

- `on` (true): the whole helper.
- `copy`, `cut`, `paste`, `translate` (true): each keycap.
- `caret_paste` (true) and `caret_s` (5, from 2 to 30).
- `look` (`both`): `both`, `icons` or `words`; a bad value goes back to `both` with the settings notice.
- `minimised` (true): show while the keyboard is minimised.
- `translate_to` (""), `translate_site` ("google"), `translate_chars` (1,000, at most the site's limit), `translate_ok` (false, set by the first Yes).
- The builder adds the timing and limit settings the detection needs, each with bounds: `settle_ms`, `read_ms`, `looks_per_s` and `consent_s` (`crates/kx-mod-selection/README.md`).
- `edit_ms` (5,000, from 1,000 to 30,000): the longest an edit or a Translate open may go unanswered; then its strip hides and the user is told.

## How the selection is found (Windows first)

- **UI Automation only,** on the focused element: its kind, whether its selection holds text, the selection's line boxes, whether it is editable, and its caret's box. No text is read, except Translate's one read at a click.
- **Driven by events, never a timer:** a focus change, a text-selection change, our presses and releases while the module wants them (`PressesWanted`), Grab's drop and our keys. A short wait after an event lets the app settle, and reads are capped per second. Idle CPU stays near 0 % (spec §10).
- **No gesture fallback (owner):** an app that does not tell its selection gets no chips. This replaces the spec's "Fallback: our mouse hook sees a drag-select or double-click gesture".
- **Reads** run on the adapter's own COM thread with a deadline, never on Qt's thread or the hook's, and a hung app gets no look until it answers.
- **macOS and Linux (P15):** the same port over Accessibility (`AXSelectedTextRange`, `AXBoundsForRange`) and AT-SPI (`Text.GetSelection`, `GetRangeExtents`).

## Privacy and security

- Selected text stays in the app unless Translate is clicked and accepted; even then it lives only in the link handed to the browser.
- Boxes, kinds and points from other apps are untrusted: every number is checked and the maths saturates (the spike's `geom` tests).
- A link's text is percent-encoded, stripped of control and hidden characters (`kx_module_api::is_hidden`), and capped; the site table is compiled in.
- The module holds only the capabilities it needs; any new one, and the change to the spec, go in a new ADR (structural change).
- `kx-security-check` writes `docs/security/kx-mod-selection.md`.

## Moving the spike (move map, P10 row)

- `core/src/selwatch.rs` and `geom.rs`: the show rules into `kx-mod-selection`, the UIA part into `kx-platform-windows`, and the pure placement (`anchor`) into `kx_module_api::ports::geom`, with which the ui's hub places the strip; `qt-kb/qml/Pill.qml` becomes kx-ui's keycap strip; the pill case of `tst_extras.qml` moves into kx-ui's `tst_selchips_app.qml`, where the shadow plugin loads (P10 item note); `sendkeys` `combo`, the pill part of `tools.rs` and the `[tools]` pill lines are deleted.
- Gate `g19pill` is gone. `g19 core` is detection only, in Notepad and Chrome; `g19 keyxtend` drives the strip on the test build's own data folder, in Notepad, Chrome, Windows Terminal and the console, never Word (owner, 2026-10-16); `g19 translate-read` and `g19 translate-beside` only read the page the owner opens.

## Left to the builder

- Module, port, event and capability names and shapes; whether the keyboard or the module sends the edit keys.
- The adapter tables (editable, read-only, terminal) and how a caret's box is found when the app gives an empty range.
- How the strip avoids the keyboard and panels, and what happens on a selection that spans two screens.
- The exact look-wait and read limits, within the bounds above.

## Tests and checks

- Unit and property tests for placement, the show and hide rules, the kind table and the link builder (EN, FR, AR text, the cap, hidden characters), all on the fake clock.
- Qt Quick tests for the strip in each theme, light and dark, each language, each size, and each `look`.
- G19 on the product: a selection in Notepad and Chrome is noticed without copying; the strip shows without taking focus; Copy copies; Cut and Paste work in Notepad; a page in Chrome shows no Cut; the caret Paste shows after a click; Windows Terminal and the console take the terminal keys; Translate asks once, and Not now opens nothing. Translate's real open, which sends text to Google, is in the owner's manual check.
- Edge cases: an empty or whitespace selection, a selection scrolled off screen, a selection over two screens, a box closed while the strip shows, Arabic text, a very long selection, a terminal whose selection clears before the click, a password box, a hung app.
