# Feature: product-version-align

> Product crate, Tauri, desktop package, and `.deb` names follow CHANGELOG, not a stale 0.1.0 pin. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: `chromaflow_0.2.0_amd64.deb` (and helper) match CHANGELOG `[0.2.0]`; Cargo/Tauri/desktop `package.json` are `0.2.0`
- ✅ Offline/error behavior: `product-release-version.sh` remains the single semver source for packaging scripts; `.template-version` stays the bootstrap pin
- ✅ Accessibility: N/A
- ✅ i18n: N/A

## Smoke scenario

1. _Given_ CHANGELOG first `[X.Y.Z]` is `0.2.0`
2. _When_ `build-chromaflow-deb.sh` stages a package (skip cargo)
3. _Then_ the artifact is `chromaflow_0.2.0_amd64.deb` and workspace versions match

## Container map

| Layer | Path |
|-------|------|
| Logic | `scripts/product-release-version.sh` `scripts/lib/product_version_align.py` `scripts/build-chromaflow-deb.sh` `scripts/build-helper-deb.sh` |
| Tests | `tests/test_chromaflow_packaging.py` |
| Docs | `packaging/README.md` `branding/product.json` |

## Tests

- Automated: yes — workspace/Tauri/npm/deb scripts align with CHANGELOG; Golden Path `examples/web` stays `0.1.0`
- Coverage: empty CHANGELOG versions still fail closed via `product-release-version.sh`

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Do not bump `.template-version` or gtk-rs. PWM unchanged.
