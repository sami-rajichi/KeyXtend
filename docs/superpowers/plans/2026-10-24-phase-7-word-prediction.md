# Phase 7: word prediction and emails (v0.4)

## Goal
- Word chips in the top bar finish the word and guess the next one, learn from the user, and fill emails, all on the PC.
- Spec §6 (approved 2026-10-24); the research report (`target/research/reports/`, summed up in ADR-0032); `docs/FEATURES.md` P7; the move map's P7 row; `docs/items/P7.md`.

## Scope
- In: `kx-mod-predict`, `kx-crypto`, the packs and their build tool, the learner, emails and `@@`, password boxes, the chips and their menu, the meter, the P3 and P4 notes in `docs/items/P7.md`, G21 aimed at the product, a new prediction gate G27.
- Out: password filling and the vault's emails (P9); the learned-words list in Settings (P13); spelling dictionaries for languages other than EN, FR and AR; a neural re-ranker; clitic-aware Arabic (ideas, after the meter's numbers).

## Design notes
- **Crates:** new `kx-mod-predict` (pure engine and actor), `kx-crypto` (seal, open, sealed files); touched: `kx-module-api`, `kx-platform-windows`, `kx-platform-fake`, `kx-mod-keyboard`, `kx-mod-mouse`, `kx-ui`, `kx-theme`, `apps/keyxtend`, `xtask`, `tools/kx-gates`, `tools/kx-target-window`.
- **Capabilities (ADR-0033):** `ReadFocusedText` gates every event that carries typed or suggested text and the field service; the new `KeyboardUi`, held only by the ui module, gates `KeyboardFaceKey`, `AssistNote` and `PointerMoved`; `SecretStore` gates the sealed store.
- **Events (all gated by `ReadFocusedText`):** `Typed` (the keyboard tells each text piece, Backspace or word break), `TypeWord` (asks the keyboard to erase a count and type a word, so the keyboard stays the only typist), `Suggested` (the lane: chips, lock note, forget mode, Undo), `ChipPicked` and `WordsAsk` (menu rows, More, Undo). `PressesWanted { on }` is ungated and carries nothing.
- **Ports:** `FieldWatch` (`ports/field.rs`): the focused box's kind (text, password, unknown, none), the N words before the cursor, and whether the text before it still ends with a word. `WordPacks` (`ports/packs.rs`): a read-only view of a language's pack. `SealedStore` (`ports/sealed.rs`): load, save and set aside a named blob, sealed by the adapter. `KeyGuard` (in `kx-crypto`): wraps the data key with the OS (DPAPI on Windows).
- **Packs:** one file per language, `<iso>.kxw` (a header, a lexicon FST from the `fst` crate mapping folded keys to word ids, the word table with spellings and frequency buckets, and the n-gram tables as sorted id arrays with 8-bit scores); built into `assets/words/`, staged into `words/` beside the program with `CREDITS.txt`.
- **Engine:** folding per script, with casing by language; a stupid-backoff scorer over at most `context_words` previous words; seed and learned counts blended by `learn_weight`; capitals from the pack's spelling or the typed letters; at most `chips` results.
- **Learner:** per-language counts and last-used days for words and chains up to N+1 words; the two-use gate; clean-up at 1.2 × cap (oldest first), the purge after `forget_days`, halving near the 16-bit count limit; a blocklist for forgotten seed words; informal spellings counted toward the correct form.
- **Settings `[predict]`:** `on` true, `learn` true, `chips` 5 (3–7), `context_words` 2 (1–4), `min_uses` 2, `max_words` 10000, `max_chains` 30000, `forget_days` 300, `learn_weight` 0.6, `max_word` 48, `max_emails` 50, `domains` (gmail.com, outlook.com, hotmail.com, yahoo.com, icloud.com), `read_ms` 150, `save_s` 30. `[keyboard] text_cap` 256 (32–1024) caps any text the keyboard types.
- **Tokens:** `icons.toml [words]` (Lucide at-sign, key-round, lock, ellipsis, x, undo-2, pause, power); `shape.toml [chip]` gains the menu button and icon gap; no per-key animation, so the CPU stays idle.
- **Texts (EN, FR, AR):** `chip.name`, `chip.more`, `chip.undo`, `chip.none`, `chip.locked`, `words.menu`, `words.forget`, `words.pause`, `words.off`, `words.on`, `notice.words-damaged`, `notice.words-missing`, `module.predict`.

## Tasks
1. **Licences and sources.** *Files:* `THIRD_PARTY.md`, `licenses/`, `deny.toml`, `.claude/skills/kx-licence-check/SKILL.md` (CC BY 2.0 FR for data, owner 2026-10-24). *Test first:* none (records). *Implement:* `kx-licence-check` for `fst`, `chacha20poly1305`, `zeroize`, the build-only decoders and `spellbook`, wordfreq, Wikipedia, Tatoeba and the EN/FR/AR Hunspell lists; confirm a plain-text Wikipedia source; ask the owner's yes per download, with sizes. *Verify:* `cargo deny check`.
2. **ADRs.** *Files:* `docs/adr/0032-word-prediction-is-a-learning-n-gram-engine.md`, `0033-typed-text-travels-only-to-modules-that-may-read-it.md`, `docs/adr/README.md`. *Implement:* the engine, packs and learner with the research's numbers; the gates above, the field service in place of an intercept event, and `FlipCaps` dropped.
3. **Module API.** *Files:* `kx-module-api/src/{capability.rs, ports.rs, ports/{field,packs,sealed,words}*.rs, ports/face.rs, ports/assist.rs}`, contract tests. *Test first:* every new event and service names its gate; `KeyboardUi` is its own capability; `Typed` and `Suggested` print redacted; `FlipCaps` is gone. *Implement:* the types. *Verify:* `cargo test -p kx-module-api`.
4. **Fakes.** *Files:* `kx-platform-fake/src/{field,packs,sealed}.rs`. *Test first:* scripted box kinds and words before the cursor; packs from bytes; a sealed store that can fail on cue. *Implement:* the three fakes. *Verify:* `cargo test -p kx-platform-fake`.
5. **`kx-crypto`** (`kx-new-module` steps for a library). *Files:* the crate, `docs/security/kx-crypto.md`. *Test first:* seal then open returns the bytes; a flipped bit, a wrong key or a short file is refused; property: no input panics `open`; a damaged file is set aside and reported once; writes replace the file whole. *Implement:* XChaCha20-Poly1305 with a fresh nonce per write, the wrapped data key, `SealedFiles` over the data folder. *Verify:* `cargo test -p kx-crypto`.
6. **Windows adapters.** *Files:* `kx-platform-windows/src/{guard_key.rs, packs.rs, field.rs, field/{uia,win,words}.rs, platform.rs}`, the moved spike `core/src/{uia,uia/act,uia/read}.rs`. *Test first:* DPAPI wraps and unwraps; a pack maps read-only (never copy-on-write) and a missing one is `None`; the Win32 password style reads as password; an unreadable box is unknown; the words-before-cursor cut keeps at most N words. *Implement:* the key guard, the file map, the UIA focus watch on its own COM thread with `read_ms` deadlines. *Verify:* `cargo test -p kx-platform-windows`.
7. **Folding and pack format.** *Files:* `kx-mod-predict/src/{fold.rs, pack.rs, pack/*.rs}`. *Test first:* `ecole` and `école` share a key; أ إ آ ٱ fold to ا and ى to ي, harakat and tatweel drop; a pack round-trips; property: a truncated or random pack is refused without panic. *Implement:* folding and the reader. *Verify:* `cargo test -p kx-mod-predict fold pack`.
8. **Pack builder and meter.** *Files:* `xtask/src/{words.rs, words/{fetch,wordfreq,dict,text,count,write,meter}.rs}`, `xtask/words.toml` (sources, hashes, caps, the n-gram budget per language). *Test first:* on tiny fixtures: the dictionary drops `ete` and keeps `été` and `a`/`à`; capitals return; French elisions reattach; digits, URLs and emails are dropped; pruning meets the budget; the meter counts saved presses on a known sentence. *Implement:* fetch with hash checks, build, credits, and the meter over held-back sentences. *Verify:* `cargo test -p xtask words`.
9. **Build the packs.** The controller runs the downloads after the owner's yes, builds EN, FR, AR and the 39 word-only packs, weighs them (≤ 22 MB together) and commits them; the sources are deleted and both drives checked.
10. **Engine and tracker.** *Files:* `kx-mod-predict/src/{score.rs, track.rs, email.rs}`. *Test first:* the best guess comes first; an unseen context backs off; capitals follow; the tracker ends words on breaks and resets on clicks and new boxes; one `@` lists domains, a typed `@@` lists emails, and an `@@` it did not type does not. *Implement:* the scorer, the word tracker machine and emails. *Verify:* `cargo test -p kx-mod-predict`.
11. **Learner.** *Files:* `kx-mod-predict/src/{learn.rs, learn/*.rs}`. *Test first:* one use is not suggested, two are; caps evict the oldest; 300 days purge (fake clock); halving keeps order; forget hides a seed word until typed twice; `انا` counts toward `أنا`; property: random use never breaks the caps. *Implement:* the learner and its byte form. *Verify:* `cargo test -p kx-mod-predict learn`.
12. **The module** (`kx-new-module`). *Files:* `kx-mod-predict/{Cargo.toml, defaults.toml, src/{lib.rs, settings.rs, actor*.rs, menu.rs}}`, `docs/security/kx-mod-predict.md`. *Test first:* the manifest needs only its three services and two capabilities; a password box gives the lock lane and learns nothing; an unknown box suggests but learns nothing; Pause and Off are saved; a pick sends one `TypeWord`; the learned file saves after `save_s` and at stop; a damaged file starts empty with the notice. *Implement:* module, actor (`kx-predict` thread), menu. *Verify:* `cargo test -p kx-mod-predict`; `cargo xtask tidy` 0/0.
13. **Keyboard.** *Files:* `kx-mod-keyboard/src/{plan.rs, run.rs, face/bus.rs, face/inner*.rs, settings.rs}`, `defaults.toml`, tests. *Test first:* each text piece, Backspace and break is told once; a `TypeWord` types through the app's route, ends a dead key's wait, refuses while Ctrl, Alt or Win is latched, and lets a one-shot Shift go; text over `text_cap` is refused. *Implement:* `Typed`, `TypeWord`, `text_cap`; the churn's Caps flip through the app's debug layout wrapper and `relabel()`. *Verify:* `cargo test -p kx-mod-keyboard`.
14. **Mouse.** *Files:* `kx-mod-mouse/src/{engine*.rs, actor*.rs}`. *Test first:* `PressesWanted` keeps the hook on and each press is told with `ours`; off lets it go when idle. *Implement:* the need flag. *Verify:* `cargo test -p kx-mod-mouse`.
15. **The app.** *Files:* `apps/keyxtend/{Cargo.toml, src/boot.rs, src/run.rs, src/layouts_debug.rs}`, `tests/life/{predict,rig,ships}.rs`, CI package lists. *Test first:* `mod-predict` is a default; the field and sealed services reach only the predict module; packs are found beside the program, and in debug in `assets/words`. *Implement:* the wiring. *Verify:* `cargo test -p keyxtend`; `cargo xtask stage` stays ≤ 80 MB.
16. **UI hub and texts.** *Files:* `kx-ui/src/{hub/bus.rs, hub/words.rs, bridges/ui*.rs, ui_module.rs}`, `i18n/{en,fr,ar}.ftl`, fixtures, `tests/texts/files.rs`. *Test first:* `Suggested` wakes the top bar once; a click sends `ChipPicked` with the lane's round, so a stale click is dropped; every new key exists in three languages. *Implement:* hub state, bridge calls, texts. *Verify:* `cargo test -p kx-ui`.
17. **Views, tokens and icons.** *Files:* `kx-ui/qml/{TopBar,Plate,Chip,WordMenu,WordList}.qml`, `kx-theme/tokens/{icons,shape}.toml`, `assets/icons/lucide/`, `qmltest/tst_words.qml`. *Test first:* chips fill the lane up to `chips` and drop the ones that do not fit; Arabic chips use the Arabic font and read right to left; the lock note, forget mode, Undo, More and the menu are found by accessible name; contrast holds in light and dark. *Implement:* the views. *Verify:* `qmltestrunner` at both scalings; `cargo test -p kx-ui -p kx-theme`.
18. **Gates.** *Files:* `tools/kx-gates/src/{g21.rs, g27.rs}`, `kx-gates-p7.toml`, `docs/spike-move-map.md`, `docs/security/dev-tools.md`. *Test first:* config checks. *Implement:* G21 on the product (no chips in the target window's password box); G27 types `hel` in EN, `eco` in FR and `ان` in AR and picks the first chip, checking the target window's text; G8 adds the predict memory part. *Verify:* `cargo test -p kx-gates`.
19. **Docs.** `ARCHITECTURE.md`, README, `docs/security/{kx-mod-keyboard,kx-mod-mouse,kx-platform-windows,kx-ui,keyxtend}.md`, `docs/items/{P7,P9,P13}.md`.
20. **Review and security.** `kx-review` on every changed file, `kx-security-check`, `/code-review` and `/security-review`; fix every finding.
21. **Gates, meter, manual check, done.** The controller runs G21, G27 and G8 with a heads-up, and the meter; then `kx-windows-manual-test` with the owner, `kx-module-done` and the records.

## Edge cases
1. A word that mixes scripts gets no chips and is never learned.
2. An Arabic word picked after typing erases exactly the units the keyboard typed (RTL caret).
3. A token longer than `max_word` gets no chips and is not learned.
4. Password box: lock lane, no learning; unknown box: chips, no learning; neither is ever read through UIA.
5. Forget, then Undo restores; a forgotten seed word comes back after two uses.
6. A click mid-word ends the word and reads at most N words; a pick whose word no longer ends the text erases nothing and refreshes the lane.
7. An `@@` already in the text never opens the email chips.
8. A layout switch mid-word ends the word and switches the language.
9. Ctrl, Alt or Win latched: no chips; Shift latched: capitalised chip, Shift lets go.
10. A damaged learned file is set aside, learning starts empty, and the notice shows once.
11. A missing EN, FR or AR pack shows `notice.words-missing`; other languages learn from nothing.
12. Digits and web addresses are never learned; emails go only to the email list.
13. The mouse module off: clicks go unseen, and the pick check of edge 6 still guards.
14. Focus moves to a new box between a key and a pick: the word resets and the lane refreshes.
15. An email longer than `text_cap` is refused.
16. Learning paused saves nothing; prediction off empties the lane and keeps the menu button.

## Security
- Capabilities: predict `ReadFocusedText` and `SecretStore`; keyboard and ui `ReadFocusedText`; ui `KeyboardUi`.
- Data: typed text and learned words (`Redacted`, never logged), emails (personal), the data key (DPAPI-wrapped).
- Threats to test: a module without the gate hears typed text or asks to type; learning in a password box; a tampered pack or learned file; a stale chip click erasing the wrong text.

## Manual Windows check
- Notepad and a browser text box: EN, FR and AR words through chips; accents and hamza come out right; the next-word guess after a space.
- `@` domains and `@@` emails; a browser login page's password box shows the lock and learns nothing.
- Menu: forget with Undo, Pause learning, Prediction off and on; a restart keeps learned words.
- A click mid-text, then a chip; a layout switch mid-word.

## Done when
- The Definition of Done in `docs/roadmap.md` holds.
- The meter shows ≥ 40% saved presses for EN and FR with 5 chips, and Arabic's number is reported; the packs are ≤ 22 MB; G8 shows the predict memory, reported to the owner.
