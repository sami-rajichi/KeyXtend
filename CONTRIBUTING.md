# Contributing

Thank you for helping! This keyboard exists for people who cannot use a normal keyboard or mouse. Every change must keep it usable with **one left click**.

## Code contributions (for now)
We welcome issues, ideas and test reports. Until the long-term licence is final (ADR-0006), we do not merge code from outside the core team.

## Before you code
- Read `ARCHITECTURE.md` and the design spec in `docs/superpowers/specs/`.
- For a new feature, open an issue first. Large changes need an ADR (`docs/adr/`).

## Set up a Windows build
- Install Visual Studio 2022 Build Tools with the C++ workload (MSVC), and Rust through `rustup`; `rust-toolchain.toml` pins the toolchain.
- Install Qt 6.11.2 for MSVC 2022 64-bit (`win64_msvc2022_64`), the version `qt_version` in the root `Cargo.toml` pins; CI installs it with aqtinstall (`.github/actions/qt`).
- In each shell that builds, set `QMAKE` to Qt's `bin\qmake.exe` and put that `bin` folder first in `PATH`. Any folder works, a portable one too; nothing goes in the registry.
- For the Qt Quick tests, run `cargo test -p kx-ui --test texts` first, then `qmltestrunner -input crates/kx-ui/qmltest` with `QML_DISABLE_DISK_CACHE=1`.
- Without Qt, leave out the app and build `kx-ui` with `--no-default-features`, as CI's Linux job does.

## Rules
- **Tests first.** Logic is tested without the OS, using `kx-platform-fake` and an injected clock. UI is tested with Qt Quick Test (`qmltestrunner`), finding controls by accessible name.
- The required checks are the jobs in `.github/workflows/ci.yml`; run the same commands locally before you push.
- `cargo xtask dco <base> <head>` checks every commit in that range carries a DCO sign-off.
- No `unsafe` outside `crates/kx-platform-*`, except cxx-qt bridge blocks in `kx-ui/src/bridges/` and the test tools in `tools/` (ADR-0013, ADR-0014).
- Never log typed text, clipboard data, passwords or transcripts.
- UI never needs scrolling, dragging or right-click. Lists are paged, 5 rows per page.
- New dependencies must have a licence compatible with GPL-3.0-or-later (see `deny.toml`). Data, fonts and models go in `THIRD_PARTY.md`.

## Commits
- Conventional Commits (`feat(scroll): …`, `fix(keyboard): …`).
- Every commit is signed off (`git commit -s`) under the Developer Certificate of Origin.

## Testing on Windows
CI cannot test uiAccess, admin windows or real apps. Follow `.claude/skills/kx-windows-manual-test/SKILL.md` and save the report in `docs/test-reports/`.

- Build the two test tools with `cargo build -p kx-target-window -p kx-gates`.
- The gates move the real mouse and keyboard, so run them only when the owner agrees. `kx-gates` with no arguments prints its commands.
- Gates that click the Qt keyboard need uiAccess on both sides. Install the keyboard and a release `kx-gates` with `cargo xtask dev-install <folder>`, then run `kx-gates` from there.
- The installed `kx-gates` reads `kx-gates.toml` and `kx-gates-p3.toml` beside it, else from `tools/kx-gates`. A relative path starts at its file's folder, so in copies beside the installed exe write the repo paths (`[paths] out`, `[apps.target] exe`, `[g22] worker_dir`) in full; `[g8] program` already points at the installed app.
