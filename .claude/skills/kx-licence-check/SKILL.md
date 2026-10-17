---
name: kx-licence-check
description: Use before adding any Rust crate, font, icon, sound, word list, dictionary, ML model or other third-party file to the KeyXtend project, and before each release. Keeps the project legally clean for a closed, sold app.
---

# Licence check

KeyXtend is view only, all rights reserved, and is sold as a closed app (ADR-0030; confirm the current decision there). Every third-party part that ships in the app or links into it must allow that; build, test and CI tools that never ship keep their own licences.

1. Find the exact licence **for the version you are adding**: `Cargo.toml` `license` field plus the LICENSE file. For models and data, read the model card and the dataset licence. For fonts, read OFL.txt and the Reserved Font Names.
2. **Allowed:**
   - MIT, Apache-2.0 (also WITH LLVM-exception), BSD-2/3-Clause, ISC, Zlib, Unicode-3.0, MPL-2.0, CC0-1.0, OFL-1.1;
   - LGPL only as a separate, replaceable library, reviewed; the app's licence must let users replace it and reverse engineer to debug that (LGPL-3.0 §4, LGPL-2.1 §6);
   - CC-BY-4.0 and CC-BY-SA-4.0 for **data files** only, shipped as separate files with attribution;
   - Qt's bundled parts, checked against Qt's SBOM: permissive notice licences inside Qt are accepted (owner, 2026-09-29) and recorded per DLL with their credits; copyleft-only parts are still refused.
3. **Refuse:**
   - any GPL or AGPL (for example Vosk ar-linto); a dual licence is fine when we take its permissive side;
   - "community", non-commercial, research-only or custom model licences (for example Moonshine Arabic tiny/base);
   - anything without a licence;
   - anything that requires telemetry, or that sends data somewhere by default (MediaPipe Tasks, ONNX Runtime usage data: disable it, or do not use it).
4. **Record it:**
   - add the item to `THIRD_PARTY.md` (name, version, licence, source URL, what it is used for, and whether it is shipped in binaries);
   - if it ships, add its licence text to `licenses/`.
5. For crates, also run `cargo deny check licenses`. If the licence is not in `deny.toml`'s allow-list, stop and ask the product owner (AskUserQuestion) before widening the list.
6. Fonts: never rename a modified or subset OFL font to a Reserved Font Name. Keep the copyright notice.
