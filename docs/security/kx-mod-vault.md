# Security note: kx-mod-vault

- **Owner:** product owner
- **Last review:** 2026-11-06 (P9 Task 27: the search line's text reaches prediction's memory while aimed at); 2026-11-05 (P9 `kx-security-check`: the coarse gate, plain copies, a fill's wrong-window gaps, the box's strength, hostile files and leaks; P9 Task 23: the capabilities, data, threats and residuals as built)
- **Status:** Reviewed
- **Scope:** `crates/kx-mod-vault`: the manifest, the settings, the tables, the entry rules, the notebook and its store, search, order and Make one, the panel, the fills, Paste, the backup (setup, writes, memory checks, Bring back, a box from another computer) and the KeePass import. It runs inside the uiAccess `keyxtend` process and reaches the OS only through its ports (ADR-0038, ADR-0039).

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `Vault` | It gates the events only: `FillLive`, `FillPanel`, `FillKey`, `FillLine`, `FillView`, `FillAct`, `FillNames`, `FillType`, `FillTyped` and `VaultEmails`. The keyboard, prediction and the ui hold it too, as an event gate (ADR-0039). | The module does not start, so Quick-fill stays closed. |
| `VaultData` | It gates `VaultStoreKey`, the notebook's own sealed store under `<data>/quickfill/` with its own key, and `BoxesKey` and `PlacesKey`, the backup's boxes and places. The vault alone holds it (ADR-0039, `manifest.rs`). | The module does not start, so Quick-fill stays closed. |
| `ReadClipboard` | It gates Paste's `ClipTake` and `ClipTaken`. The grant also opens the other gated clipboard events and the clipboard services, which it does not use; `manifest.rs` tests pin what it holds. | The module does not start, as it subscribes to `ClipTaken`. |

- It provides no service. The `Calendar` and `Random` ports it takes need no capability, and it holds no `InjectInput` or `WriteClipboard`: the keyboard types each fill.
- `FillOpen`, `FillClose`, `FillAsk`, `Notice` and `NoticeAnswered` carry no value and need no capability.
- `ships/vault.rs` in the app's life tests pins each holder and that the three services reach the vault alone.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Saved values (`Field::value`) | Secret | Memory as `Secret`, wiped when dropped; the sealed `notebook` blob | Yes, sealed by the platform (ADR-0010) | Until the user deletes the entry |
| Entry names and own labels | Sensitive | Memory as `Redacted<String>`; the sealed blob | Yes | Until the user deletes or edits them |
| The saved notebook's bytes | Secret | Built in one `Zeroizing` buffer sized first, so it never grows; read bytes are wrapped at once | Sealed at rest | Wiped when dropped |
| A notebook that opened but did not read | Secret | The sealed `notebook-aside` blob, the newest copy winning | Yes | Until the next such copy |
| A made password | Secret | Memory as `Secret` | No | Until the page drops it |
| A typed line: the search, a New or Edit line, a typed password (`Line`) | Secret while it holds a value or password | Memory, wiped when dropped, its room reserved at once so it never moves | No | Until the page leaves |
| A value on a page (`FieldValue`), a typed password (`PasswordSheet`) | Secret | One `FillView`, as `Secret`; a hidden value with text is carried only while shown, and an empty one has nothing to mask. New and Edit also carry a masked value's count of characters, as `Redacted`, for one dot per letter (owner) | No | Until the next view; the ui's copies are in `kx-ui.md` |
| The search and an Edit name on a page, a panel key's letters | Sensitive, at times Secret | One `FillView` or `FillKey`, as plain `Redacted<String>` (see Accepted) | No | Until the next view or key |
| A line's caret | Sensitive: it tells how long a typed secret is, and on a mask no more, as one press moves one dot | The `Line` (redacted in its `Debug`), one `FillView` and one click as `Redacted<u32>`, and the view's JSON and QML's click number, which the hub never logs | No | Until the next view or click |
| The search line as told (`FillLine`, ADR-0040) | Sensitive | One event to modules granted `Vault`, its text up to the caret, as plain `Redacted<String>` (see Accepted); the actor keeps the last told. While it is aimed at, the keyboard passes its text to prediction's memory (`AimedLine`), for word chips, and it is never stored, learned or logged there (`kx-mod-predict.md`). A form's or a password's line says only `Private`, with no text | No | Until the search changes or the panel closes |
| A fill's parts (`FillType`) and the emails (`VaultEmails`) | Secret | One bus event, as `Secret`, heard by `Vault` holders | No | Until each holder drops them |
| The backup password | Secret | Never stored: from its `Line` a `Secret` goes to one job, wiped as the call returns or as a stop drops the job unrun | No | One call |
| The backup blob: places by kind, name and path, the padlock, each box's stamp, mark and length, the first offer, the hold and the memory checks | Sensitive (paths hold the account name) | The sealed `backup` blob; a blob that did not read goes to `backup-aside` | Yes | Until the next save |
| The padlock's recipient and locked identity | Sensitive: the identity opens only with the password | The sealed `backup` blob, and every box | Yes; the identity also by scrypt | Until a new padlock |
| A box's sealed bytes and a KeePass file's bytes | Sensitive: already locked | Memory as `Redacted<Vec<u8>>`, on the job thread, then the worker's pipe; the box also in each place | `age` or KDBX | One job; each box until rewritten |
| A box's notebook part | Secret | `SecretBytes`, wiped when dropped, then entries | Sealed at rest | One job |
| KeePass entries | Secret | `Secret` values and `Redacted` labels from the worker; each passes the entry rules or is left out | No | Until saved or dropped |
| A box's mark (a 64-bit FNV-1a of its sealed bytes) | Public: it guards no secret | The sealed `backup` blob | Yes | Until the next write there |

- Logs carry only fixed texts and error kinds (`store.rs`, `module.rs`, `backup/state.rs`), never a value, a name, a label, a password or a path; a path stays `Redacted` and a page names a place by kind and short name alone.
- Search folds the query, names, own labels and shown values into wiped buffers sized first (`fold_any_into`); hidden values are never folded. The list's A to Z order still folds names and own labels with the shared `fold_any`, whose copies are not wiped.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| The sealed `notebook` blob, and a box's notebook part read the same way | Mark, version, every count and length checked before a slice; known kinds and labels only; every text one line with no hidden mark; ids unique; nothing after the last entry (`store/codec.rs`). A newer version is refused, untouched | The worker's caps: 10,000 entries, 100 fields, 4,000 characters a text | Property tests: random bytes, cut and changed notebooks (`store/props.rs`) |
| The sealed `backup` blob | Its mark and version, what is left checked before each slice, so random bytes give `Bad`; a damaged one starts clean with `notice.vault-backup-damaged` (`backup/codec.rs`) | `MAX_BLOB_BYTES` | Property tests (`backup/props.rs`) |
| Typed and pasted texts (`Draft`, `Line`) | A shown text loses LRM, RLM and ALM, which real Arabic copies carry; any other hidden mark, or any mark in a hidden value, is refused. Paste trims breaks at the ends, and an inner one is a space in a shown field and refuses a hidden one | `field_chars`, `max_fields`, `max_entries` and the blob's 16 MiB | Property tests (`entry/props.rs`) |
| `ClipTaken`, gated by `ReadClipboard` | Only the take waited for counts: another round or a late answer is dropped. Too long is `Refused`, busy is `ClipBusy`, and no answer is `ClipOff` | Twice `field_chars`; `paste_wait_ms` | Unit tests (`paste/`) |
| `FillAct` and `FillKey` from the ui and the keyboard | A row, field or fill click names its view's round, and an older round is dropped so nothing fills unseen; each place is checked against the page | One act per click | Property tests of the panel (`panel/props.rs`) |
| `FillTyped` | Only the waiting fill's round counts; a fill while one waits is refused, never queued | One answer per fill | Actor tests |
| `FillNames` from the ui | Only matched against table labels for search and shown nowhere | The labels table | No |
| `NoticeAnswered`, ungated | Read only for the offer and the memory check, and it carries no data (see Threats) | One per notice | Actor tests |
| What the worker gives | The host checks each frame and reply (`kx-platform.md`); a part passes the store's reader and an entry the rules below | `MAX_BOX_BYTES`, `box_mb`; `vault_ms` per call | See `keyxtend-worker.md` |
| KeePass entries | The text rules per field: custom strings start hidden and first lose LRM, RLM and ALM; a password with any mark is refused. Over-cap entries and fields are left out and counted, so one oversize value does not fail the whole import. A duplicate is one kind with the same non-empty name and User as search folds them | `MAX_KEEPASS_BYTES` before the file reaches the worker; `max_entries`, `max_fields` | Property tests (`import/props.rs`) |
| Places, folder lists and picked files | A pick is checked with `Places::state` before any password, and the data folder, above or below it, is refused (`PlaceState::Refused`). A path that is not UTF-8 is `ReadOnly`, as the blob keeps text. A box is read within `box_mb` | One job at a time on `kx-vault-box`; lists capped by the adapter's table | Unit tests over the fake places |
| The `[vault]` settings | Each range, the check days rising, and three pairs: `make_chars` and `box_min_chars` at most `field_chars`, `restart_ms` at most `restart_max_ms` (`settings.rs`) | The tops are the vault worker's caps | No |

## Threats (STRIDE-lite)
- **Spoofing:**
  - A module without `Vault` can neither hear a value nor ask a fill, as the bus refuses it; only the vault holds `VaultData`, so no other module reaches the notebook, the boxes or the places (`only_the_vault_holds_vault_data`, `the_boxes_and_the_places_reach_the_vault_alone`).
  - A forged `FillOpen`, `FillClose` or `FillAsk` only opens or closes the panel, or makes the module say its state again to gated listeners.
  - A forged `NoticeAnswered` can only open setup or a memory check, or put a check off until `later_h`, as the answer carries no value and no password.
  - A box from another computer proves nothing about who wrote it: its entries come in only after its own password opens it.
- **Tampering:**
  - A changed notebook or backup blob fails its tag (`kx-crypto.md`) or its reader, and is set aside with a notice. A notebook that did not read holds every box until Bring back or Replace's Yes (`actor/tests/backup/held.rs`).
  - The whole box is one `age` stream, so any part cut, dropped, moved or added fails; the worker reads it (`keyxtend-worker.md`).
  - A box this computer did not write, in a picked place, is asked about before any write, and No leaves it alone (`a_box_another_computer_wrote_is_never_written_over_and_its_question_shows`).
  - A merge past `max_entries` or the notebook's size brings nothing in and writes nothing, so a box that may be the only copy is never cut (`bring_back_past_the_cap_changes_nothing_and_writes_nothing`).
  - A write goes to its own temp file and replaces the box in one step, so a failed one leaves the old box whole.
- **Logging / repudiation:** only fixed texts and error kinds are logged, tested at each risk: `a_fill_and_its_answer_log_no_value`, `a_wrong_password_tries_again_and_logs_neither_password_nor_place`, `a_wrong_password_tries_again_and_logs_nothing_of_the_file`, `a_failed_padlock_says_so_and_logs_no_password_or_path`, `the_checks_password_never_shows_in_debug_or_a_log`.
- **Information disclosure:**
  - A hidden value is searched by no one and reaches a page only while shown, and it hides again as the page closes. New and Edit show how long it is, one dot per letter (owner), to anyone who sees the screen; read pages and row hints keep the fixed mask, which tells no length.
  - No value reaches the clipboard or prediction's `Typed`: the keyboard tells one `Break` per fill (`kx-mod-keyboard.md`). Prediction never stores a saved email (`kx-mod-predict.md`).
  - No line but the search brings prediction any text: a name, label, value or password line says only `PanelLine::Private`, which has no room for text, on every Edit caret and the password page (`every_caret_of_a_form_is_private_an_own_label_and_a_hidden_value_too`, `the_backup_pages_show_a_private_line_only_on_a_password`). What the user types in the search goes to prediction's memory while aimed at; the value of a row it finds never does.
  - Files copied to another PC or account open nothing, as the key is wrapped by the OS (ADR-0010).
  - A hidden run never opens the notebook: the module stays off (`a_hidden_run_keeps_quick_fill_off_with_its_note_and_writes_nothing`).
  - The box names only itself; opening it offline needs the password against scrypt at `box_work`, at least `box_min_chars` characters.
  - No value reaches a log, the clipboard or an unsealed file: the panic hook logs only a place (`apps/keyxtend/src/quiet.rs:16-19`), the module holds no `WriteClipboard`, a temp file holds sealed bytes, and the worker writes nothing. A crash dump is the one gap (`keyxtend.md`).
- **Denial of service:**
  - A crafted box or KeePass file hurts neither side. Each is capped before the worker (`crates/kx-platform-windows/src/places/file.rs:22-39`), each frame before its payload (`crates/kx-ipc/src/frame.rs:70`), and age's work and the KDF's cost before any key (`keyxtend-worker.md`). A panic or a dead worker answers `Failed`, and what comes back passes the store's caps.
  - Every box call blocks up to `vault_ms`, so jobs run one at a time on the `kx-vault-box` thread, never on the bus's or the actor's; a stop lets the running job end, runs no queued one, and wipes their passwords.
  - A panic in a job answers it as failed. A panic in the actor tells `FillLive::OFF`, so the fill key dims.
  - A place gone, full or failing waits and catches up; a wait never blocks a fill, and no memory check shows while a fill waits or a text is typed.
  - A memory check never blocks typing, and a clock set back never stops the checks.
- **Elevation:** the crate forbids `unsafe`, parses neither format and types nothing. A crafted box or KeePass file meets only the worker, which runs with lowered rights (ADR-0004, `keyxtend-worker.md`).

### By data class (P9 check)
Paths are under `crates/kx-mod-vault/src` unless they name another root.

| Data | Spoofing | Tampering | Logs | Disclosure | Denial | Elevation |
|---|---|---|---|---|---|---|
| Saved values (Secret) | Only `Vault` holders hear one (`crates/kx-module-api/src/ports/fill.rs:29`), and only the vault holds the store (`manifest.rs:26-28`) | The tag, then the reader's caps (`store/codec.rs:90-104`) | Never; a panic logs a place only | A hidden one reaches a page only while shown (`view.rs:257-264`). Copies: wiped in `crates/kx-mod-keyboard/src/run/fill.rs:151-156`, not in `crates/kx-platform-windows/src/strokes.rs:97-102` or `crates/kx-ui/src/hub/fill/view.rs:180-187` | One fill at a time, refused while one waits | The crate forbids `unsafe` (`lib.rs:5`) and holds no `InjectInput` (`manifest.rs:74`) |
| Backup password (Secret) | Goes only to the vault's own `BoxesKey` | A hidden mark is refused (`backup/setup.rs:208-219`); a wrong one opens nothing | Never (`a_failed_padlock_says_so_and_logs_no_password_or_path`) | `Line` and `Secret` are wiped, the ui's copies are not (`crates/kx-ui/src/hub/fill/view/backup.rs:188`), and the worker's stdin keeps the last frame | One job at a time; a panic answers failed (`backup/jobs/thread.rs:53-63`) | Only the worker sees it, with lowered rights |
| KeePass password (Secret) | Sent only with the file the user picked, exactly as typed, marks included and maybe empty, within `MAX_PASSWORD_BYTES` | The file is refused before the key: size, version, KDF (`apps/keyxtend-worker/src/vault/keepass.rs:45-54`) | Never (`a_wrong_password_tries_again_and_logs_nothing_of_the_file`) | As the backup password; never stored | KDF capped first (`apps/keyxtend-worker/src/vault/keepass/kdf.rs:82-91`); `vault_ms` kills a slow one | As the backup password |
| Entry names, own labels (Sensitive) | Only the ui listens to a page (`crates/kx-ui/src/hub/fill.rs:44-54`) | One line, no hidden mark (`entry/text.rs:33-37`) | `Redacted`, never logged | Plain in the ui's JSON, on screen anyway; the A to Z fold is not wiped | `field_chars`, `max_entries` | QML reads plain text only (`crates/kx-ui/tests/texts/markup.rs`) |
| Places (Sensitive) | Only the vault holds `PlacesKey`; a link is no pick (`crates/kx-platform-windows/src/places/list.rs:63-70`) | A box from elsewhere is asked about first; temp then rename (`crates/kx-platform-windows/src/places/file.rs:49-69`) | `Redacted`, never logged | A page names a kind and short name (`backup/view.rs:120-129`); the data folder is refused (`crates/kx-platform-windows/src/places/paths.rs:15-18`) | Capped reads; Quit may wait (see Accepted) | Plain file calls, no shell |

## Findings
### Must fix (blocks Definition of Done)
- None open.
- Done (P9 review): a failed notebook or backup blob save was only logged while the panel showed the change as saved, so a new password could vanish at a restart with no sign. `SaveFailed` now shows until a save works, and each is tried again every `place_check_s` (`the_note_stays_over_other_clicks_until_a_save_works`, `a_failed_blob_save_says_so_until_the_next_change_saves_it`, `a_failed_save_is_tried_again_every_place_check_s_and_logged_once`).
### Should fix (tracked issue)
- **A weak backup password:** the only rule is the count (`backup/setup.rs:212`), so `12345678` passes, and a box in a cloud folder has only scrypt and the password against offline guessing. Refuse one repeated character, a plain sequence and a short shipped list of common passwords, the list after `kx-licence-check` (`docs/items/P13.md`).
- Done (P9 security check): only the ui had a source test, so the coarse gate's reason below held untested. Prediction and the keyboard now have one each, and the ui's list names `FillKey` (`prediction_names_only_the_vaults_emails`, `the_keyboard_never_names_the_panels_view_its_clicks_or_the_emails`).
### Accepted (with reason)
- Anyone using the signed-in PC can fill, as in the browsers (ADR-0038 decision 2).
- `PanelKey::Text`, `PanelKey::Word`, `FillPage.search`, `FillLine`'s and `AimedLine`'s search and an Edit page's name travel as plain `Redacted<String>`, not wiped, as the port's shape makes them. The panel's own `Line` is wiped; a letter clicked into a password field reaches the panel through `PanelKey::Text` too, one or a few characters at a time.
- Shown values and the typed password are copied into JSON for QML, which cannot be wiped (`kx-ui.md`). The owner wants the password shown while it is typed.
- The backup blob's paths and the box's short name are Sensitive, sealed at rest, and unwiped in memory while the module runs, as they are no secret.
- The box's bytes and a KeePass file's bytes are unwiped copies, as they are locked already; the worker's own residuals are in `keyxtend-worker.md`.
- The Paste take: the history forgets its newest entry before the vault accepts the text, so a refused paste leaves no history row; when the copy's note comes after the take, that text waits in the clipboard actor as a wiped `Secret` and the next copy keeps nothing if it is the same. The system clipboard still holds the text until the next copy, and Windows' own history is the user's to manage.
- `ClipTaken` is heard by every `ReadClipboard` holder (the clipboard, the keyboard, the ui and the vault); only the vault subscribes. This is policy among first-party modules (ADR-0005).
- **The coarse gate:** the bus checks the event alone (`crates/kx-module-api/src/event.rs:117-122,147-159`), so the keyboard, prediction and the ui could each hear `FillType`, `FillKey`, `FillView` and `VaultEmails`, or send a `FillType`. Today the keyboard alone hears `FillType` (`crates/kx-mod-keyboard/src/face/bus.rs:168`) and prediction alone `VaultEmails` (`crates/kx-mod-predict/src/lib.rs:88`).
  - Not a Must fix: modules are compiled in with no plugins, and reading a value takes code that names the event. The keyboard must hear a fill, and the ui already holds shown values and can type through the face, so only prediction's reach truly widens. A gate per direction needs a new ADR (ADR-0038 decision 3).
- **Offline guessing:** a copied box meets only scrypt at `box_work` (default 2^18, 256 MiB; at most 2^20) and the password. Estimated, not measured: at most about 1,000 guesses a second on a top GPU, so a random 8 of 95 symbols takes about 200,000 years and a top-10-million password about 3 hours. The 8-character floor is the owner's choice (ADR-0038).
- **The box format:** `KXBOX`, a version, a 16-bit length, the scrypt-locked identity, then one `age` stream to the X25519 recipient, uncompressed. `age` does not prove who wrote it, but the recipient stays sealed, so a reader of a place cannot forge a box that opens; an old box put back still opens, with its date shown.
- **Quit waits for the running job:** `backup/jobs/thread.rs:77-81` and `module.rs:104` join with no timeout, and the ui ends only after them (`apps/keyxtend/src/drain.rs:52-53`). A worker call ends within `vault_ms`, and a file call on a dead drive when the OS gives up; no value is at risk.
- The kept hold: while it stands, no box is written and places show waiting, so new entries have no backup until Bring back or Replace's Yes. The note shows on every page, and an offline box may be the only copy of what the notebook lost.
- A damaged backup blob starts clean, so the hold, the padlock and the places go with it. No box is written or deleted then, and the user sets the backup up again with the notice.
- While held, Bring back asks Replace's question before it overwrites another place (owner, `an_older_box_brought_back_asks_before_it_touches_a_newer_one`); its Yes can still replace a newer box there. With no hold, the notebook keeps its own entries, so a rewrite loses only what another computer wrote, as below. A copy with the box's own file beside it keeps no place, so that file is never replaced unasked (`actor/tests/backup/beside.rs`).
- Two computers on one cloud box: each rewrites it from its own notebook and the last write wins (`when_two_computers_write_one_place_the_last_write_wins_and_nothing_asks`). A box is asked about only at a pick, as a check at every write would stop a working backup.
- A place unreachable at a new password keeps the old password's box until it is rewritten (design), and a cloud folder's old versions keep theirs for as long as the provider does.
- A wrong memory check spends a try but opens nothing; `check_tries` limits the checks, not an offline attack, which only the password and scrypt limit.
- A fill checks no box kind (owner), so a value goes wherever the caret is; the keyboard's guard after a Tab and the gaps it leaves are in `kx-mod-keyboard.md`.
- A program running as this user can unwrap the data key, so it can read and forge the sealed blobs (`kx-platform-windows.md`); the decoders treat each blob as untrusted.
