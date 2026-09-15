# Feature: desktop-unused-copy

> Cut or wire unused Lighting JSON so first-run copy is not a kernel mailing list. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: keys not on screen (`lighting.localhost`, `lighting.fusionHint`, `lighting.modulesPrompt`, unused essays) are deleted from `en.json`; `lighting.researchHelp` sits in a closed details control
- ✅ Offline/error behavior: Support extras labels still resolve; missing key falls back to the extra id
- ✅ Accessibility: research details is a native `<details>` (keyboard-reachable)
- ✅ i18n: `apps/desktop/src/locales/en.json` stays ≤300 lines

## Smoke scenario

1. _Given_ Lighting and Support are open
2. _When_ the user does not expand details
3. _Then_ unused protocol/module essays are not in the first viewport

## Container map

| Layer | Path |
|-------|------|
| View | `apps/desktop/src/locales/en.json` `ResearchList.svelte` |
| Tests | `tests/test_chromaflow_lighting.py` |

## Tests

- Automated: yes — unused keys are absent; `researchHelp` is in closed details
- Coverage: no dead `lighting.localhost` on Lighting page

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Copy only. PWM unchanged.
