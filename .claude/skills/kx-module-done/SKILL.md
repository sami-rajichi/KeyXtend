---
name: kx-module-done
description: Use before declaring any KeyXtend module, phase or pull request finished. Walks the Definition of Done from docs/roadmap.md, runs every quality gate and shows the evidence. Never claim done without it.
---

# Module Definition of Done gate

Run everything below and **paste the real output**. If any step fails, the module is not done. Fix it, or report exactly what is missing.

1. **Quality gates**, all green:
   ```
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo deny check
   cargo audit
   cargo xtask tidy
   ```
2. **Tests cover the plan:**
   - open the phase plan's "Edge cases" list and point to the test that covers each one;
   - every state machine has a property test;
   - every Slint view has a UI test that finds its controls by accessible label.
3. **Security:** `docs/security/<module>.md` is complete (run `kx-security-check` if not). No open "must fix" items.
4. **Accessibility:**
   - go through every view and confirm each action works with short left clicks only (no scroll, drag or right-click);
   - check contrast in light and dark (≥ 4.5:1);
   - Narrator can read and activate the controls.
5. **Manual Windows check:** `docs/test-reports/<date>-<module>.md` exists and records a pass on a real Windows 11 PC (use `kx-windows-manual-test`). If you cannot run it, say so plainly and leave the module "awaiting manual test".
6. **Review:**
   - run `kx-review` on every changed file; it must be all ✅;
   - run `/code-review` and `/security-review` on the branch;
   - resolve or explicitly accept every finding in the PR description.
7. **Docs:**
   - `CHANGELOG.md` (Unreleased), the module `README.md`, the user-facing README section, and a new ADR if the design changed;
   - one `docs/WORKLOG.md` entry: a timestamp plus 3–4 sentences at most.
8. **Hand off:** use `superpowers:finishing-a-development-branch`.

Report the result as a checklist with ✅/❌ and a link to the evidence for each item.
