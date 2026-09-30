# Assets

Fonts and icons that ship in the app. Every set keeps its licence file beside it. kx-ui's build compiles them into the app as Qt resources (ADR-0016): every font here, and the icons the theme tokens (`icons.toml`) name. A fresh clone builds without any download, and the app never reads this folder.

## Fonts

All fonts are SIL OFL-1.1 and ship unmodified.

- `fonts/ibmplexsans/`: IBM Plex Sans 3.201 (variable), from Google Fonts (upstream: https://github.com/IBM/plex). Reserved Font Name "Plex"; licence in `OFL.txt`.
- `fonts/ibmplexsansarabic/`: IBM Plex Sans Arabic 1.101 (Regular, Medium, SemiBold), from Google Fonts (upstream: https://github.com/IBM/plex). Reserved Font Name "Plex"; licence in `OFL.txt`.
- `fonts/notosansarabic/`: Noto Sans Arabic 2.012 (variable), from Google Fonts (upstream: https://github.com/notofonts/arabic). Licence in `OFL.txt`.
- `fonts/rubik/`: Rubik 2.300 (variable), from Google Fonts (upstream: https://github.com/googlefonts/rubik). Licence in `OFL.txt`.

## Icons

- `icons/lucide/`: 39 icons from Lucide 1.48.0 (`lucide-static`), https://lucide.dev. ISC, and MIT for the ones derived from Feather; both texts are in `LICENSE`.
- `icons/win-logo.svg`: KeyXtend's own drawing of the Windows key legend, GPL-3.0-or-later like the code. Windows and its logo are Microsoft trademarks; used only to label the Windows key.

Add a font or icon only after the `kx-licence-check` skill passes, and list it in `THIRD_PARTY.md`.
