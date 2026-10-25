# Assets

Fonts, icons and sounds that ship in the app. Every set keeps its licence file beside it. kx-ui's build compiles them into the app (ADR-0016): every font here and the icons the theme tokens (`icons.toml`) name as Qt resources, and the samples of the sounds `sounds.toml` names. A fresh clone builds without any download, and the app never reads this folder.

## Fonts

All fonts are SIL OFL-1.1 and ship unmodified.

- `fonts/ibmplexsans/`: IBM Plex Sans 3.201 (variable), from Google Fonts (upstream: https://github.com/IBM/plex). Reserved Font Name "Plex"; licence in `OFL.txt`.
- `fonts/ibmplexsansarabic/`: IBM Plex Sans Arabic 1.101 (Regular, Medium, SemiBold), from Google Fonts (upstream: https://github.com/IBM/plex). Reserved Font Name "Plex"; licence in `OFL.txt`.
- `fonts/notosansarabic/`: Noto Sans Arabic 2.012 (variable), from Google Fonts (upstream: https://github.com/notofonts/arabic). Licence in `OFL.txt`.
- `fonts/rubik/`: Rubik 2.300 (variable), from Google Fonts (upstream: https://github.com/googlefonts/rubik). Licence in `OFL.txt`.

## Icons

- `icons/lucide/`: 61 icons from Lucide 1.48.0 (`lucide-static`), https://lucide.dev, unmodified: the keys, the shortcuts layer, the panels and the top bar. ISC, and MIT for the ones derived from Feather; both texts are in `LICENSE`.
- `icons/win-logo.svg`: KeyXtend's own drawing of the Windows key legend, all rights reserved like the code. Windows and its logo are Microsoft trademarks; used only to label the Windows key.

## Sounds

All sounds are CC0-1.0 (text in `sounds/CC0-1.0.txt`), converted with `cargo xtask sounds`; `sounds/README.md` gives each file's source and command. kx-ui's build reads each file kx-theme's `tokens/sounds.toml` names with a strict reader (PCM in its `[format]`, whole frames, at most `max_ms`) and fails, naming the file, on anything else.

- `sounds/cherry-key-1.wav` … `-6.wav`: the key sound, a Cherry KC 1000 key in six takes, from "Keyboard Soundpack #1" by unicaegames (https://opengameart.org/content/keyboard-soundpack-1-typing-and-single-keystrokes).
- `sounds/mouse-click.wav`, `sounds/mouse-press.wav`: the right-click and pick-up sounds, from "Middle Mouse Click" by 1j01 (https://opengameart.org/node/186493).
- `sounds/card-down.wav`: the drop sound, from "Playing Card Sounds" by BMacZero (https://opengameart.org/node/118338).
- `sounds/recipes.toml`: KeyXtend's own candidate recipes, all rights reserved like the code; the shipping format is in kx-theme's `tokens/sounds.toml`.

## Word packs

- `words/`: the word packs (`<iso>.kxw`) that `cargo xtask words build` writes, shared under CC BY-SA 4.0 as `words/LICENSE.txt` says, with every source credited in `words/CREDITS.txt`; they ship as separate files beside the program. `words/packs.txt` lists the languages the build shipped (`cargo xtask words index` rewrites it).
- `cargo xtask words meter` counts the presses the chips save on the held-back sentences (spec §6 "Target"). Its times count only from a release build: `cargo run --release --locked -p xtask -- words meter`, the only build where `--speed` judges its ceiling.

Add a font, icon or sound only after the `kx-licence-check` skill passes, and list it in `THIRD_PARTY.md`.
