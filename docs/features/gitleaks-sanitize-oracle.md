# Feature: gitleaks-sanitize-oracle

> Scheduled Gitleaks must not fail on documented sanitizer oracle strings. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: `.gitleaks.toml` allowlists `tests/privacy_report/test_sanitize.py` next to the JSON sanitize fixtures
- ✅ Offline/error behavior: live credentials outside that path still match default Gitleaks rules
- ✅ Accessibility: N/A (CI)
- ✅ i18n: N/A

## Smoke scenario

1. _Given_ the privacy sanitizer oracle contains a fake `ghp_` token
2. _When_ Gitleaks detect runs with repo `.gitleaks.toml`
3. _Then_ that test file is allowlisted; `check-gitleaks-baseline` still requires the allowlist

## Container map

| Layer | Path |
|-------|------|
| Logic | `.gitleaks.toml` `scripts/lib/gitleaks_baseline.py` |
| Tests | `tests/test_gitleaks_baseline.py` |
| Wiring | `.github/workflows/security.yml` |

## Tests

- Automated: yes — allowlist path present; baseline required snippets include `test_sanitize`
- Coverage: SDK/home paths stay banned from the allowlist block

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

See `docs/FEATURE_MODULES.md`. PWM unchanged.
