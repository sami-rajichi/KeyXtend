# kx-mod-predict

Word prediction (spec §6, ADR-0032). Today it holds pure parts: folding and casing, so `ecole` finds `école` and `ان` finds `أن`, the word splitting the pack builder and the tracker share, the word-pack format each language's seed ships in, and the engine: the tracker, the scorer and emails. The module, its settings and its capabilities come later in P7.

- **Casing** (`casing`, rows in `src/casing.toml`): the language rows of Unicode's SpecialCasing.txt, so Turkish keeps `İ i` apart from `I ı`; other languages case as Unicode's default. The pack builder and the engine share it.
- **Folding** (`fold`): lower case by the language's casing, then the tables in `src/fold.toml`. Latin accents go, precomposed or combining, and ligatures spell out (`cœur` → `coeur`); the typographic apostrophe folds to the plain one (`l’` → `l'`); Greek final sigma folds to sigma; Arabic harakat, the superscript alef and tatweel go, أ إ آ ٱ fold to ا and ى to ي. The chip always types the stored spelling.
- **Splitting** (`split`, `script`): one rule for `cargo xtask words` and the tracker. A word is a run of letters, digits and marks; at an apostrophe a known whole word stays (`quelqu'un`), then a known elision comes off (`l'`), else the apostrophe splits. `split::places` says which piece stands alone and which joins a chain. A word mixing scripts is never a word; each script's row gives its direction and the gap between its words (UAX #14).
- **Tracker** (`track`): follows the keyboard's pieces to know the word being typed and the words before it. A space, Enter or Tab ends a word, which is learned as the build counts it; marks after it, Enter and Tab clear the context. A break, a click, a new box or a layout switch ends the word unlearned and asks for a read. Each open word reads whole and after each known elision, so `l'éc` finds `l'école`.
- **Scorer** (`score`): stupid backoff over the context, seed chains and frequencies blended with learned counts (`Learned`), capitals that follow what was typed. After an elision its followers come first; the word list fills in only when none match and a letter follows. A script without word spaces (zh, ja) gets no chips.
- **Emails** (`email`, marks in `src/email.toml`): the whole address around a typed `@` is saved, less trailing marks; domains after one `@`, known emails from their start or after a typed `@@`, the paged list.
- **Packs** (`pack`): `<iso>.kxw` files, read in place from the mapped file by `Pack`. `pack/layout.rs` defines the format (v2: the spellings hold the capitals); `PackWriter`, behind the feature `build`, writes it for `cargo xtask words`.

## Services and capabilities

None yet; the module adds them.

## Tests

`cargo test -p kx-mod-predict --features build` runs every test; without `build`, the writer's round trip and the damaged-pack tests are left out. A workspace run turns `build` on once `xtask` depends on it.

## Security

See `docs/security/kx-mod-predict.md`.
