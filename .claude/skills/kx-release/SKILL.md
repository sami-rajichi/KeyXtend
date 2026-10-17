---
name: kx-release
description: Use when preparing a tagged KeyXtend release - version bump, changelog, supply-chain checks, SBOM, attestations, checksums, signing request, winget/Scoop manifests. Never publish without the owner's explicit yes.
---

# Release

1. **Preconditions:**
   - `main` is green;
   - every module in the release passed `kx-module-done`;
   - no open "Must fix" items in `docs/security/`.
2. **Version and changelog:**
   - bump the workspace version (SemVer; pre-1.0 is `0.x`);
   - move "Unreleased" in `CHANGELOG.md` to the new version with today's date.
3. **Supply chain** (CI does this on a tag; check it locally first):
   - `cargo deny check`, then `cargo audit`;
   - `cargo auditable build --release`;
   - CycloneDX SBOM (`cargo cyclonedx`);
   - update `THIRD_PARTY.md` and `licenses/`.
4. **Build the installer:** installs to Program Files, per-machine, silent-install switch for winget.
5. **Signing:**
   - The release needs a paid signature (ADR-0030): Azure Artifact Signing, a certificate authority's certificate or the Microsoft Store. Until one is set up, the release is unsigned, and the README explains the SmartScreen warning and how to verify checksums and attestations.
   - The signing key never sits in the repository; never sign locally with a release key.
6. **GitHub Release** (draft first):
   - assets: installer, portable zip, SBOM, `SHA256SUMS`;
   - `actions/attest` build provenance;
   - release notes in plain language (EN; FR and AR if available);
   - the Qt LGPL notice and where its source is (ADR-0013);
   - the app's own user licence (ADR-0030), which lets users replace the Qt DLLs and reverse engineer to debug that (LGPL-3.0 §4).
7. **Ask the product owner** (AskUserQuestion) to approve publishing. Show the draft link and a summary.
8. **After publishing:**
   - winget manifest PR (`wingetcreate`) and Scoop Extras manifest;
   - tell users to check the files with `gh attestation verify <file> --repo <owner>/<repo>`.
