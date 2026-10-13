# Sounds

This folder holds `recipes.toml`, the recipes for KeyXtend's candidate sounds with the shipping format in its `[format]` table, and later the sounds the owner picks. Sounds made from these recipes are our own work, released under CC0 (spec §3.2); recordings come only from CC0 sources. `cargo xtask sounds render [--out <dir>]` writes every recipe to the folder `[workspace.metadata.sounds]` names in `Cargo.toml`, and `cargo xtask sounds import <in.wav> <out.wav> [--start-ms N] [--len-ms N]` converts a recording to the shipping format.
