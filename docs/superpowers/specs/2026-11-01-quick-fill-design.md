# Quick-fill and one data folder — design (P9)

Decided with the owner on 2026-11-01, from four mock-ups shown in the session. This file replaces spec §8.1 of [`2026-09-26-accessible-keyboard-design-modules.md`](2026-09-26-accessible-keyboard-design-modules.md). Every number below is a `[vault]` setting unless it says otherwise; one design serves Windows, macOS and Linux, and only the adapters differ.

## Why

- Many users keep passwords and personal details in a plain text note and type them by hand: slow with one finger, and unsafe.
- Quick-fill is a notebook of saved values. One click types a value at the caret, as the clipboard's Insert does.
- The notebook must survive a broken, lost or wiped laptop, without handing readable data to a cloud company, and without steps that tire a one-finger user.
- A KeyXtend started from inside another app's package saves into that app's hidden copy. The owner lost history and learned words that way (`docs/items/P9.md`).

## Goals, non-goals, anti-goals

- **Goals:**
  - one click fills any saved value into the box in front;
  - no unlock step;
  - a locked backup box that keeps itself up to date and is found again on a new computer;
  - one data folder, never a hidden one.
- **Non-goals in P9:** website detection or autofill by page; a browser extension; Windows Hello, Touch ID or any OS prompt; a PIN lock (an idea for P13's Settings, off by default); a sync service; a phone app; CSV import or any plain export.
- **Anti-goals (a failure even if it works):**
  - a value in the clipboard, the prediction learner, the logs or a plain file;
  - several steps or questions to fill one value;
  - data written into a hidden folder;
  - a check of the box's kind that blocks a fill.

## Part 1 — One data folder

- **Why the usual check fails:** a program started from a packaged app (Claude, Codex) has no package identity, so `GetCurrentPackageFullName` says "no package", yet its new files under AppData still go to the package's private copy. Microsoft documents the redirection rules ([behind the scenes](https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes), [flexible virtualization](https://learn.microsoft.com/en-us/windows/msix/desktop/flexible-virtualization)) and that children lose identity; field reports show they keep the redirection ([claude-code#94254](https://github.com/anthropics/claude-code/issues/94254), [codex#52698](https://github.com/openai/codex/issues/52698)). The breakaway flags do not lift it.
- **The probe (Windows adapter):** at start, when the data folder lies under the user's AppData, KeyXtend writes a small, randomly named probe file there and looks for it in each package's private mirror of that path (`%LOCALAPPDATA%\Packages\<package>\LocalCache\…`), then deletes it. Found means a **hidden run**. The builder may use the probe handle's final path instead, if it proves reliable on this PC.
- **A hidden run (owner):**
  - a banner says, in short: KeyXtend was started from inside another app, so Windows would hide what it saves; nothing is saved this time; start it from the Start menu;
  - it starts fresh and saves nothing (builder's call, the owner agreeing): default settings, and nothing read from the data folder, since Windows would show the hidden copy's stale data first;
  - the run's copies and learned words stay in memory and are gone when it closes; no log file is written;
  - Quick-fill stays closed, with a note;
  - the probe runs before the log opens and before the kernel reads settings.
- **No relaunch through Explorer (owner):** Microsoft does not document that use, and it is the move this session's tools refused as a sandbox bypass.
- **Hidden copies (owner):** a normal run looks for `KeyXtend` folders in other packages' private copies, from the adapter's table. For each one found, a banner asks once: a hidden copy of KeyXtend's data is in another app's folder (named from the package, such as Claude); delete it? Yes deletes it at once, with no recycle bin. No keeps it and never asks about that copy again.
- **macOS and Linux:** no such redirection is known; their adapters find no hidden run and no hidden copy (P15 checks the macOS App Sandbox).

## Part 2 — Quick-fill

### The notebook

- **Entries:** each has a name (a site, an app, a person, a Wi-Fi, anything), a kind and its fields.
- **Kinds,** each with an icon and starter fields, from the module's kinds table:
  - Person: Full name, Email, Phone, Address;
  - Login: User, Password, and an optional Website;
  - Wi-Fi: Network, Password;
  - Card (an ID or membership card): Number, Name;
  - Note: Text;
  - Other: no starter fields.
- **Fields:** a label, a value and a hidden flag.
  - Labels come from a labels table (Full name, Email, Phone, Address, City, Postcode, Country, Birthday, User, Password, PIN, ID number, Number, Website, Network, Text…) or are the user's own.
  - Password, PIN, ID number and Number start hidden; any field can be switched.
  - Empty fields are not saved.
- **Values** are one line, at most `field_chars` characters (1,000); a line break would send the message in chat apps.
- **Limits:** `max_entries` entries (2,000) and `max_fields` fields per entry (30).
- **Emails:** every field labelled Email also shows in the `@@` list (§6) with a key icon.

### The panel (mock-ups 1 and 4)

- **Opening:** the side block's Quick-fill key (the `fill` key, live from now on) opens and closes it. In a password box, where the word row shows no word chips, the row also shows a Quick-fill chip (builder's call). It opens above the keyboard (§3.4); opening it closes the clipboard panel, and the reverse (builder's call).
- **Header:** key icon, "Quick-fill", New, Backup, and close.
- **Filters:** All, People, Logins, Wi-Fi, Cards, Notes and By label. Other shows under All.
- **Search line:** while it is aimed at (ADR-0040), letters clicked on our keyboard go into it, as in §7. It matches entry names, labels and the values of fields that are not hidden, folding as §6 does. The top row shows word chips for it, a chip goes into it, and nothing typed there is learned (§6). Name, label, value and password lines show no chips and send prediction nothing.
- **List:** one list, 5 rows per page (§3.3), with ▲/▼ and "n of m". The most recently used entries come first, then A to Z (builder's call).
- **Rows:** kind icon, name, a hint line ("alex@example.com · ••••••••", or "Person · 6 fields"), and buttons:
  - a Login: User, Password and Both;
  - an entry with one field: that field's button;
  - any other entry: Open.
  - A click on the row itself does nothing, so a slip never types (owner).
- **Entry page:** each field with its label, its value (hidden ones as ••••••••), Show for hidden ones, and Insert. The header has Edit and Back. A shown value hides again when the page closes (builder's call).
- **By label:** a paged list of the labels in use, with counts. A label lists every value with that label across entries: the entry's name, the value and Insert.
- **New and Edit:**
  - New starts with the kind chips; the kind fills in its starter fields.
  - Name, then the fields, then "Add a field", which offers the labels table, paged, or "Your own label".
  - Each field has Show or Hide and Paste; a Password field also has Make one.
  - The highlighted field takes the letters clicked on our keyboard while the form is aimed at: it aims as it appears, and leaving it puts the aim back as it was, unless a click outside took it meanwhile or another window came to the front (ADR-0040). While the form shows, only a click takes it, so a window that pops up never gets the rest of a secret.
  - A line's own border marks it, with no ring around its row. Only an aimed line draws the caret, the underline and the ring. An empty line shows its hint until it takes the letters; only the search lines keep theirs beside the caret until a letter comes.
  - A hidden value draws one dot per letter while typed, as a website's password box does, and Show reveals the text (owner). An empty one draws none, and readers hear only the fixed mask.
  - Save and Cancel. Edit adds Delete entry, which asks once, Yes or No, and Esc means No (builder's call: a lost password cannot come back).
- **Make one:** a random password of `make_chars` characters (20) from the module's character table (letters, digits, symbols), shown so the user sees it.
- **Paste:** puts the clipboard's current text into the field, with line breaks turned into spaces, and asks the clipboard history to forget that copy at once. That way a password copied from a user's own note leaves no trace.
- **Empty notebook:** an invitation to save a first entry. On a new computer, a found box is offered first (backup below).
- **Keys** come from a key list per panel (§3.3): ↑↓ move, ←→ move the caret in a line with text and switch filters on an empty search line, Delete removes what follows the caret, Enter acts as the highlighted row's first button, and Esc backs out (search, then page, then panel). They reach the panel only while it is aimed at; else they go to the app, and so does a key with Ctrl, Alt or Win latched (ADR-0040). A press anywhere on the panel aims again, unless it follows another inside the double-click time, and a fill takes the aim away. A click on a line's text places its caret, and on a hidden line one press moves one dot. A label's values draw no search line, so letters do nothing there. The scroll pad and one notch of the mouse wheel turn pages (owner); the wheel is silent.
- **Look:** the keyboard's theme in light, dark and high contrast, every text passing the contrast check, and the mock-up's Lucide icons. Arabic flips text only.

### Filling

- **Route:** each button types its value at the caret of the box in front, through the keyboard's own route, as Unicode text, so any layout types any character. The keyboard stays the only typist (ADR-0033).
- **Both** types User, then Tab, then Password, and never Enter (owner).
- **No check of the box's kind (owner):** password, number or text, the value goes in.
- **Latches:** the keyboard lets its latches go before a fill, as before a paste (builder's call).
- **Never:** in the clipboard, told to prediction (`Typed`), logged, or in the panel's JSON unless shown.
- **After a fill,** the panel stays open with a short note, so User and then Password work on two-step login pages; the close button or Esc closes it (builder's call). A failed fill says that nothing was typed.

### Protection: no unlock step (owner)

- The notebook is sealed under the account's own key (ADR-0010's data key under DPAPI, Keychain or Secret Service), as Chrome and Edge keep saved passwords. It lives in its own folder under the data folder, with its own key, as the clipboard's shelf does.
- Files copied to another PC or account open nothing.
- There is no unlock, idle lock or OS prompt: anyone using the signed-in PC can fill.

### The backup box

- **One file,** named `KeyXtend Backup` with the box's extension, in each place the user picked.
  - It is rewritten in the background after every save, edit or delete, `box_delay_ms` after the last change (2,000).
  - A write goes to a temp file and replaces the box in one step.
- **The lock:** the user's own backup password, at least `box_min_chars` characters (8), shown while typing and typed once (owner).
- **The open padlock:** the password is never kept. KeyXtend keeps the box's public key, which locks each new copy. The box carries its private key, locked by the password.
  - Format: `age`, with an X25519 recipient for the copies and an scrypt-locked identity inside (builder's call: an audited, open format).
  - Every box is decoded in the worker (ADR-0004).
- **Places:**
  - setup asks for one place; the Backup page adds or removes places at any time, and P13's Settings shows the same page (owner);
  - suggestions come from the adapter's places table: the cloud folders found on this computer (OneDrive, Google Drive, Dropbox; iCloud Drive on macOS) and the removable drives;
  - Another folder opens a paged folder list: drives, then folders, 5 per page;
  - the data folder itself is refused, as a format wipes it.
- **A missing place** (a stick unplugged, a cloud folder gone): the Backup page and the panel's footer say "Backup waiting for E:\", and the box catches up when the place comes back.
- **Backup page:** each place's state ("Updated today 10:42", "Waiting"), Back up now, Add place, Remove place, Change password, the memory checks switch, and Bring back.
- **First offer (owner):** after the first entry is saved, one question: protect your entries with a backup? Yes leads to the setup (place, then password). Later never asks again; the Backup button stays.
- **Memory checks (owner):**
  - after `check_days` days (1, 3, 7, 14, then every 30), a banner asks whether the user still remembers the backup password, with Check and Later;
  - right: the next check waits longer;
  - wrong: the user may try again, up to `check_tries` times (5). After that the user sets a new backup password, which makes a new padlock and rewrites every box;
  - Later asks again the next day, and a check never blocks typing;
  - `memory_checks` turns them off (owner).
- **Changing the password** also makes a new padlock and rewrites every box. A box in a place not reachable then keeps the old password until it is rewritten.
- **A forgotten password:** while this computer works, the user sets a new one. With the computer gone and the password forgotten, the box cannot open; setup says so in one sentence. There is no spare code (owner).
- **New computer:**
  - while Quick-fill is empty, KeyXtend looks for boxes in the places table: names that contain `keyxtend` in any case, with the box's extension, checked by the mark inside;
  - found: "Found your box from 1 Nov on OneDrive. Bring it back?", then the password; the entries come back, and that place becomes this computer's first place;
  - several found: the newest first, paged;
  - none found: Bring back opens the paged folder list.
- **A box this computer did not make (owner):** a place picked in setup or Add place may already hold one, such as the old laptop's box in the same OneDrive folder. KeyXtend never overwrites it unasked. It asks once: a backup from another computer is here (with its date); bring its entries in? Yes asks that box's password, adds its entries, then rewrites the box with everything under this computer's padlock. No leaves the box untouched and skips that place.
- **Bring back and padlocks:** a computer with no padlock yet keeps the box's own padlock, so its password stays the same. A computer that has one keeps its own, and the box brought back is rewritten under it, so one password opens every place; the Bring back page says so in one sentence.
- **A copy brought back with the box's own file beside it (builder's call):** a cloud conflict copy brings its entries in, but a write to that folder would replace the other file. With no hold its folder stays no place until Add place asks about that file; while held, Replace's question comes first.
- **Bring back into a notebook that has entries:** entries are matched by id, and the newer copy wins (builder's call).
- **Two computers with one cloud box:** each rewrites the whole box from its own notebook, and the last write wins. P9 merges nothing beyond the case above.
- **Room for more:** the box holds named parts, Quick-fill alone in P9. The whole keyboard (settings, look, learned words) is a later session's work (owner, `docs/items/P13.md`).

### Import

- **A KeePass file** (`.kdbx`, KDBX 3.1 and 4) from another app (owner):
  - picked from the paged folder list, with its password typed;
  - decoded in the worker;
  - each entry becomes a Login when it has a password, else Other: Title is the name, UserName is User, Password is Password, URL is Website, Notes is Text with its line breaks turned into spaces, and custom strings become fields under their names;
  - an entry with the same name, kind and user as one already saved is skipped (builder's call);
  - a file that also needs a key file is refused with a note, as KeyXtend opens only password-locked files (builder's call).
- **Paste** (above) brings in a user's text note, one value at a time.

## Settings `[vault]`

`field_chars`, `max_entries`, `max_fields`, `make_chars`, `box_delay_ms`, `box_min_chars`, `check_days`, `check_tries` and `memory_checks`, with the defaults above. The places and the padlock are stored state, not settings, and are kept sealed with the notebook.

## Security

- Every value travels as `Redacted<T>`, is never logged, and buffers that hold it long are wiped. The panel's JSON carries a hidden value only while it is shown; New and Edit carry only its count of characters otherwise.
- A damaged notebook is set aside, with a notice offering Bring back from a box.
- `age` and KeePass run in `keyxtend-worker`, in a `vault` job with the media job's rights and size-limited frames: the worker both seals and opens boxes, so neither format is in the uiAccess app (builder's call).
- This computer also keeps a sealed copy of the box's password-locked private key, so memory checks work while every place is unplugged; it never keeps the password (builder's call).
- Saved emails shown in the `@@` list are never stored by prediction (builder's call).
- A hidden run never opens the notebook.
- The panel's state, keys and acts, and the fill requests, need a new capability, `Vault` (ADR).

## Edge cases for the plan

- A hidden run; a hidden copy found, deleted, kept, or no longer deletable.
- A damaged notebook file; a damaged, truncated or foreign box; a box from a newer version.
- A wrong backup password; 5 wrong memory checks; a new password while a stick is unplugged.
- A place gone, read-only, full, or slow; a cloud conflict copy (`KeyXtend Backup (1)`).
- A KeePass file that is not KDBX, KDBX 2, has a key file, or has a wrong password.
- A value over `field_chars`, holding a hidden mark (refused, as words are), or empty.
- Focus moving during a fill: the rest goes to whatever is in front then, as with the clipboard.
- A fill while a latch is on, while a dead key waits, or while the keyboard is minimised.
- 2,000 entries with search on; Arabic names and values; a label in Arabic.
- Paste with an empty clipboard, an image on the clipboard, or the clipboard module off.

## Left to the builder

- Module and port names, the event set, the folder name under the data folder, the box's extension and its inner layout.
- Exact copy for every note, in EN, FR and AR (Arabic checked).
- How the probe looks for the private mirror (every package is checked), and how the places table finds each cloud folder.
- How `field_chars` sits beside the keyboard's `text_cap`, which stays for picked words.

## Documents this changes

- ADR-0037: a run whose saves would be hidden saves nothing (adds to ADR-0015).
- ADR-0038: Quick-fill is a notebook sealed with the account key and backed up to a locked box. It supersedes ADR-0010's vault lines (user presence, Hello, the master password, and the `age` and CSV export), drops Hello from P9, deletes the spike's `hello.rs` and `fill.rs`, and retires or re-aims gate G23.
- Spec §8.1 points here; §9.1's presence line and §14 D4 change; the roadmap's P9 bullet and edge cases follow this file.

## As built (P9, 2026-11-05)

Where the build settles or differs from the text above, each as the builder's call unless it says otherwise. The owner sees the marked ones at the hands-on check.

### Capabilities and names

- ADR-0039 adds `VaultData`, which gates the notebook store, the boxes and the places; only the vault holds it. `Vault` gates the events alone, so the keyboard, prediction and the ui never reach the data services.
- The Windows package table is `redirect.toml`, read by `redirect.rs`; ADR-0037 and the plan named it `hidden.toml`, but `hidden.rs` is the one running copy's window.
- The vault's folder is `quickfill` (`VAULT_DIR`), its blobs `notebook` and `backup`, with `-aside` copies, and the box's extension `kxbox`.
- Texts for the user say "backup", never "box".

### One data folder

- The probe looks in each package's mirror, as `redirect.toml` names it; the final-path idea was not used. It runs only when the data folder lies under the local app-data folder.
- "Quick-fill stays closed, with a note": the note is the hidden-run banner and the fill key dimmed with its tip, `tip.fill-off`.
- Copies are asked about only in a normal run. Two copies of one app share one question, and the next shows after its answer.

### The panel

- Every row ends with Open, so any entry can be opened to edit: User, Password and Both for a kind's pair, Insert for an entry of one field, then Open. An entry with neither shows Open alone.
- The list keeps its order while the panel is open, and a fill moves the entry at the next opening.
- Space types a space in Quick-fill's search and lines.
- A row and a field click name their view's round, and an older round is dropped, so nothing fills unseen.
- The password chip shows only in a locked lane with the vault live. It is absent while suggestions are off or the vault is not live, as a button that does nothing misleads.
- With the vault off, the fill key dims with `tip.fill-off`, told by `FillLive`.

### Values

- `field_chars` is 1,000 by default, within 8 to 4,000, so a field holds a made or a backup password.
- At Save, Paste and KeePass, a shown value loses LRM, RLM and ALM, which real Arabic copies carry. Any other hidden mark, or any mark in a hidden value, is refused, as a password must never change silently (for the hands-on check).
- Paste trims breaks at the ends. A break inside is a space in a shown field and refuses a hidden one, for the same reason (for the hands-on check).
- Paste reads up to twice `field_chars`. Too long is `Refused` and forgets nothing; a busy clipboard is `ClipBusy`; no answer within `paste_wait_ms` is `ClipOff`. The history forgets a taken text before the vault accepts it, and a copy whose note comes after the take keeps nothing.
- A failed save of the notebook or the backup blob shows `SaveFailed` until a save works, as the change could be lost. Another note takes the line for a moment and `SaveFailed` comes back after it. The next try is at the next change, every `place_check_s` or the stop.

### Filling

- A fill that fails says "did not finish", not that nothing was typed, as a user and a Tab may have gone out already.
- After a Tab, the rest waits `tab_gap_ms`. It never goes more than `[keyboard] fill_late_ms` past due, or to another app, and an app the system cannot name counts as another; this replaces the edge case above where the rest goes to whatever is in front.
- The app is noted before the first part, so a switch before the Tab stops what has not gone out.
- `fill_late_ms` is at least twice `relabel_ms`.
- A fill while one waits inside its bound is refused, never queued. While a dead key waits it is refused too.
- A click outside our windows ends a waiting fill only while the mouse module reports presses, which the keyboard now asks for while a panel is open (ADR-0040).

### Asking notices

- An asking notice shows only its two answers. Esc never reaches a banner, as our windows take no focus (ADR-0013), so the design's "Esc means No" holds for Quick-fill's own pages, whose keys come from the keyboard.
- The first offer is a notice, `notice.vault-offer`, with Yes and Later.
- `NoticeAnswered` carries the whole `Notice`, not a key and a flag.

### The backup box

- A box is a mark, a version and the password-locked identity, then one `age` stream holding every part, so none can be cut or moved. A box with no parts is refused.
- `box_work` is 18, within 16 to 20. A newer box or an unknown part is `Unsupported`.
- Remove's question names its place, and Back up now's question while held is `Replace` (`BackupAsk`). A place dropped while Remove's question shows ends the question, as its row may now be another place.
- A box past `box_mb` cannot fit until the notebook changes, so its place waits for the next change or Back up now, with the note once per try. A restart may try once.
- A pick is checked before any password, and the data folder, above or below it, is `PlaceState::Refused`.
- `DRIVE_REMOVABLE` is what Windows calls removable, so a USB hard disk is not suggested and is reached through the folder list. Telling them apart by bus type waits (`docs/items/P9.md`).
- Cloud folders come from environment variables, known parts and, for Google Drive, the volume label; no cloud app's own config is read.
- `later_h` sets Later's wait (24), not "the next day".

### A box from elsewhere

- A foreign box is asked about only at a pick, in setup and Add place. After that, the last write wins, as the two-computers line above says, so a box "never overwritten unasked" holds at the pick, not at every write.
- The question shows the box's date from its own file (`Places::modified`). When that cannot be read, the page is the undated question, `Sheet::ForeignUndated`.
- A merge past `max_entries` or the notebook's size brings nothing in, writes nothing and shows `Full`, as rewriting after a partial merge could lose entries from what may be their only copy (for the hands-on check).
- Bring back's right password rewrites every place, so an older box can replace a newer one elsewhere; found boxes are offered newest first. Whether Replace's question belongs there is for the hands-on check.

### Reaching Bring back and Import

- The Places page always shows Bring back and Import, in setup and in Add place, as a new computer's only way to a backup in another folder is that button.
- Import is also on the empty notebook's invitation and the Backup page, so a computer with entries and no padlock can reach it.

### A damaged notebook: the hold

- A notebook that did not read holds every box, as the box may be the only copy of what it lost. Adding a place or changing the password leaves places waiting, Back up now asks once, and only its Yes or Bring back ends the hold. Bring back brings its entries in at once, but while another place would be overwritten it asks the same question first (owner); until that Yes, the hold stands and every other box stays as it is.
- The hold persists in the backup blob, so a restart keeps it.
- Import adds to a held notebook and writes no box. The note, `Paused`, shows on every page.

### Import

- A file's password goes exactly as typed, marks included, and may be empty, as our rules did not set it. A memory check and a box keep the rules a new password has.
- An entry is a Login when it keeps a password after the rules, so one whose password was refused goes as Other.
- A file's KDF is refused before any key is made when it would pass the job's memory, and its time is bounded by the host's deadline.
- Custom strings start hidden, as KeePass's protected flag is optional, and first lose only direction marks, so a real Arabic value is never dropped; another hidden mark is refused and counted (for the hands-on check).
- Fields and entries past a cap are dropped and counted, not a failure of the whole file (for the hands-on check).
- A nameless entry is never a duplicate, and a duplicate needs one kind with the same non-empty name and User as search folds them.
- A file needing a key file reads as a wrong password (`FileLocked`), as the worker cannot tell them apart. KDBX 3.1's bad padding reads the same way.

### Settings

- `[vault]` also holds `tab_gap_ms` (150, 20–2000), `paste_wait_ms` (3000, 2500–10000), `box_mb` (16, 1–64), `box_work` (18, 16–20), `vault_ms` (20000, 2000–120000), `worker_idle_s` (30, 5–3600), `restart_ms` (1000, 100–30000), `restart_max_ms` (30000, 1000–600000), `place_check_s` (60, 10–3600) and `later_h` (24, 1–168).
- `[keyboard] fill_late_ms` is 2000, within 500 to 20,000. `[kernel] kept_copies` starts empty.
