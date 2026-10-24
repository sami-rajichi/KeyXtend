# kx-mod-predict

Word prediction (spec §6, ADR-0032). Today it holds pure parts: folding, so `ecole` finds `école` and `ان` finds `أن`, the word splitting the pack builder and the tracker share, the word-pack format each language's seed ships in, and the engine: the tracker, the scorer and emails. The module, its settings and its capabilities come later in P7.

- **Folding** (`fold`): lower case, then the tables in `src/fold.toml`. Latin accents go, precomposed or combining, and ligatures spell out (`cœur` → `coeur`); the typographic apostrophe folds to the plain one (`l’` → `l'`); Greek final sigma folds to sigma; Arabic harakat, the superscript alef and tatweel go, أ إ آ ٱ fold to ا and ى to ي. The chip always types the stored spelling.
- **Splitting** (`split`, `script`): one rule for `cargo xtask words` and the tracker. A word is a run of letters, digits and marks; at an apostrophe a known whole word stays (`quelqu'un`), then a known elision comes off (`l'`), else the apostrophe splits. A word mixing scripts is never a word.
- **Tracker** (`track`): follows the keyboard's pieces to know the word being typed and the words before it: a space ends a word, which is learned and joins the context, a mark ends it unlearned, and a break, a click, a new box or a layout switch asks for a read. Each open word reads whole and after each known elision, so `l'éc` finds `l'école`.
- **Scorer** (`score`): stupid backoff over the context, seed chains and frequencies blended with learned counts (`Learned`), capitals that follow what was typed.
- **Emails** (`email`, marks in `src/email.toml`): domains after one `@`, known emails from their start or after a typed `@@`, the paged list.
- **Packs** (`pack`): `<iso>.kxw` files, read in place from the mapped file by `Pack`. `pack/layout.rs` defines the format; `PackWriter`, behind the feature `build`, writes it for `cargo xtask words`.

## Services and capabilities

None yet; the module adds them.

## Tests

`cargo test -p kx-mod-predict --features build` runs every test; without `build`, the writer's round trip and the damaged-pack tests are left out. A workspace run turns `build` on once `xtask` depends on it.

## Security

See `docs/security/kx-mod-predict.md`.
