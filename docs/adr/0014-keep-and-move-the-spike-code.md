# 14. Keep the tested spike code and move it into the product

- **Date:** 2026-09-29
- **Status:** Accepted. Supersedes ADR-0011's "No leftovers" rule for code; its rules for screenshots, logs and virtual PCs stay.

## Context

ADR-0011 planned to delete the spike code after the test round and keep only its numbers. The round grew into about 29,000 lines of reviewed, tested Rust and QML on the Qt side:

- the Qt keyboard with all three themes, motion, moving, resizing and the bubble;
- the hold engine, scroll routes, selection pill, clipboard listener, quick-fill, snip and the language key;
- the voice worker with the local and opt-in cloud engines;
- a Windows gate harness and a logging test window.

The owner judged it close to the product. Rebuilding it from zero would cost weeks and lose tested behaviour. Decision by the owner on 2026-09-29: keep the code, Qt only, and apply every project rule as it moves.

## Decision

- **Keep, then move and harden.** Each roadmap phase moves the spike parts it needs into the product (`crates/`, `apps/`, `tools/`), following `docs/spike-move-map.md`.
- **Moving is not copying.** A moved part is fitted to the architecture: a module behind ports, settings in the schema, texts in translation files, and theme values in tokens. It then meets the full Definition of Done.
- **One copy of each moved part.** The commit that moves a part deletes it from `spike/`; spike code that still needs it depends on the new crate.
- **Shared foundations are models.** The spike's config loader and clock are too widely used to move early; the product crates are built from their tested pattern, and the spike files go when their last user has moved.
- **Qt only.** `spike/slint-kb` and `spike/tauri-kb`, and the settings and harness lines that serve only them, are removed at the start of P2; they stay in the history of `spike/toolkits`.
- **Test tools live in `tools/`.** The harness moves whole into `tools/kx-gates` and `target-window` into `tools/kx-target-window` (the CI test window of spec §12). Tools never ship: they may use `unsafe` with `// SAFETY:` comments, tidy checks their size, and no shipped crate depends on them.
- **The settings files are the source.** `spike.toml`, `themes.toml`, `shape.toml` and `motion.toml` seed the settings schema defaults and the theme tokens.
- **Reaching `main`.** `spike/toolkits` merges into `main` by a pull request once the owner approves the push; P2 starts from `main` after that.
- **Still deleted:** screenshots, logs, scratch scripts and virtual PCs. The test certificate and the Program Files test install stay while uiAccess builds are tested; the owner decides when to remove them (the certificate expires on 2026-12-26).

## Consequences

- Each phase plan lists the spike files it moves, and its tests prove the moved behaviour still holds.
- The spike keeps building and its tests keep passing until its last part moves.
- Tidy does not scan `spike/`. P2 adds `tools/` to tidy's scan with the `unsafe` exception above.
- The `kx-feature` skill and the roadmap no longer call P1 throwaway.

## Alternatives

- **Delete and rebuild (ADR-0011 as written):** clean, but slow, and it throws away tested fixes.
- **Merge the spike as it is:** fast, but it would skip the kernel, the module rules and the review, so it is refused.
