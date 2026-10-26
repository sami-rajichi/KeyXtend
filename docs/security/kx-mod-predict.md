# Security note: kx-mod-predict

- **Owner:** product owner
- **Last review:** 2026-10-26 (P7 security review and its fix pass: the pick check, the box kind, the set-aside rule and the decoders; B10: the minimal erase and emails learned on leave)
- **Status:** Reviewed
- **Scope:** `crates/kx-mod-predict`: the manifest, the `[predict]` settings, the actor on its `kx-predict` thread (`actor.rs`, `actor/`), the lane and menu (`lane.rs`, `menu.rs`), the tracker (`track`), the scorer (`score`), the learner (`learn`), the email chips and their byte form (`email`), folding (`fold`) and the word-pack format (`pack`). It runs inside the uiAccess `keyxtend` process; the pack writer runs only in `cargo xtask words`, and `cargo xtask tidy` (D7) refuses `build` in the app's tree.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `ReadFocusedText` | It gates `FieldWatchKey` (the focused box's kind and the words before the cursor) and the typed-text events: `Typed`, `TypingMode`, `TypingModeAsk`, `TypeWord`, `Suggested`, `ChipPicked` and `WordsAsk`. | The module does not start (`without_read_focused_text_it_cannot_watch_the_box`), so the lane stays empty. |
| `SecretStore` | It gates `SealedStoreKey`, which keeps the learned words and emails sealed (ADR-0032). | The module does not start (`without_secret_store_it_cannot_reach_the_sealed_store`). |

- It has no `InjectInput` (`manifest.rs`): a pick asks the keyboard with one `TypeWord`, and the keyboard types it (ADR-0033).
- `WordPacksKey` is ungated: the packs are public data. `LayoutChanged`, `LayoutAsk`, `PointerPressed`, `PressesWanted`, `PressesAsk` and `Notice` are ungated too.

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| The open word and up to `context_words` words before it (`Tracker`) | Sensitive (typed text) | Memory only, in `Redacted` where it leaves the tracker | No | Until the word ends, a reset, or the stop |
| Text read before the cursor (`FieldRun::before_cursor`, a `FieldRead` with the box's kind) | Sensitive | Memory only: at most `read_chars` characters; the tracker keeps the last words as context and open word | No | The read goes at once, its last words when the word ends, at a reset or the stop; read words are never learned, and join chains only through words already learned |
| Learned words and chains (`Learner`) | Sensitive (what the user writes) | Sealed blob `predict-<iso>` through `SealedStore` (kx-crypto, XChaCha20-Poly1305) | Yes | Dropped after `forget_days` unused, at Forget, or past `max_words` and `max_chains` |
| Known emails (`Emails`) | Sensitive (personal data) | Sealed blob `predict-emails`, one for every language | Yes | Kept when the user leaves a typed address (see Information disclosure); the `max_emails` most recent |
| A blob that opened but did not read | Sensitive | Its aside blob, its name then `ASIDE_SUFFIX`, sealed the same way | Yes | Until the next copy of that blob, which wins |
| Chips (`Suggested`) and picks (`TypeWord`) | Sensitive | One bus event each, gated by `ReadFocusedText`, texts in `Redacted` | No | Until the next lane |
| `[predict]` settings, including `on` and `learn` the menu writes | Public | `settings.toml` (only the user's changes) | No | Until the user changes them |
| A word pack's seed words, frequencies, capitals and chains | Public (wordfreq, Tatoeba and Wikipedia data) | The shipped `<iso>.kxw` files, read in place from a read-only map | No | As shipped |
| The actor's counts (`STOPPED_LINE`) | Public (counts) | The log, one line when the actor quits | No | As the log |

`Tracker`, `Learner` and `Emails` print counts only in debug output, `Show`, `Plan` and `Pick` hold their texts in `Redacted`, and the actor has no `Debug`. Opened and saved blob bytes, and the live learner and emails, are plain memory, never wiped (see Accepted).

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| `Typed`, `TypingMode`, `ChipPicked`, `WordsAsk` (gated) | Typed values only, relayed to the actor's inbox; a pick of an older `round` or of no chip does nothing; a list row past the shown page is refused, and pages are clamped to what the list holds | One message per event; the inbox has no bound (see Accepted) | Menu property test (`menu/tests.rs`); tracker property test (`track/props.rs`); `tests/predict/emails.rs` |
| `LayoutChanged` and `PointerPressed` (ungated) | The language must pass `fits_iso` before it names a blob or opens a pack, and the pack's own code must match it; the layouts module tells it at its start and on `LayoutAsk`, so any start order works; a press only resets the tracker | One switch per event | No; `tests/predict/lang.rs` |
| The box (`FieldChange`, `FieldRun::kind`, `before_cursor`, `ends_with`) | Only a `Text` box is read or learned from; the kind the watch or a read (`FieldRead`) sees wins over a late focus event; a read error counts as no context. A pick keeps the exact common start of the text it replaces and the chip, by `char`, and erases only the rest. A pick that replaces typed or read text first asks a `Text` box `ends_with` all of it, even when it erases nothing: `Some(true)` erases and types; `Some(false)` sends nothing, resets and reads again; no answer (`None`) types only a plan that erases nothing, and otherwise nothing. A box that cannot confirm (`Unknown` or none) is never erased: it offers only chips that extend exactly what was typed, and next words, and a pick types only the rest | `read_ms` and `read_chars` from `[predict]`; a pick past `text_cap` sends nothing | No; `tests/predict/boxes.rs`, `lane.rs` |
| Sealed learned words (`Learner::from_bytes`) | AEAD first (kx-crypto), then magic, version, a sound language code equal to the one in front, every length against what is left, entries in order and once, nothing after the end; a spelling the learning rule (`could_learn`) refuses is dropped | The store caps a blob at `MAX_BLOB_BYTES` | Property test: random and altered bytes never panic (`learn/props.rs`) |
| Sealed emails (`Emails::from_bytes`) | Magic, version, every length against what is left, each entry UTF-8 and an email, nothing after the end; no allocation from the count | Same cap; at most `max_emails` kept | Property tests: any bytes and any change are read or refused (`email/bytes/tests.rs`) |
| Word pack bytes (`Pack::parse`) | Magic, version and every header field in range, the language code one `fits_iso` takes; each section inside the file, known and once; offsets in order and UTF-8 valid; the lexicon's CRC-32C. Each later read checks offsets again, as a mapped file could change | The adapter maps at most `MAX_PACK_BYTES`; `layout::MAX_WORDS` words, spellings of `MAX_SPELLING` bytes | Property tests over random bytes (`pack/tests.rs`), truncations and byte changes (`pack/tests/damage.rs`) |
| The pack index (`packs.txt`, through `WordPacks::ships` and `pack_of`) | Whether a missing pack shows `notice.words-missing`, and which pack serves a layout code (its alias line). `PackIndex` stops at the first part that is not UTF-8 or fails `fits_iso`, a code served twice, or a layout code that is a pack; an index that lists no pack is `Empty`. A missing, damaged or empty index ships nothing, logged by kind, and a later folder's index that clashes adds nothing | Read once per run, within `MAX_INDEX_BYTES` | Property test: any bytes are read or refused (`kx-module-api` `ports/packs/index/tests.rs`) |
| `[predict]` settings | `deny_unknown_fields`; each value within its named range, the ranges tied to the engine's limits by tests; domains checked; the scorer's and learner's own checks | Checked at load: a refused value goes back to its default with the `settings-reset` notice | No; `settings/tests.rs` tries each bound |

No `cargo fuzz` target exists: property tests stand in until the owner allows the nightly toolchain (see Accepted).

## Threats (STRIDE-lite)
- **Spoofing:** only modules holding `ReadFocusedText` can send picks or menu taps, so no other module can make the keyboard type. A forged `LayoutChanged` only switches the language: the old learner saves and the new one loads. A forged `PointerPressed` only ends the word unlearned.
- **Tampering:**
  - A blob changed by another user or PC, or by damage, fails its AEAD and loads as `Damaged`; the module starts empty and shows `notice.words-damaged` once. Programs running as this user can unwrap the key and seal blobs that pass (see Accepted), so the decoders treat every blob as untrusted.
  - A blob that opens but does not read, or names another language, is copied once to its aside blob (plan edge 10): learning starts empty, the notice shows once, and saves resume. A failed copy, an `Io` load or a `Locked` store holds that blob for the run, so it is never written over.
  - A damaged pack reads as missing. A changed index can only add or hide the missing notice, or give a layout another shipped pack; every pack still passes `Pack::parse`.
- **Logging / repudiation:** log lines hold fixed texts, error kinds, a blob kind and counts only, never a word, an email or typed text. `STOPPED_LINE` holds counts only. `no_log_line_shows_a_typed_word_a_learned_word_or_an_email` captures every line of a session and checks it.
- **Information disclosure:**
  - A password box locks the lane: the tracker follows nothing, the box is never read, and nothing is learned (`a_password_box_locks_the_lane_learns_nothing_and_is_never_read`). The kind the watch or a read sees wins at once, so the window left lasts until the platform sees the focus move (see Accepted).
  - An `Unknown` box or none still gets chips from what is typed here, but is never read or learned from. A secret typed in a box the platform does not flag can show completions of its prefix there (see Accepted).
  - Off follows, reads and learns nothing; Pause learns nothing. Both hold even when the settings write fails.
  - An email is kept only when the user leaves a full address typed here, in a text box with prediction and learning on: a space, Enter or Tab, a domain chip that completes it once its text reached the app, a click outside our windows, or focus on another box (owner, 2026-10-26). A read or partial address, or one left while the watch already sees a password box or another kind, is never kept (`tests/predict/leave.rs`), and an address counts once (`logs.rs`, `a_domain_chip_then_a_space_counts_one_email`).
  - Known emails list only after a typed `@@`; one that came from a read never lists them (`an_at_at_that_came_from_a_read_never_lists_emails`).
  - Learned words and emails rest sealed under the OS-wrapped key; a `Locked` store keeps them in memory only and saves nothing that run, with `notice.words-locked` once.
- **Denial of service:**
  - The learner is capped by `max_words` and `max_chains`, and the emails by `max_emails`; `purge` drops entries unused for `forget_days` once a day.
  - Each blob saves at most once per `save_s`, and at stop. A refused save waits `save_s` before it tries again.
  - A pack parses once per language switch, on the actor's thread, off the key and UI paths.
  - A read waits `read_ms` at most and takes `read_chars` at most.
- **Elevation:** the crate forbids `unsafe` (`lib.rs`). It never types: the keyboard checks and types each `TypeWord` (ADR-0033). A panic in the actor stops the lane (`Lane::Stopped`), lets presses go and shows `notice.module-failed`.

## Findings
### Must fix (blocks Definition of Done)
- None open.
- Done (P7 fix pass): a word typed here was erased without asking the box, so a click predict never heard (an injected click, the mouse module off) let a pick erase the user's text. Every pick that replaces typed text now asks the box, even one that erases nothing (`a_typed_word_the_box_no_longer_ends_with_erases_nothing_and_the_lane_reads_again`).
### Should fix (tracked issue)
- None open.
- Done (P7 fix pass): learning and the lane followed the last focus event only; the watch's and the read's kind now win at once.
- Done (P7 fix pass): a blob that opened but did not read was written over at the next save; it is now set aside first.
- Done (P7 fix pass): decoded learned words skipped the learning rule, and the keyboard refused only controls; `could_learn` now drops them, and the keyboard refuses `is_hidden`.
- Done (P7 fix pass): a save refused at a language switch lost that language's words; they now wait and save later.
### Accepted (with reason)
- The actor's inbox has no bound, as in the other modules: only first-party modules publish on the in-process bus, and each message is small.
- A learner past `MAX_BLOB_BYTES` fails to save as `Io` and tries again every `save_s`; the defaults stay far below the cap, and the words stay in memory for the run.
- A lexicon changed under the view after its checksum passed, or a checksum-valid malformed one, could make `fst` panic: the Windows adapter opens packs with read sharing only, release packs sit under Program Files, and the module's panic is contained.
- No `cargo fuzz` target: property tests stand in until the owner allows the nightly toolchain, as in `kx-kernel.md`.
- Programs running as this user can unwrap the data key, so they can read and forge sealed blobs (`kx-platform-windows.md`); they can screen-record the keyboard's clicks anyway, and the decoders are bounded safe Rust with property tests.
- Rollback: an older sealed copy put back passes the tag and brings back forgotten words; the format has no version counter.
- A control that never says it is a password box, or a forged UI Automation focus event, can make predict follow and learn there until the real kind shows; the learning rule and `min_uses` narrow it.
- In a box the platform does not flag, chips can complete the prefix of a secret typed here; nothing there is read or learned.
- A box that cannot confirm a pick gets only the chip's rest, which erases nothing, so an unheard click there can only add that rest at the new caret.
- Plain copies of sealed data (opened and saved bytes, the live learner and emails) are not wiped: Sensitive, not Secret, and the uiAccess process's memory sits above medium-integrity programs.
- A forged blob near `MAX_BLOB_BYTES` makes a decoder grow up to about 1.6 M entries before a duplicate or a cap ends it: tens of MB, for a moment.
