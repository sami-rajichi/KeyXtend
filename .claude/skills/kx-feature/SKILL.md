---
name: kx-feature
description: Use when the owner types /kx-feature P<n> or /kx-feature F<n>, or asks to build or improve a numbered item from docs/FEATURES.md in KeyXtend (P = v1 roadmap phase, F = later feature). Runs the whole flow - questions, plan without code, test-first build, review, security, manual check, work log and commit - for one item per session. On a finished item it builds the owner's saved ideas.
---

# Build or improve item P<n> or F<n>

- The argument is a roadmap phase (`P0`–`P15`, details in `docs/roadmap.md`) or a feature (`F1`, `F2`…).
- If it is missing, show the `Ready` items and the finished items with saved ideas in an AskUserQuestion pop-up, and ask which one.
- **P1 is done** (ADR-0013). Phases P2–P13 first move their tested spike parts into the product, following `docs/spike-move-map.md` (ADR-0014), then harden and finish them.
- **One item per session.** When this item is recorded, stop; the next item starts in a new session.

## Modes
- **Build:** the status in `docs/FEATURES.md` is `Ready` and its **needs** are done.
- **Improve:** the status is `Done` and `docs/items/<id>.md` has saved ideas not yet built.
- Otherwise tell the owner what blocks it, in 1–2 sentences, and stop.

## Preconditions
- Read `docs/items/<id>.md` first; create it from `docs/items/README.md` if it is missing.
- Earlier work is merged: if an earlier item's pull request is still open, read its checks once and ask the owner (pop-up) to merge it before starting.
- Start from the latest `main`, on a new branch or worktree (`feat/<id>-<slug>`, or `improve/<id>-<slug>`) created with `superpowers:using-git-worktrees`.

## Flow
1. **Understand.** Read the item's notes, its feature entry, the linked spec sections and `ARCHITECTURE.md`. In Improve mode, ask which saved ideas to build now (multi-select pop-up).
2. **Ask.** Use `superpowers:brainstorming` with AskUserQuestion pop-ups: several questions per round, each with a recommended option. For parts already built, ask only about what is new or what the owner wants changed. Stop when the behaviour is clear.
3. **Plan.** Write the plan with `kx-plan` (steps only, no code). Get the owner's approval.
4. **Build.** If the feature needs a new module, use `kx-new-module`. Implement test-first (`superpowers:test-driven-development`), task by task (`superpowers:subagent-driven-development`). All values go in settings, tokens or translations; nothing is hardcoded.
5. **Review.** Run `kx-review` on every changed file, then `kx-security-check` for the feature. Fix every finding.
6. **Check on Windows.** Run `kx-windows-manual-test` with the owner.
7. **Done gate.** Run `kx-module-done`, and paste the evidence.
8. **Record.**
   - Add a `docs/WORKLOG.md` entry (timestamp + 3–4 sentences).
   - Build mode: set the status in `docs/FEATURES.md` to `Done (vX.Y)`, and mark the next item `Ready`.
   - Update `docs/items/<id>.md`: remove handled notes, mark built ideas `Built (date)`. A finding for another item goes into that item's file.
   - Add a line to `CHANGELOG.md` under Unreleased.
9. **Commit** with Conventional Commits and `Signed-off-by`. **Ask the product owner before pushing** or opening a pull request (AskUserQuestion), and push only after a yes.
10. **Hand off.** Update the local `docs/NEXT-SESSION.md`, tell the owner the next command (for example `/kx-feature P3`), and stop.

## Ideas at any time
When the owner says "idea for <id>: …" during any session, add it with the date under **Ideas** in `docs/items/<id>.md`, commit it with the current work, and carry on. It is built later with `/kx-feature <id>`.
