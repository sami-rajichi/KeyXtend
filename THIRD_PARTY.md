# Third-party components

Everything we ship or bundle that we did not write ourselves. Updated by the `kx-licence-check` skill. Crates are also covered by `cargo deny` and the release SBOM.

| Component | Version | Licence | Source | Used for | Shipped |
|---|---|---|---|---|---|
| Slint | =1.18.1 | GPL-3.0-only option (of three) | https://github.com/slint-ui/slint | All windows (P1 candidate) | if it wins P1 |
| Qt 6 (base, declarative, svg, shader tools) | 6.11.2 | LGPL-3.0-only or GPL-2.0/3.0, as separate DLLs; shader tools bundle glslang and SPIRV-Cross (BSD-3-Clause, Apache-2.0, MIT) | https://doc.qt.io/qt-6/licenses-used-in-qt.html | All windows (P1 candidate) | if it wins P1 |
| cxx-qt | 0.10.0 | MIT OR Apache-2.0 | https://github.com/KDAB/cxx-qt | Rust bridge to Qt | if Qt wins P1 |
| Mesa llvmpipe with LLVM (Qt's `opengl32sw.dll`) | from Qt 6.11.2 | MIT and BSL-1.0; LLVM under its own licence | https://doc.qt.io/qt-6/licenses-used-in-qt.html | Software drawing fallback | if Qt wins P1 |
| Lucide icons | 1.48.0 (mock-up) | ISC (some MIT from Feather) | https://lucide.dev/license | Key and panel icons | yes |
| IBM Plex Sans / Sans Arabic | – | OFL-1.1 | https://github.com/IBM/plex | ET66 theme | yes |
| Noto Sans / Noto Sans Arabic | – | OFL-1.1 | https://github.com/notofonts | Native theme, Arabic | yes |
| Rubik | – | OFL-1.1 | https://github.com/googlefonts/rubik | Dolch theme | yes |
| whisper.cpp + Whisper weights | – | MIT | https://github.com/ggml-org/whisper.cpp | Local voice typing | models downloaded on demand |
| wordfreq data | – | CC-BY-SA-4.0 (data) | https://github.com/rspeer/wordfreq | Prediction seed lists | yes, separate files |
| Tesseract + tessdata_fast | – | Apache-2.0 | https://github.com/tesseract-ocr | OCR fallback | on demand |
| Inno Setup | 7.x (planned, P14) | Inno Setup License (zlib-style) | https://jrsoftware.org/isinfo.php | Windows installer | installer stub only |
| Contributor Covenant | 3.0 | CC BY-SA 4.0 | https://www.contributor-covenant.org/ | `CODE_OF_CONDUCT.md` | no |

The versions are filled in when each component is added. Planned items stay here so their licences are checked early.
- `deny.toml` allows `GPL-3.0-or-later` only. Slint's GPL option is `GPL-3.0-only`, so adding Slint to the workspace needs the owner's yes to allow it.
- Whichever toolkit wins P1 gets a full licence review of everything it ships before P2.

## Test tools (P1 only, never shipped)

| Tool | Version | Licence | Source | Used for |
|---|---|---|---|---|
| aqtinstall | master `076e165` | MIT | https://github.com/miurahr/aqtinstall | Downloads Qt |
| PresentMon | 2.6.0 | MIT | https://github.com/GameTechDev/PresentMon | Frame times (G8) |
| NVDA (portable) | 2026.2 | GPL-2.0 with exceptions | https://github.com/nvaccess/nvda | Screen-reader check (G10) |
| Tauri | 2.12.0 | MIT OR Apache-2.0 | https://github.com/tauri-apps/tauri | Quick comparison only |
| cpal | 0.18.2 | Apache-2.0 | https://github.com/RustAudio/cpal | Microphone in the spike worker (G22) |
| whisper.cpp Windows build (`whisper-server`) | b5130 (v1.9.4) | MIT; the bundle also has `SDL2.dll` (Zlib) | https://github.com/ggml-org/whisper.cpp | Local speech-to-text server (G22) |
| Whisper `base` model (`ggml-base.bin`) | SHA-256 `60ed5bc3…2efe` | MIT | https://huggingface.co/ggerganov/whisper.cpp | Local speech model (G22) |
