# kx-mod-voice

Voice typing (P11, ADR-0042 to ADR-0044 and ADR-0046 to ADR-0052): speak, and the keyboard types the text into the focused app. This crate holds the compiled-in models and services tables, the cloud rules and sealed keys, Automatic's device rule, the scorer that Compare and a card's check count words with, the dictation flow, History, the counts, the sealed store and the `[voice]` settings. It also holds the models folder, the downloads, Compare and the Voice panel's state. The module reads its settings, drops a bad service entry or pick, and starts the actor that wires every part to the ports and the bus.

- **Models table** (`models`, `models.toml`): one row per local model with its file, size, SHA-256, a download address pinned to a commit on an allowed host, its licence and credit, its memory on the processor and on a card, and its languages with each model's own code. A bad table turns local models off; it never panics.
- **Orders** (`models::order`): each tested language has a list, best first, with the day of its tests; its first is the default. The English, French and Arabic orders were ranked on the 2026-11-14 trial numbers (the owner's six clips, the voice detector at pad 300 ms). A language without a list takes the general order of the models that know it, marked untested. A model that never leads (`lead = false`, Nemotron) is first in no list.
- **Picks** (`models::pick`): a pick per language names a row that knows it, or a model file of the folder; anything else goes back to that language's default alone. A cloud pick names a service and its model as `<service>:<model>` (`cloud::PICK_MARK`).
- **Services table** (`services`, `services.toml` for the table-wide values and the upload-form rows, `services-own.toml` for the rows of their own shape, read as one table): one row per cloud service, Groq first: its request shape (the upload form, ElevenLabs, Gemini or AssemblyAI Sync) and form fields (a model may carry its own), its own HTTPS host or a typed address, its key header and test, its limits with room for the WAV header under the service's cap, its models with each language's code (often a named set from `langs.toml`, written once), its terms number, its dated facts and whether it is offered (a hidden row stays built but is never shown, picked or sent to; ADR-0050). The validator refuses a broken row with its rule in plain words; a bad table turns cloud services off and never panics.
- **Cloud rules** (`cloud`): nothing in the table is a default. A service is off until the user confirms its card while `cloud` is on, and only the user's own pick moves a language to it. `cloud` off or a hidden run offers no service, and a declined newer terms number turns the service off; either way its languages go back to their local models. A card's warning shows again under the bar's row while the first dictation at each terms number listens, holding nothing back.
- **Keys** (`keys`): each service's key is sealed alone as a blob named by its id in the shelf's `keys` folder, handed out as a `Secret` and wiped after use. Only Remove key deletes its blob (Off or a declined card keeps it), and a hidden run reads and writes none.
- **Typed lines** (`line`): the key, address and Other model lines take our keyboard's keys while aimed (ADR-0049). The typed key stays in the module, wiped on Save, Paste, Remove, a page turn or the panel closing, and the views see only its dots.
- **Other files** (`models::other`): a model file not in the table gets its memory from its size and its language code from its own facts.
- **Device** (`device`): the processor until a card passes its check; Automatic then takes the passed card with room and the best gain, a picked card serves only when present, checked and with room, and every fallback says why.
- **Score** (`score`, `score.toml`): words apart between two texts, per language. Case, punctuation and the apostrophe's forms are no difference, and digits 0 to 10 count as their number word in either gender. Arabic is strict: only the harakat, the superscript alef and tatweel go (the shared fold table's drops in the Arabic block), Arabic-Indic digits read as digits, and a hamza written apart joins its letter; ه for ة, a lost hamza or ي for ى is a word wrong, as the user would fix it.
- **History** (`history`): the texts dictated, pinned first then newest first, each with its time, language, model and recording; a later text of the run's last take replaces it (Try again), and an unpin counts as new, as in the clipboard's History. Pins sit outside `history_max`, recordings share `voice_max_mb` oldest unpinned first, a take keeps one recording, Delete all needs the `Asked` from `ask_all`, and history off, a hidden run or a password box keeps nothing. Turning a switch off keeps what is stored; Delete all removes it.
- **Counts** (`counts`): per local day, model and device (the processor or a card by its id), the dictations, the time heard and the time waited, never content; a file the table does not name counts as `Other`. Cloud dictations count by service and listed model, with no device; a typed model's id is never kept. Beside them, each card's check by card, driver, file fingerprint and language (ADR-0052), of which a card failing mid-run drops that file's alone, and Compare's last numbers per ISO language.
- **Store** (`store`): `<data>/voice/` on a sealed shelf with its own key: `texts` (a blob per text), `sounds` (recordings in blobs `<id>-<n>`) and `numbers`. A newer file stops the open before any change, damage and a crash's leftovers are removed, and a failed write is tried again with the next change.
- **Folder** (`folder`): the models folder as scanned, each file known by its fingerprint; a file named and sized as a row's serves on the processor while its fingerprint is read. A new folder forgets the old one's files, and a late fingerprint for a file no longer held is dropped. `Stock` is the flow's `Supply`. The first download offers `KeyXtend\models` on the drive with the most room; a path `path_ok` refuses is never used.
- **Downloads** (`fetch`): one at a time, only with room for the file and `min_free_mb`, and never over a file holding the row's name; each fault becomes the bar's note. A download asked while the folder is read waits for the read, so a copy under another name is not fetched again.
- **Compare** (`compare`): the set text is recorded once, then each ready model writes it in a fresh job; only numbers are kept, and a text shows only for a language with no set text. Until its microphone opens the key glows amber with no bar, then while it records the flow shows its Listening bar and red light (`In::Lend`); no dictation starts until it ends.
- **Panel** (`panel`, `view`, `options.toml`): the Voice panel's pages, 5 rows a page by default, ticked then acted on; each click gives the actor its next steps as `Do`s. A row click on rows that moved since is dropped (`told`), and `recheck` checks a picked card, or with Automatic a card checked under an older driver.
- **Actor** (`actor`): on `kx-voice`, it feeds the flow and the panel and carries out their effects in order. The microphone's runs (one call at a time each) and the slow reads go to short job threads, texts to one typer thread, so it never waits on the worker or the keyboard; the stop joins only the actor. Undo insert wants the mouse module's presses while it can act.

## Services and capabilities

| Service | Use |
|---|---|
| `Speech` (`SpeechKey`) | Records from the microphone and turns speech into text in the worker. |
| `ModelFiles` (`ModelFilesKey`) | Scans the models folder, checks a file's fingerprint and downloads a model the user asks for. |
| `SessionWatch` (`SessionWatchKey`) | Stops and wipes a dictation when the session locks or sleeps. |
| `FieldWatch` (`FieldWatchKey`) | Knows the focused box's kind: a password box shows dots, no Copy, and keeps nothing. |
| `ClipboardWriter` (`ClipboardWriterKey`) | Copy on the bar and on a history row. |
| `FolderReveal` (`FolderRevealKey`) | Shows the models folder in the file manager (ADR-0046). |
| `SealedShelf` (`VoiceShelfKey`) | The store's own sealed shelf over `<data>/voice/`, with the cloud keys in its `keys` folder. |
| `Calendar` (`CalendarKey`) | History's times and the counts' days. |
| `CloudSpeech` (`CloudSpeechKey`) | Sends a cloud dictation's pieces at Stop, or tests a key, through the worker's `speech-net` job (ADR-0047). |
| `WebLink` (`WebLinkKey`) | Opens a card's link to the service's own HTTPS page. |

| Capability | Why |
|---|---|
| `Voice` | It gates the voice key, its light, the bar, the text to type and the typed lines' events (ADR-0042 decision 8, ADR-0049). |
| `Microphone` | It gates `SpeechKey`. |
| `Network` | It gates `ModelFilesKey`, for the model downloads the user asks for, and `CloudSpeechKey`, for the services the user switched on. |
| `SecretStore` | It gates `VoiceShelfKey`, which seals `<data>/voice/`: the history, kept recordings, counts, card checks, compare numbers and cloud keys. |
| `ReadFocusedText` | It gates `FieldWatchKey`. |
| `WriteClipboard` | It gates `ClipboardWriterKey`, and `PasteText` and `TextPasted` for Paste at once. |
| `OpenFolder` | It gates `FolderRevealKey`. |
| `OpenLink` | It gates `WebLinkKey`. |
| `ReadClipboard` | It gates `ClipTake` and `ClipTaken`, the key page's Paste through the clipboard history, waiting `take_ms`. |

It provides no service, and it never injects input: the keyboard types each text. With `insert = "paste"` (Paste at once), it hands the text to the clipboard history with `PasteText`; the history checks the clipboard for a private marker as it reads it, then pastes and puts the clipboard back, and any no types the text (ADR-0048).

## Settings (`[voice]`, version 1)

The defaults live only in `defaults.toml`, each with its one-line meaning. The ranges the ports already know come from `kx-module-api` (`SpeechAsk`, `JobLimits`, `FetchAsk`, `CardUse`, and `CloudAsk` for `net_wait_s` and `net_tries`); the module's own ranges are named constants in `src/settings.rs` and `src/settings/cloud.rs`. A bad key goes back to its default with the kernel's settings notice. A bad service entry or pick goes alone: the check takes any, and every read of `[voice]` drops a bad one with `VoiceSettings::drop_bad`, raises the same notice naming `services.<id>` or `picks.<lang>`, and writes back only the key it changed. An entry in `services` keeps the day and terms number the user confirmed, and any typed address, Other model and keep-nothing switch.

| Group | Keys |
|---|---|
| Listening | `on`, `threads`, `record_max_s`, `piece_s`, `pad_ms`, `level_hz`, `nothing_s`, `silence_stop`, `silence_stop_s`, `debounce_ms` |
| Models and device | `models_dir`, `picks`, `device`, `card_margin_mb`, `card_gain`, `card_diff`, `min_free_mb`, `stall_s` |
| Typing and bar | `type_suffix`, `history_join`, `type_gap_ms`, `insert`, `paste_ms`, `take_ms`, `done_ms`, `wave_ms`, `place`, `sounds`, `read_ms`, `copy_ms`, `copy_poll_ms` |
| History | `history`, `history_max`, `keep_voice`, `voice_max_mb` |
| Worker | `job_mb`, `idle_min`, `ask_ms`, `load_s`, `warm_s`, `piece_wait_s`, `restart_ms`, `restart_max_ms` |
| Cloud | `cloud`, `services`, `net_wait_s`, `net_tries`, `facts_days` |

## Public API

`VoiceModule` (with `hidden` for a run whose saves would be hidden), `ID`, `MANIFEST`, `VoiceSettings` (with `drop_bad`, `services_value` and its `SPEC` with the range constants), `ServiceOn`, `Insert`, `INSERTS`, `KEEP_START`, `PLACES` and `place_name`; the `models`, `services`, `cloud`, `keys`, `line`, `device`, `score`, `flow`, `history`, `counts`, `store`, `folder`, `fetch`, `compare`, `panel` and `view` modules for the actor, the ui and tests.

## Tests

`cargo test -p kx-mod-voice`: unit and property tests of each part, the kernel's settings repair in `tests/voice/settings.rs`, `src/actor/tests` drives the actor over every port's fake without a thread, `tests/voice/module.rs` starts and stops the module on the fake host with a bad pick's or service entry's notice and write-back, and `tests/voice/thread.rs` runs it on its threads: the key and the light, the typer, a panic and the feature off. `src/cloud/tests/props.rs` checks that no sequence of settings changes, picks or table loads moves a language from local to a service by itself; the keys' tests make their keys at run time.

## Security

See `docs/security/kx-mod-voice.md`.
