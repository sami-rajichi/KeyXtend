# kx-mod-predict

Word prediction (spec §6, ADR-0032): the module below, over pure parts: folding and casing, so `ecole` finds `école` and `ان` finds `أن`, the word splitting the pack builder and the tracker share, the word-pack format each language's seed ships in, and the engine: the tracker, the scorer, the learner and emails.

- **Casing** (`casing`, rows in `src/casing.toml`): the language rows of Unicode's SpecialCasing.txt, so Turkish keeps `İ i` apart from `I ı`, and two ICU rules: Dutch `IJ` as one capital, Greek all capitals without the tonos. Other languages case as Unicode's default. The pack builder and the engine share it.
- **Folding** (`fold`): lower case by the language's casing, then the tables in `src/fold.toml`. Latin accents go, precomposed or combining, and ligatures spell out (`cœur` → `coeur`), Romanian and Vietnamese letters fold too (`người` → `nguoi`); the typographic apostrophe folds to the plain one (`l’` → `l'`); Greek final sigma folds to sigma; Arabic harakat, the superscript alef and tatweel go, أ إ آ ٱ fold to ا and ى to ي. The chip always types the stored spelling.
- **Splitting** (`split`, `script`): one rule for `cargo xtask words` and the tracker. A word is a run of letters, digits and marks, split at any hidden mark (`kx_module_api::is_hidden`), with ZWNJ and ZWJ kept between its letters (`کتاب‌ها`) and trimmed at its edges; at an apostrophe a known whole word stays (`quelqu'un`), then a known elision comes off (`l'`), else the apostrophe splits. `split::places` says which piece stands alone and which joins a chain. A word mixing scripts is never a word; each script's row gives its direction and the gap between its words (UAX #14). `script::main` picks a list's main script, for the builder and for `Pack::script` alike.
- **Tracker** (`track`): follows the keyboard's pieces to know the word being typed and the words before it. A space, Enter or Tab ends a word, which is learned as the build counts it; marks after it, Enter and Tab clear the context. A break, a click, a new box or a layout switch ends the word unlearned and asks for a read. Each open word reads whole and after each known elision, so `l'éc` finds `l'école`.
- **Scorer** (`score`): stupid backoff over the context, seed chains and frequencies blended with learned counts (`Learned`), capitals that follow what was typed (all capitals only when no typed letter is lower case). After an elision its followers come first; the word list fills in only when none match and a letter follows. A script without word spaces (zh, ja) gets no chips.
- **Learner** (`learn`): per language, the words and chains the user typed with their last day, capped by `max_words` and `max_chains` and purged after `forget_days`. Forget blocks a seed word until it comes back `min_uses` times; its byte form is small, versioned and checked.
- **Emails** (`email`, marks in `src/email.toml`): the whole address around a typed `@` is saved, less trailing marks, once the user leaves it: a space, Enter or Tab, a domain chip that completes it, a click outside our windows or another box, and a longer address of the same token replaces the one a domain chip taught; domains after one `@`, known emails from their start or after a typed `@@`, the paged list. Their byte form (`Emails::to_bytes`) is small and versioned; an entry that is no email is dropped, and any other cut or change is an error, never a panic.
- **Packs** (`pack`): `<iso>.kxw` files, read in place from the mapped file by `Pack`. `pack/layout.rs` defines the format (v2: the spellings hold the capitals); `PackWriter`, behind the feature `build`, writes it for `cargo xtask words`.

## The module

`PredictModule` watches the focused box, follows the keyboard's pieces and tells the lane (`Suggested`) with a new round after each change. A pick asks the keyboard to type with one `TypeWord`; the module never types itself (ADR-0033). At its stop, or when its thread panics, the lane says `Lane::Stopped`, which offers no menu. Only the language in front is loaded: its pack, parsed once per switch, and its learner, sealed as `predict-<iso>`; known emails are one blob, `predict-emails`.

- **Boxes:** a password box locks the lane and is never followed, read or learned from. An unknown box, or none, gets chips but is never read or learned from. A text box is read for its last words at a new box or a click outside our windows. When the watch, a read or a pick's check already sees another kind of box, that kind wins at once over the focus event still on its way, at each piece, lane and read; a box the watch sees as a password box is never read. A text box whose read gives no text, or whose check cannot tell, offers only chips that extend the typed word until focus moves to another box, and a box holding a selection never confirms a pick.
- **Picks:** a tap on an older lane does nothing. A pick keeps the exact common start of what it replaces and the chip, and erases and types only the rest. It types only after a text box says it still ends with what it replaces. When the box says it does not, nothing is typed, the word resets and the module reads again; when it gives no answer, only a pick that erases nothing is typed. A box that cannot tell offers only next-word chips and chips that start with exactly what was typed, and types their rest; an email it could not type without erasing is never offered, in the lane, the `@@` list or More. A chip or list row past the keyboard's `text_cap`, which `TypingMode` carries, is never shown. A latched Shift gives the chips a first capital, as the pick spends it.
- **Menu (`WordsAsk`):** Forget, then a word or known-email chip, forgets it; its Undo chip shows until the next key. Forget mode fills the lane with words or emails, with no domain or More chip. Pause and Off save `learn` and `on` through the module's own settings. The email list opens from More, with the rows the UI shows, and pages by them; a row past the page does nothing.
- **Language:** it asks `LayoutAsk` at start and follows each `LayoutChanged`; the layouts module also tells its layout once at its own start, so any start order works. A layout code the pack index lists after a pack, such as `hr` after `sh`, takes that pack; its learned file keeps the layout's code.
- **Typing mode:** it asks `TypingModeAsk` at start, and the keyboard answers with `TypingMode`; the keyboard also tells it at its own start.
- **Saving:** each blob saves `save_s` after its first change, and at stop; while learning is paused, neither the daily purge nor these timed saves write. A damaged blob starts empty with `notice.words-damaged`. A blob that opens but does not read, or names another language, does the same and is copied to its aside blob (`aside_blob`), the newest copy replacing the last; later saves resume. A blob whose copy or load fails is never written that run. A language left while its save fails keeps its words for a later save. While `on` is off, no pack or learner opens. A locked store saves nothing that run with `notice.words-locked`. A missing or unreadable pack of a language the build shipped, as the pack index lists, shows `notice.words-missing` once per language and run.

## Services and capabilities

| Service | Direction | Use |
|---|---|---|
| `FieldWatch` (`FieldWatchKey`) | needs | The focused box's kind, the words before the cursor, and whether it still ends with a read word. |
| `WordPacks` (`WordPacksKey`) | needs | Opens the pack serving the language in front, and names it from the build's index. |
| `SealedStore` (`SealedStoreKey`) | needs | Loads and saves the learned words and emails, sealed. |

| Capability | Why |
|---|---|
| `ReadFocusedText` | It gates the box and the typed-text events (`Typed`, `TypingMode`, `TypingModeAsk`, `TypeWord`, `Suggested`, `ChipPicked`, `WordsAsk`). |
| `SecretStore` | It gates the sealed store. |

`LayoutChanged`, `LayoutAsk`, `PointerPressed`, `PressesWanted` and `PressesAsk` need no capability; it answers `PressesAsk` with its current `PressesWanted`, so the mouse module may start in any order. It provides no service.

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
| `max_word` | `48` | The longest word with chips, in characters; within `MAX_WORD`, whose top and its gap fit the keyboard's smallest `text_cap` (`TEXT_CAP`). |
| `max_emails` | `50` | The known emails kept; within `MAX_EMAILS`. |
| `min_start` | `2` | The typed characters before a known email is offered; within `MIN_START`. |
| `domains` | five common domains | The domains offered after one `@`, in order; each a domain, at most the end of `DOMAINS`. |
| `read_ms` | `150` | The longest one read of the box may take; within `READ_MS`. |
| `read_chars` | `1024` | The longest text one read takes; within `READ_CHARS`. |
| `save_s` | `30` | Seconds from a first unsaved change to its save; within `SAVE_S`. |

The ranges are named constants in `src/settings.rs`, tied by tests to the engine's limits; the defaults live only in `defaults.toml`.

## Public API

`PredictModule` (`default` on the system calendar, `with_days` on another), `ID`, `MANIFEST`, `PredictSettings` (with `from_defaults` and each part's config), its `SPEC`, and the menu's keys `LEARN_KEY` and `ON_KEY`. The runtime is the `Actor`, driven by `Msg` on the `kx-predict` thread over its `Ports`; tests build it directly, with a fake `Days`, and it logs `STOPPED_LINE` with counts only when it quits, and `OTHER_LANGUAGE` for a learned file of another language. `learned_blob`, `emails_blob` and `aside_blob` name its blobs.
## Tests

`cargo test -p kx-mod-predict --features build` runs every test; without `build`, the writer's round trip and the damaged-pack tests are left out. A workspace run turns `build` on through `xtask`, which depends on it with that feature.

## Security

See `docs/security/kx-mod-predict.md`.
