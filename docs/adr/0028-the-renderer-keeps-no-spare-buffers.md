# 28. The renderer keeps no spare buffers, and G8 also reads after use

- **Date:** 2026-10-15
- **Status:** Accepted. Adds to ADR-0021: decision 7's Qt variables gain the buffer pool limit, and decision 6's G8 also samples after use.

## Context

- With Right-click on, the installed keyboard read 149.4 MB of private commit at idle after the hands-on checks, over spec §10's 140 MB. A cold start with the mode off read 137.8 MB, and the owner's copy reached 163.3 MB over one morning of use.
- An investigation in a harness running the real `App.qml` over the test stand-ins found that the overlay costs only 1 to 2 MB. The rest comes from use: each item that appears or goes in the keyboard window (a hover, a press, a lit light) makes Qt's batch renderer rebuild its batches.
- The renderer keeps the old vertex and index buffers for reuse, up to `QSG_RENDERER_BUFFER_POOL_LIMIT` bytes of each kind per window (2 MiB by default, Qt 6.11 `qsgbatchrenderer.cpp`). On the development PC's AMD driver, its own buffer pool then grows in 2 MiB steps and never shrinks.
- G8 read only an untouched keyboard, so it never saw this.

## Decision

Decided by the owner on 2026-10-15.

1. **No spare buffers.** After `harden_qt`, the app sets `QSG_RENDERER_BUFFER_POOL_LIMIT` from `[ui] buffer_pool_bytes` (0 by default, at most 8 MiB), beside ADR-0021's two variables. Each window's renderer then frees a batch's old buffers instead of keeping them, and Qt's own 2 MiB stays one setting away.
2. **G8 also reads after use.** After its cold reading, G8 sweeps the pointer over every key and turns a mouse mode on, lets the keyboard settle as before, and judges that reading against the same budgets. It also runs once with the Arabic layout in front. kx-gates builds this.

## Consequences

- Harness, three runs each: after two sweeps over every key and a heap decommit, commit was 153.9 to 155.2 MB with Qt's pool and 142.2 to 142.8 MB with none, 11 to 13 MB less. Under weak mode's caps, a mode and two sweeps added 2.7 MB instead of 6.1.
- A rebuild frame makes new buffers instead of reusing old ones. Its CPU time rose by at most about 1 ms at the median, and its p99 and worst stayed within noise, far below one frame.
- Qt reads the variable as each window's renderer is made, so windows made again after a device loss take it too. ADR-0013's window rules are unchanged.
- An outside value of the variable is cleared first, like every Qt name (ADR-0021).
- The harness is not the product. G2, G8 (key light and frames) and G12 run again on the product.
- An Arabic keyboard sat 8 to 10 MB above English in the harness, so its G8 run may show a budget the English run hides.

## Alternatives

- **A small pool, such as 256 KB:** not measured; expected between the two. It stays a settings change.
- **Release each window's graphics and decommit once the keyboard settles:** 4.8 MB back, from the heap only. A release drops pipelines, so the next frame hitches, and it must never run during typing or a hold; refused for now.
- **Slimmer keys (P13):** saves more at the base, but needs a QML rework and a look check; it stays a P13 idea.
- **Leave G8 reading only a cold keyboard:** it would pass while the keyboard breaks the budget in use; refused.
