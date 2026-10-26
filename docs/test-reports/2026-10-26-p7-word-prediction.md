# Test report: P7 word prediction

- **Date:** 2026-10-26, on the development PC.
- **Windows:** Windows 11 Home Single Language 26H2, build 26300.9711.
- **Display:** 1920×1080, 125 % (120 DPI).
- **Apps:** Notepad 11.2607.14.0, Chrome 155.0.8059.86; Qt 6.11.2.
- **Builds:**
  - `keyxtend` (3822fb5, same product as 7ac2168), release, signed and installed under `Program Files\KeyXtend-dev` with uiAccess (`check-uiaccess`: secure folder, trusted signature, uiAccess in the manifest).
  - `kx-gates` (7ac2168, then 6ac7dc5 for G27's second run), release, signed and installed beside it with uiAccess.
  - `keyxtend` again at 43bde0d, for the hands-on retest after the fixes (note 10), and at 218891e after the Definition of Done reviews (note 13).
- **Load:** the owner's usual load (WSL, Docker), left running.
- **Scope:** gates G21, G27, G8 (with its prediction part) and G7 on the installed build, and the plan's hands-on checks.

## Gates

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | `g21 chrome`, `g21 win32` | Password boxes flagged, text boxes not | Pass | |
| 2 | `g21 keyxtend` | Password boxes show the lock and no chip; a plain box shows chips | Pass | Lock with 0 chips in Win32 and Chrome; plain box 5 chips |
| 3 | `g27 keyxtend`, run 1 | EN, FR and AR prefixes offer a chip that types the word; edge 6 types nothing | Fail, then fixed | Cases passed; edge 6 failed in the gate itself (note 1) |
| 4 | `g27 keyxtend`, run 2 | As above | Pass | Chips in 357–363 ms; edge 6 "typed nothing" |
| 5 | `g8 keyxtend`, run 1 | Spec §10 budgets | Partial; owner accepts | Memory over, 3 dropped frames; see note 2 |
| 6 | `g8 keyxtend`, run 2, settings aside | As above, with the scroll part | Void for timing | 1.3 GB free on the PC; see note 3 |
| 7 | `g8 predict`, before and after the one-language rule | The predict part reported; English typing grows ≤ 3 MB; three-language growth reported | Fail, then Pass | English +1.3 / +1.6 MB; predict part 0.0 / 1.4 MB; see note 4 |
| 8 | `g7 qt` | The Arabic panel's caret stops only between letters | Pass | |

## Hands-on checks (owner)

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | Type the start of a word and pick a chip, in English, French and Arabic | Each language's words, typed whole and right | Pass | |
| 2 | Picks in a plain text box, domain chips after `@`, a password box | Picks type right; domains complete the address; the lock and no chip in the password box | Pass | Chrome and the test page |
| 3 | Type `@@` and pick the email chip | The known email replaces the `@@` | Fail, then Pass | No email was ever learned (note 8); fixed in 43bde0d |
| 4 | Picks after typed letters in Notepad | The word comes out whole | Fail, then Pass | "hhelp", "gg ame" (note 9); fixed in 43bde0d |
| 5 | Every chip-lane tap | The same sound as a key | Fail, then Pass | Chip taps were silent; the owner asked for it in P7 (note 10) |
| 6 | Start-of-text chips with nothing typed | Shown on purpose | Pass | The owner kept the five |
| 7 | Menu: Forget a word, then Undo | Undo takes the chip's place and brings the word back | Pass | |
| 8 | Menu: Pause learning on, type words, restart | Nothing learned; the switch kept over a restart | Pass | The English file stayed unchanged; `learn = false` saved at Quit |
| 9 | Menu: Prediction off, then on | An empty lane with the menu button, then chips again | Pass | |
| 10 | Menu at the smallest size | Every label whole and readable | Pass | |
| 11 | Restart, then `@@` | The learned email is kept | Pass | |
| 12 | Caps Lock, then a quick letter | A capital letter and capital chips | Pass | |
| 13 | Half a word, then a layout switch | The lane follows the new language | Pass | English to French |
| 14 | Restart | The keyboard opens where it was left | Pass | |
| 15 | Regressions: keys and chips never take focus; above the Start menu; Right-click mode keeps short clicks normal | As before | Pass | |
| 16 | The scroll pad cannot page the email list | Only the list's page buttons turn it | Not run | It needs more known emails than chips; covered by the QML tests |
| 17 | Select a word in Notepad (double-click), then tap a chip | Nothing erased or typed; the lane empties until the next key | Pass | Build 218891e |
| 18 | Notepad: a pick after typed letters, then `@@` and the email chip | Both right | Pass | Build 218891e |
| 19 | Chrome: a pick in a textarea, `@@` in an email input | Both right | Pass | Build 218891e |

## Notes

1. The installed gate got no text pattern from target-window's box, so asking for its lines failed before the pick check ran. The gate now clicks the box's own rectangle (6ac7dc5); the product's pick check was never at fault.
2. G8 run 1: start 407 ms, idle CPU 0 %, char p99 15.7 ms, key light p50 14.7 ms, churn −1.4 / +1.5 MB. Private working set 106.2 MB cold (budget 100; P5 read 101.5–103.0), commit 144.1 MB (budget 140; P5 137.2–140.5), and 111.7 / 150.0 MB after use. Three frames were lost in one press. The owner accepted the memory for P7, with P13 owning it.
3. G8 run 2 ran with both settings files aside, as in P5 (note 8), while WSL held 9.6 GB: start 1086 ms and 164 dropped frames come from that load. Memory matched run 1 (106.1 / 144.6 cold) and read 115.0 / 153.6 MB after a scroll.
4. Prediction costs about 4–6 MB of commit, on against off. Typing 30 words through chips grew +2.1 / +1.9 MB in English and +4.9 / +5.3 MB with EN, FR and AR in turn (66 words: +9.3 / +10.6). With nothing typed and no UI Automation client, 90 warm layout switches grew 0.3 MB, so the three-language growth comes from the gate reading the rebuilt keys and chips. The owner ruled the limit judges one language (98bf3b0): the last run grew +1.3 / +1.6 MB in English, and its three-language reading +8.4 / +8.1 MB over 28 of 30 words (two French chips came late).
5. The first switches to French and Arabic keep about 38 MB, with prediction on or off (139.0 → 177.0 MB commit off, 144.9 → 183.2 on). It belongs to the keyboard's layouts, not to P7; the owner made it a P13 must-fix.
6. `close qt` closed the prototype face's window, but its process stayed and needed an elevated stop (owner's yes).
7. Gate logs and test files are deleted when P7 is recorded; only these numbers are kept.
8. A click after a domain chip ended the address unlearned, so no email was ever known. The owner chose that an email is learned when the user leaves it: a domain chip, a space, Enter, Tab, a click elsewhere or a new box.
9. The new Notepad drops Backspaces sent back-to-back and reorders text sent right after them. A test in the test tab: 2 ms or more after each Backspace, and a short pause before the text, gave 10 of 10. The fix adds `[keyboard] erase_gap_ms` (10 ms), erases only the letters that differ from the chip, and stops a pick at a click elsewhere (owner's choice).
10. Hands-on rows 3–5 were retested on the build at 43bde0d (installed, uiAccess checked). The keyboard's counts at Quit: 35 picks sent, none moved, none unsure.
11. A doubled letter seen once was most likely the key repeat after a long press (`repeat_delay_ms`, 800 ms).
12. The session-end save on sign-out was not run, as signing out would stop the owner's WSL and Docker; it is unit-tested.
13. The Definition of Done reviews found that a pick could erase into a selection. Build 218891e checks the selection before any erase and checks every pick in a text box; rows 17–19 retested it in Notepad and Chrome.
