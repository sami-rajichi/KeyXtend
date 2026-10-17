# Sounds

The app's sounds, all CC0 1.0 (text in `CC0-1.0.txt`; each source page's licence field was checked on 2026-10-13), in the shipping format that `[format]` in `crates/kx-theme/tokens/sounds.toml` sets, where each event names its files. `recipes.toml` holds the tool's own candidate recipes, our own work, all rights reserved like the code: `cargo xtask sounds render [--out <dir>]` writes them to the folder `[workspace.metadata.sounds]` names in `Cargo.toml`, `sounds import` converts a recording, and `sounds join` mixes two converted files.

Shipped files, each made with `cargo xtask sounds` from the source's original file:

- `cherry-key-1.wav` … `cherry-key-6.wav`: the key sound, six takes of a Cherry KC 1000 keyboard played in turn. Source: "Keyboard Soundpack #1 [Typing and Single Keystrokes]" by unicaegames, https://opengameart.org/content/keyboard-soundpack-1-typing-and-single-keystrokes, the zip's `Single Keys/keypress-001.wav`, `-006`, `-011`, `-016`, `-021` and `-026` in that order, made with `sounds import keypress-001.wav cherry-key-1.wav` and so on.
- `mouse-click.wav`: the right-click sound, a mouse button pressed and released. Source: "Middle Mouse Click" by 1j01, made for Tracky Mouse, https://opengameart.org/node/186493, `middle-click-press.wav` and `middle-click-release.wav`, each made with `sounds import`, then joined with `sounds join press.wav release.wav mouse-click.wav --at-ms 95`.
- `mouse-press.wav`: the pick-up sound. Source: the same "Middle Mouse Click" by 1j01, `middle-click-press.wav`, made with `sounds import middle-click-press.wav mouse-press.wav`.
- `card-down.wav`: the drop sound, a playing card laid down. Source: "Playing Card Sounds" by BMacZero (Brian MacIntosh, who asks for an optional credit), https://opengameart.org/node/118338, `contact2.wav`, made with `sounds import contact2.wav card-down.wav`.
