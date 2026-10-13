# Security note: kx-mod-mouse

- **Owner:** product owner
- **Last review:** 2026-10-13 (scaffold; threats come with the engine and runtime)
- **Status:** Draft
- **Scope:** `crates/kx-mod-mouse`, which will run inside the uiAccess `keyxtend` process from P4 on.

## Capabilities
| Capability | Why needed | What breaks without it |
|---|---|---|
| `PointerHook` | It gates the system-wide pointer hook the hold engine runs on (ADR-0027). | The module does not start, so Right-click, Grab and modifier + click are off. |
| `InjectInput` | The engine sends clicks and modifier keys, and it gates the toggle and latch events the module hears and sends. | The engine's input and the keyboard's toggle and latch events are refused. |

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
