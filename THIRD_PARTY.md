# Third-party components

Everything we ship or bundle that we did not write ourselves. Updated by the `kx-licence-check` skill. Crates are also covered by `cargo deny` and the release SBOM.

| Component | Version | Licence | Source | Used for | Shipped |
|---|---|---|---|---|---|
| Slint | =1.18.1 | GPL-3.0 (option) | https://github.com/slint-ui/slint | All windows | yes |
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
