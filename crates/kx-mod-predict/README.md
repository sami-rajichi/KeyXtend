# kx-mod-predict

Word prediction (spec §6, ADR-0032). Today it holds two pure parts: folding, so `ecole` finds `école` and `ان` finds `أن`, and the word-pack format each language's seed ships in. The module, its settings and its capabilities come later in P7.

- **Folding** (`fold`): lower case, then the tables in `src/fold.toml`. Latin accents go, precomposed or combining, and ligatures spell out (`cœur` → `coeur`); the typographic apostrophe folds to the plain one (`l’` → `l'`); Greek final sigma folds to sigma; Arabic harakat, the superscript alef and tatweel go, أ إ آ ٱ fold to ا and ى to ي. The chip always types the stored spelling.
- **Packs** (`pack`): `<iso>.kxw` files, read in place from the mapped file by `Pack`. `pack/layout.rs` defines the format; `PackWriter`, behind the feature `build`, writes it for `cargo xtask words`.

## Services and capabilities

None yet; the module adds them.

## Tests

`cargo test -p kx-mod-predict --features build` runs every test; without `build`, the writer's round trip and the damaged-pack tests are left out. A workspace run turns `build` on once `xtask` depends on it.

## Security

See `docs/security/kx-mod-predict.md`.
