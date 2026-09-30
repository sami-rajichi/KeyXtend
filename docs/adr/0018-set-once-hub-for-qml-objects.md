# 18. One set-once hub for the QML objects

- **Date:** 2026-09-30
- **Status:** Accepted. A single, bounded exception to ARCHITECTURE's "no global mutable state" pattern.

## Context

QML makes the cxx-qt objects `Ui` and `Keyboard` itself (ADR-0013), through their `Default`, with no arguments. Each still needs the app's `Hub`: the settings, the keyboard face, the UI ports and the request queue.

- ARCHITECTURE names the kernel's service registry as the only global.
- Handing the hub to an object QML made needs a route from Rust into QML. Every other route costs unsafe code or more glue:
  - an initial property or context property must carry a raw `QObject` pointer in a `QVariant`, which is unsafe code in the app, outside `kx-ui/src/bridges/` (tidy F3);
  - a QML singleton is also made without arguments in cxx-qt 0.10, so it would read a global all the same;
  - making the objects in Rust and giving them to QML needs C++ glue for every object type.

## Decision

- `kx_ui::install(Arc<Hub>)` stores the hub in one `OnceLock` in `kx-ui/src/bridges/mod.rs`. Each bridge object reads it once, when QML makes it.
- **Set once:** a second `install` returns false and changes nothing. The hub is never replaced or cleared.
- **Before QML loads:** an object made before `install` warns and does nothing.
- **One per process:** the app makes one hub before any window exists. The hub's own state stays behind its lock, so the global holds only the pointer.

## Consequences

- The hub lives until the process ends, so its request queue cannot close by being dropped. `Hub::quit` drops the queue's sender after the last request, and the app's drain loop ends on its own.
- A test binary can install only one hub. The hub's logic is tested without the global (`kx-ui/tests/hub`), and the QML load test runs in its own binary.
- A second UI with its own hub would need a new ADR that supersedes this one.

## Alternatives

- **Pass a raw pointer as an initial property:** no global, but unsafe code in the app and a pointer QML could outlive; refused.
- **A QML singleton:** it still reads a global in cxx-qt 0.10, with more glue; refused.
- **Objects made in Rust and handed to QML:** C++ glue per object type and unsafe code outside the bridges; refused.
