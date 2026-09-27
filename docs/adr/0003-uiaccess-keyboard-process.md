# 3. The keyboard process itself holds uiAccess

- **Date:** 2026-09-26
- **Status:** Accepted

## Context
The product owner's top requirement: the keyboard is **above every window ever**, including the Start menu, Search and Task Manager, and it types into admin windows.

Windows z-order bands give that position only to windows owned by processes whose manifest has `uiAccess="true"`. osk.exe uses the same mechanism. Windows grants uiAccess only when all three hold:
1. The executable is Authenticode-signed with a certificate the machine trusts.
2. It is installed in a secure location (`%ProgramFiles%`, `%ProgramFiles(x86)%` or `%SystemRoot%\system32`).
3. Its manifest sets the flag.

The signature is checked even when the secure-location policy is turned off.

## Decision
- `keyxtend.exe` (the keyboard UI process) is built with `uiAccess="true"`, signed, and installed under Program Files.
- **Development:**
  - a self-signed code-signing certificate imported into the dev machine's Trusted Root store;
  - signtool;
  - install to Program Files with a dev script.
  - The certificate is removed when no longer needed.
- **Public releases:** apply to the SignPath Foundation (free for OSI-licensed projects built in CI). Fallbacks: a Certum open-source certificate; the Microsoft Store (uiAccess is a restricted capability, so approval is unverified).

## Consequences
- The uiAccess process can bypass UIPI, so it is a high-value target. It must stay small, with **no network and no parsing of untrusted data** (ADR-0004).
- A portable (unsigned, not installed) build still runs, but without the top band. It shows a banner explaining this.
- SmartScreen warns about new signed apps until they build reputation. The README explains this.
