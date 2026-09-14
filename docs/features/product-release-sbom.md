# Feature: product-release-sbom

> GitHub Release SBOM/OpenVEX for ChromaFlow uses the product version (CHANGELOG / Cargo), not `.template-version`. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: `workflow_dispatch` or tag `v0.2.x` uploads SBOM + OpenVEX to the matching GitHub Release; a template 1.5.0 Release Please PR is not required
- ✅ Offline/error behavior: tag mismatch against product version fails the job; `.template-version` stays the bootstrap template pin
- ✅ Accessibility: N/A (CI)
- ✅ i18n: N/A

## Smoke scenario

1. _Given_ CHANGELOG has `[0.2.0]` and GitHub Release `v0.2.0` exists
2. _When_ Release runs with product tag `v0.2.0`
3. _Then_ `sbom.cyclonedx.json` and `openvex.json` attach; the job does not require `.template-version` == `0.2.0`

## Container map

| Layer | Path |
|-------|------|
| Logic | `.github/workflows/release.yml` `scripts/product-release-version.sh` |
| Tests | `tests/test_chromaflow_packaging.py` |
| Docs | `docs/SECURITY_TRIAGE.md` |

## Tests

- Automated: yes — `product-release-version.sh` prints `0.2.0` from CHANGELOG; release workflow tag gate does not require `.template-version`; empty CHANGELOG versions fail closed
- Coverage: empty tag still fails closed

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Product SBOM path. Do not merge a template 1.5.0 Release Please PR. PWM unchanged.
