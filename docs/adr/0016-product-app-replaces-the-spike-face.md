# 16. The product app replaces the spike face

- **Date:** 2026-09-29
- **Status:** Accepted. Supersedes ADR-0014's "the spike keeps building" rule for `qt-kb` only. Refines ADR-0013's "kx-ui holds the theme loader".

## Context

P3 ships the first real app, `apps/keyxtend`, and moves the spike's keyboard, layout, window, theme and Qt parts into the product (ADR-0014, `docs/spike-move-map.md`).

- The spike face `qt-kb` and the product would both build a keyboard, from two code bases that drift apart.
- P3 empties `spike/core` part by part; keeping `qt-kb` building would need a shim for every move.
- The theme loader must serve the product and the spike, but a loader inside `kx-ui` needs Qt to be tested.
- Fonts and icons are staged from a folder outside the repo, so a fresh clone cannot build a face.

## Decision

Decided by the owner on 2026-09-29.

- **The spike face stops building at the start of P3.**
  - `spike/qt-kb/{Cargo.toml,Cargo.lock,.gitignore}` are deleted.
  - Its other files stay as source: build script, panel, ring, tools, voice, their QML and Qt Quick Tests. Each moves in its own phase (P3–P13).
  - The signed test copy in `C:\Program Files\KeyXtend-dev\qt-kb` stays until the owner removes it.
  - `spike/core` and `spike/worker` keep building and testing, because `kx-gates` depends on `spike-core`. ADR-0014's rule still holds for them.
- **`kx-theme` is a new crate with no Qt:** theme tokens, the look and contrast. Its tests run everywhere and spike code can use it. `kx-ui` keeps the QML, bridges, translations and the `ui` module.
- **Ports:**
  - module ports live in `kx-module-api`: `InputInjector`, `LayoutSource`, `Layouts`, `KeyboardFace`;
  - UI ports live in `kx-platform`: `WindowGuard`, `Screens`, `SystemLook`, `Backdrop`, `Instance`;
  - `kx-ui` depends only on `kx-module-api`, `kx-platform` and `kx-theme`, never on a module or an adapter.
- **Fonts and icons live in the repo** under `assets/`, each with its licence file, and are compiled into the binary as Qt resources, like the QML. Theme tokens are embedded too, so the uiAccess process reads nothing it draws from disk.
- **CI:** Linux skips the Qt crates (`kx-ui`, `keyxtend`) until P15. The Windows job installs Qt.

## Consequences

- A fresh clone has everything a build needs; the repo grows by the fonts and icons, which pass `kx-licence-check` first.
- The unmoved spike Qt source cannot be built or run until its part moves; the panel and ring Qt Quick Tests wait for P13 and P4.
- The gates that drive the spike face are aimed at the product app within P3 (`docs/spike-move-map.md`).
- `kx-ui` never sees a Win32 call or a module type, so its bridges and QML can be tested with fakes.

## Alternatives

- **Keep both faces building:** no gap in the spike's tests, but a shim per move and two keyboards to keep alike; refused.
- **Fetch fonts and icons by hash at build time:** a smaller repo, but a build then needs the network; refused by the owner.
- **Keep the theme loader in `kx-ui`:** one crate fewer, but theme tests need Qt and the spike cannot use the loader; refused.
