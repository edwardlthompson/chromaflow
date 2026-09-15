# Feature: support-toolbox

> Support first screen is two short sentences + Install all; per-item Install is secondary; errors are first-line + details. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: first Support viewport is two short sentences plus **Install all**; extras and Advanced sit in closed details; each extra’s Install is `.btn-secondary`; long errors use first line + details
- ✅ Offline/error behavior: pkexec failures still surface; Vite still cannot pkexec
- ✅ Accessibility: Install all is the primary control; destructive uninstall stays inside closed details
- ✅ i18n: `support.help` `support.toolbox`; Advanced stays off by default

## Smoke scenario

1. _Given_ Support is open on first visit
2. _When_ the user does not expand extras or competitors
3. _Then_ they see help + Install all, not a kernel changelog

## Container map

| Layer | Path |
|-------|------|
| View | `apps/desktop/src/pages/Support.svelte` `ExtrasList.svelte` `locales/en.json` |
| Tests | `tests/test_chromaflow_tauri.py` `tests/test_chromaflow_support.py` |

## Tests

- Automated: yes — `failLines`; Install all before per-item Install; no 180-char dump as the only alert
- Coverage: Advanced still `false`

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Support as a locked toolbox. PWM unchanged. Do not dump GPU I2C.
