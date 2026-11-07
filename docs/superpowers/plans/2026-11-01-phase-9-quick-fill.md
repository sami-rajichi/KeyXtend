# Phase 9: Quick-fill and one data folder (v0.6)

## Goal
- One data folder, never a hidden one; and Quick-fill: a notebook of saved values of any kind, filled at the caret in one click, with a locked backup box that keeps itself up to date.
- Design: `docs/superpowers/specs/2026-11-01-quick-fill-design.md` (approved 2026-11-01); `docs/FEATURES.md` P9; the move map's P9 row; `docs/items/P9.md`.

## Scope
- In: the hidden-run probe and hidden-copy cleanup; `kx-mod-vault`; a `vault` worker job (`age` boxes, KeePass import); the places and box ports; the panel and its pages; notices that ask; the keyboard's `fill` key and fill route; the `@@` emails; G23 aimed at the product.
- Out: a PIN lock and the whole-keyboard backup (P13 ideas); Hello, Touch ID and any OS prompt (dropped); CSV import and plain export (dropped); macOS and Linux adapters (P15).

## Design notes
- **Crates:** new `kx-mod-vault`; touched: `kx-module-api`, `kx-crypto`, `kx-ipc`, `kx-platform`, `kx-platform-windows`, `kx-platform-fake`, `kx-mod-keyboard`, `kx-mod-predict`, `kx-mod-clipboard`, `kx-ui`, `kx-theme`, `kx-kernel`, `apps/keyxtend`, `apps/keyxtend-worker`, `xtask`, `tools/kx-gates`, `spike/`.
- **Capability:** new `Vault`. It gates the notebook store, the box and places ports, and every Quick-fill event that carries values. Holders: vault; keyboard and ui as event gates; predict for `VaultEmails` only (ADR-0035 pattern). As built, ADR-0039 moves the store, boxes and places to `VaultData`, which only the vault holds; `Vault` gates the events alone.
- **Events:** `FillOpen` (side key or lane chip, no gate); `FillPanel { open }`, `FillKey(PanelKey)`, `FillView(Arc<FillPage>)`, `FillAct` (ui clicks), `FillType { round, parts }` (text and Tab parts, `Redacted`) and `FillTyped { round, ok }`; `VaultEmails`; `ClipTake { round }` and `ClipTaken { round, text }` (`ReadClipboard`; the clipboard hands over its newest text and forgets it); `NoticeAnswered { key, yes }` (no gate, no data).
- **Ports:** `VaultStore` (`Vault`, a `SealedStore` over `<data>/quickfill/` with its own key); `Boxes` (`Vault`, worker-backed: make a padlock, seal a box, open a box, check a password, read a KeePass file); `Places` (`Vault`: suggested places, a folder's subfolders, find boxes by name, read capped bytes, write a box in one step, a place's state); UI port `Hidden` (`kx-platform`: the probe, the hidden copies, delete one).
- **Notices:** `Notice` gains an optional ask (yes and no text keys); the hub sends `NoticeAnswered`, and the app's request queue takes the hidden-copy answers.
- **Store:** `<data>/quickfill/` with `sealed.key`, one sealed `notebook` blob (entries, kinds, fields, use order), one sealed `backup` blob (places, padlock, its locked key, check dates and tries).
- **Box:** `KeyXtend Backup.kxbox`: a mark and version, then `age` parts: the identity locked by scrypt, and the named parts sealed to the X25519 recipient. Only the worker reads or writes the format.
- **Hidden run:** the probe runs before the log and `Kernel::load`; then settings run from defaults in memory, no log is written, the sealed stores are `kx-crypto`'s new in-memory `MemoryStore` and `MemoryShelf`, and the vault module stays off with its note.
- **Tables:** `kx-mod-vault` `kinds.toml` (kinds, icons, starter labels), `labels.toml` (labels, hidden by default), `make.toml` (generator characters); `kx-platform-windows` `places.toml` (cloud folders by environment variable and known path) and `redirect.toml` (the packages root and mirror path), named `hidden.toml` here at first, and in ADR-0037.
- **Settings `[vault]`:** `field_chars` 1000 (8–4000, so a field holds a made or a backup password), `max_entries` 2000 (1–10000), `max_fields` 30 (1–100), `make_chars` 20 (8–64), `box_delay_ms` 2000 (250–60000), `box_min_chars` 8 (8–64), `check_days` [1, 3, 7, 14, 30], `check_tries` 5 (1–10), `memory_checks` true, `tab_gap_ms` 150 (20–2000), `box_mb` 16 (1–64), `box_work` 18 (scrypt log N, 16–20; log N 20 needs 1 GiB, so the vault job's memory cap follows the top), `vault_ms` 20000 (2000–120000). `[kernel] kept_copies` [] (hidden copies the user kept).
- **Tokens:** `icons.toml [fill]` (key-round, user, globe, wifi, id-card, sticky-note, tag, plus, archive, eye, eye-off, wand-sparkles, clipboard-paste, usb, cloud, hard-drive, folder, file-input, trash-2, arrow-left, check, x, chevron-up, chevron-down, chevron-right; new ones from Lucide); `shape.toml [fill]`; `motion.toml [fill]`; colours reuse `pop_*`.
- **Texts (EN, FR, AR, in `i18n/fill/`):** `fill.title`, `fill.kind-*`, `fill.label-*`, `fill.{new,edit,save,cancel,delete,open,insert,user,password,both,show,hide,paste,make,add-field,own-label,back,by-label,search,empty}`, `fill.typed`, `fill.not-typed`, `fill.backup-*` (setup, places, state, password, checks, bring back, found, foreign box), `fill.import-*`, `notice.{hidden-run,hidden-copy,vault-check,vault-offer,vault-damaged}`, `module.vault`.

## Tasks
1. **ADRs and docs first.** *Files:* `docs/adr/{0037,0038}-*.md`, `docs/adr/README.md`, spec §8.1, §9.1 and §14 D4, roadmap P9. *Implement:* ADR-0037 (hidden run, probe, cleanup); ADR-0038 (Quick-fill, `Vault`, no unlock, the box, Hello dropped, superseding ADR-0010's vault lines).
2. **Licences and downloads.** *Files:* `THIRD_PARTY.md`, `deny.toml`, `assets/icons/lucide/`. *Implement:* `kx-licence-check` for `age`, `keepass` and their new dependencies, and the new Lucide icons; the owner's yes per download with sizes, both drives checked. If `age` fails, ask the owner before any other format. *Verify:* `cargo deny check`.
3. **Hidden probe (Windows).** *Files:* `kx-platform/src/{ui/hidden.rs, mirror.rs}`, `kx-platform-windows/src/{redirect.rs, redirect.toml}`, fakes. *Test first:* a probe found in a package mirror is a hidden run; none is a normal run; the probe file is always deleted; copies are listed and deleted; a live test behind `KX_LIVE_HIDDEN=1` sees this session as hidden. *Verify:* `cargo test -p kx-platform -p kx-platform-windows hidden`.
4. **In-memory stores.** *Files:* `kx-crypto/src/{memory.rs, memory/*.rs}`. *Test first:* `MemoryStore` and `MemoryShelf` pass the same contract tests as the sealed files and never touch the disk. *Verify:* `cargo test -p kx-crypto`.
5. **Notices that ask.** *Files:* `kx-module-api/src/notice.rs`, `kx-ui/src/hub/notices*.rs`, `kx-ui/qml/Notice.qml`, tests. *Test first:* an ask notice shows two buttons with their texts; a click sends `NoticeAnswered` once; Esc means No. *Verify:* `cargo test -p kx-module-api -p kx-ui`; `qmltestrunner`.
6. **Hidden run in the app.** *Files:* `apps/keyxtend/src/{run.rs, boot.rs, hidden.rs}`, `kx-settings` (a no-save, no-log mode), `tests/life/hidden.rs`. *Test first:* a hidden run reads no settings, writes no file and no log, starts the vault off with its note and shows `notice.hidden-run`; a normal run asks about each copy not in `kept_copies`; Yes deletes, No adds to `kept_copies`. *Verify:* `cargo test -p keyxtend -p kx-settings`.
7. **Module API.** *Files:* `kx-module-api/src/{capability.rs, ports.rs, ports/{fill,boxes,places}*.rs}`. *Test first:* each event and service names its gate; values print redacted; `FillPage` carries a hidden value only while shown. *Verify:* `cargo test -p kx-module-api`.
8. **Fakes.** *Files:* `kx-platform-fake/src/{boxes,places,hidden}.rs`. *Test first:* scripted padlocks, boxes that open or refuse, places that vanish or fill up, hidden copies. *Verify:* `cargo test -p kx-platform-fake`.
9. **IPC and the worker job.** *Files:* `kx-ipc/src/{ask,reply,command,limits}.rs`, `apps/keyxtend-worker/src/{vault.rs, vault/{boxes,keepass}.rs}`, fixtures. *Test first:* the new frames round-trip and refuse bad lengths; a box seals and opens; a wrong password, a truncated or foreign box, KDBX 2 and key-file KeePass files fail cleanly; random bytes never panic (property). *Verify:* `cargo test -p kx-ipc -p keyxtend-worker`.
10. **Worker host for both jobs.** *Files:* `kx-platform/src/{worker.rs, boxes.rs}`, `kx-platform-windows/src/worker*.rs`. *Test first:* the `vault` job starts on demand with the media job's rights, one request at a time, within `vault_ms`; a hang is killed and restarted with backoff. *Verify:* `cargo test -p kx-platform -p kx-platform-windows`.
11. **Places (Windows).** *Files:* `kx-platform-windows/src/{places.rs, places/*.rs, places.toml}`. *Test first:* cloud folders come from the table; removable drives are listed; the data folder is refused; a find matches `keyxtend` in any case with the extension and checks the mark; a write replaces in one step; a missing or full place reports so. *Verify:* `cargo test -p kx-platform-windows places`.
12. **Notebook rules** (`kx-new-module`). *Files:* `kx-mod-vault/{Cargo.toml, defaults.toml, kinds.toml, labels.toml, make.toml, src/{lib,manifest,settings,entry,kinds,find,order,make}.rs}`, `docs/security/kx-mod-vault.md`. *Test first:* kinds give their starter fields; empty fields are not kept; limits hold; search folds as §6 and skips hidden values; use order then A to Z; Make one draws from the table at the set length. *Verify:* `cargo test -p kx-mod-vault`.
13. **Panel state.** *Files:* `kx-mod-vault/src/{panel.rs, panel/*.rs, view.rs}`. *Test first:* filters, By label and its counts, entry page, New and Edit with the highlighted field taking letters, Delete asks once, Esc backs out, 5 rows a page, Show hides again on leaving. *Verify:* `cargo test -p kx-mod-vault panel`.
14. **Fill and Paste.** *Files:* `kx-mod-vault/src/{fill.rs, paste.rs}`, `kx-mod-clipboard/src/actor/take.rs`. *Test first:* Both sends User, Tab, Password and never Enter; values never reach `Typed`; a failed fill says so; Paste turns breaks to spaces and the clipboard forgets that entry; an empty clipboard or a clipboard module off gives a note. *Verify:* `cargo test -p kx-mod-vault -p kx-mod-clipboard`.
15. **Backup.** *Files:* `kx-mod-vault/src/{backup.rs, backup/{setup,write,checks,bring,foreign}.rs}`. *Test first:* a write `box_delay_ms` after the last change; a missing place waits and catches up; first offer once; memory checks follow `check_days`, `check_tries` leads to a new padlock, `memory_checks` off stops them; a foreign box asks before any write; Bring back adopts or keeps the padlock as designed; newer entries win (fake clock). *Verify:* `cargo test -p kx-mod-vault backup`.
16. **Import.** *Files:* `kx-mod-vault/src/import.rs`. *Test first:* KeePass entries map to kinds and fields; duplicates are skipped; notes lose their breaks. *Verify:* `cargo test -p kx-mod-vault import`.
17. **Keyboard.** *Files:* `kx-mod-keyboard/{board.toml, src/{spec,panel,face/*,run/fill.rs}}`. *Test first:* the `fill` key sends `FillOpen` and lights while open; panel keys go to Quick-fill; `FillType` lets latches go, types each part as Unicode, waits `tab_gap_ms` after Tab, and tells nothing to prediction; opening one panel closes the other. *Verify:* `cargo test -p kx-mod-keyboard`.
18. **Prediction and the lane.** *Files:* `kx-mod-predict/src/email*.rs`, `kx-ui/qml/WordLane.qml`. *Test first:* `VaultEmails` show in `@@` with a key icon and are never learned; a password box's lane shows the Quick-fill chip, which sends `FillOpen`. *Verify:* `cargo test -p kx-mod-predict`; `qmltestrunner`.
19. **The app.** *Files:* `apps/keyxtend/{Cargo.toml, src/{boot.rs, vault.rs}}`, `tests/life/{vault,ships}.rs`, `xtask` stage. *Test first:* `mod-vault` is a default; each new service reaches only its holders; the worker has the `vault` job; `age` and `keepass` stay out of the app's tree (tidy D4). *Verify:* `cargo test -p keyxtend`; `cargo xtask tidy`.
20. **UI hub and texts.** *Files:* `kx-ui/src/{hub/fill.rs, hub/fill/*.rs, bridges/fill.rs}`, `i18n/fill/{en,fr,ar}.ftl`. *Test first:* one `FillView` wakes the panel once; clicks send `FillAct` with the round; every key in three languages, Arabic checked. *Verify:* `cargo test -p kx-ui`.
21. **Views and tokens.** *Files:* `kx-ui/qml/Fill{Panel,Head,Filters,Search,Row,Entry,Labels,Edit,Field,Backup,Places,Folders,Check,Import,Foot}.qml`, `kx-theme/tokens/*.toml`, `qmltest/tst_fill*.qml`. *Test first:* rows padded to 5; row buttons by kind; a row click does nothing; hidden values masked; an Arabic entry reads right to left; contrast in every theme, mode and high contrast. *Verify:* `qmltestrunner` at both scalings; `cargo test -p kx-ui -p kx-theme`.
22. **Spike move and gates.** *Files:* delete `spike/core/src/{hello,fill}.rs` and their callers, the quick-fill part of `spike/qt-kb/src/tools.rs` and `spike.toml [tools]`; `tools/kx-gates/src/g23*.rs`, `kx-gates-p9.toml`, `docs/spike-move-map.md`. *Implement:* G23 on the product in the target window: Both types the user, a Tab and the password, nothing reaches the clipboard, and no Enter is sent. *Verify:* `cargo test -p kx-gates`; `cargo test --workspace`.
23. **Docs.** `ARCHITECTURE.md`, `docs/security/*`, the design's "as built", `docs/items/{P9,P13}.md`, the README section.
24. **Review and security.** `kx-review` on every changed file, `kx-security-check`, `/code-review`, `/security-review`; fix every finding.
25. **Gates, manual check, done.** The controller runs G23 with a heads-up; `kx-windows-manual-test` with the owner; `kx-module-done`; the records.

## Edge cases
1. A hidden run: banner, nothing written (no log either), vault off; the probe file never stays.
2. A hidden copy: Yes deletes it; No keeps it and never asks again; a copy that cannot be deleted gives a note.
3. A damaged notebook: set aside, notice, Bring back offered.
4. A damaged, truncated, foreign-format or newer box: refused with a note, nothing changed.
5. A wrong backup password; 5 wrong memory checks lead to a new password; a stick unplugged then keeps the old one until rewritten.
6. A place gone, read-only or full: "waiting", then catches up.
7. A cloud conflict copy (`KeyXtend Backup (1)`) is found as a box.
8. A foreign box in a picked place: asked before any write; No skips the place.
9. KeePass: not KDBX, KDBX 2, key file, wrong password, all refused cleanly.
10. A value over `field_chars`, holding a hidden mark, or empty.
11. Focus moving during a fill: the rest goes to the app in front then.
12. A fill with Ctrl or Shift latched, a dead key waiting, or the keyboard minimised.
13. 2,000 entries with search; Arabic names, values and labels.
14. Paste with an empty clipboard, an image on it, or the clipboard module off.
15. Two computers writing one cloud box: the last write wins, nothing crashes.
16. The OS key unavailable: the vault fails with its notice and Try again.

## Security
- Capabilities as above; `ships.rs` pins each holder.
- Data: saved values (`Redacted`, never logged, never in `Typed`, never on the clipboard), the backup password (never stored), the padlock and its locked key (sealed), place paths and package names (personal).
- Threats to test: a module without `Vault` reads values or asks a fill; a value in a log or the `@@` learner; a tampered box or notebook; a crafted box or KeePass file in the worker; a box written over another computer's unasked; a hidden run writing anything.

## Manual Windows check
- From the Start menu: the hidden-copy question for Claude's copy; Yes deletes it.
- Add a Person, a Login and a Wi-Fi; fill into the target window and a browser login (User, Password, Both); By label; search in EN, FR and AR.
- Paste from a test note (never the owner's real note); Make one.
- Backup to a cloud folder or USB stick; unplug, change, plug back; a memory check; Bring back into a fresh test build.
- A USB hard disk, if one is at hand: not suggested, and reached through the folder list.
- Every theme in light, dark and high contrast; Arabic keyboard language.

## Done when
- The Definition of Done in `docs/roadmap.md`, with every edge case above tested.
- G23 passes on the product; `docs/FEATURES.md` marks P9 `Done (v0.6)` and P10 `Ready`.
