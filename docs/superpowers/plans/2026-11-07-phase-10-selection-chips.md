# Phase 10: selection chips (v0.6)

## Goal
- Selected text shows a strip of keycaps (Copy, Cut, Paste, Translate) in the keyboard's theme; a caret just clicked into shows Paste; Translate opens the browser's translate page after one consent.
- Design: `docs/superpowers/specs/2026-11-07-selection-chips-design.md` (replaces spec §8.3); mock-up `design/selection-chips.html`; `docs/FEATURES.md` P10; `docs/items/P10.md`; the move map's P10 row.

## Scope
- In: ADR-0041; the `SelectionWatch` and `WebLink` ports, their Windows adapters and fakes; `kx-mod-selection`; the keyboard's edit route; per-asker presses and a release event in the mouse module; the hub, the strip's QML, tokens, icons and texts; `[selection]` settings; G19 on the product; the spike's pill removed.
- Out: the in-app translation box (F8); sites other than Google; Search; a gesture fallback (owner: none); macOS and Linux adapters (P15); the Settings window's page (P13).

## Design notes
- **ADR-0041:** no gesture fallback; new capabilities `Selection` (gate of the chips' events: the module, the keyboard, the ui) and `OpenLink` (hand a link to the browser: the module alone); the module also holds `ReadFocusedText` for Translate's one read and `AimedLine`; Translate's consent; per-asker presses.
- **Port `ports/selection.rs`:** `SelectionWatch` starts a `SelRun` that tells each focus or selection change on any thread; `SelRun::look` gives a `SelSpot` (box id, `SelKind` Editable, ReadOnly, Terminal or Closed, whether a selection holds text, first and last line boxes, the caret box, the work area and DPI), with no text; `still(id)` re-checks a selection; `read(max)` gives `Redacted` text for Translate; `clip_holds` counts clipboard formats without reading. `SelectionWatchKey` is gated by `ReadFocusedText`.
- **Port `ports/link.rs`:** `Link` (an https link, checked, redacted `Debug`), `WebLink::open`, `WebLinkKey` gated by `OpenLink`.
- **Events (`ports/selection/events.rs`, gated `Selection`):** `ChipsView` (module to ui: chips, the spot, a round; or hidden), `ChipAct` (ui to module), `EditAsk` and `EditDone` (module and keyboard; `Edit` Copy, Cut or Paste, and a terminal flag). Ungated: `PointerReleased { ours }`; `PressesWanted` names its asker.
- **Geometry:** the spike's `anchor` moves to `kx_module_api::ports::geom`, extended to keep clear of given rectangles (the keyboard, an open panel).
- **Platform:** OS-neutral run in `kx-platform/src/selection/` (thread `kx-select`, settle wait, look cap, read deadline, a hung app skipped); Windows in `kx-platform-windows/src/selection/` with `selection.toml` (editable and read-only control types, terminal classes), and `src/link/` (`WinWebLink`, the desktop shell's `ShellExecute`, as `reveal`).
- **Shortcuts:** `shortcuts.toml` gains `copy-term` and `paste-term`; `keynames.toml` gains `insert`; G19 picks Ctrl+Insert/Shift+Insert or Ctrl+Shift+C/V.
- **Settings `[selection]`:** `on` true, `copy` `cut` `paste` `translate` true, `caret_paste` true, `caret_s` 5 (2–30), `icons` `words` true (never both off), `minimised` true, `translate_to` "", `translate_site` "google", `translate_chars` 1000 (50–5000, within the site's limit), `translate_ok` false, `settle_ms` 120 (30–1000), `read_ms` 250 (50–2000), `looks_per_s` 10 (1–30), `consent_s` 30 (5–120).
- **Module table:** `kx-mod-selection/sites.toml` (id, name, link pattern, length limit; `google` only).
- **Tokens:** `shape.toml [pill]` becomes `[chips]` (plate pad, cap gap, legend and icon sizes, gap to the text); `motion.toml` `pill` becomes `chips`, plus `chips_done` and `chips_fade`; `common.pill_hover` goes; `icons.toml [chips]` (`copy`, `scissors`, `clipboard-paste`, `languages`, `check`); Lucide `languages.svg` after `kx-licence-check`.
- **Texts (EN, FR, AR), `i18n/sel/`:** the four words and their done words, the strip's reader name, `notice.translate-ask`, `notice.translate-failed`, `module.selection`.

## Tasks
1. **ADR and pointers.** *Files:* `docs/adr/0041-selection-chips-read-no-text-and-translate-in-the-browser.md`, `docs/adr/README.md`, spec modules §8.3 pointer, roadmap Phase 10 line. *Test first:* none (docs). *Implement:* the decisions above. *Verify:* `cargo xtask tidy` 0 errors.
2. **API.** *Files:* `kx-module-api/src/{capability.rs, ports.rs, ports/selection*.rs, ports/link*.rs, ports/assist.rs, ports/words.rs, ports/geom*.rs, notice*.rs}`, contract tests. *Test first:* gates and names unique; `Link` refuses non-https, control characters and over-long text, and prints redacted; `anchor` stays on the area, clears the given rectangles when there is room, never overflows (property). *Implement:* types, events, `anchor`. *Verify:* `cargo test -p kx-module-api`.
3. **Fakes.** *Files:* `kx-platform-fake/src/{selection.rs, link.rs}`. *Test first:* scripted spots and changes are told; `still`, `read` and `clip_holds` answer as scripted; opened links are recorded. *Implement:* `FakeSelectionWatch`, `FakeWebLink`. *Verify:* `cargo test -p kx-platform-fake`.
4. **OS-neutral run.** *Files:* `kx-platform/src/selection/{mod,run,tests}.rs`. *Test first:* a burst of changes gives one look after `settle_ms`; looks stay within `looks_per_s`; a read past `read_ms` is dropped and the box skipped until it answers; nothing runs while idle. *Implement:* the run over an adapter `SelProbe`. *Verify:* `cargo test -p kx-platform selection`.
5. **Windows selection.** *Files:* `kx-platform-windows/src/selection/{mod,win,uia,kinds,tests}.rs`, `selection.toml`, `platform.rs`. *Test first:* the kinds table (Notepad's edit, a read-only document, Windows Terminal, the console, a password box, our own window); boxes come with no text call; the caret box when the range is empty. *Implement:* focus and text-selection handlers on the run's COM thread, `still`, `read`, `CountClipboardFormats`. *Verify:* `cargo test -p kx-platform-windows selection`.
6. **Windows link and terminal keys.** *Files:* `src/link/{mod,win}.rs`, `shortcuts.toml`, `keynames.toml`, `shortcut/tests.rs`. *Test first:* only https reaches the shell; the terminal ids map to their keys. *Implement:* `WinWebLink`, the ids. *Verify:* `cargo test -p kx-platform-windows link shortcut`.
7. **Mouse presses.** *Files:* `kx-mod-mouse/src/{lib.rs, actor/quiet.rs, engine/note.rs}`, `kx-mod-predict` asker field, tests. *Test first:* two askers keep the hook told until both stop, in any order; a release while wanted tells `PointerReleased`. *Implement:* the asker set and the event. *Verify:* `cargo test -p kx-mod-mouse -p kx-mod-predict`.
8. **Keyboard edit route.** *Files:* `kx-mod-keyboard/src/{face/bus.rs, face/edit.rs, run/edit.rs}`, `tests/keyboard/edit.rs`. *Test first:* `EditAsk` sends its id with no held keys and leaves latches as they were; the terminal flag picks the terminal id; prediction hears one `Break`; `EditDone` answers each round; a held key's repeat ends. *Implement:* `Typing::edit`. *Verify:* `cargo test -p kx-mod-keyboard edit`.
9. **Module rules** (`kx-new-module`). *Files:* `kx-mod-selection/{Cargo.toml, defaults.toml, sites.toml, src/{lib,manifest,settings,chips,strip,link}.rs}`, tests and property tests. *Test first:* chips per kind and settings; the strip shows, moves and hides on each rule, and the caret Paste ends on its timer (fake clock); the link is percent-encoded, hidden characters dropped, cut at a whole word, EN/FR/AR; settings bounds and the icons/words rule. *Implement:* the pure parts. *Verify:* `cargo test -p kx-mod-selection`.
10. **Module actor.** *Files:* `kx-mod-selection/src/actor/*.rs`, `tests/selection/*.rs`, `docs/security/kx-mod-selection.md`. *Test first:* a selection tells `ChipsView`; Copy re-checks with `still` and sends `EditAsk` or hides; an aimed panel line hides; Translate asks once, opens only after its own Yes within `consent_s`, saves `translate_ok`, never reads in a closed box; a forged `NoticeAnswered` opens nothing; a panic tells hidden. *Implement:* the `kx-selection` actor. *Verify:* `cargo test -p kx-mod-selection`; `cargo xtask tidy`.
11. **The app.** *Files:* `apps/keyxtend/{Cargo.toml, src/boot.rs, src/run.rs}`, `tests/life/{main,rig,ships}.rs`, `.github/workflows/ci.yml`. *Test first:* `mod-selection` is a default; `ReadFocusedText`, `Selection` and `OpenLink` reach exactly their holders; the watch and the link reach only the module. *Implement:* the wiring. *Verify:* `cargo test -p keyxtend`; `cargo xtask stage`.
12. **UI hub and texts.** *Files:* `kx-ui/src/{hub/sel.rs, hub/bus.rs, bridges/ui.rs, bridges/ui/wake.rs, i18n.rs, i18n/keys.rs}`, `i18n/sel/{en,fr,ar}.ftl`, `apps/keyxtend/tests/agree/{texts,icons}.rs`. *Test first:* `ChipsView` wakes once; the place uses the strip's measured size and clears the keyboard and panels; closed hides, minimised follows `minimised`; a click publishes `ChipAct`; every text exists in three languages. *Implement:* state, placement, invokables, texts. *Verify:* `cargo test -p kx-ui -p keyxtend`.
13. **Views, tokens and icons.** *Files:* `kx-ui/qml/{SelChips,SelKey,App}.qml`, `qtfiles.rs`, `kx-theme/tokens/{shape,motion,icons,themes}.toml`, `kx-theme/src/{extras,shape*,motion,common,hc}.rs`, `assets/icons/lucide/languages.svg`, `THIRD_PARTY.md`, `qmltest/tst_selchips.qml`. *Test first:* each theme's cap kind, light and dark; one key row tall at each size; icons or words off; Arabic words in the theme's Arabic font with the order kept; the tick and fade, none under reduced motion; the shadow fades with the strip (from the spike's `tst_extras`); no focus taken. *Implement:* the strip after the mock-up. *Verify:* `qmltestrunner` at both scalings; `cargo test -p kx-theme -p kx-ui`.
14. **Spike removal.** *Files:* `spike/core/src/{selwatch,geom,sendkeys,voice,config,lib}.rs`, `spike/qt-kb/{qml/Pill.qml, qml/Extras.qml, qmltest/tst_extras.qml, src/tools.rs}`, `spike/spike.toml`, `docs/spike-move-map.md`. *Test first:* the spike still builds and its tests pass. *Implement:* delete the pill and `combo`; `PillPx` moves into `voice.rs` (its last user); `geom` keeps `centre` and `line_points`. *Verify:* `cargo test -p spike-core -p qt-kb`.
15. **Gates.** *Files:* `tools/kx-gates/src/{g19.rs, g19chips.rs, cli.rs, cfgfile.rs, config.rs}`, `kx-gates-p10.toml`. *Test first:* the config check; the gate's own unit tests. *Implement:* G19 on the test build: a selection in Notepad and Chrome is seen with no copy; the strip shows without taking focus; Copy, Cut and Paste in Notepad; no Cut on a Chrome page; the caret Paste; the terminal keys in Windows Terminal and the console. *Verify:* `cargo test -p kx-gates`.
16. **Docs.** `ARCHITECTURE.md`, crate READMEs, `docs/security/{kx-platform-windows,kx-mod-keyboard,kx-mod-mouse,kx-ui,keyxtend}.md`, `docs/items/{P10,P13,F8}.md`.
17. **Review and security.** `kx-review` on every changed file, `kx-security-check`, `/code-review` and `/security-review`; fix every finding.
18. **Gates, manual check, done.** The controller runs G19 with a heads-up, then `kx-windows-manual-test` with the owner, then `kx-module-done` and the records.

## Edge cases
1. An empty selection shows nothing; a whitespace-only one is caught at Translate's one read and opens nothing (no text is read before a click).
2. A selection scrolled off screen has no box: no strip; scrolled back: it shows.
3. A selection near the screen's bottom puts the strip above; one under the keyboard keeps clear of it.
4. A selection over two screens uses the last line's screen and DPI.
5. The box closes while the strip shows: it hides at the next change.
6. A terminal whose selection clears before the click: `still` refuses and nothing is sent.
7. A password box, an unknown box and our own panel lines: no strip, no read.
8. A hung app: its look times out, no strip, and other apps keep working.
9. A very long selection: Translate cuts at `translate_chars`, at a whole word.
10. Arabic, French and mixed text in the link; hidden characters dropped.
11. Translate: Not now opens nothing; a late or forged Yes opens nothing; Yes after `consent_s` opens nothing.
12. Latched Shift or Ctrl stays latched after a chip; Chrome never gets Ctrl+Shift+C.
13. Minimised: chips show (or not, by `minimised`); closed to the tray: none.
14. The caret Paste: an empty clipboard shows none; it ends on a key, a click, a caret move or `caret_s`.
15. `icons` and `words` both off in the file: the repair turns words on with the settings notice.
16. The module off: no strip, the hook stops only when no other asker wants presses.

## Security
- Capabilities: `ReadFocusedText` (the watch, `AimedLine`), `Selection` (the chips' events: module, keyboard, ui) and `OpenLink` (module only); no `InjectInput`, the keyboard stays the only typist.
- Data: selection boxes and kinds (public geometry); the selected text only at a Translate click, redacted, never logged or kept; the link lives only on its way to the shell.
- Threats: untrusted boxes (saturating maths); a forged consent answer; a link to a non-https or table-less site; Ctrl+C to a terminal; reading in a password box; a hung app stalling the thread; the browser inheriting uiAccess (the desktop shell opens it).

## Manual Windows check
- In Notepad: select a word, a line and several lines; Copy, Cut and Paste each; the strip in Native, ET66 and Dolch, light and dark, at S, M and L.
- In Chrome: a page shows Copy and Translate only; Translate's first click asks; Yes opens Google Translate with the text; EN, FR and AR.
- A click in an empty Notepad line with something copied shows Paste, which goes after 5 s or on a key.
- Select with Shift and the arrows on our keyboard, and with Grab; minimise the keyboard and select again.
- Windows Terminal: Copy and Paste work and a running command keeps running.

## Done when
- The Definition of Done in `docs/roadmap.md` holds.
- G19 passes on the product in Notepad, Chrome and both terminals, and the owner's manual check passes.
