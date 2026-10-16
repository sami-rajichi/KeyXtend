# Third-party components

Everything we ship or bundle that we did not write ourselves. Updated by the `kx-licence-check` skill. Crates are also covered by `cargo deny` and the release SBOM.

| Component | Version | Licence | Source | Used for | Shipped |
|---|---|---|---|---|---|
| Slint | =1.18.1 | GPL-3.0-only option (of three) | https://github.com/slint-ui/slint | P1 candidate, dropped (ADR-0013) | no |
| Qt 6 modules, the 12 DLLs `cargo xtask stage` ships: Core, Gui, Network, OpenGL, Qml, Qml Meta, Qml Models, Qml Worker Script, Quick, Quick Effects, Quick Shapes and Svg. Plugins: `platforms/qwindows.dll`, `imageformats/qsvg.dll`, and the QML plugins of `QtQuick`, `QtQuick.Effects` and `QtQuick.Shapes`, with their `qmldir` files and the one of `QtQml` | 6.11.2 | LGPL-3.0-only (Qt also offers GPL-2.0-only, GPL-3.0-only or commercial), as separate DLLs | https://doc.qt.io/qt-6/licenses-used-in-qt.html | All windows (ADR-0013); `qsvg` draws the SVG icons | yes, as separate DLLs |
| Parts inside `Qt6Core.dll` | Qt 6.11.2 | `LicenseRef-BSD-3-Clause-with-PCRE2-Binary-Like-Packages-Exception` (PCRE2 10.47); `BSD-2-Clause` (PCRE2 JIT, SHA-3 endian helper); `BSD-3-Clause` (double-conversion, easing curves, RFC 6234 SHA-384/512); `Zlib` (zlib 1.3.2); `Unicode-3.0` (Unicode data, CLDR); `MIT` (TinyCBOR); `Apache-2.0` (Tika MIME types); `CC0-1.0 OR Apache-2.0` (BLAKE2); `CC0-1.0` (MD4, MD5, Keccak, SipHash, tl::expected); `LicenseRef-SHA1-Public-Domain` (SHA-1) | Qt's SBOM, `sbom/qtbase-6.11.2.spdx.json` | Text, regular expressions, hashes | yes, inside the DLL |
| Parts inside `Qt6Gui.dll` | Qt 6.11.2 | `FTL` (FreeType 2.14.3 and its gray rasterizer, offered as `FTL OR GPL-2.0-only`; we take `FTL`); `MIT` (HarfBuzz 14.3.0, FreeType's BDF reader, MD4C, the D3D12 and Vulkan memory allocators, the D3D12 mipmap generator, OpenGL and OpenGL ES headers, WebGradients); `MIT AND MIT-open-group` (FreeType's PCF reader); `Libpng AND libpng-2.0` (libpng 1.6.58); `Zlib` (zlib); `BSD-3-Clause` (Adobe Glyph List); `BSD-2-Clause AND Imlib2` (smooth scaling); `X11 AND HPND` (X Server helper); `Apache-2.0` (Emoji Segmenter); `Apache-2.0 OR MIT` (Vulkan registry); `LicenseRef-ICC-License` (sRGB colour profile) | Qt's SBOM, `sbom/qtbase-6.11.2.spdx.json` | Fonts, text shaping, images, drawing | yes, inside the DLL |
| Parts inside `Qt6Network.dll` | Qt 6.11.2 | `BSD-3-Clause` (libpsl); `MPL-2.0` (Public Suffix List data); `Zlib` (zlib) | Qt's SBOM, `sbom/qtbase-6.11.2.spdx.json` | Imported by Qt Qml and Qt Quick | yes, inside the DLL |
| Parts inside `Qt6Qml.dll` | Qt 6.11.2 | `BSD-2-Clause` (JavaScriptCore macro assembler) | Qt's SBOM, `sbom/qtdeclarative-6.11.2.spdx.json` | The QML engine | yes, inside the DLL |
| Parts inside `Qt6Svg.dll` | Qt 6.11.2 | `HPND-sell-variant` (XSVG) | Qt's SBOM, `sbom/qtsvg-6.11.2.spdx.json` | SVG drawing | yes, inside the DLL |
| Parts inside `platforms/qwindows.dll` | Qt 6.11.2 | `LicenseRef-Lcs-Telegraphics` (Wintab API); it reaches FreeType through `Qt6Gui.dll` and holds no copy | Qt's SBOM, `sbom/qtbase-6.11.2.spdx.json` | Pen tablet input | yes, inside the plugin |
| Parts inside `imageformats/qjpeg.dll` | Qt 6.11.2 | `IJG AND BSD-3-Clause` (libjpeg-turbo 3.2.0) | Qt's SBOM, `sbom/qtbase-6.11.2.spdx.json` | JPEG images | only if `qjpeg.dll` ships |
| Mesa llvmpipe with LLVM (Qt's `opengl32sw.dll`) | Mesa 11.2.2, LLVM 3.6.2 | MIT and BSL-1.0; LLVM under its own licence. Not in Qt's SBOM, so it is checked before it ever ships | https://doc.qt.io/qt-6/licenses-used-in-qt.html | Software drawing fallback | only if `opengl32sw.dll` ships |
| cxx-qt, cxx-qt-lib | 0.10.0 | MIT OR Apache-2.0 | https://github.com/KDAB/cxx-qt | Rust bridge to Qt | yes, built in |
| cxx | 1.0.202 | MIT OR Apache-2.0 | https://github.com/dtolnay/cxx | The C++ bridge under cxx-qt | yes, built in |
| cxx-qt-build, cxx-qt-gen, cxx-qt-macro, qt-build-utils, cxx-build, cxx-gen, cxxbridge-flags, cxxbridge-macro, cxxbridge-cmd | 0.10.0 (cxx-build and the cxxbridge crates 1.0.202, cxx-gen 0.7.202) | MIT OR Apache-2.0 | https://github.com/KDAB/cxx-qt, https://github.com/dtolnay/cxx | Generate the bridge code | build time only, not linked in |
| serde | 1 | MIT OR Apache-2.0 | https://github.com/serde-rs/serde | Reading and writing settings (P2) | yes, built in |
| toml | 1.1.6 | MIT OR Apache-2.0 | https://github.com/toml-rs/toml | The settings file format (P2) | yes, built in |
| thiserror | 2.0.21; 1.0.69 through cxx-qt | MIT OR Apache-2.0 | https://github.com/dtolnay/thiserror | Error types in the libraries (P2) | yes, built in |
| tracing, tracing-subscriber | 0.1.44, 0.3.23 | MIT | https://github.com/tokio-rs/tracing | The redacted log (P2) | yes, built in |
| windows and its sub-crates: windows-core, windows-collections, windows-future, windows-implement, windows-interface, windows-link, windows-numerics, windows-result, windows-strings, windows-sys, windows-threading | 0.62.2; sub-crates 0.62.2, 0.3.2, 0.3.2, 0.60.2, 0.59.3, 0.2.1, 0.3.1, 0.4.1, 0.5.1, 0.61.2, 0.2.1 | MIT OR Apache-2.0 | https://github.com/microsoft/windows-rs | Win32 calls in the Windows adapter (from P3) and the test tools | yes, built in |
| embed-manifest | 1.5.1 | MIT | https://codeberg.org/carey/embed-manifest | Puts the Windows manifest (uiAccess, DPI) into each exe | build time only, not linked in |
| Fluent: fluent-bundle, fluent-syntax, fluent-langneg, unic-langid, unic-langid-impl, intl-memoizer, intl_pluralrules, type-map, self_cell, smallvec, rustc-hash, tinystr, memchr | 0.16.0, 0.12.0, 0.13.1, 0.9.6, 0.9.6, 0.5.3, 7.0.2, 0.5.1, 1.3.0, 1.16.2, 2.1.3, 0.8.4, 2.8.3 | MIT OR Apache-2.0, except self_cell (Apache-2.0 OR GPL-2.0-only, used as Apache-2.0), tinystr (Unicode-3.0) and memchr (Unlicense OR MIT, used as MIT) | crates.io; each crate's repository | UI translations (added in P3) | yes, built in |
| Lucide icons (39 SVGs from `lucide-static`, in `assets/icons/lucide/`) | 1.48.0 | ISC (some MIT from Feather) | https://lucide.dev/license | Key, panel, pill, voice, theme-button and language-list icons | yes |
| IBM Plex Sans / Sans Arabic | 3.201 / 1.101 (Google Fonts) | OFL-1.1, Reserved Font Name "Plex" | https://github.com/IBM/plex | ET66 theme | yes, unmodified |
| Noto Sans Arabic | 2.012 (Google Fonts) | OFL-1.1 | https://github.com/notofonts/arabic | Arabic fallback in all three themes (the Latin Noto Sans is not bundled: Native uses Segoe UI on Windows) | yes |
| Rubik | 2.300 (Google Fonts) | OFL-1.1 | https://github.com/googlefonts/rubik | Dolch theme, Latin and Arabic (Noto Sans Arabic is its Arabic fallback) | yes |
| Keystrokes of a Cherry KC 1000 keyboard: "Keyboard Soundpack #1 [Typing and Single Keystrokes]" by unicaegames, six single keys as `assets/sounds/cherry-key-1.wav` … `-6.wav` | – | CC0-1.0 | https://opengameart.org/content/keyboard-soundpack-1-typing-and-single-keystrokes | The key sound (P4) | yes, built into the app from P4 |
| "Middle Mouse Click" by 1j01, as `assets/sounds/mouse-click.wav` and `mouse-press.wav` | – | CC0-1.0 | https://opengameart.org/node/186493 | The right-click and pick-up sounds (P4) | yes, built into the app from P4 |
| "Playing Card Sounds" by BMacZero (Brian MacIntosh), `contact2.wav` as `assets/sounds/card-down.wav` | – | CC0-1.0 | https://opengameart.org/node/118338 | The drop sound (P4) | yes, built into the app from P4 |
| whisper.cpp + Whisper weights | – | MIT | https://github.com/ggml-org/whisper.cpp | Local voice typing | models downloaded on demand |
| whisper.cpp Windows build (`whisper-server`) | b5130 (v1.9.4) | MIT; the bundle also has `SDL2.dll` (Zlib) | https://github.com/ggml-org/whisper.cpp | Local speech server in the voice worker (tested in G22) | P11 decides: shipped or downloaded |
| cpal | 0.18.2 | Apache-2.0 | https://github.com/RustAudio/cpal | Microphone in the voice worker (tested in G22) | yes, built into the worker (P11) |
| wordfreq data | – | CC-BY-SA-4.0 (data) | https://github.com/rspeer/wordfreq | Prediction seed lists | yes, separate files |
| Tesseract + tessdata_fast | – | Apache-2.0 | https://github.com/tesseract-ocr | OCR fallback | on demand |
| Inno Setup | 7.x (planned, P14) | Inno Setup License (zlib-style) | https://jrsoftware.org/isinfo.php | Windows installer | installer stub only |
| Contributor Covenant | 3.0 | CC BY-SA 4.0 | https://www.contributor-covenant.org/ | `CODE_OF_CONDUCT.md` | no |

The versions are filled in when each component is added. Planned items stay here so their licences are checked early.
- Qt 6.11.2 was reviewed on 2026-09-29 against its own SBOM files (`sbom/*.spdx.json` in the Qt install): every shipped module and plugin is LGPL-3.0-only. The permissive parts bundled inside them are accepted as part of Qt (owner, 2026-09-29); shipped Qt files without a "Parts inside" row bundle none.
- Only the Qt files the app loads ship: `[workspace.metadata.stage]` in `Cargo.toml` lists them, measured on the running release app in P3 (Task 20), and the rows marked "only if" are not staged.
- Credits the release adds to the docs: FreeType (`FTL`) asks for a line crediting The FreeType Project, Imlib2 (smooth scaling in `Qt6Gui.dll`) one acknowledging its use, and libjpeg (`IJG`) one crediting the Independent JPEG Group if `qjpeg.dll` ships. Every other bundled part needs its notice in `licenses/`, and the Public Suffix List (`MPL-2.0`) a pointer to its source.
- Qt's LGPL-3.0 DLLs ship unmodified and replaceable. Their licence texts and a note on how to swap them go into `licenses/` at release (ADR-0013, `kx-release`).
- `Qt6Network.dll` ships because Qt Qml and Qt Quick import it; the stage check refuses it once nothing does, or when anything else imports it. No TLS, network-information or QML debugging plugin ships, and the stage check refuses them.
- Qt's build tools (moc, rcc, qsb) are GPL-3.0-only with the Qt GPL exception and run at build time only. Qt Shader Tools bundles glslang and SPIRV-Cross (BSD-3-Clause, Apache-2.0, MIT and others); none of the shipped DLLs load it, so it does not ship.
- Fluent's full tree is 20 crates; the other seven are proc-macro helpers (displaydoc, syn, quote, proc-macro2, unicode-ident) and thiserror with its macro crate, all MIT OR Apache-2.0 (unicode-ident adds Unicode-3.0). The Fluent tree and the cxx-qt tree (63 crates, with the build tools) passed `cargo deny check licenses` against `deny.toml` on 2026-09-29.
- Fonts and icons are in the repo under `assets/`, each set with its licence file (ADR-0016). kx-ui's build compiles them into the app as Qt resources: every font, and the icons the theme tokens name.

## Test tools (never shipped)

| Tool | Version | Licence | Source | Used for |
|---|---|---|---|---|
| aqtinstall | master `076e165` | MIT | https://github.com/miurahr/aqtinstall | Downloads Qt, on the dev PC and in CI |
| py7zr | 1.1.3 | LGPL-2.1-or-later | https://github.com/miurahr/py7zr | Unpacks Qt's archives for aqtinstall |
| PresentMon | 2.6.0 | MIT | https://github.com/GameTechDev/PresentMon | Frame times (G8) |
| NVDA (portable) | 2026.2 | GPL-2.0 with exceptions | https://github.com/nvaccess/nvda | Screen-reader check (G10) |
| Tauri | 2.12.0 | MIT OR Apache-2.0 | https://github.com/tauri-apps/tauri | Quick comparison only; removed in P2 |
| proptest | 1.11.0 | MIT OR Apache-2.0 | https://github.com/proptest-rs/proptest | Property tests |
| Whisper `base` model (`ggml-base.bin`) | SHA-256 `60ed5bc3…2efe` | MIT | https://huggingface.co/ggerganov/whisper.cpp | Local speech model (G22) |

## CI actions (never shipped)

Each runs only on GitHub's runners, pinned to a full commit SHA; Dependabot proposes updates (`docs/security/ci.md`).

| Action | Version | Licence | Source | Used for |
|---|---|---|---|---|
| actions/checkout | 7.0.1 | MIT | https://github.com/actions/checkout | Fetches the repo |
| Swatinem/rust-cache | 2.9.2 | LGPL-3.0 | https://github.com/Swatinem/rust-cache | Caches cargo's builds between runs |
| taiki-e/install-action | 2.87.26 | Apache-2.0 OR MIT | https://github.com/taiki-e/install-action | Installs cargo-deny and cargo-audit |
| jurplel/install-qt-action | 4.4.1 | MIT | https://github.com/jurplel/install-qt-action | Installs Qt through aqtinstall on the Windows jobs (P3) |
| actions/setup-python | 6.3.0, pinned inside install-qt-action | MIT | https://github.com/actions/setup-python | Python for aqtinstall |
