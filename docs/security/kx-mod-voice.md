# Security note: kx-mod-voice

- **Owner:** product owner
- **Last review:** 2026-11-15 (P11 task 24: a refused Copy shows its line on the bar, and a text typed in part says typing stopped; before it task 20: the actor, the module thread, the jobs and the typer that wire every part to the ports and the bus; before it task 14 part b: `OpenFolder`, the clips' languages from the speech port and the port's new notes; before it task 19: the models folder, the downloads, Compare and the Voice panel's state, reviewed and fixed; before it task 18: History with its recordings, the counts, the card checks, Compare's numbers and the sealed store; before it task 16: the models table, the orders, the device rule, the strict scorer and `[voice]` with its per-language pick repair)
- **Status:** Draft
- **Scope:** `crates/kx-mod-voice`: the manifest, the `[voice]` settings, the models table (`models.toml`, `models`), the device rule (`device`), the scorer (`score.toml`, `score`), the flow (`flow`), History, the counts and the store (`history`, `counts`, `store`), the models folder (`folder`), the downloads (`fetch`), Compare (`compare`), the panel's state (`panel`, `view`, `options.toml`), and the module and its actor (`module`, `actor`), which wire them to the ports and the bus. It runs inside the uiAccess `keyxtend` process; the speech jobs and downloads run in `keyxtend-worker` (ADR-0042).

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `Voice` | An event gate (ADR-0042 decision 8): the key, its light, the bar's view, its clicks and the text to type. | The module cannot show the bar or hand text to the keyboard. |
| `Microphone` | It gates `SpeechKey`, which records and transcribes in the worker. | The module does not start (`every_service_it_needs_is_gated_by_a_capability_it_holds`). |
| `Network` | It gates `ModelFilesKey`, for the model downloads the user asks for, and nothing else. | The module does not start; no model can be downloaded. |
| `SecretStore` | It gates `VoiceShelfKey`, the store's own shelf over `<data>/voice/` with its own key: the history, kept recordings, counts, card checks and compare numbers. | The module does not start. |
| `ReadFocusedText` | It gates `FieldWatchKey`, so a password box shows dots, no Copy, and keeps nothing. | The module does not start. |
| `WriteClipboard` | It gates `ClipboardWriterKey`, for Copy on the bar and a history row. | The module does not start. |
| `OpenFolder` | It gates `FolderRevealKey`, which shows the models folder in the file manager: a plain folder only, never a file or a link (ADR-0046). The actor checks the path (`FolderPath::new`) and calls it in a job. | The module does not start. |

- It has no `InjectInput` (`it_holds_the_plans_capabilities_and_never_injects_input`): the keyboard stays the only typist.
- `SessionWatchKey` and `CalendarKey` are ungated, as they carry no content (ADR-0042 decision 9).
- Every service comes through `ModuleCx::service`, so the manifest's gates hold; the events it hears from others (`Typed`, `PointerPressed`, `PressesWanted`, `PressesAsk`, `LayoutChanged`, `LayoutAsk`) need only gates it holds (`the_events_it_hears_from_others_need_only_gates_it_holds`).

## Threads and what each holds
| Thread | Runs | Holds | Waits on |
|---|---|---|---|
| The kernel's, at start and stop | `[voice]`, the services, the watches, the relays; the stop sends Quit and joins the actor | Nothing after the start | The actor only, which waits on no job or text |
| `kx-voice`, the actor | The flow, the panel, the store, Copy, the folder's scan, make and remove, the downloads' start and buttons | The text on the bar and its pieces (`Secret`), History (`Secret`), the facts, the runs between calls | Its inbox, up to the flow's next deadline; Copy up to `copy_ms`; the disk for the scan, the shelf and settings writes |
| `kx-voice-job`, one per job, never joined | A run's open and each call (one at a time per run, in order), the devices, a file's fingerprint (one at a time, each file once while queued or read) and facts, a card's check, the folder reveal | The run while it calls; a kept recording (`SecretBytes`) until it reaches the inbox | The worker, within the port's own waits |
| `kx-voice-type`, the typer, never joined | Publishes each `TextAsk` in order, so one text is in flight; the keyboard types it on this thread. Texts still queued when the actor lets it go are never typed | The text being typed (`Secret`) | The keyboard, for the whole text |

- Every tell (a run's notes, a download's notes, the watches, the bus's relays) only posts to the inbox. A relayed key or click outside our windows carries no text: `Typed` and `PointerPressed` arrive as `Elsewhere`.
- A job's port panic counts as `Internal` and is told as a fault, even when the run panics again as it closes (`a_port_that_panics_in_a_call_tells_a_fault_and_frees_the_run`). A panic on the actor hides the bar, darkens the key, lets presses go and raises `MODULE_FAILED`; its runs close as it unwinds (`a_panic_on_the_actors_thread_hides_the_bar_closes_the_microphone_and_tells_the_user`).
- At the stop the watches go first, the flow closes every run, a failed save (History, counts, checks or scores) is tried once more, downloads pause (their parts stay) and the runs left close as they drop. A run out in a job closes when the job ends; the stop never waits for it or for the typer (`a_text_goes_out_on_the_typers_thread_and_a_held_keyboard_never_holds_the_stop`).
- With `on` off the module asks for no service, starts no thread, shows no bar and lights no key (edge 36, `with_the_feature_off_no_bar_shows_no_key_lights_and_nothing_starts`).
- Logs carry fixed texts and error kinds only, never a text, a recording, a file name or a folder (`a_failed_copy_save_reveal_or_run_logs_no_text_or_folder`); a settings write is logged by `WriteError::kind`.

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
| The bus's events (`VoiceKey`, `VoiceAct`, `VoiceOpen`, `VoiceAsk`, `LayoutChanged`, `TextDone`, `Typed`, `PointerPressed`, `PressesAsk`) | Relays only post to the inbox; a click names the round it saw, the bar's buttons, the pointer and Try again go to the flow and the rest to the panel; a `TextDone` counts only for a round the typer gave, mapped back to the flow's own, and History's texts get none | One text in flight; the flow's waits from `[voice]` | `src/actor/tests/*.rs`, `tests/voice/thread.rs` |
| The watches' notes (`FieldChange`, `SessionNote`) | Only the box's kind and id are read, never its text (`read_chars` 0); a lock or a sleep stops every run | `read_ms` per look | `src/actor/tests/faults.rs` |
| The runs' answers and the store's open | A refused open, a failed call or a job that cannot start becomes the run's fault note; `Newer` or a shelf error keeps nothing this run with `VOICE_LOCKED`, damage tells `VOICE_DAMAGED`; the recording is never fetched while it would not be kept | The port's waits from `JobLimits` | `src/actor/tests/faults.rs`, `src/actor/tests/store.rs` |
| A model file's facts from the worker (`FactsAsk`) | Its language code matched by its part before `-` or `_`, case ignored | — | Unit tests |
| A card check's result (`CheckDone`) | Texts scored before speed; a zero card time reads as the fastest, never a division by zero; a check runs only on a file whose fingerprint is read, and a card gone (`NoCard`) keeps no verdict | — | Unit tests in `src/device/tests.rs`, `src/panel/tests/device.rs` |
| Any text to the scorer | Lower-cased per language; Arabic drops only the harakat, superscript alef and tatweel and joins a hamza written apart, keeping ة, the hamza forms and ى as letters | Two-row edit distance, linear memory | Property test: any text scores without panic, matches itself and is symmetric |
| The voice shelf's blobs (`texts`, `sounds`, `numbers`) | A mark and a version first: another mark or an older version is damaged, a newer one stops the open before any change and no store exists to write over it. Every length is checked against the bytes left; a text's id must match its blob's name; dates must exist, card ids and fingerprints parse, a row id follows the table's rule, an ISO code fits. Each check matches a rule the writing side keeps, so nothing the store writes reads back damaged. A damaged blob is removed and counted, a recording with a blob missing loses its blobs, a blob no text claims goes. Writes come before removes, so a crash leaves leftovers, never a lost pin (`a_crash_between_the_writes_and_the_removes_loses_no_pin`). | One blob each, within `MAX_BLOB_BYTES`; counts drop their oldest days to fit; a text past one blob is refused once, never retried | Property tests in `src/store/tests/props.rs`: random bytes, a cut or changed blob, and a shelf of random blobs never panic; every form round-trips |

## Threats (STRIDE-lite)
- **Spoofing:** a renamed copy of a model is known by its fingerprint (`Table::printed`), not its name. A card's verdict attaches only to a fingerprint read from the file, never to a file served by its name and size.
- **Tampering:** every address is HTTPS, on a listed host, pinned to a 40-hex commit, and ends in the row's file; every row carries a 64-hex SHA-256 and its size. A table that breaks any rule turns local models off and never panics. A download asks only for a row's own address, host list, size and SHA-256 (`Row::fetch`), and the worker checks the file before it is used; a wrong size or fingerprint deletes the part and tells the Print note. A download never replaces a file holding the row's name; that file must be deleted first. The shelf seals each blob (ADR-0010); the store refuses another mark, a newer version and a text whose id differs from its name.
- **Logging / repudiation:** the scorer and the check hold texts in `Zeroizing` or `Redacted`; no text reaches a log or an error.
- **Information disclosure:** the scorer keeps no text, and a check keeps only its verdict and timings. A hidden run, a password box and `history` off keep nothing (`Rules::of`; the flow sends no keep from a password box or a hidden run, and never asks for the sound with `history` off, so it stays in the worker); `Debug` of the store, History and the codec shows no text or sound, and nothing is logged. Compare keeps numbers only; its texts are `Secret` and wiped after scoring. The microphone never records unseen: Compare's recording shows the Listening bar and lights the key, as a dictation does.
- **Denial of service:** the actor never waits on the worker or the keyboard: runs, reads and checks run in jobs, texts on the typer, and the stop joins only the actor. A run takes one call at a time, in order, so a slow worker queues calls instead of racing them. The tables are read once at first use; a bad settings value resets alone, and a bad pick goes without the other languages' picks. Automatic never picks a card that has not passed its check or lacks room, and says why it uses the processor. The store's limits come from `[voice]` and `MAX_BLOB_BYTES`; a failed write waits for the next change instead of looping.
- **Elevation:** a model file name cannot climb out of the folder or name a device; `models_dir` passes `path_ok`, and so does every folder the folder list opens or the panel makes. The folder reveal gets only a checked `FolderPath`, and the file manager runs as the user's own program (ADR-0046).

## Findings
### Must fix (blocks Definition of Done)
- None for this task.
### Should fix (tracked issue)
- Task 22 serves `VoiceShelfKey` over `<data>/voice/` with its own key, in memory in a hidden run (ADR-0037), and builds the module with `VoiceModule::hidden()` there.
### Fixed in task 20
- The manifest requires `VoiceShelfKey`, never the clipboard's `SealedShelfKey` (`it_keeps_its_store_on_its_own_shelf_and_dates_by_the_calendar`).
- A failed save shows the `VOICE_UNSAVED` notice once, and the stop tries the pending changes again (`a_failed_save_tells_once_and_is_tried_again_at_the_stop`).
- The edge-31 test takes `SOUND_HZ`, `SAMPLE_BYTES` and `MAX_SOUND_BYTES` from the speech port.
- The typer drops texts still queued at the stop, so an old typer never types beside a new one (`texts_still_queued_when_the_actor_stops_are_never_typed`).
- A file queued or being fingerprinted is not queued again by the next scan, so a large model is hashed once (`a_file_is_fingerprinted_once_however_often_the_folder_is_read`).
- A failed counts, checks or scores save is tried again at the stop (`a_count_that_failed_to_save_is_saved_again_at_the_stop`).
- Copy wipes its own copy of the text once the clipboard has it.
- A failed Copy is logged by kind and shows its line on the bar until a Copy goes through (`a_copy_the_clipboard_refuses_shows_its_line_on_the_bar`); History's Copy shows the fault band.
- A run that panics in a call and again as it drops still answers its lane, so the flow is never stuck waiting.
### Accepted (with reason)
- Job threads are never joined and may outlive the stop, as in the other modules; each ends within the port's own waits, and its run closes then.
- At the stop or a panic, a run busy in a job closes only when its call returns, so an opening microphone can stay on up to `ask_ms` after the light goes off.
- A panic on the actor does not retry failed saves; the stop does.
- The stop's join of the actor has no time limit, as in selection and clipboard; the actor waits only on bounded disk and clipboard work.
- With no keyboard, the typer's round map keeps a few bytes per text until the stop.
- `VOICE_DAMAGED` names texts even when a numbers blob was the damaged one; that blob is removed, which loses numbers only, never a text.
- No keyboard means no `TextDone`: the bar keeps the text with Copy and Undo stays dim, as the keyboard is part of every start. The typer never synthesizes an answer, since the keyboard may send a real one later from its pieces' thread.
- The devices are read at the start only when a card could serve (a card picked, or Automatic with a stored check), so a plain start never starts the worker; until then every model counts as fitting, and the job's memory cap still holds.
- Turning `history` or `keep_voice` off keeps what is already kept, as Delete all is the way to remove it; the panel may say so. The design is silent, and the house's switches (Pause learning, Prediction off) keep their data too.
- An unpin counts as dictated then, as the clipboard's History counts it as copied then, so an unpinned recording's date becomes the unpin's.
- After a crash, an id whose blobs were all removed may be used again; no blob of it is left, so nothing pairs wrongly.
- A pick that is not text, such as a number, fails the read of the whole `picks` table, so the kernel resets it with the notice; a bad text pick, such as a removed row, goes alone.
- The check takes any text pick, so a bad one written to the file is dropped at the module's next read of `[voice]`, not refused at the write.
- A file named and sized as a row's serves on the processor before its fingerprint is read, so dictation works at start without hashing gigabytes first. The file sits in the user's own folder, runs in the capped job, and a wrong fingerprint moves it to the other files once read.
- A name held by another file ticks its row so Delete file is at hand, and shows its own note (`BarHint::Taken`).
- A picked file still being read (`Lack::Checking`) shows its own note (`BarHint::Checking`).
- A first download with no fixed drive shows the Folder note (edge 21); the folder list is empty too, so there is nothing to offer.
- The table's `hosts` follow today's redirect (`us.aws.cdn.hf.co`); a region whose CDN host differs fails with the Net note, never a silent fetch. Check another region before the hosts are final.
