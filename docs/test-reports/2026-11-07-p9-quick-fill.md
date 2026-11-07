# Test report: P9 quick-fill and one data folder

- **Date:** 2026-11-06 and 2026-11-07, on the development PC.
- **Windows:** Windows 11 Home Single Language 26H2, build 26300.9711.
- **Display:** 1920×1080, 125 % (120 DPI).
- **Apps:** Chrome 157.0.8103.38 with a local test login page, Task Manager (administrator), Snipping Tool, Narrator, Explorer, OneDrive; Qt 6.11.2.
- **Builds:**
  - `keyxtend` and `keyxtend-worker`, release, staged, signed and installed under `Program Files\KeyXtend-dev` with uiAccess, at each round's commit; the rounds of 2026-11-07 ran on 9938fe68 and the final one on 90bd7076 (`check-uiaccess`: secure folder, trusted signature, uiAccess in the manifest).
  - The gate test build (`--features gate-input`, no uiAccess) and its worker, with its own data folder, at 5a9ac47f and 90bd7076, driven by the plain `kx-gates`.
- **Load:** the owner's usual load (WSL, Docker), left running.
- **Scope:** G23 on the product, G9 on the dev install, and the plan's hands-on checks.

## Gates

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | `g23` at 5a9ac47f | Both, Password and User arrive exact, with no Enter, no submit and the clipboard unchanged | Pass | Lengths 12 and 10 |
| 2 | `g23` at 90bd7076 | As above; the panel stays open and the data is removed after | Pass | 0 Enter, 0 submits; layout back; the mouse guard dropped 3,235 moves and passed 240 |
| 3 | `g9` on the dev install at 90bd7076 | Every flag step keeps its guard and the window in front | Pass for every flag step | Its outline part did not run, so the tool's overall answer is no (note 2) |

## Automated checks at 90bd7076

| Check | Result |
|---|---|
| `cargo fmt --all --check` | Clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | Clean |
| `cargo test --workspace` | 82 binaries, 7,222 passed, 0 failed |
| `cargo deny check` | Advisories, bans, licences and sources ok |
| `cargo audit` | Ok, with the two allowed "unmaintained" warnings (`basic-toml`, `proc-macro-error2`) |
| `cargo xtask tidy` | 0 errors, 11 warnings about files over 300 lines |
| Qt Quick tests, 3 runs each at scaling on and scaling 0 | 789 passed, 0 failed, 1 skipped (pictures), 0 warnings, in all 6 |
| `cargo xtask stage` | 69 files; the download is 44 MB of the 80 MB budget, 103 MB installed |
| Repeated runs of the tests fixed for failing now and then | Window group 0 of 60, UI Automation test 0 of 60, churn test 0 of 200 (note 1) |

## Hands-on checks (owner)

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | Start from the Start menu; the question about a hidden copy | One data folder; the copy deleted on Yes | Pass | The copy from the P8 report's note 4 is gone, checked by path |
| 2 | Open the panel, its invitation, New Login | The caret follows each click; the search suggests words; a password box shows no value of its own | Pass | The look is accepted for now |
| 3 | Fill Both, Password and User into the test page | Each lands right, with no Enter and nothing sent | Pass | |
| 4 | Fill into Task Manager's search (administrator) | The value lands | Pass | |
| 5 | New Person and Wi-Fi with an own label; By label; Open on every row | As designed | Pass | |
| 6 | Search in English, French and Arabic | Each finds its entries | Pass | |
| 7 | Paste plain and its rules; Make one; a form's caret and Cancel | As designed | Pass | |
| 8 | A picture alone on the clipboard, Paste beside a field | "The clipboard holds no text to paste." and the field unchanged | Pass | At 90bd7076 |
| 9 | Back up to OneDrive and to a folder from the list; a change | The change rewrites both boxes | Pass | |
| 10 | A missing place, by renaming its folder | It says so and comes back alone within a minute | Pass | No USB stick was at hand (note 3) |
| 11 | Bring back from a folder; rows, Back up now, Remove | The entries return; each place acts alone | Pass | |
| 12 | The new computer offer; a damaged notebook; Replace's question, No and Yes | As designed | Pass | |
| 13 | A second fresh run with a box from elsewhere; its question, No and Yes | As designed | Pass | |
| 14 | KeePass import | A file that needs a key file is refused; the sample brings 2 in and leaves 3 out | Pass | The 3 are right; the note's wording stays as written |
| 15 | Change the backup password | The boxes take the new one | Pass | |
| 16 | After a click on a row: Up and Down, and one wheel notch | The highlight moves; one notch turns one page, silently | Pass | Quick-fill and Clipboard history |
| 17 | Left and Right in a search with text; a letter in the middle; a click on the text; Delete and Backspace | They act at the caret; an empty search switches the filters | Pass | Quick-fill and Clipboard history |
| 18 | Arabic in the search (`كتاب` + `لا`), clicks and letters | The caret follows; letters stay joined | Pass | |
| 19 | A suggestion chip in the middle of a line | Suggestions follow the caret; the chip lands at it | Pass | |
| 20 | A hidden line: one press, a click on a dot, Delete | One dot each | Pass | The Arabic caret on an empty hidden line sits on the right |
| 21 | Space in Quick-fill's search and in Clipboard's | Types at the caret in English and Arabic; on Clipboard's empty search it ticks the highlighted row | Pass | At 90bd7076 |
| 22 | Shift, Space, then a letter | A small letter | Pass | At 90bd7076 |
| 23 | A fill after a row click; fast repeated taps on Both | The value lands in the page and the next letter goes to the page | Pass | |
| 24 | The Mail entry's values; the Backup pages in Arabic | Right values; right Arabic | Pass | |
| 25 | The three themes in light and dark, and high contrast | The panel follows each look | Pass | |
| 26 | Narrator in Quick-fill and Clipboard history | It says names, buttons work with it on, and it never speaks the real password on a hidden line | Pass | At 90bd7076 |
| 27 | Regressions: focus stays in the app; above the Start menu; Right-click mode | As before | Pass | |
| 28 | Quick-fill after the test data was deleted | It opens empty, with its invitation and no backup | Pass | At 90bd7076 |

## Notes

1. A window test that failed in about 1 run of 12 found a real bug: Windows can place a lift with its owned windows after another thread's hidden IME window out of the topmost band, move nothing and still say yes. `lift` now sends the window up alone when it stayed out, which may also explain the older G9 note on installed-shadow lifts (not proven).
2. G9 reports `pass:false` only as its outline part says "no room above the pad": the keyboard sits near the top of the screen, as in the P5 report's note 8. Its flag steps all show the guard on, nothing missing and the front window kept, with nothing left to put back or left open.
3. Not checked by hand: a real USB stick (a renamed folder stood in), USB hard disks (left for later), Google Drive (not tried), Bring back of a conflict copy (tests in `actor/tests/backup/beside.rs`), and the memory check after a day (fake-clock tests; the system clock is never changed). The live picture test is opt-in and runs on CI, so check 8 stands for it here.
4. The owner saw joined words in the chips in one app's text field before typing, and nowhere else; this is noted in `docs/items/P13.md`.
5. All test data was deleted with the owner's yes: the notebook and backup blobs, the test folders and the OneDrive box.
6. P9 reached GitHub as one commit, so the commit IDs here come from its full history, kept on the development PC.
