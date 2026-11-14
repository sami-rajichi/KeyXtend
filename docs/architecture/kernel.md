# The kernel contract

Moved from [ARCHITECTURE.md](../../ARCHITECTURE.md), which keeps the overview, the code map and the invariants.

```rust
pub struct Manifest {
    pub id: ModuleId,                        // "keyboard", "clipboard", …
    pub version: &'static str,               // the crate version
    pub requires: &'static [ServiceId],      // services it consumes
    pub provides: &'static [ServiceId],      // services it registers
    pub capabilities: &'static [Capability], // what it may do (InjectInput, ReadClipboard, …)
    pub settings: Option<&'static SettingsSpec>, // version, defaults.toml, migrations, validator
}

pub trait Module: Send + 'static {
    fn manifest(&self) -> &'static Manifest;
    fn start(&mut self, cx: &mut ModuleCx<'_>) -> Result<(), ModuleError>; // handlers leave the bus before `stop` runs, services after it
    fn stop(&mut self) {}
}
```

- **Lifecycle** of each module: `Pending → Starting → Active | Failed`, then `Active → Stopping → Stopped`. The table `MOVES` in `crates/kx-kernel/src/lifecycle.rs` is the one source.
  - A failure puts the module in `Failed`. It never crashes the app.
  - Containing a panic needs unwinding, so no build profile may set `panic = "abort"`.
  - Dependents stop before their providers do.
  - `Kernel::retry` (Try again) moves `Failed → Starting`, and restarts the features that wait on it.
  - Switching a module off stops the active modules that need it, then the module itself, which ends `Stopped`. From `Pending` or `Failed` the module is skipped, with no failure notice.
  - Switching it on moves `Stopped → Starting`. A module that waits for a switched-off module stays `Stopped`, with no notice.
- **Services** are traits in `kx-module-api` (ports). Examples: `InputInjector`, `LayoutSource`, `Layouts`, `KeyboardFace`, `FieldWatch`, `WordPacks`, `SealedStore`, `Clipboard`, `ClipboardWriter`, `SealedShelf`, `Pictures`, `Boxes`, `Places`, `Random`, `SelectionWatch`, `WebLink`, `SpeechToText`, `OcrEngine`, `ScrollTarget`, `PointerHook`, `Shortcuts` and `PowerDialog` (ADR-0031).
  - `cx.service::<K>()` returns the service only if the manifest `requires` it **and** the requester holds the capability stored by the service's provider. A request for another capability is refused, and only the kernel builds a module's `Grants`.
  - A provider must itself hold the capability that gates its service.
- **Time:** `Clock` and `Mono` live in `kx-module-api`, because modules may use only that crate. The real clock comes from the platform adapter; tests use `kx-platform-fake`.
- **Platform:** `kx-platform` holds `Platform` and `AppDirs`, which carry the data folder rule (ADR-0015).
- **Events** are typed, and an event may name a capability (`Event::CAPABILITY`) that a module needs to subscribe, intercept or publish it:
  - *notify* events are broadcast (`KeyActivated`, `LayoutChanged`, `ModeChanged`, `ModuleStateChanged`);
  - *intercept* events are an ordered chain that can stop propagation. No module intercepts today: a password box is the field watch's `Password` kind, which each module asks for itself (ADR-0033).
- **Settings** (ADR-0015):
  - one `settings.toml` in the data folder holds one flat section per module, each with its own `version`. Only the user's changes are stored, and the defaults live in the module's `defaults.toml`;
  - migrations are pure functions tested with golden files;
  - a bad value resets alone, a damaged file is set aside and the last good copy comes back, and each repair shows a notice. A file locked by another program is not damage: defaults run in memory and nothing is saved that run;
  - a Settings-window change restarts the module and the modules that need it, or tries a failed module again unless it waits for a provider. A module saves values it sets itself (size, position) through its `SettingsHandle`, with no restart; `set_all` saves values that belong together (x, y, placed) in one checked write;
  - `Kernel::load` reads the settings before any module starts, so the app opens its log first; `Kernel::start` then starts the modules, and `boot` does both.
