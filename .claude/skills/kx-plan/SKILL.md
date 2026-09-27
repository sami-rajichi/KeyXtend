---
name: kx-plan
description: Use to write any KeyXtend implementation plan (a roadmap phase, a module or a feature from docs/FEATURES.md). Plans list steps only - no code blocks - to save tokens and avoid writing code twice. Overrides the code-in-plan default of superpowers:writing-plans.
---

# KeyXtend plan (steps, no code)

Follow the structure and discipline of `superpowers:writing-plans`: small tasks, TDD order, exact file paths, verification per task. There is one override, from the product owner: **no code in the plan**. No code blocks, no function bodies, no test bodies. Code is written once, during implementation.

## File
Save the plan as `docs/superpowers/plans/YYYY-MM-DD-<phase-or-feature>.md`. Keep it short: aim for under 150 lines.

## Sections
1. **Goal:** 1–2 sentences, and a link to the spec section or `docs/FEATURES.md` entry.
2. **Scope:** what is in and what is out, as bullets.
3. **Design notes:**
   - the crates and files touched;
   - new ports or traits **by name** with a one-line purpose;
   - new settings keys with their defaults (these go in the settings schema, never hardcoded);
   - new theme tokens and new translation keys.
4. **Tasks:** a numbered list. Each task has:
   - *Files*;
   - *Test first*: what the test proves, one sentence;
   - *Implement*: what to build, 1–3 sentences;
   - *Verify*: the exact command and the expected result.
5. **Edge cases:** a numbered list. Every item must map to a test; `kx-module-done` checks this.
6. **Security:** the capabilities needed, the data classes handled, the threats to test (from `kx-security-check`).
7. **Manual Windows check:** the steps the owner will confirm (for `kx-windows-manual-test`).
8. **Done when:** a link to the Definition of Done, plus anything specific to this plan.

## Rules
- Every sentence is 1–2 lines. Use no paragraphs.
- Name interfaces and files; describe behaviour. Never paste code.
- Ask the product owner to approve the plan with AskUserQuestion before implementing.
