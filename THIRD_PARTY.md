# Third-party components

Everything we ship or bundle that we did not write ourselves. Updated by the `kx-licence-check` skill. Crates are also covered by `cargo deny` and the release SBOM.

| Component | Version | Licence | Source | Used for | Shipped |
|---|---|---|---|---|---|
| Slint | =1.18.1 | GPL-3.0-only option (of three) | https://github.com/slint-ui/slint | P1 candidate, dropped (ADR-0013) | no |
| Qt 6 (base, declarative, svg, shader tools) | 6.11.2 | LGPL-3.0-only or GPL-2.0/3.0, as separate DLLs; shader tools bundle glslang and SPIRV-Cross (BSD-3-Clause, Apache-2.0, MIT) | https://doc.qt.io/qt-6/licenses-used-in-qt.html | All windows (ADR-0013) | yes, as separate DLLs |
| cxx-qt | 0.10.0 | MIT OR Apache-2.0 | https://github.com/KDAB/cxx-qt | Rust bridge to Qt | yes, built in |
| Mesa llvmpipe with LLVM (Qt's `opengl32sw.dll`) | from Qt 6.11.2 | MIT and BSL-1.0; LLVM under its own licence | https://doc.qt.io/qt-6/licenses-used-in-qt.html | Software drawing fallback | yes |
| Lucide icons (37 SVGs from `lucide-static`) | 1.48.0 | ISC (some MIT from Feather) | https://lucide.dev/license | Key, panel, pill and voice icons | yes |
| IBM Plex Sans / Sans Arabic | 3.201 / 1.101 (Google Fonts) | OFL-1.1, Reserved Font Name "Plex" | https://github.com/IBM/plex | ET66 theme | yes, unmodified |
| Noto Sans / Noto Sans Arabic | – / 2.012 (Google Fonts) | OFL-1.1 | https://github.com/notofonts | Native theme, Arabic fallback | yes |
| Rubik | 2.300 (Google Fonts) | OFL-1.1 | https://github.com/googlefonts/rubik | Dolch theme (Latin only; its Arabic uses Noto Sans Arabic) | yes |
| whisper.cpp + Whisper weights | – | MIT | https://github.com/ggml-org/whisper.cpp | Local voice typing | models downloaded on demand |
| whisper.cpp Windows build (`whisper-server`) | b5130 (v1.9.4) | MIT; the bundle also has `SDL2.dll` (Zlib) | https://github.com/ggml-org/whisper.cpp | Local speech server in the voice worker (tested in G22) | P11 decides: shipped or downloaded |
| cpal | 0.18.2 | Apache-2.0 | https://github.com/RustAudio/cpal | Microphone in the voice worker (tested in G22) | yes, built into the worker (P11) |
| wordfreq data | – | CC-BY-SA-4.0 (data) | https://github.com/rspeer/wordfreq | Prediction seed lists | yes, separate files |
| Tesseract + tessdata_fast | – | Apache-2.0 | https://github.com/tesseract-ocr | OCR fallback | on demand |
| Inno Setup | 7.x (planned, P14) | Inno Setup License (zlib-style) | https://jrsoftware.org/isinfo.php | Windows installer | installer stub only |
| Contributor Covenant | 3.0 | CC BY-SA 4.0 | https://www.contributor-covenant.org/ | `CODE_OF_CONDUCT.md` | no |

The versions are filled in when each component is added. Planned items stay here so their licences are checked early.
- Qt gets a full licence review of everything it ships (`kx-licence-check`) before it enters the product workspace in P3.
- Qt's LGPL-3.0 DLLs ship unmodified and replaceable, with their licence texts.

## Test tools (never shipped)

| Tool | Version | Licence | Source | Used for |
|---|---|---|---|---|
| aqtinstall | master `076e165` | MIT | https://github.com/miurahr/aqtinstall | Downloads Qt |
| PresentMon | 2.6.0 | MIT | https://github.com/GameTechDev/PresentMon | Frame times (G8) |
| NVDA (portable) | 2026.2 | GPL-2.0 with exceptions | https://github.com/nvaccess/nvda | Screen-reader check (G10) |
| Tauri | 2.12.0 | MIT OR Apache-2.0 | https://github.com/tauri-apps/tauri | Quick comparison only |
| Whisper `base` model (`ggml-base.bin`) | SHA-256 `60ed5bc3…2efe` | MIT | https://huggingface.co/ggerganov/whisper.cpp | Local speech model (G22) |
