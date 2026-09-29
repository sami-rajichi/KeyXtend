# Project rules for agents

This is **KeyXtend**, the eXtended keyboard: an accessible on-screen keyboard, written in Rust, Windows first. Every window is Qt 6 Quick (QML) over a Rust core through cxx-qt (ADR-0013). The tested P1 code moves into the product phase by phase (ADR-0014, `docs/spike-move-map.md`). Read these before doing anything:

1. `docs/superpowers/specs/2026-09-26-accessible-keyboard-design.md`: what we build.
2. `ARCHITECTURE.md`: where things are, and the invariants.
3. `docs/adr/README.md`: why things are this way.
4. `docs/roadmap.md`: the current phase and the Definition of Done.

## Working with the product owner

- Explain decisions in plain language.
- Ask questions with the **AskUserQuestion pop-up**: several questions per round, each with a recommended option.
- Back design choices with real references.
- Arabic must always be correct.
- Personal notes for this machine and owner live in `CLAUDE.local.md` (not in the repo).

## Owner's rules (always apply; they override any skill)

1. **Nothing hardcoded.**
   - Every value comes from one source of truth:
     - timings, sizes, speeds and limits come from the settings schema, with its defaults in one place;
     - colours, fonts and spacing come from theme tokens;
     - user-visible text comes from translation files;
     - OS mappings come from adapter tables.
   - No magic numbers or strings in logic.
   - Changing a value must never need edits in two places.
2. **Short and clear.**
   - Names are short and meaningful.
   - Comments, doc strings, log messages and UI texts are one or two sentences, never paragraphs.
   - Files stay small: aim for ≤ 300 lines. At 400, split the file. Functions stay ≤ 40 lines.
3. **Review every file.** Before any commit, run the `kx-review` skill on every changed file. It checks design pattern, correctness, tests, security, the no-hardcode rule and the size rule.
4. **Work log.**
   - Every implemented feature or fixed issue adds one entry to `docs/WORKLOG.md`.
   - An entry is a timestamp plus **3–4 sentences at most**.
5. **Plans contain no code.**
   - Plans list steps, files, interfaces by name, tests and edge cases. Never code blocks.
   - The code is written once, during implementation. This saves tokens and avoids drift.
   - Use the `kx-plan` skill, which overrides the code-in-plan default of `superpowers:writing-plans`.
6. **Git.**
   - Commit each finished feature.
   - **Never push to GitHub, create a repo, or publish without the owner's explicit yes** for that action.
7. **Work items** come from `docs/FEATURES.md` (P0–P15 for v1, then F1+), one item per session. The owner types `/kx-feature P<n>` or `/kx-feature F<n>`.
   - Each item's notes and the owner's saved ideas live in `docs/items/<id>.md`. Read it first; a finding for another item goes into that item's file.
   - When the owner says "idea for P13: …", save it there and carry on. `/kx-feature P13` builds saved ideas later.

## How we work

- **One module at a time**, in roadmap order. Each phase has a plan in `docs/superpowers/plans/`, written with `kx-plan`, before any code.
- **Test first** (`superpowers:test-driven-development`). State machines take an injected `Clock`. Never write a test that sleeps.
- Finish a module only when the **Definition of Done** in `docs/roadmap.md` is met. Use the `kx-module-done` skill.
- Before claiming anything works, run the checks and show their output (`superpowers:verification-before-completion`).
- Work on a branch or worktree, never directly on `main`. Commit messages use Conventional Commits and a DCO `Signed-off-by`.
- Structural change means a new ADR. Never edit an accepted ADR; supersede it.
- Before adding any dependency, font, icon set, word list or model, use the `kx-licence-check` skill.

## Never do this (invariants; `cargo xtask tidy` enforces most of them)

- Add `unsafe` outside `kx-platform-*`, except cxx-qt bridge blocks in `kx-ui/src/bridges/` and the test tools in `tools/` (ADR-0013, ADR-0014). Every `unsafe` block needs a `// SAFETY:` comment.
- Let a `kx-mod-*` crate depend on another module, on the kernel or on a platform crate.
- Add network, TLS or media-decoding crates to the `keyxtend` app. They belong in `keyxtend-worker` only (ADR-0004).
- Log typed text, clipboard content, secrets or transcripts. Use `Redacted<T>`.
- Build UI that needs scrolling, dragging, sliders or right-click (ADR-0008). Lists are paged, 5 rows per page.
- Call `requestActivate()` on a no-focus window, or drop the window guard or the `Qt.WindowDoesNotAcceptFocus` flag (ADR-0013).
- Take focus from the target app. Every window we own is non-activating, except Settings.
- Send dictated text, clipboard data or anything else to the network unless the user turned that feature on.
- Ship API keys, or add telemetry.

## Commands (after Phase 0)

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
cargo audit
cargo xtask tidy
```

## Windows specifics

- Testing uiAccess needs:
  - the dev self-signed certificate in Trusted Root;
  - a signed build;
  - an install under Program Files (`cargo xtask dev-install`, created in Phase 1).
- A plain `cargo run` build works, but it does not stay above the Start menu.
- Manual test results go to `docs/test-reports/YYYY-MM-DD-<module>.md`.

## Knowledge graph

Once code exists, run `/graphify` to build `graphify-out/`. After changing code, run `graphify update .`.
