# kx-mod-vault

Quick-fill (design `docs/superpowers/specs/2026-11-01-quick-fill-design.md`, ADR-0038): a notebook of saved values of any kind, sealed with the account's own key, that fills a value at the caret in one click. This crate holds the notebook, its panel, the fills, Paste, the backup box, its memory checks, Bring back and the KeePass import. Anyone using the signed-in PC can fill, as in the browsers.

- **Tables** (`kinds`): `kinds.toml` gives each kind its icon token, its filter and its starter fields; `labels.toml` lists the labels in Add a field's order, which start hidden and which feed the `@@` list. Their ids name the Fluent texts (`fill.kind-<id>`, `fill.label-<id>`).
- **Entries** (`entry`): a stable random id, a name, a kind, its fields (a label, a one-line value, a hidden flag), when it changed and when it was last filled, as wall times that never step back. An own label is never blank. Empty fields are not kept. A shown text loses LRM, RLM and ALM; any other hidden mark, a line break among them, or any mark in a hidden value is refused with the `Refused` note. Values are wiped when dropped, and a typed `Line` reserves its room at once.
- **Notebook** (`book`): at most `max_entries` entries of at most `max_fields` fields, saved in one sealed blob; `Full` past either or past the blob's cap.
- **Store** (`store`): the notebook in one versioned blob `notebook` of the sealed store behind `VaultStoreKey`. A damaged blob starts empty with `notice.vault-damaged`; one that opens but does not read is kept in `notebook-aside` first; one from a newer version fails the start, untouched.
- **Search** (`find`): names, labels and shown values, folded as the clipboard's search folds them into wiped buffers; a hidden value is never searched, and a table label matches by the text the ui gives for it.
- **Order** (`order`): the last used first, then A to Z by folded name; the filters by kind; By label's counts and values.
- **Make one** (`make`): `make_chars` characters from `make.toml`'s letters, digits and symbols, each group at least once, every draw without bias.
- **Fill** (`fill`): a click becomes a `FillType` with its own round; a login's Both types user, Tab, password, never Enter. A fill while one waits is refused, never queued; a typed one moves the last use and saves, and the list keeps its order until the panel opens again.
- **Paste** (`paste`): `ClipTake` asks the clipboard history for its text, read within twice `field_chars`. Breaks at its ends go; one inside becomes a space in a shown field and refuses a hidden one, as a password must never change silently, then the field's rules apply. No text, a text too long, a busy clipboard, or no answer within `paste_wait_ms` each shows its own note, and a late answer is dropped.
- **Backup** (`backup`): setup picks a suggested place or a folder from the list, then a new password of at least `box_min_chars`, typed once and shown while typed, makes the padlock. `box_delay_ms` after the last change the notebook is sealed once and written to every place; one gone or full waits, named in the footer, and catches up on a change, when the panel opens and every `place_check_s`. A box past `box_mb` cannot fit until the notebook changes, so its place waits for a change or Back up now, and a restart may try once. Its own blob `backup` keeps the places, the padlock, each box's mark and length, the first offer, the hold and the memory checks; a damaged one starts clean with `notice.vault-backup-damaged`. While the notebook did not read, the hold writes no box until Back up now's Yes or Bring back, even after a restart. Another computer's box in a place being picked asks first, dated from its own file (`Places::modified`), or undated when that fails (`Sheet::ForeignUndated`); its Yes asks that box's password and brings its entries in, and No leaves it. A box is checked only at a pick: after that, two computers on one place each rewrite it, and the last write wins.
- **Memory checks** (`backup::checks`): a banner asks for the backup password each step of `check_days` after a new padlock or the last right answer, the last step repeating; Later, or leaving the check, asks again after `later_h`. Each wrong answer uses one of `check_tries`, and after the last a new password rewrites every box. No banner shows while a fill waits or a text is typed, and a clock set back never stops the checks.
- **Bring back** (`backup::bring`): an empty notebook offers the boxes found in the suggested places, newest first; else Bring back, on the Backup page and always on the Places page, lists them, or the folder list picks a backup file. The box is read within `box_mb` and opened in the worker; a computer with no padlock keeps the box's own, else the box is rewritten under this one. Entries merge by id, and the newer change wins whole. A merge past `max_entries` or the notebook's size brings nothing in and writes nothing, as that box may be the only copy. Bring back ends the hold and writes every place; another computer's box leaves a hold standing.
- **Import** (`import`, `backup::keepass`): Import, always reachable from the empty notebook's invitation, the Backup page and the Places page, and writing no box while held, lists the files of `import.toml`'s extension and asks the file's password, sent exactly as typed and maybe empty; the file is read within `MAX_KEEPASS_BYTES` on the job thread and opened only in the worker. Each value keeps the entry rules. Custom strings start hidden, as a file protects one only when asked to, and first lose LRM, RLM and ALM, so an Arabic value is never lost, while a password with any mark is refused, as it must never change silently. What the rules refuse, an entry with no field, a duplicate (one kind, with the same non-empty name and User as search folds them) and what passes a cap are left out and counted with the worker's drops; anything new saves once and rewrites the boxes.
- **Actor** (`kx-vault` thread): runs the panel, sends fills and takes, saves after each change, shows `SaveFailed` until a failed save works and tries it again every `place_check_s`, raises each memory check due, tells `FillLive` at start, on `FillAsk` and as it stops or panics, and tells `VaultEmails` at start, on `FillAsk` and after each change that alters them.

## Services and capabilities

| Service | Use |
|---|---|
| `SealedStore` (`VaultStoreKey`) | Keeps the notebook and the backup blob sealed under their own key. |
| `Calendar` (`CalendarKey`) | Stamps each change and each fill. |
| `Random` (`RandomKey`) | New ids and Make one. |
| `Boxes` (`BoxesKey`) | Makes the padlock, seals and opens the box, checks a password and reads a KeePass file in the worker, on the module's own job thread, `kx-vault-box`. |
| `Places` (`PlacesKey`) | Lists the backup places and folders, finds and dates boxes, and reads and writes them, on the same job thread. |

| Capability | Why |
|---|---|
| `Vault` | It gates the panel's events (ADR-0038 decision 3). |
| `VaultData` | It gates the notebook store, the boxes and the places; the vault alone holds it (ADR-0039). |
| `ReadClipboard` | It gates `ClipTake` and `ClipTaken`, for Paste. |

It provides no service, and it never injects input: the keyboard types each fill.

## Settings (`[vault]`, version 1)

The defaults live only in `defaults.toml`, each with its one-line meaning; the ranges are named constants in `src/settings.rs`, and the tops of `field_chars`, `max_entries`, `max_fields`, `box_mb` and `box_work` are the vault worker's caps. A field holds a made and a backup password, and `box_config` gives the vault job its waits.

| Group | Keys |
|---|---|
| Notebook | `field_chars`, `max_entries`, `max_fields`, `make_chars` |
| Backup box | `box_delay_ms`, `box_min_chars`, `box_mb`, `box_work`, `place_check_s`, `vault_ms`, `worker_idle_s`, `restart_ms`, `restart_max_ms` |
| Memory checks | `check_days`, `check_tries`, `later_h`, `memory_checks` |
| Fill | `tab_gap_ms`, `paste_wait_ms` |

## Public API

`VaultModule` (with `running`), `ID`, `MANIFEST`, `VaultSettings` and its `SPEC`; the parts above are public for the panel and for tests.

## Tests

`cargo test -p kx-mod-vault`: unit and property tests of each part over `kx-platform-fake`, and `tests/vault` runs the module on the fake host.

## Security

See `docs/security/kx-mod-vault.md`.
