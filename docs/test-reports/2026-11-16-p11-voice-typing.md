# Test report: P11 voice typing, part 1

- **Date:** 2026-11-16, on the development PC.
- **Windows:** Windows 11 Home Single Language 26H2, build 26300.9711.
- **Display:** 1920×1080, 125 % (120 DPI), one screen.
- **Hardware:** an NVIDIA RTX 4060 Laptop card with 8 GB (driver 592.00, the engine's `Vulkan1`) and the processor's built-in AMD graphics.
- **Apps:** Notepad 11.2609.11.0 (the gates' own documents), the owner's own text fields for the dictations; Qt 6.11.2.
- **Builds:**
  - `keyxtend`, release, staged, signed and installed under `Program Files\KeyXtend-dev` with uiAccess, at 63cd384f.
  - The voice gate build (`--features gate-voice`) beside the gate worker (`--features gate,card`), each copy on a data folder of its own, driven by `kx-gates` with the `voice` feature; the last card run used the branch's final code, with the fixes in note 6.
  - The gate test build (`--features gate-input`, no uiAccess) for `g19 hung`, driven by the plain `kx-gates`.
- **Load:** the owner's usual load (WSL, Docker), left running.
- **Scope:** G22 on the product and the staged worker, the sandbox proofs, the window guard's open finding, and the plan's hands-on checks.

## Gates

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | `g22 bench`, normal | Each language's default writes its check clip within the card's limit | Pass | 0 words wrong; real-time factor 0.22 Arabic, 0.06 English and French |
| 2 | `g22 bench`, weak (2 cores at 30 %, 4 GB) | Speed recorded, never a fail | Recorded | Factor 3.81 English, 3.37 French; Arabic's default is Not for this PC (note 1) |
| 3 | `g22 staged` | Edges 28, 29 and 39 on a copy of the stage | Pass | 35 module snapshots, none refused, the planted part never mapped; the floor part alone wrote English with 0 of 14 wrong; only the processor offered |
| 4 | `g22 keyxtend`, processor | Each clip typed into target-window, which keeps the foreground; the bar above the keyboard, on top, never activated | Pass | 0 wrong in each language; first text 2.2 s Arabic, 1.2 s English, 1.1 s French |
| 5 | `g22 keyxtend`, the card (`Vulkan1`) | As row 4, each text written by the card | Pass | Last run: 0 wrong in each language, each text written by the card; first text 1.3 s Arabic, 11.2 s English, 38.5 s French (notes 6 and 7) |
| 6 | `boxed https` | The installed worker's fetch job, in its own token and job, reaches a table address and refuses a one-byte answer by its head, saving nothing | Pass | |
| 7 | `boxed folder` | The same job reads back and names a small part, with no network | Pass | |
| 8 | `g19 hung` | The hung app's cases pass; the z-order is printed if the keyboard is covered | Pass | After the hang and after the close, the test keyboard stayed in the topmost run with no window above it |

## Hands-on checks (owner, installed keyboard)

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | The signed uiAccess build over the Start menu | The keyboard and the bar stay above it and never take focus | Pass | |
| 2 | The Dictation key with no model yet | The key shows; the bar's note says what to do; the gear opens the panel above the bar | Pass | |
| 3 | Download Parakeet (0.74 GB) into the chosen folder | Pause and Resume work; the model is ready | Pass | Through Hugging Face's redirects, which the product follows only under `hf.co` |
| 4 | Cancel a download; a file already in the folder | Cancel goes back; the file is recognised and shows ready | Pass | |
| 5 | Dictate in English | The wave follows the voice, the key glows red only while listening, Stop types the text | Pass | |
| 6 | Dictate in French and in Arabic | Correct text; the Arabic bar reads right to left | Pass | |
| 7 | Undo insert; Cancel, then Transcribe anyway | Both as designed | Pass | |
| 8 | A password box; locking the PC while listening | Dots only; the lock stops listening | Pass | |
| 9 | Device: the card, then Automatic and the processor | The card check passes; dictation works on each | Pass | Note 2 |
| 10 | A dictation longer than 1 min 30 s | Typed in full | Pass | Note 3 |
| 11 | Try again with another model; Compare models | Same recording, new text; results for each model | Pass | |
| 12 | History: Insert (2), Pin, Delete, Delete all; Keep recordings on, then off | Each as designed | Pass | |
| 13 | The tray's Voice typing row; open the models folder | The panel opens; Explorer opens the folder | Pass | |
| 14 | The Arabic panel pages | Right to left, correct Arabic | Pass | |
| 15 | A bar note on two lines at 640 wide | Readable in three languages | Pass | The owner keeps it as it is |
| 16 | ET66 dark's switch track (1.17:1, with its On or Off word) | Clear enough | Pass | The owner's judgement |
| 17 | Native, ET66 and Dolch, light and dark, at S, M and L | Every part reads | Pass | |
| 18 | Narrator on the bar and the panel | It says each button's name | Pass | |
| 19 | The system's microphone sign | Off whenever no bar shows | Pass | |
| 20 | Five minutes after the last dictation | The worker is gone from the task list | Pass | It left 291 s after the owner said the dictation was typed |

## Notes

1. The weak copy is 2 cores at 30 % each, 4 GB and only the plain processor parts. Cohere Transcribe Arabic needs 5120 MB, so it is Not for this PC there, and the Models page says local models will be very slow (owner, 2026-11-15).
2. The owner asked whether the card was really used. A read of the card's process list showed our worker on the RTX 4060; the model leaves memory five minutes after the last dictation, by design. Canary felt a little slower on the card than on Automatic, likely the one-time compile of a model family's programs on a card.
3. A first long dictation seemed not to type; the owner's retry of more than 1 min 30 s typed in full, and the owner judged the first try their own slip.
4. This run found that a change of language never checked the new language's model on the card; 63cd384f adds that check, before the hands-on run.
5. The gate build stages no voice files, so `g22 keyxtend` read the detector and check clips through the workspace fallback that a plain `cargo run` uses.
6. Eight card runs: the first seven found two faults, fixed before the last run. A focus change told out of order left the voice module with no box, so a text only showed (runs 2 to 5); the module now looks at the box in front again. A fresh voice job's first answer could pass the 3 s wait (runs 3 and 7; 1.0 to 2.3 s on an idle PC); it now has `load_s`. Run 6 passed before either fix, and one owner click stopped run 1.
7. The last card run's English and French texts came 11.2 s and 38.5 s after Stop, against 0.4 to 0.9 s in runs 6 and 7. G22 sets no time limit, and the cause is not known yet.
8. All test data was deleted: the gates' data folders and Notepad documents, the trial folder with the owner's voice clips, the first speed test's clips, and the research files. The ten speech models stay on this PC for part 2's gates.
