---
name: kx-licence-check
description: Use before adding any Rust crate, font, icon, sound, word list, dictionary, ML model or other third-party file to the KeyXtend project, and before each release. Keeps the project legally clean for GitHub distribution and SignPath signing.
---

# Licence check

The project licence is GPL-3.0-or-later (ADR-0006; confirm the current decision there).

1. Find the exact licence **for the version you are adding**: `Cargo.toml` `license` field plus the LICENSE file. For models and data, read the model card and the dataset licence. For fonts, read OFL.txt and the Reserved Font Names.
2. **Allowed:**
   - MIT, Apache-2.0 (also WITH LLVM-exception), BSD-2/3-Clause, ISC, Zlib, Unicode-3.0, MPL-2.0, CC0-1.0, OFL-1.1, GPL-3.0(-or-later);
   - LGPL only when dynamically linked and reviewed;
   - CC-BY-4.0 and CC-BY-SA-4.0 for **data files** only, shipped as separate files with attribution.
3. **Refuse:**
   - GPL-2.0-only; AGPL (for example Vosk ar-linto);
   - "community", non-commercial, research-only or custom model licences (for example Moonshine Arabic tiny/base);
   - anything without a licence;
   - anything that requires telemetry, or that sends data somewhere by default (MediaPipe Tasks, ONNX Runtime usage data: disable it, or do not use it).
4. **Record it:**
   - add the item to `THIRD_PARTY.md` (name, version, licence, source URL, what it is used for, and whether it is shipped in binaries);
   - if it ships, add its licence text to `licenses/`.
5. For crates, also run `cargo deny check licenses`. If the licence is not in `deny.toml`'s allow-list, stop and ask the product owner (AskUserQuestion) before widening the list.
6. Fonts: never rename a modified or subset OFL font to a Reserved Font Name. Keep the copyright notice.
