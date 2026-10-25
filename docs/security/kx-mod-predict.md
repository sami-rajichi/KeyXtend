# Security note: kx-mod-predict

- **Owner:** product owner
- **Last review:** 2026-10-25 (P7 Task 12 and its fix round 1: the module, the tracker, the learner, the emails and the pack index)
- **Status:** Draft
- **Scope:** `crates/kx-mod-predict`: the manifest, the `[predict]` settings, the actor on its `kx-predict` thread (`actor.rs`, `actor/`), the lane and menu (`lane.rs`, `menu.rs`), the tracker (`track`), the scorer (`score`), the learner (`learn`), the email chips and their byte form (`email`), folding (`fold`) and the word-pack format (`pack`). It runs inside the uiAccess `keyxtend` process; the pack writer runs only in `cargo xtask words`, and `cargo xtask tidy` (D7) refuses `build` in the app's tree.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `ReadFocusedText` | It gates `FieldWatchKey` (the focused box's kind and the words before the cursor) and the typed-text events: `Typed`, `TypingMode`, `TypeWord`, `Suggested`, `ChipPicked` and `WordsAsk`. | The module does not start (`without_read_focused_text_it_cannot_watch_the_box`), so the lane stays empty. |
| `SecretStore` | It gates `SealedStoreKey`, which keeps the learned words and emails sealed (ADR-0032). | The module does not start (`without_secret_store_it_cannot_reach_the_sealed_store`). |

- It has no `InjectInput` (`manifest.rs`): a pick asks the keyboard with one `TypeWord`, and the keyboard types it (ADR-0033).
- `WordPacksKey` is ungated: the packs are public data. `LayoutChanged`, `LayoutAsk`, `PointerPressed`, `PressesWanted`, `PressesAsk` and `Notice` are ungated too.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| The open word and up to `context_words` words before it (`Tracker`) | Sensitive (typed text) | Memory only, in `Redacted` where it leaves the tracker | No | Until the word ends, a reset, or the stop |
| Text read before the cursor (`FieldRun::before_cursor`) | Sensitive | Memory only: at most `read_chars` characters, cut to the last words by the tracker | No | Dropped after the read |
| Learned words and chains (`Learner`) | Sensitive (what the user writes) | Sealed blob `predict-<iso>` through `SealedStore` (kx-crypto, XChaCha20-Poly1305) | Yes | Dropped after `forget_days` unused, at Forget, or past `max_words` and `max_chains` |
| Known emails (`Emails`) | Sensitive (personal data) | Sealed blob `predict-emails`, one for every language | Yes | The `max_emails` most recent |
| Chips (`Suggested`) and picks (`TypeWord`) | Sensitive | One bus event each, gated by `ReadFocusedText`, texts in `Redacted` | No | Until the next lane |
| `[predict]` settings, including `on` and `learn` the menu writes | Public | `settings.toml` (only the user's changes) | No | Until the user changes them |
| A word pack's seed words, frequencies, capitals and chains | Public (wordfreq, Tatoeba and Wikipedia data) | The shipped `<iso>.kxw` files, read in place from a read-only map | No | As shipped |
| The actor's counts (`STOPPED_LINE`) | Public (counts) | The log, one line when the actor quits | No | As the log |

`Tracker`, `Learner` and `Emails` print counts only in debug output, `Show`, `Plan` and `Pick` hold their texts in `Redacted`, and the actor has no `Debug`.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `Typed`, `TypingMode`, `ChipPicked`, `WordsAsk` (gated) | Typed values only, relayed to the actor's inbox; a pick of an older `round` or of no chip does nothing; a list row past the shown page is refused, and pages are clamped to what the list holds | One message per event; the inbox has no bound (see Accepted) | Menu property test (`menu/tests.rs`); tracker property test (`track/props.rs`); `tests/predict/emails.rs` |
| `LayoutChanged` and `PointerPressed` (ungated) | The language must pass `fits_iso` before it names a blob or opens a pack, and the pack's own code must match it; the layouts module tells it at its start and on `LayoutAsk`, so any start order works; a press only resets the tracker | One switch per event | No; `tests/predict/lang.rs` |
| The box (`FieldChange`, `before_cursor`, `ends_with`) | Only a `Text` box is read or learned from; a read error counts as no context; a pick of a read word sends only when `ends_with` answers `Some(true)` | `read_ms` and `read_chars` from `[predict]` | No; `tests/predict/boxes.rs`, `lane.rs` |
| Sealed learned words (`Learner::from_bytes`) | AEAD first (kx-crypto), then magic, version, a sound language code equal to the one in front, every length against what is left, entries in order and once, nothing after the end | The store caps a blob at `MAX_BLOB_BYTES` | Property test: random and altered bytes never panic (`learn/props.rs`) |
| Sealed emails (`Emails::from_bytes`) | Magic, version, every length against what is left, each entry UTF-8 and an email, nothing after the end; no allocation from the count | Same cap; at most `max_emails` kept | Property tests: any bytes and any change are read or refused (`email/bytes/tests.rs`) |
| Word pack bytes (`Pack::parse`) | Magic, version and every header field in range, the language code one `fits_iso` takes; each section inside the file, known and once; offsets in order and UTF-8 valid; the lexicon's CRC-32C. Each later read checks offsets again, as a mapped file could change | The adapter maps at most `MAX_PACK_BYTES`; `layout::MAX_WORDS` words, spellings of `MAX_SPELLING` bytes | Property tests over truncations and byte changes (`pack/tests/damage.rs`) |
| The pack index (`packs.txt`, through `WordPacks::ships`) | Only whether a missing pack shows `notice.words-missing`; each line must be UTF-8 and pass `fits_iso`, and a missing or damaged index ships nothing, logged by kind | Read once per run, mapped within `MAX_PACK_BYTES` | Property test: any bytes are read or refused (`kx-module-api` `ports/packs/index/tests.rs`) |
| `[predict]` settings | `deny_unknown_fields`; each value within its named range, the ranges tied to the engine's limits by tests; domains checked; the scorer's and learner's own checks | Checked at load: a refused value goes back to its default with the `settings-reset` notice | No; `settings/tests.rs` tries each bound |

## Threats (STRIDE-lite)
- **Spoofing:** only modules holding `ReadFocusedText` can send picks or menu taps, so no other module can make the keyboard type. A forged `LayoutChanged` only switches the language: the old learner saves and the new one loads. A forged `PointerPressed` only ends the word unlearned.
- **Tampering:** a changed sealed blob fails its AEAD and loads as `Damaged`; the module starts empty and shows `notice.words-damaged` once. A blob of another language, or a byte form that does not read, counts as damaged too (`a_bad_byte_form_or_another_languages_file_is_damaged`). A damaged pack reads as missing. A changed index can only add or hide the missing notice; every pack still passes `Pack::parse`.
- **Logging / repudiation:** log lines hold fixed texts, error kinds, a blob kind and counts only, never a word, an email or typed text. `STOPPED_LINE` holds counts only. `no_log_line_shows_a_typed_word_a_learned_word_or_an_email` captures every line of a session and checks it.
- **Information disclosure:**
  - A password box locks the lane: the tracker follows nothing, the box is never read, and nothing is learned (`a_password_box_locks_the_lane_learns_nothing_and_is_never_read`).
  - An `Unknown` box or none still gets chips from what is typed here, but is never read or learned from.
  - Off follows, reads and learns nothing; Pause learns nothing. Both hold even when the settings write fails.
  - Known emails list only after a typed `@@`; one that came from a read never lists them (`an_at_at_that_came_from_a_read_never_lists_emails`).
  - Learned words and emails rest sealed under the OS-wrapped key; a `Locked` store keeps them in memory only and saves nothing that run, with `notice.words-locked` once.
- **Denial of service:**
  - The learner is capped by `max_words` and `max_chains`, and the emails by `max_emails`; `purge` drops entries unused for `forget_days` once a day.
  - Each blob saves at most once per `save_s`, and at stop. A refused save waits `save_s` before it tries again.
  - A pack parses once per language switch, on the actor's thread, off the key and UI paths.
  - A read waits `read_ms` at most and takes `read_chars` at most.
- **Elevation:** the crate forbids `unsafe` (`lib.rs`). It never types: the keyboard checks and types each `TypeWord` (ADR-0033). A panic in the actor empties the lane, lets presses go and shows `notice.module-failed`.

## Findings
### Must fix (blocks Definition of Done)
- None open.
### Should fix (tracked issue)
- None open.
### Accepted (with reason)
- The actor's inbox has no bound, as in the other modules: only first-party modules publish on the in-process bus, and each message is small.
- A learner past `MAX_BLOB_BYTES` fails to save as `Io` and tries again every `save_s`; the defaults stay far below the cap, and the words stay in memory for the run.
- A lexicon changed under the view after its checksum passed could make `fst` panic: the Windows adapter opens packs with read sharing only, and the module's panic is contained.
