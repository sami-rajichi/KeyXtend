# Test report: P0 Foundation

- **Date:** 2026-09-27
- **Windows:** Windows 11 Home Single Language 25H2, build 26200.9457
- **Display scale:** 125 % (1920×1080 panel)
- **Machine:** the development PC (Ryzen 7 7735HS, 24 GB)
- **Tools:** Rust 1.98.1 MSVC, cargo-deny 0.20.2, cargo-audit 0.22.2, Git 2.44.0
- **Scope:** P0 has no app yet. The keyboard regression checks (focus, Start-menu band, Right-click mode) start in P1.

| # | Action | Expected result | Result | Notes |
|---|---|---|---|---|
| 1 | Run `cargo fmt --all --check` | Exit 0 | Pass | |
| 2 | Run `cargo clippy --workspace --all-targets --locked -- -D warnings` | Exit 0, no warnings | Pass | |
| 3 | Run `cargo test --workspace --locked` | All tests pass | Pass | 97 passed, 0 failed |
| 4 | Run `cargo deny --locked check` | All four sections ok | Pass | advisories, bans, licenses, sources ok |
| 5 | Run `cargo audit` | No vulnerabilities | Pass | 48 crates, 1271 advisories loaded |
| 6 | Run `cargo xtask tidy` | 0 errors | Pass | 0 errors, 0 warnings |
| 7 | Run `cargo xtask dco main HEAD` | 0 failing commits | Pass | |
| 8 | Compare the user PATH and the Uninstall registry keys with the snapshot taken before the installs | Unchanged | Pass | No Rustup entry, no new variables |
| 9 | Check that no tool folder exists on C: | `C:\Users\ASUS\.rustup` and `.cargo` absent | Pass | An accidental copy (623 MB) was found and deleted with the owner's yes |
| 10 | Check free space | Nothing of ours on C: | Pass | C: 172.05 → 171.05 GB. The drop is outside our folders; for example, a 158 MB browser update landed in Temp. D: 762.29 → 760.86 GB |
| 11 | The product owner opens the pull request on GitHub | Every check shows a green tick | Pending | Filled in after the push |
