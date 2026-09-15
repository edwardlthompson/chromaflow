# Feature: lighting-picker-layout

> All devices color/effect controls use the unused width on the Lighting card. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: with All devices open, the wheel stays square; sliders, hex/effects, and gauges grow to fill leftover card width
- ✅ Offline/error behavior: Vite preview still lays out; no extra IPC
- ✅ Accessibility: controls stay ≥44px; keyboard order wheel → channels → effects → gauges
- ✅ i18n: no new copy in CSS

## Smoke scenario

1. _Given_ chromaflow-gui is on Lighting with All devices selected
2. _When_ the window is ~1280px and ~1920px wide
3. _Then_ the picker row fills the card; per-device rows still expand below

## Container map

| Layer | Path |
|-------|------|
| View | `apps/desktop/src/lib/DeviceList.svelte` `apps/desktop/src/app.css` |
| Tests | `tests/test_chromaflow_lighting.py` |

## Tests

- Automated: yes — CSS/grid classes for the All devices picker row
- Coverage: `openId === "all"` layout only

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Layout only. PWM unchanged. Do not dump GPU I2C.
