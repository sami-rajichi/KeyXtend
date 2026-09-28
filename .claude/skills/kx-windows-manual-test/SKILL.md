---
name: kx-windows-manual-test
description: Use when a module or phase needs verification on a real Windows 11 PC - things CI cannot test, like uiAccess z-order, admin windows, focus, real apps (Notepad, Word, Chrome, Terminal), DPI and Narrator. Produces docs/test-reports/<date>-<topic>.md.
---

# Manual Windows test

Give **one short step at a time**. Prefer steps you can run yourself (PowerShell, scripts), and ask the product owner only to look and confirm. Collect the answers through AskUserQuestion (Pass, Fail, or Something else).

## Setup (once per machine)
1. The dev certificate: `cargo xtask dev-cert` creates a self-signed code-signing certificate and imports it into LocalMachine\Root and TrustedPublisher. Admin approval is needed; explain to the owner what it does before asking.
2. `cargo xtask dev-install`: builds release, signs, and copies to `C:\Program Files\KeyXtend-dev\`.
3. Confirm uiAccess is active: the process token has `TokenUIAccess` (`cargo xtask check-uiaccess`).

## Test script
- Build it from the module plan's checklist, plus the relevant gates G1–G16 in `docs/roadmap.md`.
- Always include these regression checks:
  - keys never take focus;
  - the keyboard stays above the Start menu;
  - Right-click mode leaves short clicks normal.
- Each step: **Action**, **Expected result**, **Result** (Pass/Fail), **Notes**. Take screenshots where useful (Snipping Tool or our own Snip).

## Report
- Write `docs/test-reports/YYYY-MM-DD-<topic>.md` containing: Windows build (`winver`), display scale, the apps and versions tested, and the table of results.
- File a GitHub issue for each failure, linked from the report.
- Remove the dev certificate from Trusted Root when a test cycle ends, if the owner prefers (`cargo xtask dev-cert --remove`).
