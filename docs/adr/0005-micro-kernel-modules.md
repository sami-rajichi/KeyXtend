# 5. Micro-kernel with independent "Lego" modules

- **Date:** 2026-09-26
- **Status:** Accepted

## Context
The product owner wants each feature to be an independent package that can be built, tested and switched on or off by itself, like DeepSeek Harness (MIT), where everything is a plugin (Cordis framework). Security matters, so each module should have least privilege.

## Decision
- A small kernel (`kx-kernel`) that owns:
  - the module registry and dependency-ordered lifecycle;
  - the typed event bus (notify and intercept);
  - the service registry;
  - capability grants;
  - versioned settings and the health monitor.
- Each feature is a crate `kx-mod-*`. It declares a `Manifest` (requires, provides, capabilities, settings version) and depends only on `kx-module-api`.
- Modules are compiled in: one Cargo feature per module. They are switched on or off at runtime from settings.
- Third-party runtime plugins are **not** in v1. If ever added, they will run as WASM with explicit capability grants.

## Consequences
- Features can be built and tested one at a time against `kx-platform-fake`.
- `xtask tidy` enforces the dependency direction (ARCHITECTURE.md invariants).
- Capabilities are policy inside one process, not a sandbox. Real isolation comes from processes (ADR-0004).
