# Security note: CI and supply chain

- **Owner:** product owner
- **Last review:** 2026-09-27
- **Status:** Reviewed

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `contents: read` (workflow token) | Checkout the repo and read metadata. | The workflow cannot fetch the code to build. |
| No other permissions, no secrets | The workflow never writes to the repo or calls an authenticated API. | Nothing; this is the point. |
| `persist-credentials: false` on every checkout | Keeps the token out of the checked-out `.git` config. | A later step could read or push with the leftover token. |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|
| Repo source and commit metadata | Public | GitHub Actions runner (ephemeral) | In transit (TLS) | Deleted when the job ends |
| `GITHUB_TOKEN` | Secret | GitHub-managed, per run | Yes | Expires at job end |

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|
| Pull-request code and events | Fork PRs run with a read-only token and no secrets; SHAs pass through `env:`, never inline in `run:` | Standard GitHub Actions job timeout | No |
| Dependabot PRs | Same as any pull request; commits are exempt from the DCO check | Standard GitHub Actions job timeout | No |
| crates.io (via `cargo`) | `cargo deny check` and `--locked` pin sources and versions | N/A | No |
| RustSec advisory DB (via `cargo audit`) | Signed, versioned advisory database | N/A | No |
| `cargo metadata` JSON and `git log` output (read by `xtask`) | Parsed into typed values; every error is reported, never a panic | N/A (developer tool) | Property tests only |

## Threats (STRIDE-lite)
- **Spoofing:** a hijacked action tag. Mitigation: every action is pinned to a full commit SHA, and Dependabot opens the update PRs.
- **Tampering:** a malicious crate. Mitigation: `deny.toml` allow-lists licences and sources, `cargo audit` checks advisories, and builds run with `--locked`. Dependabot waits 7 days before proposing a new release.
- **Logging / repudiation:** an unsigned commit on `main`. Mitigation: the `dco` job plus a branch ruleset block the merge.
- **Information disclosure:** a pushed secret. Mitigation: GitHub push protection blocks the push before it lands, and the workflow token cannot read other repos or org secrets.
- **Denial of service:** a burst of pushes queues many runs. Mitigation: a new push cancels the superseded pull-request run, and every job has GitHub's standard timeout.
- **Elevation:** template injection through `${{ }}` inside a `run:` step. Mitigation: untrusted values are passed only through `env:`, never interpolated into `run:`.

## Findings
### Must fix (blocks Definition of Done)
None.

### Should fix (tracked issue)
Add a workflow linter (actionlint or zizmor) to CI later.

### Accepted (with reason)
Dependabot commits are exempt from the DCO check, because they are bot-authored and reviewed through their PR.

The DCO exemption matches the commit author *name*, so it could be spoofed. This is accepted while ADR-0006 bars outside code, because the product owner reviews every PR.

The `xtask` parsers are not fuzzed. They are developer tools that never ship, so property tests that prove they never panic are enough.
