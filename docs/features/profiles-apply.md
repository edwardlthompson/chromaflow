# Feature: profiles-apply

> A saved profile applies its named curve set and RGB name to hardware (G-PROF). Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Apply on a profile runs the same take-over confirm as Cooling, writes the named curve set through the watchdog, and applies the named RGB to lamps; Save still only writes `~/.config/chromaflow/`
- ✅ Offline/error behavior: Vite cannot take PWM; conflict banner blocks take-over; never silent 0%; failsafe remains `pwm*_enable=2`
- ✅ Accessibility: Apply is a labeled button; confirm dialog uses existing `cooling.takeoverAsk`
- ✅ i18n: `profiles.apply` / `profiles.applyNeedGui` in `apps/desktop/src/locales/en.json`

## Smoke scenario

1. _Given_ a profile `quiet` with curve set Quiet and RGB Solid
2. _When_ the user clicks Apply in the native GUI with no competing fan daemon
3. _Then_ Cooling take-over matches Quiet and Lighting matches the RGB name; chromaflowd is not killed

## Container map

| Layer | Path |
|-------|------|
| Logic | `apps/desktop/src/pages/Profiles.svelte` existing `pwm_takeover` / `lighting_broadcast` invokes |
| Tests | `tests/test_chromaflow_tauri.py` |
| Docs | `docs/PRODUCT_GAPS.md` G-PROF |

## Tests

- Automated: yes — Apply invokes `pwm_takeover` and lighting apply; no `set_pwm` symbol
- Coverage: missing native GUI shows applyNeedGui

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

G-PROF. Never silent 0%. Never `set_pwm`. Do not restart chromaflowd.
