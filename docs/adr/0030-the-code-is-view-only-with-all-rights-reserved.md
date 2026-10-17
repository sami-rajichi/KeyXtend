# 30. The code is view only, with all rights reserved

- **Date:** 2026-10-17
- **Status:** Accepted (owner, 2026-10-17). Supersedes ADR-0006 and ADR-0003's line on free signing.

## Context

- ADR-0006 chose GPL-3.0-or-later "for now" and kept a closed path open: no outside code was ever merged.
- The owner wants to sell KeyXtend, and wants no one else to copy, clone or sell it. The repository should stay public (owner, 2026-10-17).
- Without a licence, no one may use, copy, change or share code ([Licensing a repository](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/licensing-a-repository)). GitHub's terms still let its users view and fork public code on GitHub, and give GitHub its own rights ([GitHub Terms](https://docs.github.com/en/site-policy/github-terms/github-terms-of-service), §D.4–D.5).
- P0 to P5 were published under the GPL, whose grant cannot be taken back (GPLv3 §2). P5's code, commit 0b38616, stays in the public history with that licence, so anyone may still take it from there.
- The code is written by an AI model under the owner's direction (CLAUDE.md rule 8). The US Copyright Office protects only human authorship, judged case by case, so how far copyright covers this code is a question for the lawyer, country by country ([Copyright and AI, Part 2](https://www.copyright.gov/ai/Copyright-and-Artificial-Intelligence-Part-2-Copyrightability-Report.pdf)).
- Every third-party part that ships allows a closed, sold app (`cargo deny list`, `THIRD_PARTY.md`, 2026-10-17):
  - the crates are MIT, Apache-2.0, Unicode-3.0 or Zlib; `self_cell` is used as Apache-2.0, and `memchr`, `termcolor` and `winapi-util` as MIT;
  - the fonts are OFL-1.1, the icons ISC and MIT, and the sounds CC0;
  - Qt is LGPL-3.0, as separate DLLs.
- SignPath Foundation signs only OSI-licensed projects with no proprietary parts ([conditions](https://signpath.org/terms.html)).

## Decision

1. From 2026-10-17, after commit 0b38616, all of KeyXtend is **view only, all rights reserved**, held by "sami-rajichi". This is the owner's choice; a legal name or company can replace it before sales.
2. `LICENSE` says, in short:
   - anyone may read the code and keep a copy only for reading it;
   - no one may use, build, copy, change, share, sell or train models on it, in whole or part, without the owner's written permission;
   - it adds no right beyond GitHub's terms, and grants no right to the name;
   - versions up to 0b38616 keep the GPL;
   - permission is asked through an issue in the repository.
3. The app will ship under the owner's own end-user licence (EULA), written before the first release. Like any tester's build before it, it must let users replace the Qt DLLs and allow reverse engineering to debug that change (LGPL-3.0 §4).
4. Third-party parts keep their own licences. Their notices ship in `licenses/` as `THIRD_PARTY.md` says.
5. No outside code is merged; if that ever changes, each contributor first signs an agreement that gives the owner the rights (a CLA). The DCO sign-off stays as the project's own record and CI check.
6. Everything that ships in the app or links into it must allow a closed, sold app:
   - no GPL or AGPL;
   - LGPL only as a separate, replaceable library;
   - share-alike data (CC-BY-SA) only as separate files;
   - build, test and CI tools that never ship keep their own licences.
7. Code that checks a paid licence key, and any signing secrets, live outside this repository, as public code would make them easy to bypass.
8. The repository stays public, so main's ruleset and CI's free minutes stay. Each item still merges by its own pull request.

## Consequences

- The project is no longer open source, so the README and the spec say "source-available".
- No one else may legally build or run the app. Testers get the owner's builds with a short written permission that meets LGPL-3.0 §4.
- Free signing ends, and uiAccess needs a trusted signature. P14 picks one of:
  - Azure Artifact Signing ($9.99 a month, [page](https://azure.microsoft.com/en-us/products/artifact-signing)), open to individuals only in the US and Canada ([Windows signing options](https://learn.microsoft.com/en-us/windows/apps/package-and-deploy/code-signing-options));
  - a certificate authority's certificate;
  - the Microsoft Store, though its approval of uiAccess is unverified (ADR-0003).
- Any public signature shows the owner's validated legal name.
- P14 also settles how the sold app is delivered and unlocked, and where releases are built and signed.
- The new terms cover only work after 0b38616. The owner decides, with the lawyer, whether the GPL history stays public.
- Copyright protects human-written code and artwork, not ideas. Protection may also lean on the EULA, on decision 7's private parts and on a trademark for the name.
- A lawyer reads `LICENSE` and the EULA before the first sale. The lawyer also covers:
  - the AI authorship question;
  - the governing law;
  - terms for the ideas people send;
  - room for security research.

## Alternatives

- **PolyForm Strict 1.0.0:** a ready text, with no changes or sharing. But it lets anyone build and use the app free for personal use, which undercuts selling to individuals ([PolyForm](https://github.com/polyformproject/polyform-licenses)).
- **PolyForm Shield 1.0.0:** free use and changes, but no competing product. It is too open for a sold app ([Shield](https://polyformproject.org/licenses/shield/1.0.0)).
- **FSL-1.1:** no competing use, but each version becomes Apache-2.0 or MIT after two years ([FSL](https://fsl.software/)).
- **Open core:** a public base with private extra modules. It is no longer needed, as nothing in the public repository may be used without permission.
- **A private repository:** on GitHub Free, it loses main's ruleset and limits CI to 2,000 minutes a month ([rulesets](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-rulesets/about-rulesets), [included usage](https://docs.github.com/en/billing/reference/product-usage-included)).

## The change (one pull request, after P5 merged)

- `LICENSE`: the new text.
- `Cargo.toml`, every crate's inherited field and `spike/Cargo.toml`: `license-file` points at `LICENSE` in place of the GPL identifier.
- `deny.toml`: GPL-3.0-or-later leaves the allowed list, and our own unpublished crates are exempt.
- `README.md`: "source-available" in the first line, the old mock-up link, and the Licence section.
- `CONTRIBUTING.md`: no outside code, an agreement first if that changes, building only with permission, the DCO's role, and decision 6.
- `.github/ISSUE_TEMPLATE/permission.yml`: a short form for asking permission, as `LICENSE` says.
- `ARCHITECTURE.md`'s invariants and `docs/security/ci.md` name this ADR. The `kx-licence-check`, `kx-new-module` and `kx-release` skills follow it.
- `assets/README.md` and `assets/sounds/README.md`: our own drawing and recipes are all rights reserved, like the code.
- The spec's Goal and row D2, the roadmap's signing line, and P14's notes.
- `docs/adr/README.md` and the status lines of ADR-0006 and ADR-0003 name this ADR.
- `CHANGELOG.md` and `docs/WORKLOG.md`: one entry each.
- History stays as it is: old plans, research, the design lab page and the P0–P3 work log.
