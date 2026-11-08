# Security note: kx-mod-selection

- **Owner:** product owner
- **Last review:** 2026-11-08 (P10 Task 10 security check, its first fix round and the Task 17 fix round: the actor, the module and Translate's consent, against the code)
- **Status:** Reviewed
- **Scope:** `crates/kx-mod-selection`: the manifest, the pure rules (`chips`, `strip`, `link`, `settings`), the actor on its `kx-selection` thread, and the short `kx-selection-open` thread each Translate opens. It runs inside the uiAccess `keyxtend` process and reaches the OS only through the `SelectionWatch` and `WebLink` ports (ADR-0041).

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `ReadFocusedText` | It gates `SelectionWatchKey`, so the module can see the focused box's selection with no text and read it once for Translate. It also gates the keyboard's `Typed`, `TypingModeAsk` and `AimedLine`, which end the caret's Paste and hide the strip over a panel line (`manifest.rs:20`). | The start fails, so no strip shows (`each_missing_gate_fails_the_start_and_watches_nothing`). |
| `Selection` | It gates the chips' events: `ChipsView` to the ui, `ChipAct` from it, `EditAsk` to the keyboard and `EditDone` back. | The start fails before the watch starts, so nothing is watched (`module.rs:73`). |
| `OpenLink` | It gates `WebLinkKey`, which hands Translate's https link to the desktop shell. Only this module holds it. | The start fails, as Translate is one of the module's keycaps. |

- The module holds no `InjectInput`: the keyboard sends every edit's keys (`it_holds_the_three_gates_and_never_injects_input`).
- The events it hears (`PointerPressed`, `PointerReleased`, `AssistChanged`, `BoardPlace`, `PressesAsk`, `NoticeAnswered`) and sends (`PressesWanted`, `BoardPlaceAsk`, `AssistAsk`, `Notice`) need no capability (`the_events_it_uses_need_only_the_gates_it_holds`). Their fields are flags, a place, Grab's state, an asker or a notice, so none carries text.
- `Typed` carries the user's text, so its relay keeps only whether a piece was a break (`module.rs:157`; `a_typed_piece_reaches_the_inbox_only_as_whether_it_was_a_break`).

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| A look's `SelSpot` (box id, kind, line boxes, screen) | Sensitive (where the user works) | Memory only, in the strip machine; `ChipsView` carries the boxes to the ui; `Debug` hides them | No | Until the next look, press or failure |
| The selected text, read once at a Translate click after the yes | Sensitive | Memory only: `Redacted` from the port, then a `Secret` (`Redacted<Zeroizing<String>>`) while the link is made, so no format shows it (`actor/translate.rs`, `fetch`); the link builder's cleaned and encoded copies are plain strings | No | The `Secret` is wiped when `fetch` returns; the working copies are freed, not wiped (see Accepted) |
| The Translate link, which holds the encoded text | Sensitive | Memory only: a `Redacted` `Link`, moved to the open's thread | No | Until the shell returns; then the browser holds it, by the user's consent |
| `translate_ok` and the other `[selection]` values | Public | The settings file, through the module's `SettingsHandle` | No | Until the user changes them |
| The question `notice.translate-ask` with the site's name, and `notice.translate-failed` / `notice.keys-not-sent` | Public | Memory only, bus events | No | Not kept |

The module keeps no history, writes no other file and logs no text, link or box.

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| The watch's tell | It only sends `Wake`; the look runs on the actor's thread (`module.rs:118-129`) | The platform paces looks by `settle_ms` and `looks_per_s` | `the_watchs_tell_only_wakes_and_the_look_runs_on_the_actors_turn` |
| `look`, `still`, `read`, `clip_holds` answers | Typed results; a failed look is `Lost` (hidden), a failed read is `translate-failed`; a `Closed` box shows no chip and is never asked for a clip or read | Each call within `read_ms` on the platform; a read capped at the smaller of `translate_chars` and the site's limit, plus one character (`actor/translate.rs:121`) | Strip property tests (`strip/props.rs`); actor tests below |
| The selected text | `Site::link` drops hidden marks, turns breaks into spaces, cuts at a whole word within `translate_chars`, the site's limit and `link_max`, and percent-encodes every byte outside the unreserved set; a read that hit its cap goes through `Site::cut_link`, which also ends at a whole word when cleaning made it shorter. `Link::new` checks https, characters and length | `translate_chars` (50–5000) and `MAX_LINK_BYTES` | `each_link_fits_its_caps_and_ends_at_a_whole_word`, `a_capped_reads_link_never_keeps_a_word_the_read_cut` (proptest), `link/tests/cut.rs` |
| `sites.toml` | Compiled in and checked at first use; a bad table turns Translate off and never panics | Fixed at build time | `link/sites/tests.rs` |
| The system's display language tag (`SelectionModule::new(os)`, Ruling J) | `Site::target` gives the site's own code, its fallback or one lower-case language part that passes `tag_ok`, for any string | A tag of at most `TAG_CHARS` | `any_tag_gives_a_language_tag_and_never_panics` (proptest) |
| Bus events (`ChipAct`, `EditDone`, `Typed`, `AimedLine`, pointer, Grab, place, answers) | Relayed into one inbox; an act needs the live round and its keycap, an answer must equal the raised question | One message per event; the inbox has no bound (see Accepted) | Actor tests over the fakes |

## Threats (STRIDE-lite)
- **Spoofing (a forged answer):** `NoticeAnswered` is ungated and names no sender, so any first-party module could publish one; the bus is in-process, so no other program can.
  - Only an answer equal to the whole raised notice (key, module, answers and the site's name) counts, only while it is pending and within `consent_s` on the injected clock, and only its Yes saves `translate_ok` (`actor/translate.rs:64-87`). A late Yes saves nothing and says `notice.translate-failed`.
  - Tests: `a_forged_or_other_answer_opens_nothing_and_leaves_the_question`, `a_yes_within_consent_s_opens_and_a_later_one_does_not`, `a_late_yes_says_the_page_did_not_open_and_a_late_later_says_nothing`, `later_opens_nothing_and_the_next_click_asks_again`, `the_first_translate_asks_by_the_sites_name_and_reads_nothing_before_the_yes`.
  - A forged `ChipAct` needs `Selection` and the live round, and does only what the click would; Translate still needs the yes (`an_act_or_an_answer_for_another_round_does_nothing`).
  - Two questions with no id, and the ungated `BoardPlace`, are Accepted below.
- **Tampering (a box swapped during consent):** the user may move while the question waits. The Yes reads only while the strip asked about still shows, as a moved or new selection gets a new round. Then a fresh look must find the same box of the same kind, and `still` must hold, right before the one read (`actor/translate.rs:83-87`, `:113-118`, `actor/acts.rs:50-63`).
  - Tests: `a_yes_after_the_selection_moved_saves_and_opens_nothing`, `the_box_is_checked_again_after_the_wait_right_before_the_read`, `a_box_that_turned_closed_during_the_wait_is_never_read`, `a_selection_gone_by_the_click_is_never_read`, `a_click_while_the_question_waits_points_its_yes_at_the_latest_strip`.
  - The read names the box `still` just confirmed, and the platform answers `Closed` in the same job when another box has focus, so a focus move between the check and the read sends nothing (`a_read_for_a_box_focus_left_is_closed_with_no_text_call_in_the_same_job`).
  - Copy and Cut check `still` before the keyboard sends a key; Paste needs a fresh look of the same editable or terminal box (`a_cut_whose_selection_went_hides_and_sends_nothing`, `a_terminal_takes_its_own_keys_and_one_whose_selection_cleared_gets_none`, `paste_looks_again_and_needs_the_same_editable_or_terminal_box`, `paste_on_a_box_focus_left_sends_nothing`).
  - The gap between that check and the keyboard's send is Accepted in `kx-mod-keyboard.md` ("Check, then send").
  - `sites.toml` is compiled in; the settings are validated, and a bad value goes back to its default (`tests/selection/settings.rs`). `translate_ok` is Accepted below.
- **Logging / repudiation:** the module logs fixed texts and error kinds only: a refused watch (`module.rs:127`), an unsaved consent, a failed read, a failed open and a panicked open (`actor/translate.rs:80`, `:140`, `:155`, `:159`), and a panicked actor (`module.rs:52`). `SelError` and `LinkError` print fixed texts with no box or link; `WriteError` logs its kind.
  - Test: `a_failed_read_open_or_save_logs_no_text_link_or_box` captures every line and finds no text, link scheme or box row. The inbox never holds typed text (`a_typed_piece_reaches_the_inbox_only_as_whether_it_was_a_break`).
- **Information disclosure:** no text is read before a click, and only Translate reads, after the user's one yes; Copy, Cut and Paste read nothing, and the caret Paste asks only the clipboard's formats (`a_click_at_a_bare_caret_shows_paste_when_the_clipboard_holds_anything`).
  - A `Closed` box (a password box, our own windows) shows no strip and is never read (`a_terminal_never_shows_cut_and_a_closed_box_shows_nothing_and_is_never_read`).
  - The text goes only to the site the user agreed to, by name, in their own browser; nothing goes to the network from this process.
- **Denial of service (an over-long or hostile selection, a hung app):**
  - A long selection is read once with a cap and cut at a whole word (`a_long_selection_is_cut_within_translate_chars_at_a_whole_word`); hidden marks and mixed scripts are cleaned and encoded (`arabic_and_french_are_encoded_and_hidden_marks_dropped`); a blank one opens nothing (`a_blank_selection_opens_nothing_and_tells_nothing`).
  - A hung app's look or read times out within `read_ms`: the strip hides until it answers again, and Translate says it failed (`a_failed_look_hides_until_the_app_answers_again`, `a_failed_read_says_the_page_did_not_open`).
  - A shell that holds the open holds only its own thread; the actor goes on and the stop never waits on it (`a_held_open_holds_only_its_own_thread_and_the_stop_never_waits_on_it`). A refused open or a thread that did not start says so (`a_refused_open_says_the_page_did_not_open_and_shows_no_tick`, `an_open_whose_thread_did_not_start_says_the_page_did_not_open`).
  - An edit in flight blocks only a second click on the same strip, so a keyboard that never answers leaves the next strip working. A failed edit or open is told even after a newer act took its place (`a_failed_edit_for_an_older_strip_still_tells_the_user`, `a_failed_open_for_an_older_strip_still_tells_the_user`).
  - An edit or open still unanswered after `edit_ms` on the injected clock hides its strip and says `notice.keys-not-sent` or `notice.translate-failed`, so no strip stays dead with no word (Ruling AA, `actor/tests/deadline.rs`). A late answer then ticks nothing, though a late no tells the user again.
  - A panic in the actor hides the strip, lets presses go and raises `notice.module-failed` (`a_panic_on_the_actors_thread_hides_the_strip_lets_presses_go_and_tells_the_user`).
- **Elevation:** with uiAccess the keyboard could send keys into an admin window, so no key goes without the user's click on a live keycap of the focused box, re-checked first; the module itself never injects input.
  - The link is an https link that `Link::new` checked, and `WinWebLink` hands it to the desktop shell, so the browser starts with Explorer's rights, never ours (`kx-platform-windows/src/link/win.rs`).
  - The crate forbids `unsafe` (`lib.rs:4`); no code outside the tests unwraps data from outside.

## Findings
### Must fix (blocks Definition of Done)
- None open.
### Should fix (tracked issue)
- None open.
### Accepted (with reason)
- `NoticeAnswered` and `ChipAct` name no sender: every module is first-party and compiled in, and a forged one does no more than the user's own click and answer could.
- The inbox and the open threads have no bound: events come from the user's clicks and the paced watch, and each open thread ends when the shell returns.
- The link builder's own working copies of the text are freed, not wiped: the text is Sensitive, not Secret, and the user's yes sends it to the browser anyway.
- No distinct Esc signal exists (Ruling O), so every `Break` from our keyboard (an arrow, Esc, a shortcut) hides a selection strip until the next look. The app's own event brings it back after Shift+arrows or Select all (`our_esc_as_a_break_hides_a_selection_strip`).
- `translate_ok` is the user's consent in the plain settings file, so a process running as the user could set it and skip the question. That is the trust boundary of every other setting.
- Two questions raised at different times are equal `Notice`s with no id, so a Yes on an old banner still open answers the newer question. The site and the text are the same, so it sends nothing the user did not choose.
- `BoardPlace` is ungated, so a forged `Open` while the keyboard is in the tray shows chips there. Only first-party, compiled-in modules can publish it, and the chips still need the user's click.
