# Security note: kx-mod-voice

- **Owner:** product owner
- **Last review:** 2026-11-15 (P11 task 14 part b: `OpenFolder`, the clips' languages from the speech port and the port's new notes; before it task 19: the models folder, the downloads, Compare and the Voice panel's state, reviewed and fixed; before it task 18: History with its recordings, the counts, the card checks, Compare's numbers and the sealed store; before it task 16: the models table, the orders, the device rule, the strict scorer and `[voice]` with its per-language pick repair)
- **Status:** Draft
- **Scope:** `crates/kx-mod-voice`: the manifest, the `[voice]` settings, the models table (`models.toml`, `models`), the device rule (`device`) and the scorer (`score.toml`, `score`). History, the counts and the store (`history`, `counts`, `store`), the models folder (`folder`), the downloads (`fetch`), Compare (`compare`) and the panel's state (`panel`, `view`, `options.toml`) are pure parts the actor wires in task 20, which extends this note. It runs inside the uiAccess `keyxtend` process; the speech jobs and downloads run in `keyxtend-worker` (ADR-0042).

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `Voice` | An event gate (ADR-0042 decision 8): the key, its light, the bar's view, its clicks and the text to type. | The module cannot show the bar or hand text to the keyboard. |
| `Microphone` | It gates `SpeechKey`, which records and transcribes in the worker. | The module does not start (`every_service_it_needs_is_gated_by_a_capability_it_holds`). |
| `Network` | It gates `ModelFilesKey`, for the model downloads the user asks for, and nothing else. | The module does not start; no model can be downloaded. |
| `SecretStore` | It seals `<data>/voice/`: the history, kept recordings, counts, card checks and compare numbers. | The module cannot keep History, recordings, counts, card checks or Compare's numbers. |
| `ReadFocusedText` | It gates `FieldWatchKey`, so a password box shows dots, no Copy, and keeps nothing. | The module does not start. |
| `WriteClipboard` | It gates `ClipboardWriterKey`, for Copy on the bar and a history row. | The module does not start. |
| `OpenFolder` | It gates `FolderRevealKey`, which shows the models folder in the file manager: a plain folder only, never a file or a link (ADR-0046). The actor calls it in task 20. | The module does not start. |

- It has no `InjectInput` (`it_holds_the_plans_capabilities_and_never_injects_input`): the keyboard stays the only typist.
- `SessionWatchKey` is ungated, as it carries no content (ADR-0042 decision 9).

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| The models table: files, sizes, SHA-256, addresses, licences, languages, memory | Public | Compiled in (`models.toml`) | No | As shipped |
| The scoring rules: the Arabic mark span, joined pairs, digit maps, counting words | Public | Compiled in (`score.toml`) | No | As shipped |
| Texts given to the scorer (a set text, a heard text, a card check's two texts) | Sensitive (spoken words) | Memory only: tokens in `Zeroizing`, check texts in `Redacted<Zeroizing<String>>` | No | Wiped when the score returns; never logged or stored |
| A card check's verdict and day, by card, driver and file fingerprint | Public (ids and numbers) | Sealed `numbers/checks` | Yes (shelf) | One per card and file; that file's alone is dropped when the card fails on it mid-run (`Out::Fell`) |
| History's texts, with time, language and model | Secret (spoken words) | Sealed `texts/<id>`, one blob each; in memory as `Secret` (wiped on drop) | Yes (shelf) | `history_max` unpinned, pins until deleted; never in a hidden run, from a password box or with `history` off |
| Kept recordings, 16 kHz mono sound | Secret | Sealed `sounds/<id>-<n>`, blobs within `MAX_BLOB_BYTES`; in memory as `SecretBytes` (wiped on drop) | Yes (shelf) | Only with `keep_voice` and `history`; one per take, all within `voice_max_mb`, oldest unpinned first. With `history` off, a password box or a hidden run the flow never asks the worker for it |
| Counts per day, model and device | Public (numbers; a card by its engine id; a file the table does not name is `Other`, without its name) | Sealed `numbers/counts` | Yes (shelf) | Oldest days drop past one blob |
| Compare's last numbers per language | Public (ids and numbers, never texts) | Sealed `numbers/scores` | Yes (shelf) | One run per ISO language; a board in another code is not kept, as the store reads back only ISO codes |
| `[voice]` settings, including the per-language picks | Public | `settings.toml` (only the user's changes) | No | Until the user changes them |
| The models folder's file names, sizes, fingerprints and other files' facts | Sensitive (the user's own file names) | Memory only (`Folder`); views carry names as `Redacted` | No | Until the folder changes; read again on each scan |
| Compare's recording and each model's text | Secret (spoken words) | The recording stays in the worker's job; pieces in memory as `Secret` until scored, then wiped | No | Only the numbers are kept (`Act::Save`); a text shows only for a language with no set text and goes at Compare again, Back or Close |

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `[voice]` in `settings.toml` | `deny_unknown_fields`; every number in its range, the shared ones from `kx-module-api` (`SpeechAsk`, `JobLimits`, `FetchAsk`, `CardUse`); `device` through `DevicePick::parse`; `place` from `PLACES`; `models_dir` empty or `path_ok`; `type_suffix` at most 8 characters with no hidden mark. A bad key goes back to its default with the kernel's settings notice. `picks` is read as text per language; each read drops, alone, an entry that is not an ISO code naming a row that knows it or a plain model file name, raises the settings notice naming `picks.<lang>`, and writes the rest back. | — | `tests/voice/settings.rs` (edges 19, 35), `tests/voice/module.rs` (the notice and write-back); unit tests in `src/settings/tests.rs` |
| A pick's or the folder's model file name | A model extension, no `/`, `\` or `:`, no hidden mark, no device name (`RESERVED_NAMES`) as its stem | Its size within `CPU_MB`, capped at its size plus `job_mb` (edge 37) | Unit tests in `src/models/tests/fit.rs` |
| The folder scan and each file's fingerprint (`ModelFiles`) | Names pass `file_ok`; a row's file is the one with its fingerprint; a file with a row's name and size serves only on the processor while its fingerprint is read, and another fingerprint makes it an other file; a file whose fingerprint fails is left out. A fingerprint counts only for a file of this folder by its full path and scanned size, and a new folder forgets the old one's fingerprints. | One read per new name and size; a download waits for the read | `src/folder/tests.rs`, `src/folder/tests/supply.rs`, `src/panel/tests/folder.rs` |
| A folder picked in the folder list or the first-download offer | `path_ok`: an absolute drive path, no `..`, no device path (`\\.\`, `\\?\`), no reserved name and no control character, else refused with the Folder note (edge 38); a folder that cannot be made shows its note and is not used | — | `a_relative_device_or_climbing_folder_is_refused`, `the_folder_list_picks_a_folder_and_refuses_a_climbing_one`, `a_folder_with_a_control_character_is_refused` |
| A download's notes (`FetchNote`) | Only for the live row; another row's note is dropped; Done only after the worker's size and SHA-256 check; each fault maps to one note (`fetch::hint`) | One download at a time; room for the file plus `min_free_mb` before it starts (edge 21) | `src/fetch/tests.rs`, `src/panel/tests/folder.rs` |
| Compare's notes and the panel's clicks | Compare drops a note of another run id; a row click names only a row of the view it saw, and is dropped once those rows moved (`Panel::told`); a button acts only when the view offers it (`acts`) | Compare's recording within `record_max_s` | `src/compare/tests.rs`, `src/panel/tests.rs`, `src/panel/tests/lists.rs` |
| Compare's hold on the microphone (`In::Lend`) | While it records the flow shows the Listening bar and lights the key; the key, Stop and Cancel go to Compare (`Out::Lent`); no dictation starts; a lock, a sleep, Quit or the feature off cancels it | — | `src/flow/tests/lend.rs` |
| A model file's facts from the worker (`FactsAsk`) | Its language code matched by its part before `-` or `_`, case ignored | — | Unit tests |
| A card check's result (`CheckDone`) | Texts scored before speed; a zero card time reads as the fastest, never a division by zero; a check runs only on a file whose fingerprint is read, and a card gone (`NoCard`) keeps no verdict | — | Unit tests in `src/device/tests.rs`, `src/panel/tests/device.rs` |
| Any text to the scorer | Lower-cased per language; Arabic drops only the harakat, superscript alef and tatweel and joins a hamza written apart, keeping ة, the hamza forms and ى as letters | Two-row edit distance, linear memory | Property test: any text scores without panic, matches itself and is symmetric |
| The voice shelf's blobs (`texts`, `sounds`, `numbers`) | A mark and a version first: another mark or an older version is damaged, a newer one stops the open before any change and no store exists to write over it. Every length is checked against the bytes left; a text's id must match its blob's name; dates must exist, card ids and fingerprints parse, a row id follows the table's rule, an ISO code fits. Each check matches a rule the writing side keeps, so nothing the store writes reads back damaged. A damaged blob is removed and counted, a recording with a blob missing loses its blobs, a blob no text claims goes. Writes come before removes, so a crash leaves leftovers, never a lost pin (`a_crash_between_the_writes_and_the_removes_loses_no_pin`). | One blob each, within `MAX_BLOB_BYTES`; counts drop their oldest days to fit; a text past one blob is refused once, never retried | Property tests in `src/store/tests/props.rs`: random bytes, a cut or changed blob, and a shelf of random blobs never panic; every form round-trips |

## Threats (STRIDE-lite)
- **Spoofing:** a renamed copy of a model is known by its fingerprint (`Table::printed`), not its name. A card's verdict attaches only to a fingerprint read from the file, never to a file served by its name and size.
- **Tampering:** every address is HTTPS, on a listed host, pinned to a 40-hex commit, and ends in the row's file; every row carries a 64-hex SHA-256 and its size. A table that breaks any rule turns local models off and never panics. A download asks only for a row's own address, host list, size and SHA-256 (`Row::fetch`), and the worker checks the file before it is used; a wrong size or fingerprint deletes the part and tells the Print note. A download never replaces a file holding the row's name; that file must be deleted first. The shelf seals each blob (ADR-0010); the store refuses another mark, a newer version and a text whose id differs from its name.
- **Logging / repudiation:** the scorer and the check hold texts in `Zeroizing` or `Redacted`; no text reaches a log or an error.
- **Information disclosure:** the scorer keeps no text, and a check keeps only its verdict and timings. A hidden run, a password box and `history` off keep nothing (`Rules::of`; the flow sends no keep from a password box or a hidden run, and never asks for the sound with `history` off, so it stays in the worker); `Debug` of the store, History and the codec shows no text or sound, and nothing is logged. Compare keeps numbers only; its texts are `Secret` and wiped after scoring. The microphone never records unseen: Compare's recording shows the Listening bar and lights the key, as a dictation does.
- **Denial of service:** the tables are read once at first use; a bad settings value resets alone, and a bad pick goes without the other languages' picks. Automatic never picks a card that has not passed its check or lacks room, and says why it uses the processor. The store's limits come from `[voice]` and `MAX_BLOB_BYTES`; a failed write waits for the next change instead of looping.
- **Elevation:** a model file name cannot climb out of the folder or name a device; `models_dir` passes `path_ok`, and so does every folder the folder list opens or the panel makes.

## Findings
### Must fix (blocks Definition of Done)
- None for this task.
### Should fix (tracked issue)
- The platform gives one sealed shelf (`SealedShelfKey`, the clipboard's). The voice store needs its own, over `<data>/voice/` with its own key: a `VoiceShelfKey` in `kx-module-api`, gated `SecretStore`, for task 14 or 20 to add and the manifest to require. A hidden run gets it in memory (ADR-0037), so `Store::open` reads nothing from disk there.
- A failed remove waits in memory for the next change, so a Delete or Delete all whose remove fails comes back after a restart. Task 20 shows a note when `apply` fails and tries the pending changes again before the module stops.
### Accepted (with reason)
- Turning `history` or `keep_voice` off keeps what is already kept, as Delete all is the way to remove it; the panel may say so. The design is silent, and the house's switches (Pause learning, Prediction off) keep their data too.
- An unpin counts as dictated then, as the clipboard's History counts it as copied then, so an unpinned recording's date becomes the unpin's.
- The edge-31 test holds copies of kx-ipc's `SOUND_HZ`, `SAMPLE_BYTES` and `MAX_SOUND_BYTES`, as a module may not reach `kx-ipc`; they go once `kx-module-api` carries them.
- After a crash, an id whose blobs were all removed may be used again; no blob of it is left, so nothing pairs wrongly.
- A pick that is not text, such as a number, fails the read of the whole `picks` table, so the kernel resets it with the notice; a bad text pick, such as a removed row, goes alone.
- The check takes any text pick, so a bad one written to the file is dropped at the module's next read of `[voice]`, not refused at the write.
- A file named and sized as a row's serves on the processor before its fingerprint is read, so dictation works at start without hashing gigabytes first. The file sits in the user's own folder, runs in the capped job, and a wrong fingerprint moves it to the other files once read.
- A name held by another file ticks its row so Delete file is at hand, and shows its own note (`BarHint::Taken`).
- A picked file still being read (`Lack::Checking`) shows its own note (`BarHint::Checking`).
- A first download with no fixed drive shows the Folder note (edge 21); the folder list is empty too, so there is nothing to offer.
- The table's `hosts` follow today's redirect (`us.aws.cdn.hf.co`); a region whose CDN host differs fails with the Net note, never a silent fetch. Check another region before the hosts are final.
