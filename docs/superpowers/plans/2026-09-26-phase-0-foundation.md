# Phase 0 — Foundation (plan)

## Goal
An empty but strict workspace: tools, Git, lint and licence rules, `xtask`, CI and community files.
Source: `docs/roadmap.md` Phase 0 and item P0 in `docs/FEATURES.md`.

## Scope
**In:**
- Tools in `D:\dev`: Rust 1.98.1 (rustup), cargo-deny and cargo-audit; GitHub CLI only at the GitHub step.
- The existing Visual Studio 2022 C++ compiler and Windows SDK on C: are reused (decided in round 1).
- `git init`, the baseline commit on `main`, and the branch `feat/P0-foundation`.
- Workspace config files, `xtask` (`tidy`, `dco`, `licences`, `dist` stub), CI, Dependabot.
- `LICENSE`, `CODE_OF_CONDUCT.md`, `CHANGELOG.md`, issue forms, PR template, `CODEOWNERS`.
- ADR-0012: the installer is Inno Setup 7 (decision D8).
- The public GitHub repo and its protections, **only after the owner's yes**.

**Out:** app and library crates (P2+), Qt, CMake and Ninja (P1), installer build, SBOM and signing (P14).

## Design notes
- **Root files:**
  - `Cargo.toml`: virtual workspace; members `xtask`, `crates/*`, `apps/*`; edition 2024; licence GPL-3.0-or-later.
  - `[workspace.lints]`: deny `unsafe_code`, so only `kx-platform-*` crates can allow it and every other crate root forbids it; warn `missing_docs`, clippy pedantic, `undocumented_unsafe_blocks`, `unwrap_used`, `expect_used`, `dbg_macro`, `todo`, `print_stdout`.
  - `rust-toolchain.toml` (1.98.1, rustfmt, clippy), `rustfmt.toml`, `.cargo/config.toml` (the `xtask` alias).
  - `clippy.toml`: function limit of 40 lines (`too-many-lines-threshold`), unwrap and expect allowed in tests.
  - `deny.toml`: the ADR-0006 licence list, crates.io as the only source, wildcards denied, yanked crates denied.
  - `.gitignore`, `.gitattributes` (LF in the repo).
- **Single sources of truth:**
  - Tidy limits and lists live in `[workspace.metadata.tidy]` in `Cargo.toml`: file line limits, crate prefixes, banned network and media crates, the app name, scanned extensions.
  - DCO bot exemptions live in `[workspace.metadata.dco]`.
  - The 40-line function limit lives only in `clippy.toml`.
- **`xtask` crate** (a developer tool, never shipped). Dependencies are `serde` and `serde_json`; `proptest` is for tests only.
  - `workspace.rs`: reads `cargo metadata` into a small model (packages, dependency kinds, resolved graph, crate root files, tidy config).
  - `tidy/deps.rs`: rules D1 module isolation, D2 only apps pick platforms and modules, D3 no network crates in `keyxtend`, D4 no media decoders in `keyxtend`.
  - `tidy/files.rs`: rules F1 file length (warn > 300, fail > 400), F2 `#![forbid(unsafe_code)]` in every crate root outside `kx-platform-*`.
  - `dco.rs`: every non-merge, non-bot commit in a range has a `Signed-off-by` that matches its author.
  - `licences.rs` runs `cargo deny check licenses` and `cargo deny list`. `dist.rs` is a stub that fails with "arrives in P14".
- **SAFETY comments and function length** are enforced by clippy, so tidy does not repeat them.
- **CI** `.github/workflows/ci.yml`:
  - jobs `check` (Windows + Ubuntu: fmt, clippy, test, tidy), `supply-chain` (deny, audit, also weekly), `dco` (pull requests);
  - actions pinned to full commit SHAs, token read-only, `pull_request` trigger only.
- **Dependabot** `.github/dependabot.yml`: Cargo and Actions, weekly, grouped.
- No ports, settings keys, theme tokens or translation keys: P0 has no app code.

## Tasks
1. **Tools.**
   - *Files:* `D:\dev\env.ps1`, `D:\dev\INSTALLED.md`.
   - *Implement:* record C:/D: free space, then install rustup with `--no-modify-path` into `D:\dev`. Delete its "Installed apps" registry entry and turn off self-update.
   - *Verify:* `rustup --version` works in a session after `. D:\dev\env.ps1`; the user PATH and the Uninstall key are unchanged.
2. **Git.**
   - *Files:* `.gitignore`, `.gitattributes`.
   - *Implement:* `git init -b main` and a repo-local identity (the owner's GitHub username + GitHub no-reply email). Commit the licence and git settings as the baseline, then branch `feat/P0-foundation`.
   - *Verify:* `git log --format=%an%n%ae%n%b` shows the username, the no-reply email and a Signed-off-by line.
3. **Workspace skeleton.**
   - *Files:* root config files, `xtask/Cargo.toml`, `xtask/src/main.rs`.
   - *Test first:* an unknown subcommand returns a usage error.
   - *Implement:* the root files from the design notes, plus subcommand dispatch.
   - *Verify:* `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` are clean.
4. **Supply-chain tools.**
   - *Files:* `deny.toml`.
   - *Implement:* `cargo install --locked cargo-deny cargo-audit`, built from crates.io into `D:\dev\cargo`.
   - *Verify:* `cargo deny check` and `cargo audit` both pass.
5. **Workspace model.**
   - *Files:* `xtask/src/workspace.rs`, `xtask/tests/fixtures/*.json`.
   - *Test first:* metadata fixtures load into the model; a missing or malformed tidy section gives a clear error.
   - *Implement:* parse `cargo metadata --format-version 1` output into the model.
   - *Verify:* `cargo test -p xtask workspace` passes.
6. **Tidy dependency rules D1–D4.**
   - *Files:* `xtask/src/tidy/deps.rs`.
   - *Test first:* each rule fails on a bad fixture and passes on a good one (edge cases 1–7).
   - *Implement:* rules as pure functions returning violations; the D3/D4 closure follows normal deps only.
   - *Verify:* `cargo test -p xtask deps` passes.
7. **Tidy file rules F1–F2.**
   - *Files:* `xtask/src/tidy/files.rs`.
   - *Test first:* limits, CRLF, a commented-out attribute and a non-UTF-8 file behave as in edge cases 8–12.
   - *Implement:* walk `apps/`, `crates/` and `xtask/`, skipping `target/`.
   - *Verify:* `cargo test -p xtask files` passes.
8. **Tidy command.**
   - *Files:* `xtask/src/tidy/mod.rs`.
   - *Test first:* errors give a non-zero exit, warnings only print.
   - *Implement:* run all rules and print one line per violation (rule id, place, message).
   - *Verify:* `cargo xtask tidy` passes on the real workspace.
9. **DCO check.**
   - *Files:* `xtask/src/dco.rs`.
   - *Test first:* unit tests for edge cases 13–17; property test: a message passes if and only if it has a matching trailer.
   - *Implement:* read the commits of `<base>..<head>` from `git log` and check each one.
   - *Verify:* `cargo xtask dco main HEAD` passes on the branch.
10. **`licences` and `dist`.**
    - *Files:* `xtask/src/licences.rs`, `xtask/src/dist.rs`.
    - *Test first:* `dist` returns the "arrives in P14" error.
    - *Implement:* `licences` wraps cargo-deny; `dist` is the stub.
    - *Verify:* `cargo xtask licences` passes; `cargo xtask dist` exits non-zero with its message.
11. **CI and Dependabot.**
    - *Files:* `.github/workflows/ci.yml`, `.github/dependabot.yml`.
    - *Implement:* the jobs from the design notes, with each action SHA looked up from its release tag.
    - *Verify:* checked on GitHub in task 14.
12. **Community and legal files.**
    - *Files:* `LICENSE` (GPL-3.0 from gnu.org), `CODE_OF_CONDUCT.md` (Covenant 3.0, reports through GitHub's "Report content"), `CHANGELOG.md`, `.github/ISSUE_TEMPLATE/*`, `.github/pull_request_template.md`, `README.md`, `THIRD_PARTY.md`.
    - *Verify:* the texts match their sources, and the PR template lists the Definition of Done.
13. **Decisions and security notes.**
    - *Files:* `docs/adr/0012-installer-inno-setup.md`, `docs/adr/README.md`, spec §14 D8, `docs/security/ci.md`.
    - *Implement:* record Inno Setup 7 and its reasons. The index notes that the toolkit result becomes ADR-0013.
    - *Verify:* `kx-review` passes on every changed file.
14. **GitHub (only after the owner's yes).**
    - *First:* check that the public docs describe the product and its users, not the owner.
    - *Implement:* install `gh` in `D:\dev\gh`; the owner signs in. Create the public repo `KeyXtend`, push `main`, then push the branch and open a PR.
    - *Implement, repo settings:*
      - secret scanning and push protection;
      - private vulnerability reporting and reported content;
      - Dependabot alerts;
      - `CODEOWNERS`;
      - a ruleset on `main`: PR required, the CI checks green, no force push or deletion, linear history.
    - *Verify:* every CI job is green on Windows and Ubuntu, and `gh api` shows the ruleset active.

## Edge cases
1. A `kx-mod-*` crate depends on another `kx-mod-*`: D1 error.
2. A `kx-mod-*` depends on `kx-kernel` or `kx-platform-windows`: D1 error.
3. A `kx-mod-*` has a dev-dependency on `kx-platform-fake`: allowed.
4. A non-app library depends on `kx-platform-windows`: D2 error. An app doing the same is allowed.
5. `reqwest` reaches `keyxtend` only transitively: D3 error. The same crate in `keyxtend-worker` is allowed.
6. A banned crate is only a dev-dependency of `keyxtend`: allowed, because it is not shipped.
7. A workspace with no crates except `xtask` passes every rule.
8. A file of exactly 400 lines passes, 401 fails, and 301 only warns.
9. A CRLF file counts the same lines as its LF copy (property test).
10. `// #![forbid(unsafe_code)]` inside a comment does not count: F2 error.
11. A `kx-platform-*` crate without the attribute is allowed.
12. A non-UTF-8 source file is reported without a panic; `target/` is never scanned.
13. A commit without a sign-off fails the DCO check.
14. A sign-off whose email differs from the author's fails.
15. A lower-case `signed-off-by:` trailer passes, as git treats trailers case-insensitively.
16. Merge commits and `dependabot[bot]` commits are skipped.
17. An empty commit range passes.

## Security
- **Capabilities:** none. There is no app code yet.
- **Data:** no user data. The only secret is the owner's GitHub token, which stays in Windows Credential Manager (gh default) and never goes in the repo or CI.
- **Threats** (`docs/security/ci.md`):
  - a hijacked action tag (SHA pins);
  - a malicious fork PR (`pull_request` trigger, read-only token, no secrets);
  - a bad or unknown crate (deny sources, audit, `Cargo.lock` committed);
  - a secret pushed by mistake (push protection);
  - an unsigned commit on `main` (DCO job plus ruleset).

## Manual Windows check
1. I run the six gates on this PC and paste the output into `docs/test-reports/2026-09-26-p0-foundation.md`.
2. I show that the user PATH and the Uninstall registry key are unchanged, and the C:/D: space before and after.
3. The product owner opens the PR page and confirms that every check shows a green tick.

## Done when
- The `docs/roadmap.md` Definition of Done holds, where it applies. There is no UI, so the Slint, accessibility and contrast items are N/A.
- The PR is merged into `main` after the owner's yes, and CI is green on `main`.
- P0 is `Done (v0.0.0)` and P1 is `Ready` in `docs/FEATURES.md`.
