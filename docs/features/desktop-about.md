# Feature: desktop-about

> In-app About/version (no Settings/light theme this quarter). Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Support footer About details shows CHANGELOG/product version and MIT pointer; no fifth rail page; header crumb is still the page name
- ✅ Offline/error behavior: version is bundled from `package.json`; no network
- ✅ Accessibility: About is a labeled details control
- ✅ i18n: `app.about` `app.version` `app.license`

## Smoke scenario

1. _Given_ chromaflow-gui is open
2. _When_ the user opens About on Support
3. _Then_ they see the current CHANGELOG version without leaving Cooling/Lighting

## Container map

| Layer | Path |
|-------|------|
| View | `apps/desktop/src/pages/Support.svelte` `lib/version.js` |
| Tests | `tests/test_chromaflow_tauri.py` |

## Tests

- Automated: yes — version import matches packaging/CHANGELOG helper already used by SBOM
- Coverage: no light-theme toggle; no command palette

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

About/version only. Do not add Golden Path Settings chrome or a light theme. PWM unchanged.
