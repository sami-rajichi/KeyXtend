# kx-mod-predict

Word prediction (spec §6, ADR-0032): the module below, over pure parts: folding and casing, so `ecole` finds `école` and `ان` finds `أن`, the word splitting the pack builder and the tracker share, the word-pack format each language's seed ships in, and the engine: the tracker, the scorer, the learner and emails.

- **Casing** (`casing`, rows in `src/casing.toml`): the language rows of Unicode's SpecialCasing.txt, so Turkish keeps `İ i` apart from `I ı`; other languages case as Unicode's default. The pack builder and the engine share it.
- **Folding** (`fold`): lower case by the language's casing, then the tables in `src/fold.toml`. Latin accents go, precomposed or combining, and ligatures spell out (`cœur` → `coeur`); the typographic apostrophe folds to the plain one (`l’` → `l'`); Greek final sigma folds to sigma; Arabic harakat, the superscript alef and tatweel go, أ إ آ ٱ fold to ا and ى to ي. The chip always types the stored spelling.
- **Splitting** (`split`, `script`): one rule for `cargo xtask words` and the tracker. A word is a run of letters, digits and marks; at an apostrophe a known whole word stays (`quelqu'un`), then a known elision comes off (`l'`), else the apostrophe splits. `split::places` says which piece stands alone and which joins a chain. A word mixing scripts is never a word; each script's row gives its direction and the gap between its words (UAX #14). `script::main` picks a list's main script, for the builder and for `Pack::script` alike.
- **Tracker** (`track`): follows the keyboard's pieces to know the word being typed and the words before it. A space, Enter or Tab ends a word, which is learned as the build counts it; marks after it, Enter and Tab clear the context. A break, a click, a new box or a layout switch ends the word unlearned and asks for a read. Each open word reads whole and after each known elision, so `l'éc` finds `l'école`.
- **Scorer** (`score`): stupid backoff over the context, seed chains and frequencies blended with learned counts (`Learned`), capitals that follow what was typed. After an elision its followers come first; the word list fills in only when none match and a letter follows. A script without word spaces (zh, ja) gets no chips.
- **Learner** (`learn`): per language, the words and chains the user typed with their last day, capped by `max_words` and `max_chains` and purged after `forget_days`. Forget blocks a seed word until it comes back `min_uses` times; its byte form is small, versioned and checked.
- **Emails** (`email`, marks in `src/email.toml`): the whole address around a typed `@` is saved, less trailing marks; domains after one `@`, known emails from their start or after a typed `@@`, the paged list. Their byte form (`Emails::to_bytes`) is small and versioned; a cut or changed form is an error, never a panic.
- **Packs** (`pack`): `<iso>.kxw` files, read in place from the mapped file by `Pack`. `pack/layout.rs` defines the format (v2: the spellings hold the capitals); `PackWriter`, behind the feature `build`, writes it for `cargo xtask words`.

## The module

`PredictModule` watches the focused box, follows the keyboard's pieces and tells the lane (`Suggested`) with a new round after each change. A pick asks the keyboard to type with one `TypeWord`; the module never types itself (ADR-0033). Only the language in front is loaded: its pack, parsed once per switch, and its learner, sealed as `predict-<iso>`; known emails are one blob, `predict-emails`.

- **Boxes:** a password box locks the lane and is never followed, read or learned from. An unknown box, or none, gets chips but is never read or learned from. A text box is read for its last words at a new box or a click outside our windows.
- **Picks:** a tap on an older lane does nothing. A word read from the box is typed only after the box says it still ends with it; otherwise the module reads again.
- **Menu (`WordsAsk`):** Forget, then a word chip, forgets that word, and an Undo chip shows until the next key. Pause and Off save `learn` and `on` through the module's own settings. The email list opens from More, with the rows the UI shows, and pages by them; a row past the page does nothing.
- **Language:** it asks `LayoutAsk` at start and follows each `LayoutChanged`; the layouts module also tells its layout once at its own start, so any start order works.
- **Saving:** each blob saves `save_s` after its first change, and at stop. A damaged blob starts empty with `notice.words-damaged`, and a locked store saves nothing that run with `notice.words-locked`. A missing or unreadable pack of a language the build shipped, as the pack index lists, shows `notice.words-missing` once per language and run.

## Services and capabilities

| Service | Direction | Use |
|---|---|---|
| `FieldWatch` (`FieldWatchKey`) | needs | The focused box's kind, the words before the cursor, and whether it still ends with a read word. |
| `WordPacks` (`WordPacksKey`) | needs | Opens the language in front's pack, and tells whether the build shipped one. |
| `SealedStore` (`SealedStoreKey`) | needs | Loads and saves the learned words and emails, sealed. |

| Capability | Why |
|---|---|
| `ReadFocusedText` | It gates the box and the typed-text events (`Typed`, `TypingMode`, `TypeWord`, `Suggested`, `ChipPicked`, `WordsAsk`). |
| `SecretStore` | It gates the sealed store. |

`LayoutChanged`, `LayoutAsk`, `PointerPressed` and `PressesWanted` need no capability. It provides no service.

## Settings (`[predict]`, version 1)

| Key | Default | Meaning |
|---|---|---|
| `on` | `true` | Word chips are shown; the menu's Off writes it. |
| `learn` | `true` | Finished words and emails are learned; the menu's Pause writes it. |
| `chips` | `5` | The most chips in the lane; within `CHIPS`. |
| `context_words` | `2` | The previous words a guess reads; within `CONTEXT_WORDS`, up to the pack's longest chain less one. |
| `min_uses` | `2` | The uses before a learned word is offered; within `MIN_USES`, for the scorer and the learner alike. |
| `max_words` | `10000` | The learned words kept per language; within `MAX_WORDS`. |
| `max_chains` | `30000` | The learned chains kept of each length; within `MAX_CHAINS`. |
| `forget_days` | `300` | Days unused before a learned entry goes; within `FORGET_DAYS`. |
| `learn_weight` | `0.6` | The most weight the learned counts reach; within `LEARN_WEIGHT`, the scorer's own bound. |
| `learn_ramp` | `2000` | The typed words at which the learned counts weigh half their most; within `LEARN_RAMP`. |
| `backoff` | `0.4` | Each step to a shorter context multiplies a score by this; within `BACKOFF`, the scorer's own bound. |
| `max_word` | `48` | The longest word with chips, in characters; within `MAX_WORD`. |
| `max_emails` | `50` | The known emails kept; within `MAX_EMAILS`. |
| `min_start` | `2` | The typed characters before a known email is offered; within `MIN_START`. |
| `domains` | five common domains | The domains offered after one `@`, in order; each a domain, at most the end of `DOMAINS`. |
| `read_ms` | `150` | The longest one read of the box may take; within `READ_MS`. |
| `read_chars` | `1024` | The longest text one read takes; within `READ_CHARS`. |
| `save_s` | `30` | Seconds from a first unsaved change to its save; within `SAVE_S`. |

The ranges are named constants in `src/settings.rs`, tied by tests to the engine's limits; the defaults live only in `defaults.toml`.

## Public API

`PredictModule`, `ID`, `MANIFEST`, `PredictSettings` (with `from_defaults` and each part's config), its `SPEC`, and the menu's keys `LEARN_KEY` and `ON_KEY`. The runtime is the `Actor`, driven by `Msg` on the `kx-predict` thread over its `Ports`; tests build it directly, with a fake `Days`, and it logs `STOPPED_LINE` with counts only when it quits. `learned_blob` and `emails_blob` name its blobs.
## Tests

`cargo test -p kx-mod-predict --features build` runs every test; without `build`, the writer's round trip and the damaged-pack tests are left out. A workspace run turns `build` on once `xtask` depends on it.

## Security

See `docs/security/kx-mod-predict.md`.
