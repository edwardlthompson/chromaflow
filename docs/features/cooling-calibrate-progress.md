# Feature: cooling-calibrate-progress

> Auto-calibrate shows a progress bar, not only a status sentence. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: while Auto-calibrate runs, a determinate bar (n/total headers) updates with `cooling.calibrateProgress`
- ✅ Offline/error behavior: abort still failsafe via existing sweep; never silent 0%; Vite still `cooling.needsNative`
- ✅ Accessibility: `role="progressbar"` with `aria-valuenow` / `aria-valuemax`; status text remains
- ✅ i18n: reuse `cooling.calibrate*` keys

## Smoke scenario

1. _Given_ take-over is on and no conflicts
2. _When_ the user clicks Auto-calibrate
3. _Then_ a bar advances per header instead of only a text line

## Container map

| Layer | Path |
|-------|------|
| Logic | `apps/desktop/src/lib/coolingSweep.js` |
| View | `apps/desktop/src/pages/Cooling.svelte` `app.css` |
| Tests | `tests/test_chromaflow_cooling_ui.py` |

## Tests

- Automated: yes — progressbar markup; no `set_pwm`; NVIDIA still skipped by `sweepQueue`
- Coverage: live sweep stays HUMAN smoke

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Progress UI only. Live sweep stays HUMAN smoke. Never silent 0%.
