# kx-mod-voice

Voice typing (P11, ADR-0042 to ADR-0044): speak, and the keyboard types the text into the focused app. This crate holds the compiled-in models table and its order per language, Automatic's device rule, the scorer that Compare and a card's check count words with, the dictation flow, History, the counts, the sealed store and the `[voice]` settings. It also holds the models folder, the downloads, Compare and the Voice panel's state. The module itself is a skeleton that reads its settings, drops a bad pick, and waits; the actor that wires it all comes in plan task 20.

- **Models table** (`models`, `models.toml`): one row per local model with its file, size, SHA-256, a download address pinned to a commit on an allowed host, its licence and credit, its memory on the processor and on a card, and its languages with each model's own code. A bad table turns local models off; it never panics.
- **Orders** (`models::order`): each tested language has a list, best first, with the day of its tests; its first is the default. The English, French and Arabic orders were ranked on the 2026-11-14 trial numbers (the owner's six clips, the voice detector at pad 300 ms). A language without a list takes the general order of the models that know it, marked untested. A model that never leads (`lead = false`, Nemotron) is first in no list.
- **Picks** (`models::pick`): a pick per language names a row that knows it, or a model file of the folder; anything else goes back to that language's default alone.
- **Other files** (`models::other`): a model file not in the table gets its memory from its size and its language code from its own facts.
- **Device** (`device`): the processor until a card passes its check; Automatic then takes the passed card with room and the best gain, a picked card serves only when present, checked and with room, and every fallback says why.
- **Score** (`score`, `score.toml`): words apart between two texts, per language. Case, punctuation and the apostrophe's forms are no difference, and digits 0 to 10 count as their number word in either gender. Arabic is strict: only the harakat, the superscript alef and tatweel go (the shared fold table's drops in the Arabic block), Arabic-Indic digits read as digits, and a hamza written apart joins its letter; ه for ة, a lost hamza or ي for ى is a word wrong, as the user would fix it.
- **History** (`history`): the texts dictated, pinned first then newest first, each with its time, language, model and recording; a later text of the run's last take replaces it (Try again), and an unpin counts as new, as in the clipboard's History. Pins sit outside `history_max`, recordings share `voice_max_mb` oldest unpinned first, a take keeps one recording, Delete all needs the `Asked` from `ask_all`, and history off, a hidden run or a password box keeps nothing. Turning a switch off keeps what is stored; Delete all removes it.
- **Counts** (`counts`): per local day, model and device (the processor or a card by its id), the dictations, the time heard and the time waited, never content; a file the table does not name counts as `Other`. Beside them, each card's check by card, driver and file fingerprint, of which a card failing mid-run drops that file's alone, and Compare's last numbers per ISO language.
- **Store** (`store`): `<data>/voice/` on a sealed shelf with its own key: `texts` (a blob per text), `sounds` (recordings in blobs `<id>-<n>`) and `numbers`. A newer file stops the open before any change, damage and a crash's leftovers are removed, and a failed write is tried again with the next change.
- **Folder** (`folder`): the models folder as scanned, each file known by its fingerprint; a file named and sized as a row's serves on the processor while its fingerprint is read. A new folder forgets the old one's files, and a late fingerprint for a file no longer held is dropped. `Stock` is the flow's `Supply`. The first download offers `KeyXtend\models` on the drive with the most room; a path `path_ok` refuses is never used.
- **Downloads** (`fetch`): one at a time, only with room for the file and `min_free_mb`, and never over a file holding the row's name; each fault becomes the bar's note. A download asked while the folder is read waits for the read, so a copy under another name is not fetched again.
- **Compare** (`compare`): the set text is recorded once, then each ready model writes it in a fresh job; only numbers are kept, and a text shows only for a language with no set text. While it records, the flow shows its Listening bar and light (`In::Lend`), and no dictation starts until it ends.
- **Panel** (`panel`, `view`, `options.toml`): the Voice panel's pages, 5 rows a page by default, ticked then acted on; each click gives the actor its next steps as `Do`s. A row click on rows that moved since is dropped (`told`), and `recheck` checks a picked card, or with Automatic a card checked under an older driver.

## Services and capabilities

| Service | Use |
|---|---|
| `Speech` (`SpeechKey`) | Records from the microphone and turns speech into text in the worker. |
| `ModelFiles` (`ModelFilesKey`) | Scans the models folder, checks a file's fingerprint and downloads a model the user asks for. |
| `SessionWatch` (`SessionWatchKey`) | Stops and wipes a dictation when the session locks or sleeps. |
| `FieldWatch` (`FieldWatchKey`) | Knows the focused box's kind: a password box shows dots, no Copy, and keeps nothing. |
| `ClipboardWriter` (`ClipboardWriterKey`) | Copy on a history row. |

| Capability | Why |
|---|---|
| `Voice` | It gates the voice key, its light, the bar and the text to type (ADR-0042 decision 8). |
| `Microphone` | It gates `SpeechKey`. |
| `Network` | It gates `ModelFilesKey`, for the model downloads the user asks for, and nothing else. |
| `SecretStore` | It seals `<data>/voice/`: the history, kept recordings, counts, card checks and compare numbers. |
| `ReadFocusedText` | It gates `FieldWatchKey`. |
| `WriteClipboard` | It gates `ClipboardWriterKey`. |

It provides no service, and it never injects input: the keyboard types each text.

## Settings (`[voice]`, version 1)

The defaults live only in `defaults.toml`, each with its one-line meaning. The ranges the ports already know come from `kx-module-api` (`SpeechAsk`, `JobLimits`, `FetchAsk`, `CardUse`); the module's own ranges are named constants in `src/settings.rs`. A bad key goes back to its default with the kernel's settings notice. A bad pick goes alone: the check takes any text pick, and every read of `[voice]` drops a bad one with `VoiceSettings::drop_bad_picks`, raises the same notice naming `picks.<lang>`, and writes the rest back.

| Group | Keys |
|---|---|
| Listening | `on`, `threads`, `record_max_s`, `piece_s`, `pad_ms`, `level_hz`, `nothing_s`, `silence_stop`, `silence_stop_s`, `debounce_ms` |
| Models and device | `models_dir`, `picks`, `device`, `card_margin_mb`, `card_gain`, `card_diff`, `min_free_mb`, `stall_s` |
| Typing and bar | `type_suffix`, `type_gap_ms`, `done_ms`, `wave_ms`, `place`, `sounds` |
| History | `history`, `history_max`, `keep_voice`, `voice_max_mb` |
| Worker | `job_mb`, `idle_min`, `ask_ms`, `load_s`, `warm_s`, `piece_wait_s`, `restart_ms`, `restart_max_ms` |

## Public API

`VoiceModule`, `ID`, `MANIFEST`, `VoiceSettings` (with `drop_bad_picks`) and its `SPEC` with the range constants and `PLACES`; the `models`, `device`, `score`, `flow`, `history`, `counts`, `store`, `folder`, `fetch`, `compare`, `panel` and `view` modules for the actor, the ui and tests.

## Tests

`cargo test -p kx-mod-voice`: unit and property tests of each part, the kernel's settings repair in `tests/voice/settings.rs`, and `tests/voice/module.rs` starts and stops the module on the fake host, with a bad pick's notice and write-back.

## Security

See `docs/security/kx-mod-voice.md`.
