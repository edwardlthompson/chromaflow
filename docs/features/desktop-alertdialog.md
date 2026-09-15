# Feature: desktop-alertdialog

> Replace `window.confirm` chrome with an in-app `role="alertdialog"`; keep the PWM/Support decision. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: take-over, 0%, competing-controller uninstall, and device-report confirms use an in-app dialog, not `window.confirm`
- ✅ Offline/error behavior: Cancel leaves PWM and apt untouched; Vite still cannot take PWM or pkexec
- ✅ Accessibility: `role="alertdialog"`, labelled title, Escape cancels; `:focus-visible` yellow rings
- ✅ i18n: reuse `cooling.takeoverAsk` `cooling.zeroAsk` `support.competitorsConfirm` `lighting.reportConfirm`; `app.cancel` `app.continue`

## Smoke scenario

1. _Given_ Cooling take-over is unchecked
2. _When_ the user checks Take over
3. _Then_ the in-app dialog asks to continue; Cancel does not call `pwm_takeover`

## Container map

| Layer | Path |
|-------|------|
| Logic | `apps/desktop/src/lib/pwm.js` `confirmDialog.js` |
| View | `apps/desktop/src/lib/ConfirmDialog.svelte` |
| Tests | `tests/test_chromaflow_cooling_ui.py` `tests/test_chromaflow_tauri.py` |

## Tests

- Automated: yes — no `window.confirm` in product pages; alertdialog markup; no `set_pwm`
- Coverage: 0% still requires the second confirm

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Keep the confirm *decision*. Never silent 0%.
