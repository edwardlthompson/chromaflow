# Feature: desktop-scale

> One type scale (14 / 16 / 12) and spacing (4 / 8 / 12 / 16); leftover 44px hit targets; `.btn-secondary`. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: body 16px, UI chrome 14px, meta 12px; magic `0.82rem` / `0.75rem` / `0.72rem` / `0.62rem` collapse to that scale; spacing uses 4/8/12/16px tokens; chips 36px in a 44px row; swatches 44px; Hide/Clone/Install item use `.btn-secondary`
- ✅ Offline/error behavior: Vite layout unchanged besides scale; no extra IPC
- ✅ Accessibility: plus/hide stay `--fc-hit` 2.75rem; `:focus-visible` yellow rings stay; yellow is not used as text
- ✅ i18n: no new user copy in CSS

## Smoke scenario

1. _Given_ Cooling and Lighting are open
2. _When_ the window is ~1280px
3. _Then_ chips/swatches meet the hit-target rule and Hide is visually secondary

## Container map

| Layer | Path |
|-------|------|
| View | `apps/desktop/src/app.css` `apps/desktop/src/fc-tokens.css` |
| Tests | `tests/test_chromaflow_cooling_ui.py` `tests/test_chromaflow_tauri.py` |

## Tests

- Automated: yes — `--fc-*` type/space tokens; `.btn-secondary`; chip min-size
- Coverage: no Golden Path teal in `app.css`

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Scale only. Do not restyle to teal. PWM unchanged. 22rem cooling tiles and yellow selection stay.
