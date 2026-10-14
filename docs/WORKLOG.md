# Work log

Newest first. One entry per implemented feature or fixed issue: a timestamp plus 3–4 sentences at most.

## 2026-10-14 07:00 — P4: the spike's hold engine and ring test deleted
No gate uses the spike's hold engine, its `[hold]` and `[ring]` settings, its ring test or kx-test-support's frame stats any more, so they are deleted. The screen point `Pt` that the spike and kx-gates still share moved into its own spike file. The spike's test strip lost its ring switch, and the move map and the P4 notes now say G12 times the product overlay with PresentMon.

## 2026-10-14 06:30 — P4: G12 judges lost frames as G8 does
G12 now counts the ring's lost frames by G8's frame rule, through the same helper. It skips a swap chain's first present and one replaced within a refresh, so the overlay's first frame no longer fails a smooth ring. When `g8 hook-weak`'s test build does not quit, kx-gates ends its job, lets go of anything left down, and then starts the owner's copy again. It also ends the job and lets go when the test build fails to start.

## 2026-10-14 06:00 — P4: G12 needs every ring to draw, review fixes
G12 now fails unless the overlay drew frames in every press and none of them went unshown, and it leaves out any point a test build window already covers before Right-click turns on. Its PresentMon capture is checked at load to cover every press at its longest, and G5's and G8 hook's rounds to fit the mouse guard's limit. When `g8 hook-weak`'s test build does not quit, kx-gates ends its job and then lets go of either button and the modifiers. PresentMon rows without a time are now skipped and counted in the result.

## 2026-10-14 05:00 — P4: G12 under the product's overlay, and G8's hook p99
G12 now presses 200 points of target-window under the test build's own pointer overlay, each press long enough for the ring to play yet still a click, and checks every click arrives and nothing blocks it. PresentMon, shared with G8, records the test build, and the overlay's frames are judged only inside the ring's time, so the keyboard's frames never count. The new `g8 hook` and `g8 hook-weak` run G5's cases on the test build and hold the hook's logged p99 to spec §10's 1 ms. G6's face sizes are now checked at load, and a G18 error no longer shows an empty part.

## 2026-10-14 04:00 — P4: G6 scrolls in the document, cleanup on every path
G6's stand-in face now sits over the document's lower part, as the keyboard does, so its scroll point lands in the document's body, and G6 stops when that point is off the document; with no UI Automation scroll pattern there it skips only that route, and its result gives the point as an offset in the window. G5 and G18 turn Right-click off even after a failed case, and G18 reads the latch at once after its click. G5 also refuses a drag inside the still radius at the screen's DPI, and target-window refuses a hang cap at Windows' hung-window mark.

## 2026-10-14 03:00 — P4: G5, G17, G18, G6 and the hand try drive the product
G5, G17 and G18 now drive the product's test build, turning Right-click, Grab and the latches on with its own keys, and kx-gates' own copy of the hold engine is gone. G5 parks target-window clear of the keyboard, measures the still radius at the screen's DPI, adds a hung-target case, and judges the hook by its logged numbers; its admin-window case is now the owner's manual check. G6 polls the pointer for its scroll target until the product's tracker comes in P5, and the hand try reads the hook's numbers from the owner's running copy.

## 2026-10-14 01:30 — P4: no console for any release build, review fixes
Every release build, the gate-input test build too, now opens no console window, which would have taken the foreground; a second build flag (`kx_no_console`) carries this, and the uiAccess flag keeps only the manifest and the load pinning. kx-gates now refuses to read its settings or start the test build when Windows will not say whether it has uiAccess. When the owner's copy is not started again after the test build, kx-gates prints one line saying to start it from the Start menu. Nested `target` folders are now ignored by git.

## 2026-10-14 00:30 — P4: the gates' test build, the p4 settings and the guard's replay rule
kx-gates can now start the product's gate-input test build, from the plain kx-gates only and with a data folder of its own, after asking the running copy to quit, and starts that copy again when the test build ends. It reads the test build's settings, turns its mode and latch keys on by clicking them, and reads the hook's numbers from its log; the new `kx-gates-p4.toml` holds these settings. With uiAccess, kx-gates now reads its five settings files only beside its exe, the gate-input build never asks for uiAccess, and the mouse guard passes the test build's replayed presses only for the gates that drive it (G5, G12, G17 and G18).

## 2026-10-13 23:40 — P4: the overlay keeps the tip, a badge-sized shadow
Showing the pointer overlay now joins a quiet guard run, so a key's tip the owner is reading stays and a banner or list keeps its place in front. The badge casts the mock-up's own small shadow, a new `[common] badge_shadow` token each palette takes and high contrast drops, and the overlay window only leaves room for it. Flipped near an edge, the badge keeps the base offset, and the ring's timing tests now check the animation lengths directly.

## 2026-10-13 23:05 — P4: the pointer overlay, its ring and badge
While Right-click or Grab is on, one click-through window follows the pointer: the hold ring (moved from the test build) plays once per hold, its waves running the hold time less the ring's delay in the theme's ring colour, and bursts once when it fires, with the burst skipped in reduced motion. A pill badge beside the pointer shows the mode's key icon, adds "Holding" while Grab carries, moves out with a large Windows pointer, flips near the work area's edges (which the hub now sends with the state) and fades out over our windows. The app reads the state once on the frame after each wake and makes the window only while a mode is on; Right-click and Grab, dimmed while mouse assist is off, no longer press or send.

## 2026-10-13 22:15 — P4: the keys light after the keyboard, review fixes
The keyboard's wake on a mouse mode change or a used latch is now heard only while a keyboard is attached, by handlers added after the keyboard's own, so the window always reads the new state instead of sometimes waiting for the next relabel. A hold or a fire that comes while no mode is on no longer shows a ring or a burst. The pointer badge loses its radius size, as a pill takes half its height, and a test checks that the mouse module's stop comes before the UI ends.

## 2026-10-13 21:30 — P4: the pointer overlay's state, and keys that light at once
The hub now keeps what the pointer overlay will draw: the mouse modes, the pointer while a mode is on, a hold's ring (its waves run for the hold time less the ring's delay) and the burst when it fires, read by QML as one JSON with the Windows pointer size, newly read from `CursorBaseSize`. A run of pointer moves costs one wake per read of that state, nothing is kept while no mode is on, and no point reaches a log. A mouse mode change or a used latch now wakes the keyboard window, which reads its state at once instead of at the next relabel. The badge's sizes and fade, and the texts "Pointer helper" and "Holding" in English, French and Arabic, are ready for the overlay itself.

## 2026-10-13 20:35 — P4: key and mouse sounds
Every key press on the keyboard now plays the key sound once, after the key went out, taking the six Cherry takes in turn; a held key's repeat, the top bar and the language list stay silent. A hold that fires, Grab's pick-up and its drop play their own sounds from the mouse module's notes on the bus. Each sound has its own `[ui]` switch (all on), `sound_volume` (50) scales every gain under the Windows volume, and `sound_idle_ms` sets how long the speaker stays ready, with a floor of one second so it never rests under a sound.

## 2026-10-13 19:30 — P4: sounds as checked, compiled-in data
A new token file, kx-theme's `sounds.toml`, holds the shipping format, the speaker's voices and each sound event's files and balance: the key's six Cherry takes in turn, the mouse click, the mouse press and the card laid down. kx-ui's build reads each named file with a strict reader that refuses anything but whole 16-bit mono PCM frames at the token rate within the length limit, fails naming the file, and embeds the samples 2-byte aligned with a table of events, so the app never reads audio at run time. `cargo xtask sounds` now reads the format from that file through the same checks and reads recordings with the same RIFF chunk walker, and a recipe file that still holds `[format]` is refused.

## 2026-10-13 18:30 — P4: a Speaker port with an XAudio2 adapter
A new `Speaker` UI port plays compiled-in mono PCM16 clips: `play` only puts a message on a bounded queue, so the UI never waits on audio, and `UiPorts` now carries it (nothing calls it until the sounds are wired). On Windows a thread of its own loads `XAudio2` from System32 at the first play, keeps a few voices per clip so quick plays overlap, replaces a busy voice rather than queue behind it, and stops the engine after the configured silence. A lost device or a failed start rebuilds the engine once per play, a bad clip is skipped, each failure kind warns once, and dropping the speaker destroys the voices before the engine, waiting a second at most. Tests drive every rule over a fake system and never play sound; the fake speaker records what it was asked.

## 2026-10-13 17:15 — P4: the app turns mouse assist on, with a panic release and a gate-only build
The app now ships the mouse module by default and hands it Windows' mouse hook, and its panic hook asks that hook to let go while it keeps running, so a panic during a Grab carry never leaves a button down and one the kernel catches never ends mouse assist. A gate-only build (`gate-input`) takes `--sim-input`, which makes the engine act on the gate tool's input instead of the hand's and logs a warning; every other build refuses that flag like any unknown argument. The stage check refuses a program that holds the flag, `dev-install` refuses to sign one, and CI now runs the real-hook test and the gate build's tests.

## 2026-10-13 15:32 — P4: the Windows mouse hook host for the hold engine
`kx-platform-windows` now runs the hold engine on real low-level mouse and keyboard hooks, on a raised thread of their own: the hand's input reaches the engine (or, only in `sim-input` builds, the gate tool's `SIM_TAG` input), and our own `TAG` and `REPLAY` input only ends the wait for its echo. Input the engine lets through waits behind our unechoed input in a bounded queue where moves coalesce and buttons keep a headroom, an Up that had to pass ahead of its queued press is sent again after it, and an Esc press from the hand or our keyboard reaches the engine. Lagging hooks go back after the engine lets go, judged by when the callbacks ran, a panic never reaches Windows, and dropping a run sends the release, then anything the host's own record (which the hand's own Ups also clear) shows still held, before removing the hooks, waiting at most one second. Tests drive every rule over a fake OS; the one real test only installs and removes the hooks.

## 2026-10-13 14:25 — P4: the keyboard's Right-click and Grab keys, and its latches on the bus
The side keys Right-click and Grab now ask the mouse module to flip its mode over the bus, light while it is on, and stay dimmed with the tip "Mouse assist is off" until that module says it runs; at start the keyboard tells its latches and asks for that state, and it never depends on the mouse module. Every change of the latched modifiers goes out on the bus, newest last from any thread, and a mouse click that used them lets them go as the `modifiers` setting says. A change from the bus redraws at the next relabel, and the new tips read naturally in English, French and Arabic.

## 2026-10-13 13:26 — P4: the mouse module runs its engine on the hook only while needed
The mouse module now runs an actor thread that starts the pointer hook when Right-click, Grab or a latched modifier needs it, and stops it once the engine is idle, logging the hook's numbers and never a point. It hears the keyboard's toggles, latches and state requests over the bus, and tells the keyboard and the UI the state, hold, grab and pointer notes; moves coalesce to one in flight, always at the latest point. Right-click is saved when the user changes it, not while Grab pauses it, and comes back at once after a restart. When Windows refuses the hook for a mouse key or a restored Right-click, a new notice in English, French and Arabic says to press the key again; a refusal for a latch only logs a warning, once per streak.

## 2026-10-13 12:57 — P4: a fake mouse hook, and `max_acts` on the engine port
`kx-platform-fake` now has `FakePointerHook`, a stand-in for the Windows mouse hook that the mouse module's tests drive on their own thread, with time only from the test. It runs the engine's `on` then `settle` for each fed event and records the input they send in order, counts starts, stops and wakes, and calls `release` when the run is dropped. The engine port gained `max_acts`, the most input one call may push, so a host reserves its buffer once; the fake panics when an engine pushes past it.

## 2026-10-13 12:16 — P4: the hold engine with Grab at once and modifier + click
`kx-mod-mouse` now has the pure hold engine: a still hold right-clicks, Grab picks up at the next press with no wait and drops on the next click, and a click on our windows, Esc or a toggle cancels a carry, with Right-click coming back after. The keyboard's latched modifiers go down around the next left click outside our windows in any mode, a latch change mid-click never alters that click, and the still radius scales by the press's DPI. Property tests over random input found that a late hold's right-click could reach the app after the press that revealed it; that press now follows it. The engine also lets go of stale modifiers or an awaited release when the hook misses one, and its hot path never allocates.

## 2026-10-13 12:10 — P4: the mouse module's crate and its `[mouse]` settings
The new `kx-mod-mouse` crate declares the mouse module: it needs the pointer hook and holds the `PointerHook` and `InjectInput` capabilities, and its start reads `[mouse]` and keeps the hook for the engine to come. `[mouse]` holds Right-click mode, the hold time (0.5 s to 3.0 s, 1.5 s by default), the still radius, the ring's delay and the hook's health and echo times, each with a range, and the ring must always show before the shortest hold. The crate joined the workspace and CI's Linux list, with a README and a draft security note. The `kx-new-module` skill now names `boot.rs` and the flat `qml/` folder.

## 2026-10-13 11:34 — P4: the pointer hook port and the mouse-assist bus events
`kx-module-api` now has the `PointerHook` port of ADR-0027: a module hands its `PointerEngine` to the platform, which runs it on the hook thread, and dropping the returned `HookRun` is the only way to stop. Eight new bus events let the keyboard, the mouse module and the UI talk without calling each other; the toggle and latch events need `InjectInput`. A click's point can show what was clicked, so the notes and the pointer move hide every point in their debug output, and the latch event shows only how many are held.

## 2026-10-13 11:03 — P4: sound candidates tool (`cargo xtask sounds`)
`cargo xtask sounds render` turns the recipes in `assets/sounds/recipes.toml` into WAV candidates from filtered noise and damped partials, with three jittered variants per key sound, the same bytes on every run. `sounds import` turns an 8, 16 or 24-bit or float, mono or stereo WAV into the shipping format, 48 kHz 16-bit mono, trimmed, faded and peak-normalised, and refuses a damaged file without a panic. Ten first recipes (four keys, two clicks, two pick-ups, two drops) wait for the owner's ear beside the CC0 recordings. The owner chose four CC0 recordings, now in `assets/sounds/`: a Cherry key in six takes, a mouse click joined from press and release with the new `sounds join`, a mouse press for pick-up and a card laid down for drop.

## 2026-10-13 00:35 — P3: G26 waits for the labels instead of reading them once
G26's first CI run passed German, Russian and Persian, but read Hebrew's keys once, 550 ms after the switch, while the runner's cold debug build had not yet listed them; seconds later all 48 keys were found and 100 Hebrew characters typed intact. The gate now reads again until every key is listed and named right, up to `[g26] label_wait_ms`, and reports the time taken as `labels_ms`. The next run passed all four: Hebrew's labels were right after 7.4 s against 0.64–0.68 s for the others, and P14 holds that slow first switch.

## 2026-10-12 23:48 — P3: text measures the same at every display scale
The first CI run of the Qt Quick tests, at 100 % scale, found Dolch's AltGr label shrunk a step and its `_` touching the 8 on the French and Arabic boards. Labels are drawn as exact curves, but Qt still measured them with pixel-snapped (hinted) widths, which only differ at 100 %; every label is now laid out unhinted, as it is drawn. All 345 Qt Quick tests pass at 125 % and at 100 %, and the board's pictures at 125 % are pixel for pixel the same as before.

## 2026-10-12 22:53 — P3: property tests for the UI's state machines, and screen-reader names
The device rebuilds, the guard's tries, the tray slot and the app's mode, Quit and waker lifecycle each gained a property test over any sequence of their inputs, with limits read from the `[ui]` defaults. The Qt Quick tests now find a chip by the word it shows, in English and Arabic, and check the bubble's and the move catcher's accessible names. No property test found a product bug.

## 2026-10-12 23:20 — P3: the first CI run's two failures
GitHub's Windows runner keeps its temp folder under a short (8.3) name, so the dev shortcut's remove step, which compared paths as text, missed its own shortcut; it now compares long paths, and its tests compare resolved paths. On Ubuntu a kx-ui test missed a warning because `tracing` cached a log line as unheard while another test thread installed its own capture; one capture now serves the whole test binary and sorts warnings by thread. The full suite passes here with a short temp folder, and the capture tests passed 40 runs in a row.

## 2026-10-12 22:52 — P3: the Caps Lock light follows a press outside the keyboard
The owner's hand check found that Caps Lock pressed outside KeyXtend left its light unchanged until the next key click, as nothing asked for a redraw. The relabel tick now also redraws the state when Caps Lock changed since the last state shown, so the light follows within `relabel_ms`. Narrator read the keys and top-bar buttons clearly in the same check.

## 2026-10-12 22:34 — P3: the branch review's three fixes
A let-go now ends a key's hold only when that key is the one held, so a late let-go from another key never stops a repeat. Both settings files now share one range check in `kx-module-api`. The frame log's setting and the kx-ui security note now say its press times show the typing rhythm, so it stays off while typing anything private.

## 2026-10-12 22:20 — P3 done: the keyboard core (v0.1)
The keyboard types English, French and Arabic from a no-focus uiAccess window, with modifiers, key repeat, act on press, three looks, the bubble and the tray. The owner passed every hands-on check, and G1–G4, G9–G11 and G25 passed; G13 and G8 are partial with the owner's acceptance (test report 2026-10-12). Start-up at about 527 ms against 500 and a frame count that disagrees with the screen after a reboot go to P14; a tip that misses after a look change goes to P13.

## 2026-10-12 17:46 — P3: G8 says which tip key misses and which churn step grows
G8's churn now polls through each tip rest and reports each key's hits and show times, with warm-up round 1 kept apart, and reads memory after each step of a round with a fitted slope. It showed the miss is whichever key is hovered third after the bubble and theme steps, not Shift, and that the themes step carries most of the commit growth.

## 2026-10-12 15:11 — P3: a pressed key lights as fast as Windows' own keyboard (ADR-0025, ADR-0026)
A pressed key now draws its press frame first and types right after, so its light shows in a median of 14.2–14.6 ms on the 144 Hz development PC, against 28.1 ms before and 14.2–14.5 ms for Windows' On-Screen Keyboard measured the same way. G8's rule now matches Windows' keyboard: the median within 2 frames and the slowest within 3, each plus the median screen copy. Frames keep the refresh wait, as showing them at once dropped 142 frames in G8 (ADR-0026). A frame log (`[ui] frame_log`) times each step of a press for later work.

## 2026-10-12 13:20 — P3: letters type into scrcpy, the phone mirror
scrcpy reads letters and numbers only as real key presses, so the keyboard's text never reached the phone while Space, Enter and Backspace did. `[keyboard] key_apps` now lists `scrcpy.exe` by default, so it gets key presses in its own layout, and the owner confirmed typing in WhatsApp through it. A one-click switch to add any app to that list is saved for P13.

## 2026-10-12 08:30 — P3: keys act at the press, so a slip never cancels them (ADR-0024)
A key now types the moment the button goes down, as on a real keyboard and Windows' On-Screen Keyboard, so a hand that slips while the button is down no longer cancels or redirects it; a held key repeats until let-go wherever the pointer slips. Every other control acts at let-go wherever the release lands, and the move grip and corner ignore a double click's second click. G2 now clicks like a person, each press held 0.4 s with a 2 px slip, and the owner's hands-on check passed with natural slips in English, French and Arabic.

## 2026-10-11 22:09 — P3: the gates guard the owner's mouse and count frames only while keys animate
While a gate drives the mouse, a guard drops the owner's own mouse input and counts it; one physical click gives the mouse back and stops the gate (owner's choice). The guard ends the gate when Windows silently drops its hook, which happened while osk.exe was running, and it stops after a time limit. G8 now counts lost frames only while a pressed key sinks and rises, timed by each frame's present, and the pop-up churn compares the medians of its first and last rounds.

## 2026-10-11 19:53 — P3: a pressed key shows at once, in its own shade
A pressed key sinks on the next frame, so a quick click always shows. Light looks and high contrast give it its own darker shade, made from one shared press token; the owner approved it from rendered pictures.

## 2026-10-11 17:10 — P3: hold a key to repeat it
Holding a character key, Space, Enter, Backspace, Delete or an arrow types it again after 0.8 s, then 10 times a second, as on a real keyboard; with Ctrl, Alt or Win latched, a held key acts once. The delay, the speed and an off switch are settings (`[keyboard] repeat`, `repeat_delay_ms`, `repeat_every_ms`), which P13's settings window will explain in EN, FR and AR. A `Repeat` state machine with an injected clock decides each tick, and one timer per window ticks only while a key is held.

## 2026-10-11 16:20 — P3: G8's churn warms up for five rounds and lists each round
G8's pop-up churn now runs five warm-up rounds before it measures (owner's decision), and its result lists the memory after each measured round, so a failed run shows a steady climb or a single jump. A round that fails keeps the readings taken so far, and the app's debug churn shows a key's tip, the language list and the bubble each round.

## 2026-10-11 11:40 — P3: freed heap pages go back after a look change (ADR-0023)
After a theme or light/dark change, the app asks Windows once to give back the heap pages the old look freed, through the platform's memory port. The first look change still keeps about 19 / 31 MB in Native, which P13 investigates.

## 2026-10-11 10:43 — P3: three leaks fixed in the theme switch
Qt 6.11's soft shadow never freed its default material, a theme switch let QML read the new look before its fonts loaded, and each shown window kept the old look's cached graphics. All three are fixed, and only debug builds hold the churn's test QML.

## 2026-10-11 01:20 — P3: the key diet, each key makes only what it shows
Each key builds only the parts its kind needs, an icon provider tints the icons so no icon needs its own effect, and glows and lights exist only while they show. Each icon is a texture of its own, so the saved pictures repeat exactly.

## 2026-10-10 23:15 — P3: texts carry no hidden bidi marks (ADR-0022)
Texts no longer carry LRM and RLM marks, which made Qt search every font for them. The language list draws OS names plainly, and Arabic joins a list with و, so Latin names keep their order.

## 2026-10-10 20:50 — P3: one graphics device for every window (ADR-0021)
All windows share one graphics device and draw on the GUI thread, which took idle memory from about 215 MB to about 127 MB in a harness and stopped each pop-up from adding a device of its own. Only the look's fonts load, from Rust before QML, and a lost graphics device rebuilds the windows where the user left them. The budget is now 100 MB working set and 140 MB commit (G8).

## 2026-10-10 18:00 — P3: fix round F, windows and G9
After a scale or screen change, the keyboard keeps its place in pixels and is placed again from fresh screens, also when the change comes during a move or resize, so a moved keyboard is never lost off-screen. The shadow hides during a live resize instead of trailing it, and the guard brings back a dropped window's whole family, owners included, and checks again after another program's call ends. G10 keeps no other app's labels, and G9 records each window's owner and guard state; G9's pass waits for the next installed run.

## 2026-10-10 17:30 — P3: fix round F, key faces in Arabic and Dolch
Arabic letters sit centred again, and a key whose base is an Arabic letter shows no AltGr corner, as on Windows' own keyboard; the owner chose this from rendered pictures. A key that types nothing with Shift or AltGr shows empty and a click sends nothing, from one rule in the OS-free core, while a Ctrl, Alt or Win latch still sends the shortcut. Corner legends are 20 % smaller with their own top, bottom and side margins, a Quick test fails when any corner touches a main legend, and a new picture test (`KX_PICTURES`) renders every theme for review.

## 2026-10-10 16:10 — P3: G8 measures commit, a warm-up and pop-up churn
G8 now reads private commit beside the working set and judges both (100 MB and 140 MB, ADR-0021), after a set warm-up and never trimming. A churn of tips, the bubble, the theme cycle and the List design's language list checks that memory stays flat, after PresentMon's capture so frame drops are judged as before. G11's top scale is 175 %, the largest Windows offers on the development PC's screen.

## 2026-10-10 12:50 — P3: fix round E, display scale, Arabic marks and legends, and the GPU
The windows now keep their size and place across a display scale change: Qt left each native window as it was, so a new `Rescale` part sets the bound values again, and the keyboard cuts itself and is placed again in the new pixels. In Dolch, marks on their dotted circle take Noto Sans Arabic (Rubik lacks U+25CC), and a wide Arabic letter over an AltGr legend rises 6 px so they no longer touch. The app now pins Qt to the lowest-power GPU through DXGI; it was already on the AMD iGPU here, and the large memory comes from one Direct3D device per shown window, a finding for a later ADR.

## 2026-10-10 11:08 — P3: fix round D, the last fixes before the module-done gate
The window guard now stops a repeat lift per window, so a window dropped inside another window's lift is lifted back too, the likely cause of G9's lost shadow, and G10 matches right Shift as Windows reports it. A resize saves only a size it changed and never one the settings refuse, the Keys route sends `AltGr` as left Ctrl and right Alt, the D-pad stays inside its columns, and placement keeps the resize corner on the work area. A key takes its tip back on leave even when its note went, a failed G10 step lets its latch go, and G13 finds the keyboard before it reads the look settings. G10's texts now name the keys it skips, and the keyboard's security note says the P3 manual check must still confirm who can press a key.

## 2026-10-10 10:30 — P3: screen readers can toggle latch keys and language rows
UI Automation's Toggle on a checkable control asks Qt for its toggle action, not the press, so a reader that toggled Shift or a language row did nothing. Keys and rows now answer both actions the same way, and a dimmed key still ignores both. The reader checks moved into their own Quick test file.

## 2026-10-02 14:18 — P3: kx-review fixes for the gates, the test window and the docs
G10 now presses every key the way a screen reader does, except dead keys and keys that type nothing in the layout in front, and checks what reaches the test window, and G13 stops early when the keys cannot take the accent and compares only the keys' solid faces. G1 to G3 now give a verdict (G2 also checks the caret), G19 and G23 report partial when part of the gate did not run, and G23 now tests the password. The test window logs each key and has a name screen readers read, G26 types every character key, and G9 and the launchers put back or close exactly what they changed or opened. The security notes are marked reviewed, later work is in the item files, and the move map shows where P3 left each spike part.

## 2026-10-02 13:04 — P3: round B's second pass, a size floor and one row source
A hand-edited `size_min` now has a floor (0.8, where every key cell stays 24 px across), as `size_max` has a ceiling; a value past either goes back to its default with the reset notice, and the keyboard's defaults name both bounds. A size step that lands on another screen fits that screen, the fake sink refuses a key the defaults lack as the store does, and a `Legend`'s debug print hides what Shift and Caps Lock show. The drawn check takes the board's row from the keyboard face instead of a copy in the theme, so the accent waits for the face, and the theme README names the check's limit at the capital height. The stage errors moved into their own `StageError`.

## 2026-10-02 12:24 — P3: the look's contrast measured where the text sits
The drawn check now places every legend as the keyboard lays it out, from the board's row, the theme's gap and its font's metrics, so it no longer asks for colours darker than readability needs. Every colour the owner approved is back unless it still fails where its text sits, and those moved by the smallest step that passes: the light rings, the Dolch teals, some legend_2 greys, and Dolch dark's hover at 4 % (Native and ET66 keep 8 %). The lit size letter is plain text with a ring-coloured pill under it, Cycle's current dot is larger, and the bubble's icon, the tray menu's icons and a hovered row's badge are checked too. The plain-text check reads one-line and quoted text elements, and the README accepts the see-through plate until Settings turns it on (P13).

## 2026-10-02 11:25 — P3: kx-review fixes for the modules, kernel, platform and tools
The keyboard now opens no larger than its screen: `size_max` has a ceiling, the size step and presets must fit and rise, and a size too big for the screen it lands on is shrunk for the run without being saved. Debug output hides latches and Caps Lock, both modules share one Arabic script table, a property run pins the dead-key wait and what each key types, and the bubble comes back when the keyboard module restarts while minimised. Settings refuse a key the defaults lack, a file nested too deep counts as damaged, tidy D6 bans toml's `unbounded` feature, kernel errors are logged by kind, and the fake host and sink refuse what the kernel refuses. On Windows the guard refuses kept styles that never land, the tray reports a message it cannot queue, a hidden window's loop ends once stopping, and the stage check refuses any file off its list.

## 2026-10-02 10:05 — P3: kx-review fixes for the keyboard's look and QML
kx-theme now checks every text, icon and mark the keyboard really draws, on its cap's gradient, under hover and press and on the see-through plate, and the Dolch, ET66 and Native tokens were nudged until all pass; the Carousel's faded side names are the owner's one named exception. A system accent stays only while the keyboard stays readable, and high contrast draws flat caps with Windows' own colour pairs. Every control now gives UI Automation its text key as its id, the move catcher's title is translated, `close` became `putAway`, and new Quick tests cover the two-click move and resize and units on a scaled screen. A static test keeps all QML text plain, the guard holds no borrow while it moves windows, and an open language list closes on a click even with one layout left.

## 2026-10-02 00:30 — P3: final review fixes for saving, the language key and tests
After a save fails on a locked settings file, moving or resizing the keyboard back now changes the value the store keeps, so stop no longer saves the abandoned one. With one layout installed, the language key does nothing in every design, the List included, and its Arabic tip reads «لا يوجد سوى لغة واحدة مثبّتة للوحة المفاتيح». A full message queue can no longer hang quitting, a refused `[ui]` value is logged by its kind, and `Typing`'s debug print hides the typing state. Several tests now prove what they claim (the locked file, a poisoned lock, a blocked module, NaN pixels, the plate token, golden layout ids), and the window tests find keys by their UI Automation id.

## 2026-10-01 23:57 — P3: final review fixes for tools, docs and item notes
The stage now counts its 80 MB budget in MB of 10⁶ bytes, as G8 and spec §10 do, and a test keeps G8's folder limit equal to the stage's. G10 and G26 now expect a key that types Arabic marks to be named as the product names it, in words or on a dotted circle, and two long tests, a G9 comment and the spike clock's unit are tidied. The roadmap, spec, security notes, ADR-0016, CONTRIBUTING, credits and work log wording follow the review. P3's item file now lists the owner's look, tray, tip and Arabic AltGr checks and G8's accepted weak-mode `partial`, and the notes for later items moved to P6, P7, P11, P13 and P14.

## 2026-10-01 23:40 — P3: final review fixes for Arabic marks and the language key
Arabic AltGr marks now sit on a dotted circle in the Arabic font, and screen readers name mark keys by their Arabic names. With one layout installed, the language key's tip and description say so. The keyboard relabels as soon as it shows, the window guard reads the pop-up's title at each try, and relabel_ms takes 50 ms at the fastest. Small review gaps are closed: a second-Quit test, a tip-alone z-order test, a Qt wide-environment probe and several doc fixes.

## 2026-10-01 — P3: the docs pass
The spec, roadmap, architecture map, move map, README and CONTRIBUTING now describe the keyboard as built, with the Qt set-up and the P3 edge cases. The three P3 security notes are corrected, and docs name settings instead of repeating their values. `kx-review` now fails a `tracing` call on an input path that could log typed text. Each P3 item note is marked handled or waiting for the manual check, or moved to the item that owns it.

## 2026-10-01 — P3: the late review's fixes for gates G8–G26
G8 now keeps the PresentMon it starts and reads the CSV only once PresentMon has stopped, and a copy it cannot open or cap gets Quit until it listens or ends. G8 and G9 size the keyboard back after every run, and G9 fails when no size press changed anything. G9 also writes click-through on to every window, and the guard must refuse it where the window took clicks at first. G13 joins a part that changed while the keyboard followed, and the G26 job prints its logs after any failure or timeout.

## 2026-10-01 — P3: the guard keeps the keyboard on top and clickable, and settings are peeked safely
The window guard now lifts a guarded window straight back when another program drops it out of the topmost band, which `HWND_NOTOPMOST` does without `WM_STYLECHANGING`. It keeps click-through only on windows that had it when first guarded, so no program can make the keyboard click-through. `kx_settings::peek_section` reads a section through the store's own reader, so migrations run, a bad value takes its default and a newer version gives the defaults; `[keyboard] relabel_ms` gets a ceiling, so a hand-edited value cannot stall the labels or the gates.

## 2026-10-01 — P3: the review's fixes for gates G8–G26
Weak mode now caps the job at 30 % of each allowed core's time, and the spike's voice worker gets the same fix; `g8 weak` reports `partial`, never a pass, until software drawing is decided. G8 measures idle CPU per core and memory in MB, fails lost characters, starts PresentMon itself when `[g8] presentmon_exe` names it, and asks a copy it could not cap to quit. G13 needs dark mode, the accent and high contrast to each change, G9 sizes the keyboard back to its first size and toggles click-through only where it was, and G11 judges one build's series. The G26 CI job prints the logs after a failure, and the roadmap points G26's Japanese part to F16.

## 2026-10-01 — P3: the gate review's fixes for `close` and G25
`kx-gates close keyxtend` now posts Quit to the running copy's command window and starts nothing, so no program can borrow the gates' uiAccess rights. G2, G4, G10 and G25 wait one relabel period read from the running copy's own settings, and record that wait in their result lines. G25 clicks past the language key's back part, using the theme's `back_share`, and skips the List design with a reason. A new `kx_settings::peek_section` reads one section the way a later copy of the app does, and both the app and the gates use it.

## 2026-10-01 — P3: gates G8–G11, G13 and G26
G8 times the product's start-up from its own log, idle memory and CPU, its folder size, key highlights, resizes, click-to-character and a bus publish against spec §10, with a `weak` mode capped like the spike's, and reads a PresentMon CSV when the owner records one. G9 checks the window flags after outside changes, hiding, showing and resizing; G10 walks and presses every key by UI Automation; G11 records size and sharpness per display scale; G13 times how fast the keyboard follows Windows' look and saves pictures for the owner. G26 checks German, Russian, Hebrew and Persian labels and typing on GitHub's Windows runner, where `--layout` now reports a layout that did not come back. The top bar's buttons and the bubble give UI Automation their `bar.<id>`, and `capture-layouts` follows the layouts module's scan list.

## 2026-10-01 — P3: gates G2–G4 and G25 aim at the product
Each key now gives UI Automation its board id (`esc`, `c-0x001E`, `lang`), and gates G2, G3 and G4 find the product keyboard and click each key at the centre of the box UI Automation reports, so the top bar no longer shifts their clicks. G25 clicks the product's language key, after one relabel period so the key steps from the layout in front. `kx-gates close keyxtend` runs the app's own `--quit` and waits for its process to end, while `close qt` still closes the frozen spike face; the `--quit` flag now comes from the one command table the app and the gates share.

## 2026-10-01 — P3: the spike keeps no keyboard part
The spike's keyboard controller (`kbctl`), `sysui`, `langinfo`, `facecfg` and `lookcfg` are deleted, with the face-only helpers left in `layout`, `screen`, `window` and `place`. `spike.toml` drops `[look]`, `[size]`, `[keys]`, `[bar]`, `[assets]` and the guard lines, and keeps `[keyboard]` only as the frozen face's title and stage-1 geometry for the gates and the tools row. Gate G7 now reads the face's Settings key name from `kx-gates.toml [g7] opener`, and spike-core no longer depends on kx-theme or kx-mod-layouts.

## 2026-10-01 — P3: CI builds and tests the Qt app
The Windows check now installs the Qt that the stage settings pin (a pinned action over the reviewed aqtinstall), sets `QMAKE`, runs every workspace test, the Qt Quick tests (any warning fails the job) and `cargo xtask stage`. The Linux check names its packages in lists that must cover the workspace, builds kx-ui without Qt and leaves the app out. Tests that need a font Windows ships share one rule that skips them by name on an image without it, and the label-fit test moved to `tst_fit.qml`. A G26 job stays skipped until its gate exists.

## 2026-10-01 — P3: pop-ups land in front, and tips hide for the guard
The window guard port now takes a `top` title, so a banner or tray menu that shows again at its old place lands in front of the keyboard; a native test shows a menu shown again under the keyboard and in front once named. A shown tip now hides as a guard run starts, since a guard run that lifts it also lifts the keyboard over the banner, which a second native test pins. A tip's lift is now checked to find the tip shown, and static checks keep the window titles distinct across the three languages and every window file topmost. The keyboard's place helpers moved to `Spots.qml`, so `KbWindow.qml` has room again.

## 2026-10-01 — P3: the stage review's small fixes
`dev-install` now works out the Start-menu shortcut before it signs, installs or removes anything, so an `APPDATA` outside the user's profile stops it with nothing changed. The stage settings refuse `libs` or `plugins` that are not `.dll` files, which the check would never read, and a network library or importer missing from `libs`, so a rename cannot leave the guard idle. The agree test now finds the app's own QML module by the URI kx-ui exports, and requires every plugin a staged `qmldir` names, optional ones too, unless a library the program links registers that module (today only Qt Qml).

## 2026-10-01 — P3: tips in front of every window, and a steady guard test
A tip now lifts itself alone to the front as it shows (`SWP_NOOWNERZORDER`, through the window guard port), so a top-bar tip no longer hides under the banner or the language list that the guard lifted after the keyboard; a native test shows the old bug and that the keyboard stays under the banner. Each tip now stays on the screen under its button, so a keyboard across two monitors keeps every tip by its key. The flaky guard test now starts its window topmost, as Qt makes ours: under load it failed 1958 of 3000 rounds before and 0 of 5000 after, and 2000 runs of the window tests passed. A Qt Quick test checks that every guarded window asks Qt to stay on top, a rule now in the guard's doc and `ARCHITECTURE.md`.

## 2026-10-01 — P3: the stage review fixes
The stage check now lets only Qt Qml and Qt Quick import Qt's network library, takes only the reviewed Qt 6.11.2, and a `kx-ui` test keeps network addresses and `XMLHttpRequest` out of the app's QML. A stage that fails its copy or check is removed, and `dev-install` re-runs the check on an app folder before it signs anything. An agree test maps every QML import to a staged `qmldir`, and the dev shortcut must land inside the user's profile.

## 2026-10-01 — P3: the tip fix pinned by a native test
A Windows test now lets the guard lift a window over two hidden tips and checks that only the tip it owns stays above it; it fails when the tip has no owner. The Qt Quick tests also check each tip's no-focus, click-through and tool-tip flags, a quick sweep over four keys, and the bubble's wait and screen, and every hover test lets the real pointer pass through its windows (`pointer.js`). Each tip now stays on its own window's screen, so the bubble's tip no longer borrows the keyboard's. Top-bar tips under the banner or the language list are noted in `docs/items/P3.md`.

## 2026-10-01 — P3: `cargo xtask stage` and the Start-menu shortcut
`cargo xtask stage` builds KeyXtend in release and stages it in `target/stage/keyxtend` with only the 21 Qt files it was measured to load (46 MB of the 80 MB budget), replacing the spike's PowerShell staging. Its check reads every staged program's import table and refuses TLS, network-information and QML debugging plugins, any import found neither in the stage nor in System32, and any library nothing imports. A run of the staged release app loaded everything from the stage and nothing from planted Qt paths, `PATH` or the working folder, and wrote nothing to `%LOCALAPPDATA%`. `dev-install` now adds a "KeyXtend (dev)" shortcut to the user's own Start menu, and `--remove` deletes it.

## 2026-10-01 — P3: key tips no longer hide behind the keyboard
Each run of the window guard (when the keyboard, a banner, a menu, the list or the bubble shows) lifts the keyboard above every other window, and the tip window, owned by no window, then showed behind it, so the tips of keys, the D-pad and the resize corner vanished. The keyboard now owns its tip and the bubble owns its own, so Windows keeps each tip above its window (ADR-0013); a new language still closes both. Qt Quick tests check the owners and hover every tipped control in English, French and Arabic, at the smallest size and in another theme.

## 2026-10-01 — P3: key icons back on whole pixels
Only named keys that show a label are now centred exactly; keys that show an icon are centred by anchors again, so their icons sit on whole pixels of their key, as in P1. A test checks both at each size preset in every theme, and that a key centres again when a theme drops or brings back its icon. The label fit test now also runs each preset, in case one sits off the size steps.

## 2026-10-01 — P3: Qt's variables cleared by the names Qt reads
Qt reads the C runtime's narrow copy of the environment, where the ANSI code page can turn a fullwidth `ＱＴ_` or a non-ASCII last letter into a plain Qt name, so such names could slip past the clearing. The app now clears every name with a Qt prefix in that copy, and its manifest names the UTF-8 code page outright, under which no name narrows to a Qt one; tests check both. One cfg from the build script, set with the manifest's uiAccess, now also turns on the pinned load paths and hides the console.

## 2026-10-01 — P3: the keyboard speaks the language of the layout in front
The words on keys, tips, screen-reader names, the banner, the language list and the size letters now follow the language of the layout in front, so an English layout shows Esc and Del even on a French Windows; the tray icon and its menu keep the system's language. A layout language without texts gets English, as does the time before a layout names one, and `[ui] language` can still pin one. Arabic now flips text only: no bar, list, banner or menu mirrors. ADR-0020 records the owner's decision and supersedes ADR-0017's language rule.

## 2026-10-01 — P3: the app's start hardened for uiAccess
The app now clears Qt's variables before Qt starts and turns off Qt Quick's shader cache on disk, so no other program of the user can load code into the uiAccess process or leave a cache it reads. Release builds load libraries only from the app's folder and System32, and Qt's plugins and QML modules only from the app's folder. A later copy now honours the user's `[ui] send_tries` and `send_wait_ms` and runs itself when the running copy quits meanwhile, a failed boot is logged at the kernel's defaults, and a panic's place reaches standard error until the log opens.

## 2026-10-01 — P3: the first real app types
`apps/keyxtend` now runs the kernel, the layouts, keyboard and UI modules and Qt over the Windows adapter, and types into other apps without taking focus. A second copy hands Show or Quit to the running one, with tries timed by the new `[ui] send_tries` and `send_wait_ms`, and stops when no copy answers. The log opens between the settings load and the modules' start, a panic logs only its place, and Quit stops the modules, ends the UI and removes the tray icon. Tests cover the start-to-stop order over fakes, the texts, icons, paints and scan codes the board needs, and that the language shown always matches the legends.

## 2026-10-01 — P3: key labels centred exactly
Named-key labels are now centred by hand, since anchors rounded their centre to whole pixels and pushed some labels up to 0.8 px past Dolch's top face. The fit test now checks every size step from the smallest to the largest, in one hidden window per theme and language. The size preset limit moved to the keyboard face port, so the UI no longer depends on the keyboard module.

## 2026-09-30 — P3: tray menu review fixes
The tray menu now stays open for `[ui] tray_grace_ms` (500 ms) when another window comes to the front just after it opens, as the Windows 11 hidden icons can do, and a press on the banner or the language list closes it too. The icon's changes go to the taskbar from its own thread, so a hung Explorer can no longer freeze the keyboard, and the icon is sized and redrawn for the taskbar screen's scale. Arabic rows in the language list no longer clip, and the root's wiring moved into `App.qml`, where Qt Quick tests check every way the menu closes. ADR-0019 records the tray decision, and ADR-0016 is back to its accepted text.

## 2026-09-30 — P3: labels fit Dolch's top face
On Dolch's sculpted caps, a named key's label now fits the lighter top, plus a 1 px allowance per side from a theme token, which is how far P1's AltGr already ran. The fit test covers every size preset and the largest size, and checks that the theme's own font draws each label. The keyboard's settings now refuse more size presets than the top bar names.

## 2026-09-30 — P3: notice banner, tray icon, language list and theme button
The keyboard now shows notices in a themed banner that queues them, counts what waits and offers Try again only when the app can restart what failed. A tray icon, drawn as a theme badge, opens a menu with Show keyboard and Quit, through a new `Tray` port with a Windows adapter and a fake. The language key gains its Cycle design (globe, name, one dot per layout) and its List design, whose list pages five rows and is driven by the board's arrows, Enter and Esc; the top bar gains a theme button. A press on the keyboard also closes the menu, and the Arabic UI mirrors the banner, the menu and the list.

## 2026-09-30 — P3: size buttons and key labels fixed
After a Smaller or Bigger step between presets, no size button stays lit, and each preset in the settings gets its own button. English and Arabic keys print the P1 words again (Caps, AltGr, Back), and every label shrinks to fit its cap, down to a theme token's size, so French words such as "Échap" stay on the key's face. The Qt Quick tests now load the bundled fonts and icons as the app holds them, and any warning fails them.

## 2026-09-30 — P3: Quit stays the last request
Try again now reaches the app while the hub holds its lock, so a second copy's Quit can no longer overtake it, and a race test pins the order. If the app's request thread is gone, Quit now ends the keyboard's windows itself. The supply-chain notes and the security note were corrected.

## 2026-09-30 — P3: the keyboard's views move into the product
The keyboard window, its keys, top bar, D-pad, resize corner, bubble and tooltips moved from the test build into `kx-ui` with the same look, now drawn from the keyboard module's board and the translated texts. Keys that come later are dimmed and say "Coming in a later version", the Arabic UI mirrors the top bar, and fonts and icons are compiled into the app. Three P1 findings are fixed: text is gray-edged without ClearType fringes, hover shows a Highlight outline in high contrast, and icons are tinted with public Qt Quick only. Qt Quick tests now check the views over the real look, texts and board.

## 2026-09-30 — P3: UI review fixes: one quit path
Quit now has one path, from the tray or from a second copy: it reaches the app once and then closes the request queue, so the hidden keyboard process can always end. ADR-0018 records the one set-once hub the QML objects share. Tidy F3 now finds allows written over several lines and is not fooled by raw strings or nested comments, with property tests. The guard starts from its first try on each run, silent failures now warn, and `cargo deny` is clean.

## 2026-09-30 — P3: one table of hidden marks
The log and the notices now share one table of the characters that can split or reorder a line (`kx_module_api::HIDDEN_MARKS`, with `is_hidden` adding every control character). The log escapes them as before; notices now drop line breaks and control characters too, not only bidi controls, so a setting name holding a line break can no longer split a notice.

## 2026-09-30 — P3: the UI's Qt build and the `ui` module
`kx-ui` now builds the QML module `KeyXtend.Ui` with two bridge objects moved from the test keyboard: `Ui` for the look, texts, notices and commands, and `Keyboard` for the board, clicks, the no-focus guard and the window's shape. A new `ui` module owns the `[ui]` settings (theme, mode, language, timings and the contrast floor, which can never go below WCAG AA) and a hub that shows notices in the user's language and hands Try again and Quit to the app. Tidy's new rule F3 allows unsafe code in `kx-ui` only in its bridges, each block with a `// SAFETY:` comment. The texts, settings and hub still build and test without Qt, for the Linux checks.

## 2026-09-30 — P3: text review fixes
Arabic tips and screen-reader names now call keys that print a Latin word "مفتاح" and that word (مفتاح Esc, مفتاح Shift), keeping Microsoft's terms for the rest; the kernel's settings carry the name KeyXtend. French notices name the part in guillemets, and a reset that can name no setting now says only how many. Bidi controls inside names are dropped before they reach a text, and ADR-0017 was corrected on Qt Linguist, plurals and language choice.

## 2026-09-30 — P3: English, French and Arabic texts
Every text the keyboard shows now comes from one Fluent file per language, compiled into the new `kx-ui` crate: key names and labels, the top bar, the tray menu, notices, and module and theme names (ADR-0017). The UI follows the Windows display language (EN, FR or AR, else English), and a text missing in French or Arabic falls back to English. French keys print French words (Échap, Suppr, Verr. Maj) and Arabic keys the English words Arabic keyboards print; Arabic counts follow CLDR's plural forms, with a fix for a library bug from 103 up. Notice arguments are now typed (text, count, list), and the theme names and the space bar's joiner moved into the translations.

## 2026-09-30 — P3: the keyboard face and the language key
The keyboard now gives the screen everything it draws: each key's box, name and icon, the legends and lights, and dimmed side keys marked "Coming in a later version". The language key works in all three designs: Carousel, Cycle, and a List the arrows, Enter and Esc drive, five rows a page, with Arabic shown right to left. A click made while the app's layout just changed sends nothing and redraws, so the user clicks again, and failed sends or switches show a notice. Golden files pin the English, French and Arabic views for the Qt screens to come.

## 2026-09-29 — P2 done: kernel, settings and test tools
The kernel starts the feature modules in dependency order, keeps a failed or crashing module from stopping the rest, and offers Try again; modules talk through a typed event bus and share services gated by capabilities. Settings live in one readable `settings.toml` in the data folder that holds only the user's changes (ADR-0015): a bad value resets alone, a damaged file is set aside and the last good copy comes back, and the log keeps this run and the previous one with sensitive fields hidden. The P1 test tools moved into `tools/kx-gates` and `tools/kx-target-window`, and the Slint and Tauri prototypes are gone. Reviews found and fixed about 60 issues, 720 tests pass, and on Windows G1, G2, G5 and G12 passed again and the owner confirmed the settings file and its repair.

## 2026-09-29 — One item per session, with notes per item
Each item from `docs/FEATURES.md` is now built in its own session, starting from the latest `main` once the previous item's pull request is merged. Every phase has a notes file in `docs/items/` that says what the test round already built, what earlier sessions left for it, and the owner's saved ideas. Saying "idea for P13: …" in any session saves an improvement there, and `/kx-feature P13` on a finished item builds the saved ideas later. The notes from the P1 hand-off moved into these files, so nothing is lost or done twice.

## 2026-09-29 — CI fix for pull request #2
The Windows check failed because GitHub starts our PowerShell scripts from PowerShell 7, whose module path hides the certificate drive from Windows PowerShell; the dev tools now clear that path for every script they run. The Ubuntu check failed on a test-only constant that only the Windows tests use, so it now exists only there. The Windows failure was reproduced on this PC with the same error and passes after the fix, and a new test checks that the path is cleared.

## 2026-09-29 — P1 closed: Qt chosen, tested code kept
The toolkit test round is closed: Qt 6 Quick with a Rust core draws every window (ADR-0013, with every gate's numbers), and Slint and Tauri are dropped. By the owner's decision about 29,000 lines of tested spike code are kept and moved into the product phase by phase, each part fitted to the architecture and rules as it moves (ADR-0014, `docs/spike-move-map.md`). The roadmap, spec, architecture map, skills and licence list now describe Qt, and the gates not run move to P3, P14 and P15 (G15 earlier once the Qt keyboard builds on macOS and Ubuntu). P2 is ready.

## 2026-09-29 — P1: the hold ring (G12), icons and darker ring colours
A "Ring test" switch on the Qt face loops the mock-up's hold ring at the pointer in a click-through window, timed from the hold setting, and records its frame gaps. The G12 run passed: 200 of 200 clicks under the ring reached the app behind it, at 167 frames a second (slowest frame 7.9 ms); the G7 typing run also passed. The Copy pill, "Transcribing…" and the words bar now carry the mock-up's Lucide icons, and the ET66 and Dolch light rings are darker, reaching 3:1 on a plain window. Two review passes found about 40 issues, all fixed.

## 2026-09-28 — P1: the Arabic test panel for G7
The Settings key now opens a right-to-left Arabic panel on the Qt face, a stand-in for Settings: a header with a close button, five rows a page with page buttons, and an Arabic text field, in each theme's light and dark look and with system colours in high contrast. It is the only window that takes focus; it opens above the keyboard, lined up with its right edge, and hands focus back to the app when it closes. Tests pass (core 259, harness 116, Qt 8, 12 offline QML checks), and a live check through the accessibility API, with no typing into apps, confirmed the Arabic, the paging, the focus and the caret reader. Three review rounds found about 32 issues, all fixed; the G7 typing run waits for the owner's yes.

## 2026-09-28 — P1: the keyboard moves as in the mock-up
Each mock-up animation the spike has a part for now plays on the Qt face, timed from one new `motion.toml`: key presses, colour fades, a red mic key whose light pulses while recording, legends rising and language names sliding the way the key stepped, the D-pad's Stop popping in, and the pill and caption bar popping in, with a shimmer while words are on their way. Reduced motion, from the keyboard setting or Windows' animation switch, makes every length 0, and no loop runs at 0 ms. 13 new core tests and a scratch QML test of the real files, with full and reduced timings, pass; a live start with theme switches logged no errors, and the microphone was never used. A review of every changed file found about 20 small issues: all are fixed except three kept on purpose, listed in the hand-off notes.

## 2026-09-28 — P1: the Copy pill, voice caption and snip overlay follow the theme
The three small windows beside the keyboard now take each theme's look, light and dark, as in the mock-up: a badge-coloured Copy pill with a shadow, a caption bar with a red recording dot and the Arabic font for Arabic words, and a snip overlay that dims outside the region, with a size tag and a hint bar. Colours the mock-up writes once for all themes live in one shared `[common]` table, with system colours in high contrast, and their sizes in `shape.toml`. A preview of all six looks and live runs (pill on a real selection, overlay open and cancelled with no file left) passed; the microphone was never used. A review found 4 small issues, all fixed.

## 2026-09-28 — P1 stage 2 part B: the full keyboard on Qt
The Qt face now draws the designed keyboard in all three themes, light and dark, with tooltips, hover and press tints, sticky modifiers, the language carousel, click-move-click moving and resizing, and minimise to a bubble in a bottom screen corner. The owner's look check passed after colour fixes; their decisions (no labels under icons, no Esc undo, a remembered state, the settings strip only when asked) are in the spec and roadmap. Dead keys now reach the app as key presses so Windows adds the accent (the owner typed ê), Caps Lock results come from the layout, and a live bug where a move never ended at a screen edge was fixed. Three review rounds found about 45 issues, all fixed, including latches an accent key ignores and arrows that must keep an accent waiting.

## 2026-09-28 — P1 stage 2 part A: look tokens, geometry, size and system look
All three themes, each light and dark, now live in `spike/themes.toml`, and a review checked every value against the mock-up. spike-core gained toolkit-free parts that load and check those tokens, lay out the keyboard at any size, handle −/+, S/M/L and click-move-click resizing, and read Windows' dark mode, accent colour and high contrast. The fonts (IBM Plex, Rubik, Noto Sans Arabic) and 31 Lucide icons were downloaded with their hashes recorded. Eleven text colours in the mock-up are under 4.5:1 (ten in Dolch, one in ET66 light); a test pins that list until the owner decides at the look check.

## 2026-09-28 — P1: voice probe (G22)
A worker process now owns the mic, runs whisper.cpp's server once at start (base model, 0.8 s load) and can send clips to Groq whisper-large-v3 when cloud is turned on; both faces got a Mic button and a click-through caption bar. On both faces the bar stayed on top, Notepad kept focus, the words arrived 0.8–1.3 s after Mic stop, and the worker does not inherit uiAccess. Notepad lost most of a sentence typed in one batch, so dictated text is typed 10 ms per character on its own thread; speed against clip length was local 0.14, weak mode 0.36 and cloud 0.08, and base is weak for French and Arabic. A five-part review found about 35 issues, all fixed but two accepted, among them a loopback-only server address and length-capped worker messages.

## 2026-09-27 — P1: tools row, Copy pill, quick-fill and snip probes (G19 pill, G23, G24)
Both faces got a tools row (Fill user, Fill password, Snip), a Copy pill beside a selection, Windows Hello quick-fill and a full-screen snip overlay, built on shared spike-core code. On both faces the pill sat within 1 px of its place, left the app's focus alone even right after a click on the keyboard, and copied the line. The snip saved the picked region with 0 differing pixels, and with Hello not set up, Fill typed nothing. Slint needed its unstable winit hook to open windows without focus and a retry to lift a new window above the keyboard (Qt did both itself); a five-part review found about 40 issues, all fixed.

## 2026-09-27 — P1: selection, clipboard, password and language probes (G19, G20, G21, G25)
A line selected in Notepad or a Chrome text box (English, French and Arabic) is read back through UI Automation without touching the clipboard, and a pill position is worked out from its screen box. A clipboard listener heard all 25 test copies, skipped the 5 marked as private and pasted an older one back, keeping the owner's clipboard and removing test copies from the Win+V list. Password boxes are recognised in Chrome and in a Win32 box, the language key cycles English, French and Arabic in the app in front, and Ctrl+C/V/Z, Win+V and Alt+Tab work (Win+V only with uiAccess). Three reviews found about 50 issues, all fixed, including arrow-type keys now sent as extended keys and the listener opening the clipboard with its own window.

## 2026-09-27 — P1: Grab, modifier+click and scroll probes (G17, G18, G6)
The owner tried Right-click and Grab with their own mouse: both work. Automatic runs passed in Notepad, Chrome and Explorer: Grab selects text and moves a file (Esc cancels), latched Shift and Ctrl change the next click, and UI Automation, a posted wheel and a SendInput wheel all scroll a window the pointer is not on. Word was not tested: a fresh Word process shows a subscription-check dialog over the document. Two reviews found about 30 issues, all fixed.

## 2026-09-27 — P1: hold-to-right-click engine and G5
The hold engine turns a 1.5-second still press into a right-click, or into a grab in Grab mode, and passes normal clicks and drags through unchanged. It runs in low-level mouse and keyboard hooks on their own thread. G5 replays clicks, drags and holds into the test window, including one run as administrator: both passed 20/20, with the hook taking at most 0.1 ms. Two reviews found about 30 issues, all fixed.

## 2026-09-27 — P1: no-focus, top-band and admin tests
Slint and Qt both passed G2: 1000 clicks each, the target kept focus every time, and keys typed in about 4 ms (13 ms at worst). Both stayed above Start, Search and Task Manager (G3). The new `harness g4` opens Terminal as administrator and clicks 200 keys into it: both typed into it with uiAccess alone, Qt 200/200 and Slint 199/200 (the owner touched the mouse then). Covered keys are now left out of G2 and G4, and the click code they share lives in `clicks.rs`.

## 2026-09-27 — P1: test keyboards and the typing test
Slint and Qt test keyboards run with uiAccess; Tauri was dropped because WebView2 will not start under uiAccess. The harness typed 1000 random English, French, Arabic and AltGr characters into five apps: the test window, Chrome, Terminal and Word received every one (Word's AutoCorrect curls quotes), and Notepad drops a few only in fast bursts. The harness now checks where the keyboard focus really is, saves a screenshot when a pop-up blocks it, and moves on to the next app. A four-part review found about 65 issues in the spike code, all fixed or explained.

## 2026-09-27 — uiAccess dev tools
`cargo xtask dev-cert` makes a test signing certificate and a launcher the owner opens to trust it. `dev-install` signs a build and copies it into Program Files, and `check-uiaccess` explains why uiAccess is missing. Every admin step checks its inputs before the Windows prompt and can only touch `Program Files\KeyXtend-dev`. Two review rounds found about 60 issues, all fixed; 225 tests pass.

## 2026-09-27 — P0 Foundation done
The Rust workspace, its strict lints and the `cargo xtask` checks (tidy, dco, licences, dist stub) are in place, with 97 tests. CI runs fmt, clippy, tests and tidy on Windows and Ubuntu, plus cargo-deny, cargo-audit and a DCO check; all four jobs passed on the first pull request. The public repo protects `main` with a ruleset, blocks pushed secrets and accepts private vulnerability reports.

## 2026-09-27 — Public docs describe the product
The README, spec §1 and the project docs now describe KeyXtend and its target users, not one person. The GitHub username appears only in the repo address and CODEOWNERS. Personal notes for this PC and owner moved to the git-ignored `CLAUDE.local.md`, and `docs/NEXT-SESSION.md` is no longer tracked.

## 2026-09-26 — Toolkit re-check for macOS and Linux
Three research passes re-checked the UI toolkit for all three systems. Qt 6 Quick with a Rust core is now recommended over Slint, and Tauri is rejected on evidence (WebView2 fails under uiAccess, and it uses about 200–320 MB of memory). Phase 1 became a measured test round (ADR-0011): Qt vs Slint, with a quick Tauri check, on the development PC, in weak mode, on GitHub's Mac and Linux machines and in a Linux virtual PC. The owner set a download limit of 80 MB, chose the system accent colour for Native Adaptive, and made the Linux helpers opt-in.

## 2026-09-26 — Project named, moved to D:, rules added
The project is now called KeyXtend ("the eXtended keyboard"): Claude's recommendation, built on the owner's "Extended Keyboard" idea, and the owner can still rename it. The code prefix is `kx-`. New rules were added: nothing hardcoded, short texts, small files, review every file, plans without code, and push only with the owner's yes. New skills: `kx-review`, `kx-plan`, `kx-feature`. The future features F1–F12 are listed in `docs/FEATURES.md`.

## 2026-09-26 — Stack and architecture verified
Three research passes verified Rust + Slint (no Windows showstopper), free local AI (whisper.cpp), licences (GPL-3.0-or-later) and free signing (SignPath). The keyboard process itself must hold uiAccess, so untrusted data and the network moved to worker processes. The spec, `ARCHITECTURE.md`, ADRs 0001–0010 and the roadmap (P0–P15) were written.

## 2026-09-26 — Mock-up v2
The mock-up was rebuilt after the owner's review. Right-click now keeps normal clicks, the scroll pad has two speeds (third tap stops), and modifiers work like osk.exe. Clipboard and quick-fill lists are paged five rows at a time and work with the arrow keys, and panels scale with the keyboard. Three themes remain, each with a soft dark version.

## 2026-09-26 — Requirements and mock-up v1
Six rounds of pop-up questions defined the keyboard, the mouse helpers, voice, clipboard, the vault, snip and the window rules. Mock-up v1 compared five keyboard styles built from real references.
