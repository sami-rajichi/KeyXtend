---
name: kx-review
description: Use before every commit in KeyXtend, and whenever the owner asks for a review. Reviews every changed file against the owner's rules - design pattern, correctness, tests, security, nothing hardcoded, short names and texts, small files - and reports a pass/fail table.
---

# KeyXtend file-by-file review

Review **every** changed file: `git diff --name-only main...HEAD` plus uncommitted changes. Read each file in full, not only the diff. For large changes, give each file to its own reviewer subagent in parallel and merge the results.

## Checks per file

| # | Check | Fails when |
|---|---|---|
| 1 | **Pattern** | It breaks the hexagonal layering, module isolation, the enum state machine or the actor/handle pattern (ARCHITECTURE.md), or it duplicates logic that already exists |
| 2 | **Correct** | Logic errors, unhandled `Result`, a panic on outside input, races, a blocking call in the mouse hook or on the UI thread |
| 3 | **Tested** | New logic has no unit test, a state machine has no property test, an edge case listed in the plan has no test, or a test sleeps |
| 4 | **Secure** | It fails anything in `kx-security-check` §3: `unsafe`, logging secrets, unchecked sizes, network or untrusted parsing in `keyxtend`, missing zeroize |
| 5 | **No hardcode** | A literal timing, size, colour, font, speed, limit, path, URL or user-visible string appears outside settings defaults, theme tokens, translation files or adapter tables. `0`, `1` and obvious indices are fine |
| 6 | **Short** | A name is unclear or overly long; a comment, doc string, log line or UI text is longer than 2 sentences |
| 7 | **Small** | A file is over 400 lines (aim ≤ 300) or a function is over 40 lines |
| 8 | **Accessible** | UI needs scroll, drag, a slider or right-click; a control has no accessible label; contrast is below 4.5:1 |
| 9 | **Docs** | A public item has no one-line doc; the design changed without an ADR; `docs/WORKLOG.md` has no entry |

## Also run
`cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo xtask tidy`. Paste the real output.

## Report (keep it short)
- A table with one row per file and ✅/❌ for each of the 9 checks.
- Then only the failures: `file:line`, what is wrong, the fix, one or two sentences each.
- Fix every ❌ before committing. If a finding is accepted rather than fixed, say why in one sentence.
