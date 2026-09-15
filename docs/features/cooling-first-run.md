# Feature: cooling-first-run

> Kinder first Cooling visit: ask to take over fans before the board is a wall of checkboxes. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: first live visit with writable PWM and no take-over yet offers “Take over these fans?” with Quiet; later visits restore recipe take-over
- ✅ Offline/error behavior: Vite cannot take PWM; conflicts skip the prompt; never silent 0%; same `cooling.takeoverAsk` / `cooling.zeroAsk` confirms
- ✅ Accessibility: labeled primary button, not a hidden checkbox
- ✅ i18n: `cooling.firstRun` / `cooling.firstRunApply`

## Smoke scenario

1. _Given_ a fresh session, live inventory, no conflicts
2. _When_ Cooling opens
3. _Then_ the user can take over Quiet in one confirm

## Container map

| Layer | Path |
|-------|------|
| Logic | `apps/desktop/src/pages/Cooling.svelte` |
| Tests | `tests/test_chromaflow_cooling_ui.py` |

## Tests

- Automated: yes — first-run copy and Quiet apply reuse; no `set_pwm`
- Coverage: prompt omitted when `conflicts.length`

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

First-run kindness. Never silent 0%. Do not restart chromaflowd.
