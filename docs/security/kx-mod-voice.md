# Security note: kx-mod-voice

- **Owner:** product owner
- **Last review:** 2026-11-14 (P11 task 16: the models table, the orders, the device rule, the strict scorer and `[voice]` with its per-language pick repair)
- **Status:** Draft
- **Scope:** `crates/kx-mod-voice`: the manifest, the `[voice]` settings, the models table (`models.toml`, `models`), the device rule (`device`) and the scorer (`score.toml`, `score`). The module is still an idle skeleton; the flow, the sealed store, the downloads and the panel come in plan tasks 17 to 20, and each extends this note. It runs inside the uiAccess `keyxtend` process; the speech jobs and downloads run in `keyxtend-worker` (ADR-0042).

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `Voice` | An event gate (ADR-0042 decision 8): the key, its light, the bar's view, its clicks and the text to type. | The module cannot show the bar or hand text to the keyboard. |
| `Microphone` | It gates `SpeechKey`, which records and transcribes in the worker. | The module does not start (`every_service_it_needs_is_gated_by_a_capability_it_holds`). |
| `Network` | It gates `ModelFilesKey`, for the model downloads the user asks for, and nothing else. | The module does not start; no model can be downloaded. |
| `SecretStore` | It seals `<data>/voice/`: the history, kept recordings, counts, card checks and compare numbers (task 18). | Nothing yet; from task 18 the module cannot keep a history. |
| `ReadFocusedText` | It gates `FieldWatchKey`, so a password box shows dots, no Copy, and keeps nothing. | The module does not start. |
| `WriteClipboard` | It gates `ClipboardWriterKey`, for Copy on the bar and a history row. | The module does not start. |

- It has no `InjectInput` (`it_holds_the_plans_capabilities_and_never_injects_input`): the keyboard stays the only typist.
- `SessionWatchKey` is ungated, as it carries no content (ADR-0042 decision 9).

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| The models table: files, sizes, SHA-256, addresses, licences, languages, memory | Public | Compiled in (`models.toml`) | No | As shipped |
| The scoring rules: the Arabic mark span, joined pairs, digit maps, counting words | Public | Compiled in (`score.toml`) | No | As shipped |
| Texts given to the scorer (a set text, a heard text, a card check's two texts) | Sensitive (spoken words) | Memory only: tokens in `Zeroizing`, check texts in `Redacted<Zeroizing<String>>` | No | Wiped when the score returns; never logged or stored |
| A card check's verdict and timings | Public (numbers) | Memory only here; sealed from task 18 | No | Until replaced |
| `[voice]` settings, including the per-language picks | Public | `settings.toml` (only the user's changes) | No | Until the user changes them |

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `[voice]` in `settings.toml` | `deny_unknown_fields`; every number in its range, the shared ones from `kx-module-api` (`SpeechAsk`, `JobLimits`, `FetchAsk`, `CardUse`); `device` through `DevicePick::parse`; `place` from `PLACES`; `models_dir` empty or `path_ok`; `type_suffix` at most 8 characters with no hidden mark. A bad key goes back to its default with the kernel's settings notice. `picks` is read as text per language; each read drops, alone, an entry that is not an ISO code naming a row that knows it or a plain model file name, raises the settings notice naming `picks.<lang>`, and writes the rest back. | — | `tests/voice/settings.rs` (edges 19, 35), `tests/voice/module.rs` (the notice and write-back); unit tests in `src/settings/tests.rs` |
| A pick's or the folder's model file name | A model extension, no `/`, `\` or `:`, no hidden mark, no device name (`RESERVED_NAMES`) as its stem | Its size within `CPU_MB`, capped at its size plus `job_mb` (edge 37) | Unit tests in `src/models/tests/fit.rs` |
| A model file's facts from the worker (`FactsAsk`) | Its language code matched by its part before `-` or `_`, case ignored | — | Unit tests |
| A card check's result (`CheckDone`) | Texts scored before speed; a zero card time reads as the fastest, never a division by zero | — | Unit tests in `src/device/tests.rs` |
| Any text to the scorer | Lower-cased per language; Arabic drops only the harakat, superscript alef and tatweel and joins a hamza written apart, keeping ة, the hamza forms and ى as letters | Two-row edit distance, linear memory | Property test: any text scores without panic, matches itself and is symmetric |

## Threats (STRIDE-lite)
- **Spoofing:** a renamed copy of a model is known by its fingerprint (`Table::printed`), not its name.
- **Tampering:** every address is HTTPS, on a listed host, pinned to a 40-hex commit, and ends in the row's file; every row carries a 64-hex SHA-256 and its size. A table that breaks any rule turns local models off and never panics. The download itself and its fingerprint check come in task 19.
- **Logging / repudiation:** the scorer and the check hold texts in `Zeroizing` or `Redacted`; no text reaches a log or an error.
- **Information disclosure:** the scorer keeps no text, and a check keeps only its verdict and timings.
- **Denial of service:** the tables are read once at first use; a bad settings value resets alone, and a bad pick goes without the other languages' picks. Automatic never picks a card that has not passed its check or lacks room, and says why it uses the processor.
- **Elevation:** a model file name cannot climb out of the folder or name a device; `models_dir` passes `path_ok`.

## Findings
### Must fix (blocks Definition of Done)
- None for this task.
### Should fix (tracked issue)
- None yet.
### Accepted (with reason)
- A pick that is not text, such as a number, fails the read of the whole `picks` table, so the kernel resets it with the notice; a bad text pick, such as a removed row, goes alone.
- The check takes any text pick, so a bad one written to the file is dropped at the module's next read of `[voice]`, not refused at the write.
