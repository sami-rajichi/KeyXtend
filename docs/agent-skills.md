# Agent skills for this project

Skills are instruction packs that Claude loads for a type of task. There are two groups.

## Already installed (superpowers plugin and built-ins)

| When | Skill |
|---|---|
| Any new idea or feature before code | `superpowers:brainstorming` |
| Turning an approved design into steps | `kx-plan` (the `superpowers:writing-plans` structure, but no code in plans) |
| Carrying out a plan in this session, task by task with review | `superpowers:subagent-driven-development` |
| Carrying out a plan in a separate session | `superpowers:executing-plans` |
| Writing any code | `superpowers:test-driven-development` |
| Any bug or failing test | `superpowers:systematic-debugging` |
| Before saying "done" | `superpowers:verification-before-completion` |
| Isolated branch per module | `superpowers:using-git-worktrees` |
| Independent tasks in parallel | `superpowers:dispatching-parallel-agents` |
| Asking for / answering review | `superpowers:requesting-code-review`, `superpowers:receiving-code-review` |
| Merging a module | `superpowers:finishing-a-development-branch` |
| Creating or improving our own skills | `superpowers:writing-skills` |
| Review of a branch | `/code-review` (`/code-review ultra` for a deep multi-agent cloud review, billed) |
| Security review of a branch | `/security-review` |
| Code knowledge graph | `/graphify` (once code exists) |

## Project skills (in `.claude/skills/`, 2026-09-26)

| Skill | Use it when |
|---|---|
| `kx-feature` | **Main entry point:** `/kx-feature P3` or `/kx-feature F2` runs the whole flow for one item from `docs/FEATURES.md`, one item per session; on a finished item it builds the owner's saved ideas |
| `kx-plan` | Writing any plan: steps only, no code |
| `kx-review` | Before every commit: checks every changed file for pattern, correctness, tests, security, no hardcoding, short texts, small files |
| `kx-new-module` | Starting any `kx-mod-*` crate or platform adapter |
| `kx-module-done` | Before calling a module, phase or PR finished |
| `kx-security-check` | Designing or finishing a module; touching IPC, secrets, hooks, input, network |
| `kx-licence-check` | Before adding any crate, font, icon, word list or model; before a release |
| `kx-windows-manual-test` | Verifying on a real Windows PC (uiAccess, admin windows, real apps) |
| `kx-release` | Preparing a tagged release |
| `kx-theme` | Adding or changing a theme, colour, font, spacing or motion token |

## Suggested later

- **`kx-spike-report`:** turns spike findings into ADR updates.
- **`kx-i18n`:** adding or reviewing EN/FR/AR UI strings and RTL mirroring.
- **`kx-perf-budget`:** measures size, memory, start time and latency against spec §10 and fails when a budget is exceeded.
