# 6. Licence: GPL-3.0-or-later for the app

- **Date:** 2026-09-26
- **Status:** Accepted for now (2026-09-26). Revisit before v1.0.

## Context
- The project is published on GitHub for anyone to use.
- Slint is free for open-source applications under GPLv3; its alternative, the royalty-free licence, requires attribution and is not OSI-approved.
- The SignPath Foundation (free code signing) requires OSI licences and no proprietary components.
- Our dependencies are MIT, Apache-2.0, ISC, MPL-2.0, OFL (fonts) and CC-BY-SA (word lists as data). All can be combined with GPLv3.

## Decision
- The application and the repository as a whole: **GPL-3.0-or-later**.
- Reusable library crates that we may publish (prediction engine, voice wrapper): **MIT OR Apache-2.0**.
- Contributions under the DCO (`Signed-off-by`), with no CLA.
- Fonts, icons, word lists and models keep their own licences. They are listed in `THIRD_PARTY.md`, and their licence texts are shipped.

## Option considered: permissive source, GPLv3 binaries
Slint's licence says: "Use Slint for free under the GPLv3 on any platform; your own files can stay MIT or Apache-2.0". Its FAQ adds that the work *as a whole* must be under the GPL.

So we could license our source as MIT OR Apache-2.0 and distribute the binaries under GPLv3. That is more flexible: forks could choose Slint's royalty-free licence, which requires an AboutSlint widget or a "Made with Slint" badge. But it also allows closed-source derivatives of our code.

We recommend GPL-3.0-or-later for the whole app: it is simpler, and it keeps derivatives of an accessibility tool open, as OptiKey (GPL-3.0) does. The owner decides. Either way, add the "Made with Slint" badge to the releases page.

## Owner's direction (2026-09-26)

For now: open source, free for everyone, code and app. The owner may later prefer to keep the code private and publish only the app.

What keeps that option open:
- As the copyright holder, the owner can publish **future** versions under other terms. Versions already published under the GPL stay GPL.
- **Outside code contributions would block that**, unless each contributor signs an agreement allowing relicensing (a CLA). A DCO sign-off is not enough. So until the licence is final, we do not merge outside code.
- A closed app would need Slint's royalty-free licence (desktop allowed, with an AboutSlint notice) or a paid Slint licence.

Where to publish: GitHub Releases, then winget and Scoop, then the Microsoft Store (free developer account; the Store signs the MSIX, but uiAccess needs Microsoft's approval, which is unverified). Android's Play Store does not apply to a Windows keyboard.

## Consequences
- **Banned:** GPL-2.0-only code, which is incompatible with Apache-2.0 in GPLv3 combinations. Also banned: AGPL models (for example Vosk ar-linto), and "community" or non-commercial model licences (for example Moonshine Arabic tiny/base).
- `deny.toml` allow-list: MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-2/3-Clause, ISC, Zlib, Unicode-3.0, MPL-2.0, CC0-1.0, OFL-1.1, GPL-3.0-or-later, LGPL (dynamic only, reviewed case by case).
