# Security note: CI and supply chain

- **Owner:** product owner
- **Last review:** 2026-10-02 (P3: Qt on the Windows job, the staged app and the G26 job)
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
| Qt binaries (`.github/actions/qt`, Windows jobs) | aqtinstall checks each archive's SHA-256 against Qt's own server; the version comes from the stage settings, and `cargo xtask stage` refuses any other | Cached per Qt version and runner OS | No |
| aqtinstall and its Python packages (pip, at run time) | aqtinstall is pinned to the reviewed commit and py7zr to one version; the rest resolve from PyPI at run time (Accepted below). py7zr is `==1.1.3`, not upstream's `==1.1.0` Bad7zFile workaround, since that commit asks `py7zr>=1.1.3`; the action's setup-python `>=3.14.0` is a floor (the dev venv is 3.13) | N/A | No |

## Threats (STRIDE-lite)
- **Spoofing:** a hijacked action tag. Mitigation: every action is pinned to a full commit SHA, and Dependabot opens the update PRs.
- **Tampering:** a malicious crate. Mitigation: `deny.toml` allow-lists licences and sources, `cargo audit` checks advisories, and builds run with `--locked`. Dependabot waits 7 days before proposing a new release.
  - cargo-deny 0.20.2 and cargo-audit 0.22.2 come pinned by version through the SHA-pinned install-action.
- **Tampering:** a tampered Qt download or Python package. Mitigation: hash checks against Qt's server, pinned aqtinstall and py7zr, and check jobs that hold no secret and publish nothing.
- **Logging / repudiation:** an unsigned commit on `main`. Mitigation: the `dco` job plus a branch ruleset block the merge.
- **Information disclosure:** a pushed secret. Mitigation: GitHub push protection blocks the push before it lands, and the workflow token cannot read other repos or org secrets.
  - A failed G26 run prints the app's log, target-window's log and the result lines. Mitigation: the app never logs typed text, and target-window holds only the gate's own test characters and keys.
  - The Windows job builds the release app with its uiAccess manifest (`cargo xtask stage`) and uploads nothing, so no unsigned uiAccess build leaves CI.
- **Denial of service:** a burst of pushes queues many runs. Mitigation: a new push cancels the superseded pull-request run; the G26 job stops after 60 minutes, the Qt Quick step after 10, and the rest at GitHub's standard timeout.
- **Elevation:** template injection through `${{ }}` inside a `run:` step. Mitigation: untrusted values are passed only through `env:`, never interpolated into `run:`.
- **Tampering:** the G26 job runs pull-request code that adds languages to the runner, starts the debug app and moves the runner's mouse. Mitigation: the runner is thrown away after the job, which holds only the read-only token and no secret.

## Findings
### Must fix (blocks Definition of Done)
None.

### Should fix (tracked issue)
Add a workflow linter (actionlint or zizmor) to CI later.

A release workflow that ships the Qt it installs must pin aqtinstall's Python packages by hash (`kx-release`, P14). It also installs Qt fresh (`cache: false`) and restores no CI cache.

### Accepted (with reason)
Dependabot commits are exempt from the DCO check, because they are bot-authored and reviewed through their PR.

The DCO exemption matches the commit author *name*, so it could be spoofed. This is accepted while ADR-0006 bars outside code, because the product owner reviews every PR.

The `xtask` parsers are not fuzzed. They are developer tools that never ship, so property tests that prove they never panic are enough.

aqtinstall's own dependencies (requests, beautifulsoup4 and others) are not pinned. The Windows jobs hold only the read-only token and ship nothing. A bad release on a `push: main` run could still poison the Qt and cargo caches that later runs restore, which is why a release never restores them (Should fix above).
