---
name: kx-feature
description: Use when the owner types /kx-feature P<n> or /kx-feature F<n>, or asks to build a numbered item from docs/FEATURES.md in KeyXtend (P = v1 roadmap phase, F = later feature). Runs the whole flow - questions, plan without code, test-first build, review, security, manual check, work log and commit - one item at a time.
---

# Build item P<n> or F<n>

- The argument is a roadmap phase (`P0`–`P15`, details in `docs/roadmap.md`) or a feature (`F1`, `F2`…).
- If it is missing, show the `Ready` items from `docs/FEATURES.md` in an AskUserQuestion pop-up and ask which one.
- **P1 is the throwaway toolkit test round.** It lives on the `spike/toolkits` branch and produces numbers in ADR-0011, not merged code.

## Preconditions
- The feature's **status** in `docs/FEATURES.md` is `Ready` and its **needs** are done. If not, tell the product owner what blocks it, in 1–2 sentences, and stop.
- `main` is green, and you are on a new branch or worktree (`feat/F<n>-<slug>`) created with `superpowers:using-git-worktrees`.

## Flow
1. **Understand.** Read the feature entry, the linked spec sections and `ARCHITECTURE.md`.
2. **Ask.** Use `superpowers:brainstorming` with AskUserQuestion pop-ups: several questions per round, each with a recommended option. Stop when the behaviour is clear.
3. **Plan.** Write the plan with `kx-plan` (steps only, no code). Get the owner's approval.
4. **Build.** If the feature needs a new module, use `kx-new-module`. Implement test-first (`superpowers:test-driven-development`), task by task (`superpowers:subagent-driven-development`). All values go in settings, tokens or translations; nothing is hardcoded.
5. **Review.** Run `kx-review` on every changed file, then `kx-security-check` for the feature. Fix every finding.
6. **Check on Windows.** Run `kx-windows-manual-test` with the owner.
7. **Done gate.** Run `kx-module-done`, and paste the evidence.
8. **Record.**
   - Add a `docs/WORKLOG.md` entry (timestamp + 3–4 sentences).
   - Set the item's status in `docs/FEATURES.md` to `Done (vX.Y)`, and mark the next item `Ready`.
   - Add a line to `CHANGELOG.md` under Unreleased.
9. **Commit** with Conventional Commits and `Signed-off-by`. **Ask the product owner before pushing** or opening a pull request (AskUserQuestion), and push only after a yes.
