# 12. Installer: Inno Setup 7

- **Date:** 2026-09-26
- **Status:** Accepted (product owner, 2026-09-26, decision D8).

## Context
- uiAccess needs a per-machine install under Program Files, a signed binary, a winget silent install, and an uninstaller.
- The download budget is 80 MB.
- The owner may later ship a free closed binary.

## Decision
Inno Setup 7 (latest 7.1.0, 2026-08-12). The installer is built in P14.

## Reasons
- The licence is permissive (zlib-style, "any purpose, including commercial").
- Since 6.5.0 ([changelog](https://jrsoftware.org/files/is6-whatsnew.htm)), a paid licence is only *requested* from commercial users above USD 5,000 a year, and is not required ([terms](https://jrsoftware.org/isorder.php)).
- winget supports the `inno` install type and adds the silent switches itself.
- Arabic and French translations are official ([official translations](https://jrsoftware.org/files/istrans/)).
- VS Code and Git for Windows both use it.

## Options considered
- **NSIS 3.12** (zlib): the fallback, through cargo-packager if needed.
- **WiX 7** (MS-RL; OSMF EULA with a fee above $10k a year of revenue): needs the .NET SDK.
- **MSIX:** uiAccess apps cannot run from MSIX.
- **Velopack:** Setup.exe installs per-user only; per-machine installs need its MSI, which is built with WiX, so it brings WiX's costs.

## Consequences
- Add Inno Setup to `THIRD_PARTY.md` when P14 adds it.
- Buy the optional commercial licence only if closed-source revenue ever passes $5k a year.

## Sources
- https://jrsoftware.org/files/is/license.txt
- https://jrsoftware.org/files/is6-whatsnew.htm
- https://jrsoftware.org/isorder.php
- https://jrsoftware.org/files/is7-whatsnew.htm
- https://learn.microsoft.com/en-us/windows/package-manager/package/manifest
- https://jrsoftware.org/files/istrans/
- https://github.com/microsoft/vscode/tree/main/build/win32
- https://github.com/git-for-windows/build-extra/tree/main/installer
- https://github.com/wixtoolset/issues/issues/8974
- https://robmensching.com/blog/posts/2026/02/04/osmf-v11/
- https://nsis.sourceforge.io/License
- https://github.com/microsoft/msix-packaging/issues/486
- https://docs.velopack.io/packaging/installer
