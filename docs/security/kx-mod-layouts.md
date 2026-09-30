# Security note: kx-mod-layouts

- **Owner:** product owner
- **Last review:** 2026-09-30
- **Status:** Draft
- **Scope:** `crates/kx-mod-layouts`, which runs inside the uiAccess `keyxtend` process from P3 on.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `InjectInput` | The layout source it reads is gated by `InjectInput`, because its switch request acts on the app in front; the `Layouts` service it provides is gated the same way, and a provider must hold its service's capability. | The module fails to start, so the keyboard has no key labels and no language switch. |

## Data handled
| Data | Class (Secret/Sensitive/Public) | Stored where | Encrypted | Retention |
|---|---|---|---|---|

## Inputs from outside (files, IPC, OS callbacks, clipboard, network)
| Input | Validation | Size/time limit | Fuzzed |
|---|---|---|---|

## Threats (STRIDE-lite)
- **Spoofing:**
- **Tampering:**
- **Logging / repudiation:**
- **Information disclosure:**
- **Denial of service:**
- **Elevation:**

## Findings
### Must fix (blocks Definition of Done)
### Should fix (tracked issue)
### Accepted (with reason)
