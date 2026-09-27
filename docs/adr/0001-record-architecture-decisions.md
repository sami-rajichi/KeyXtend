# 1. Record architecture decisions

- **Date:** 2026-09-26
- **Status:** Accepted

## Context
The project will be built step by step, over many sessions and by several agents and people. Decisions made in one session must survive into the next ones.

## Decision
We record every significant decision as an ADR in `docs/adr/`, using this format: Context, Decision, Consequences, Alternatives.
- ADRs are never rewritten. To change a decision, a new ADR supersedes the old one, and the old one's status becomes "Superseded by N".

## Consequences
- Agents read `docs/adr/` before proposing structural changes.
- Every pull request that changes a decision adds an ADR.
