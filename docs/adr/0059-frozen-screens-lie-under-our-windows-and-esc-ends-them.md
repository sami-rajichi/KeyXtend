# 59. The frozen screens lie under our windows, and the hand's Esc ends them

- **Date:** 2026-11-25
- **Status:** Accepted (owner, 2026-11-26). Adds to ADR-0056 (frozen screens, shielded windows) and ADR-0057 (Snip takes keys after a press); supersedes nothing.

## Context

- In the owner's run, Snip's frozen screen trapped him. The capture bar, its hint and the keyboard bubble showed, but under the frozen screen, so no click reached them, and nothing else closed it.
- Each frozen screen is a topmost window Qt makes for each snip. Windows puts a new topmost window on top of the others, and showing a hidden window keeps its place. Our other windows were made at start, so the frozen screens came out above them, and nothing moved them back.
- Our keyboard's Esc reaches Snip only after a press on its windows (ADR-0057), and a press could not reach them. People with a physical keyboard expect Esc to end a screen snip, as in Windows' Snipping Tool.

## Decision

1. **Under our windows.** While frozen screens show and no system dialog sits in front, they lie under every other always-on-top window of ours: Snip's bar, hint, strip and panel, and the keyboard or its bubble.
   - The window guard's `lay_low(title)` puts them there on the UI thread with `SetWindowPos`, never activating a window. A window that already lies right does not move.
   - The hub asks as each window of ours shows, as each Snip stage shows, and after a frozen screen moves or changes scale. QML gives the frozen screens' title, as it does for a shield.
   - Guard runs spare Snip's windows with their shadows and tips, so while a system dialog shows they stay where Qt drops them, under it and out of the topmost band. Once the dialog closes, the next view lays the frozen screens under ours again.
2. **The hand's Esc ends them.** A new `EscWatch` port hears the Esc key system-wide, only while frozen screens show and no system dialog sits in front.
   - On Windows it is a low-level keyboard hook on its own thread, `kx-esc-watch`, which starts and stops with the frozen screens.
   - The hook reads only the key's code and marks: it compares the code with Esc and the marks with ours. It skips our own input (our keyboard's `TAG` and the pointer hook's replays) and a held key's repeats, and logs nothing. Another program's Esc, as Voice Access or another on-screen keyboard sends it, counts as the hand's.
   - The hook only posts a message to its window. That window's procedure then runs the hub's callback on the same thread.
   - Each press sends `SnipClick::Esc`. Over frozen screens it does what our keyboard's Esc does: it closes the Snip panel, ends a pick, or closes a strip, asking first for a picture not kept.
   - At the close question over frozen screens, Esc from either source means Don't save, so a stray Esc only asks and a second always ends the snip. The editor's question keeps Back.
   - While a region's frame has a handle held, the first Esc only drops the hold, and the next goes on as above (plan 2026-11-26).
3. **Esc still reaches the app.** The hook never drops the key, as our pointer hook never does (ADR-0027).
   - Our windows never take the focus, so the app in front heard every key during a snip anyway.
   - A dropped key down whose key up passes, or the reverse, can leave the app with a stuck key when the watch stops in between.
   - A hook that eats keys system-wide is a power we do not need, and security tools treat it as a key blocker.
4. **At any other time, Esc is the app's.** During the countdown, a recording, in the editor, at rest, or with a system dialog in front, no watch runs. A late `Esc` there does nothing. Our keyboard's Esc still follows ADR-0057.
   - In the editor, our keyboard's Esc drops a held Crop handle first, then Crop's waiting frame, then reaches the close question (plan 2026-11-26).
5. **No timer ends a snip.** The bar's Close, our keyboard's Esc and the hand's Esc are the ways out. Snip's `keyboard = "hidden"` setting stays.
6. **Other systems.** macOS and Linux implement `EscWatch` in P15. Where the system lets no app hear keys, as on Wayland, the watch is refused and the other ways out stay.

## Consequences

- The bar and the keyboard always take clicks over the frozen screens.
- A system-wide keyboard hook now runs while frozen screens show. The security notes list it, and no key, text or count leaves it.
- The hook's thread also runs the hub's callback, short locks and a bus send, while every key system-wide waits, so it must stay short: Windows quietly removes a hook whose callback runs too long. It gets no health check (ADR-0027 decision 6).
- Voice Access and other assistive programs can end a snip with Esc, as the hand can.
- No guard holds Snip's windows in the band, as in most snips before, when no guard run came while they showed. A program that drops one out of the band leaves it there.
- Only a live run proves the order over real windows, with uiAccess and on mixed-DPI screens, and a real key's path through the hook.

## Alternatives considered

- **Show the frozen screens first, then the others:** the bar and keyboard already exist, and showing keeps their place, so they would still lie under.
- **Make the frozen screens non-topmost:** every app window would then cover them.
- **An automatic close after a time:** the owner refused it. A user who reads slowly would lose the snip.
- **Drop the Esc key:** see decision 3.
- **Guard Snip's windows like the others:** a guard run while frozen screens show would lift them back over a system dialog and the keyboard, and trap the user again.
- **Esc means Back at the close question:** Esc after Esc would only show and hide the question, and never end the snip.
- **Skip every injected Esc:** Voice Access and other on-screen keyboards could not end a snip.
