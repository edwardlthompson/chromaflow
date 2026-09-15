# Feature: lighting-apply-feedback

> Lighting Apply keeps the list usable: toast, optimistic swatch, per-row busy. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Apply updates the row swatch immediately; only that row’s Apply shows a spinner; other lamps stay clickable; `.fc-toast` reports success (2.2s)
- ✅ Offline/error behavior: Vite shows `lighting.applyNeedGui`; failed apply reverts the swatch and uses `role="alert"`
- ✅ Accessibility: per-row spinner is `aria-busy` on that control; disabled buttons are not `cursor: wait`
- ✅ i18n: reuse `lighting.applyOk` / `lighting.applying`

## Smoke scenario

1. _Given_ Lighting lists two lamps
2. _When_ the user Applies color on one
3. _Then_ the other row’s Apply stays enabled and a toast confirms

## Container map

| Layer | Path |
|-------|------|
| Logic | `apps/desktop/src/pages/Lighting.svelte` `DeviceList.svelte` |
| View | `apps/desktop/src/app.css` |
| Tests | `tests/test_chromaflow_lighting.py` `tests/test_apply_status.py` |

## Tests

- Automated: yes — `busyKey` not a global Lighting Apply disable; toast helper
- Coverage: install-engine still uses page `busy`

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Perceived performance. Never `set_pwm`. Do not dump GPU I2C.
