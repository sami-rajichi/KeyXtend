# 27. The hold engine decides in the hook, behind a port

- **Date:** 2026-10-13
- **Status:** Accepted.

## Context

- Spec §5.1 asked the mouse hook's callback to do "no work beyond a queue push". Windows needs the callback's answer (pass or drop the event) before it returns, and the hold engine must drop a press it holds back, so the answer cannot wait for another thread.
- P1's gate G5 ran the engine inside the callback: 20 of 20 holds in four rounds, plain and admin, with a hook p99 of 0.066 ms and 0.053 ms (ADR-0013).
- Windows silently removes a low-level hook that exceeds `LowLevelHooksTimeout`, and says nothing.
- Modules never call the OS (ADR-0005), and the engine must run on macOS and Linux later (P15).
- The keyboard runs with uiAccess (ADR-0003). If it replayed injected input, a program with fewer rights could pass its clicks through us into an admin window.

## Decision

Decided with the owner's approval of P4's design on 2026-10-13.

1. **A port.** `PointerHook` in `kx-module-api`, gated by the `PointerHook` capability, takes a `PointerEngine` from a module and runs it on the platform's hook thread. Windows uses `WH_MOUSE_LL`, plus `WH_KEYBOARD_LL` for Esc; the fake drives it in tests; macOS (`CGEventTap`) and Linux (XInput2) follow in P15.
2. **The engine decides in the callback.** One pure step runs there: no waiting on locks, no I/O, no logging and no allocation, as the host lends it a reused buffer.
   - Our input goes out after the callback returns, from the hook thread, in one `SendInput` batch whose `dwExtraInfo` carries `REPLAY`, a mark of its own: our hook skips it, and kx-gates' mouse guard drops it unless the gate drives the engine.
   - Events the engine lets through wait behind our input until it comes back through the hook, so nothing overtakes it (from P1).
3. **Only physical input reaches the engine.** Injected events, ours or another program's, always pass untouched. Only a test build (the app's `gate-input` feature with `--sim-input`) lets the gate tool's tagged input stand in for a hand, and `cargo xtask stage` refuses that feature.
4. **The hook runs only while needed:** a mode is on, a modifier is latched, or a carry is in progress. The mouse module starts and stops it.
5. **Our own windows.** The host asks Windows which window is under each press, and whether our process owns it.
6. **Health check.** Every `[mouse] health_ms` while the hook runs, the host compares Windows' last-input time with the last event its hooks saw. When they missed input for longer than `health_slack_ms`, it puts both hooks back.
7. **Timing.** The host keeps a fixed histogram of callback times. When the hook stops, the module logs the event count, the p99 and the reinstalls (numbers only, never a point), and G8 reads that line.
8. **Letting go.** Stopping the hook lets go of everything held: the button and any modifiers. The app's panic hook asks the hook thread to let go and waits briefly.

## Consequences

- Spec §5.1's hook discipline now reads "a pure step in the callback", and §10's 1 ms p99 budget is measured on the product's hook (G8).
- G5, G12, G17, G18 and the hand try run against the product's test build, not the spike.
- The engine is tested with unit and property tests and a fake clock. The host's ordering, timer and health logic is tested over a fake OS.
- In Right-click mode, a normal click reaches the app at its release, held back for the click's length (about 400 ms for the owner, ADR-0024).
- A forced kill during a carry leaves the button down until the next physical click.

## Alternatives

- **Queue the event and decide on another thread:** impossible, as Windows needs the answer before the callback returns.
- **Drop every event and replay all of it from another thread:** doubles each event's delay, and turns all mouse input into injected input, which some apps and games treat differently.
- **Raw Input (`WM_INPUT`):** it sees input but cannot drop it.
- **Our window boxes kept by the UI:** stale for a moment after a move, and it misses tips and lists.
- **A hook that is always on:** it would watch the mouse when nothing needs it.
