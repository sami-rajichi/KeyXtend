# Phase 8: clipboard history (v0.5)

## Goal
- A private clipboard history in a paged, themed panel: search, filters, tick-then-Insert, pins and a preview page, sealed on disk in day folders.
- Spec §7 in `2026-09-26-accessible-keyboard-design-modules.md` (approved 2026-10-30); `docs/FEATURES.md` P8; the move map's P8 row; `docs/items/P8.md`.

## Scope
- In: `kx-mod-clipboard`, `kx-ipc`, `apps/keyxtend-worker` (`media` only), the Windows clipboard and worker adapters, the sealed shelf, the panel, the history shortcut, G20 aimed at the product, a G8 clipboard part, and the P7 CI test that fails on main.
- Out: editing `skip_apps` and the other `[clipboard]` values in a window (P13); macOS and Linux adapters (P15); thumbnails of copied image *files*; video length; the voice, OCR and update workers (P11, P12, P14).

## Design notes
- **Crates:** new `kx-mod-clipboard`, `kx-ipc`, `apps/keyxtend-worker`; touched: `kx-module-api`, `kx-crypto`, `kx-platform`, `kx-platform-windows`, `kx-platform-fake`, `kx-mod-keyboard`, `kx-mod-predict`, `kx-ui`, `kx-theme`, `apps/keyxtend`, `xtask`, `tools/kx-gates`.
- **Capabilities (ADR-0034):** new `WriteClipboard`. `ReadClipboard` gates watching and the events that carry content (`ClipKey`, `ClipView`, `ClipAct`); `WriteClipboard` gates writing and `PasteAsk`; `SecretStore` gates the shelf. Holders: clipboard all three; keyboard `ReadClipboard` and `WriteClipboard` (event gates only, no service); ui `ReadClipboard`.
- **Events:** `ClipOpen` (keyboard: the side key or the history chord toggles the panel), `ClipPanel { open }` (lights the key, starts panel keys), `ClipKey` (a nav, Tick, Erase or a search character, `Redacted`), `ClipView` (the page to draw), `ClipAct` (ui clicks), `PasteAsk { round }` and `Pasted { round, ok }` (the keyboard stays the only typist), `HistoryWanted { on }` and `HistoryAsk` (as P7's `PressesWanted`).
- **Ports:** `Clipboard` (`ports/clip.rs`, `ReadClipboard`): a watch that tells copies (parts: text, HTML, RTF, image bytes with codec, file paths; the app; skipped, cleared or too big), plus file facts (size or gone). `ClipboardWriter` (`WriteClipboard`): sets parts with our marker, files always as a copy. `SealedShelf` (`ports/shelf.rs`, `SecretStore`): sealed blobs in named folders: load, save, remove, drop a folder, list, and room (used, free, drive name). `Pictures` (`ports/pictures.rs`, no gate): bytes in, small RGBA pixels out, within a deadline. UI port `Reveal` (`kx-platform`): shows a folder in the system's file manager.
- **Store:** `<data>/clipboard/` with one key file; folders `yyyy-mm-dd` and `pinned`, each with a sealed `list` (titles, times, sizes, folded search text, hashes) and one blob per entry, split in parts under `MAX_BLOB_BYTES`, plus a small thumbnail blob. The AEAD's extra data is folder and name, so a moved file will not open.
- **Worker:** `keyxtend-worker media` beside the program, started on demand with uiAccess cleared and normal integrity, in a job object with a memory cap that kills it with us; one request at a time over its inherited pipes (`kx-ipc` frames with version, kind, id and capped length); stopped when idle, restarted with backoff. It decodes PNG (`png` crate) and DIB (our reader), refuses anything over `max_pixels` before decoding, and scales down.
- **Folding:** prediction's `fold`, `casing` and `fold.toml` move to `kx-module-api::text`, which modules may use (tidy's `module_allowed`); prediction and search share one table.
- **Settings `[clipboard]`:** `keep_days` 7 (1–7), `media_days` 1 (1–7), `max_entries` 1000 (1–5000), `max_mb` 1024 (64–16384), `min_free_mb` 2048 (256–65536), `max_entry_mb` 32 (1–64), `clear_follow_s` 120 (10–600), `skip_apps` [], `join_breaks` 2 (1–3), `join_rule` "" (≤ 80 characters), `paste_gap_ms` 150 (20–2000), `history_key` true, `search_chars` 1000, `title_chars` 300, `preview_chars` 20000, `thumb_px` 128, `preview_px` 960, `max_pixels` 100000000, `media_ms` 3000, `worker_idle_s` 30, `worker_mb` 512, `open_ms` 250, `poll_ms` 10, `save_ms` 500. The media extensions are a module table, `media.toml`.
- **Tokens:** `icons.toml [clip]` (bundled: clipboard-clock, clipboard-paste, search, eye, folder-open, x, check, chevron-up, chevron-down; new from Lucide: file-text, image, film, file-audio, link, pin, pin-off, trash-2, hard-drive, arrow-left); `shape.toml [clip]` (panel width, row, thumb, tick box, search line, chips, buttons, gaps); `motion.toml [clip]` (tick pop, highlight move, row leave, page turn, all off with reduced motion); colours reuse the `pop_*` tokens, and new ones only where a check fails.
- **Texts (EN, FR, AR):** `clip.title`, `clip.kept`, `clip.search`, `clip.filter-{all,text,images,files,media}`, `clip.{preview,pin,unpin,delete,delete-all,insert,back,yes,no}`, `clip.confirm`, `clip.empty`, `clip.{today,yesterday}`, `clip.day-{1..7}`, `clip.date`, `clip.size`, `clip.size-small`, `clip.storage`, `clip.open-folder`, `clip.chars`, `clip.expires`, `clip.note-{too-big,disk,pinned-full,paused-key}`, `num.decimal`, `notice.clip-{damaged,locked}`, `module.clipboard`.

## Tasks
1. **P7's CI test.** *Files:* `kx-ui/qmltest/tst_wordmenu.qml`, maybe `WordMenu.qml`. *Test first:* reproduce under load (`superpowers:systematic-debugging`). *Implement:* the root cause, never a longer wait alone. *Verify:* `qmltestrunner` at both scalings, then CI.
2. **Licences and downloads.** *Files:* `THIRD_PARTY.md`, `deny.toml`, `assets/icons/lucide/`. *Implement:* `kx-licence-check` for `png` and `fdeflate` and the 10 Lucide icons; the owner's yes per download with sizes, both drives checked. *Verify:* `cargo deny check`.
3. **ADR-0034.** *Files:* `docs/adr/0034-*.md`, `docs/adr/README.md`. *Implement:* the capability, events, the three ports, the store layout, the worker's rights and pipe, the folding move, and how macOS and Linux fill each port.
4. **Module API.** *Files:* `kx-module-api/src/{capability.rs, event*.rs, ports.rs, ports/{clip,shelf,pictures}*.rs, text*.rs}`, `kx-mod-predict` imports. *Test first:* every new event and service names its gate; `ClipKey` and `ClipView` print redacted; folding tests pass from the new home. *Implement:* the types and the move. *Verify:* `cargo test -p kx-module-api -p kx-mod-predict`.
5. **Fakes.** *Files:* `kx-platform-fake/src/{clip,shelf,pictures}.rs`. *Test first:* scripted copies, skips and clears; recorded writes; a shelf that fails on cue and reports room; pictures that answer, fail or hang. *Verify:* `cargo test -p kx-platform-fake`.
6. **`kx-ipc`.** *Files:* the crate, `docs/security/kx-ipc.md`. *Test first:* frames round-trip; a wrong version, kind or length is refused; property: random bytes never panic and never allocate past the cap. *Verify:* `cargo test -p kx-ipc`.
7. **Sealed shelf.** *Files:* `kx-crypto/src/{shelf.rs, shelf/*.rs}`. *Test first:* save then load per folder; a file moved to another folder or name will not open; remove and drop leave nothing; bad folder names are refused; a damaged blob is set aside once. *Verify:* `cargo test -p kx-crypto`.
8. **The worker.** *Files:* `apps/keyxtend-worker/src/{main.rs, media.rs, media/{png,dib,scale}.rs}`, fixtures. *Test first:* PNG and 24/32-bit DIB fixtures give the right pixels; scaling keeps within `thumb_px`; truncated, huge and random inputs fail cleanly (property). *Verify:* `cargo test -p keyxtend-worker`.
9. **Worker host.** *Files:* `kx-platform/src/{pictures.rs, pictures/*.rs}`, `kx-platform-windows/src/{worker.rs, worker/*.rs}`. *Test first:* starts on demand, one request at a time; a hang past `media_ms` kills and restarts with backoff; bad replies are refused; idle stop; on Windows the child's token has no uiAccess and medium integrity, and only the pipe handles are inherited. *Verify:* `cargo test -p kx-platform -p kx-platform-windows`.
10. **Windows clipboard.** *Files:* `kx-platform-windows/src/{clip.rs, clip/{watch,read,write,marks,app}.rs, clip.toml}`, the moved `tools/kx-gates/src/{clip,cliplisten,cliptext}.rs`. *Test first:* a marked copy is skipped unread; an empty or blank clipboard is `Cleared` with its app; formats over the cap are `TooBig`; a write carries our marker; files go with the copy effect; file facts see a deleted file. *Verify:* `cargo test -p kx-platform-windows clip`.
11. **History rules.** *Files:* `kx-mod-clipboard/src/{entry,hash,order,keep,follow}.rs`. *Test first:* pinned first, newest first; a duplicate moves up; unpin counts as now; calendar sweep at midnight (fake clock); the caps drop the oldest unpinned; pinned filling a cap refuses with a note; the free floor; follow-the-clear and `skip_apps`. *Verify:* `cargo test -p kx-mod-clipboard`.
12. **Store.** *Files:* `kx-mod-clipboard/src/{store.rs, store/{list,parts,codec}.rs}`. *Test first:* lists round-trip; a 32 MB entry splits and joins; a hand-deleted day leaves the list; a damaged list sets aside one day and tells once; deletes remove every part; property: a random list never panics. *Verify:* `cargo test -p kx-mod-clipboard store`.
13. **Search and panel state.** *Files:* `kx-mod-clipboard/src/{find,panel,view}.rs`. *Test first:* `ECOLE` finds `école` and `ارسلت` finds `أرسلت`; filters; ticks keep their order across searches; Esc clears, then closes; Delete all counts and spares pinned; the view holds 5 rows a page. *Verify:* `cargo test -p kx-mod-clipboard`.
14. **Insert and the actor** (`kx-new-module`). *Files:* `kx-mod-clipboard/{Cargo.toml, defaults.toml, media.toml, src/{lib,manifest,settings,insert,media,actor*}.rs}`, `docs/security/kx-mod-clipboard.md`. *Test first:* the manifest; tick order; separators are pasted text, never Enter; `paste_gap_ms` between steps; the last item stays; a gone file drops its entry; `HistoryWanted` follows `history_key`; thumbnails queue to `Pictures`. *Verify:* `cargo test -p kx-mod-clipboard`; `cargo xtask tidy` 0/0.
15. **Keyboard.** *Files:* `kx-mod-keyboard/{board.toml, src/{spec,panel,face/*,run}.rs}`, `kx-platform-windows/src/shortcuts.toml`, tests. *Test first:* the side key sends `ClipOpen`; with the panel open Space ticks, Backspace erases, a letter goes to search and not to the app; the language list still types Space; `PasteAsk` lets latches go and sends paste once; the history chord opens ours only while wanted. *Verify:* `cargo test -p kx-mod-keyboard`.
16. **The app.** *Files:* `apps/keyxtend/{Cargo.toml, src/boot.rs, src/dirs.rs}`, `tests/life/{clip,ships}.rs`, `xtask` (stage, dev-install, tidy lists), root `Cargo.toml`. *Test first:* `mod-clipboard` is a default; each service reaches only its holder; the worker is staged beside the app with no uiAccess; `png` is banned from the app. *Verify:* `cargo test -p keyxtend`; `cargo xtask stage` ≤ 80 MB.
17. **UI hub and texts.** *Files:* `kx-ui/src/{hub/clip.rs, bridges/clip*.rs, ui_module.rs}`, `i18n/{en,fr,ar}.ftl`. *Test first:* one `ClipView` wakes the panel once; clicks send `ClipAct` with the view's round; sizes read "2,41 Mo" in French; every key in three languages. *Verify:* `cargo test -p kx-ui`.
18. **Views and tokens.** *Files:* `kx-ui/qml/Clip{Panel,Head,Search,Filters,Row,Foot,Preview,Confirm,Room}.qml`, `kx-theme/tokens/{icons,shape,motion,themes}.toml`, `qmltest/tst_clip*.qml`. *Test first:* 5 rows padded; tick numbers; "Insert (3)"; an Arabic row reads right to left; preview pages without scrolling; the empty line; contrast in every theme, mode and high contrast. *Verify:* `qmltestrunner` at both scalings; `cargo test -p kx-ui -p kx-theme`.
19. **Gates.** *Files:* `tools/kx-gates/src/{g20*.rs, g8clip.rs}`, `kx-gates-p8.toml`, `docs/spike-move-map.md`. *Implement:* G20 on the product in the target window and a Notepad tab of its own: marked copies absent, tick two, Insert (2), a blank line between; Win+V opens ours; the installed worker has no uiAccess. G8 measures memory with 1,000 entries. *Verify:* `cargo test -p kx-gates`.
20. **Docs.** `ARCHITECTURE.md`, `docs/security/*`, spec §7 "as built", `docs/items/{P9,P13}.md`.
21. **Review and security.** `kx-review` on every changed file, `kx-security-check`, `/code-review`, `/security-review`; fix every finding.
22. **Gates, manual check, done.** The controller runs G20 and G8 with a heads-up; `kx-windows-manual-test` with the owner; `kx-module-done`; the records.

## Edge cases
1. A 200 MB image: not kept, a note, nothing written.
2. `max_entries = 10000` in the file resets alone to 1,000 with the repair notice.
3. A marked copy is never read; a copy followed by a clear within `clear_follow_s` is deleted; a `skip_apps` copy is never kept.
4. A deleted file drops its entry at the next look or insert.
5. Focus moves between ticking and Insert: the paste goes to the app in front.
6. Under `min_free_mb` free: nothing new kept, a note; keeping resumes with space.
7. A day folder deleted by hand: its entries leave, nothing crashes.
8. Midnight while running: the folder past `keep_days` goes; a clock set back keeps entries and order.
9. The worker crashes, hangs or sends bad pixels: the row keeps its type icon, and the worker restarts with backoff.
10. A damaged day list: that day is set aside, the notice shows once, other days stay.
11. A duplicate copy moves up and stays pinned if pinned; Unpin counts as now; pinned filling a cap refuses with a note.
12. Ticks survive search and filter; Delete all spares pinned and names the count.
13. Insert with Ctrl or Shift latched: the latches let go, and one plain paste goes out.
14. Win+V with `history_key` off or the module off goes to Windows.
15. The OS key is unavailable: the module fails with its notice and Try again.
16. Files cut in Explorer are inserted as a copy.
17. Another app holds the clipboard open: a read gives up quietly after `open_ms`; a write stops Insert with a note.

## Security
- Capabilities as above; the app's `ships.rs` pins each holder.
- Data: clipboard content and search text (`Redacted`, never logged), file paths and app names (personal), the shelf key (DPAPI-wrapped).
- Threats to test: a module without the gate reads content or triggers a paste; a marked copy read; a tampered or moved sealed file; a decompression bomb or crafted DIB; the worker keeping uiAccess or extra handles; a flood of copies (caps and save debounce hold).

## Manual Windows check
- Copy text in EN, FR and AR, a link, an image (a browser and Snip), files in Explorer; open with the side key and with Win+V.
- Search with and without accents and harakat; tick across two searches; Insert (2) into Notepad (own tab) and a browser box; paste images and files.
- Pin, Delete, Delete all with its question; preview a long text and an image; Open folder shows the day folders; the storage line.
- Every theme in light, dark and high contrast; Arabic keyboard language.

## Done when
- The Definition of Done in `docs/roadmap.md`, with every edge case above tested.
- G20 and G8 pass on the product; main's CI is green again; `docs/FEATURES.md` marks P8 `Done (v0.5)` and P9 `Ready`.
