# Feature: rail-icons

> Left rail uses large icons, not tab names. The four buttons share height evenly as the window grows or shrinks. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Cooling, Lighting, Profiles, and Support are icon buttons that split the rail’s vertical space equally; icons scale with that slot; hover/title still names the page
- ✅ Offline/error behavior: N/A (chrome only; no I/O)
- ✅ Accessibility: each button has `aria-label` from i18n; keyboard still reaches the rail; current page stays `aria-current="page"`
- ✅ i18n: reuse `cooling.title` / `lighting.title` / `profiles.title` / `support.title` (no new copy in CSS)

## Smoke scenario

1. _Given_ the native GUI is open
2. _When_ the window height changes
3. _Then_ the four rail buttons stay equal height, icons grow or shrink with the slot, and the visible page still matches the highlighted icon

## Container map

| Layer | Path |
|-------|------|
| View | `apps/desktop/src/lib/RailNav.svelte` `apps/desktop/src/app.css` `apps/desktop/src/App.svelte` |
| Tests | `tests/test_chromaflow_tauri.py` |
| Wiring | App.svelte rail swap ≤10 lines |

## Tests

- Automated: yes — rail markup has SVG + `aria-label`; CSS `repeat(4, calc((100vh - 3rem) / 4))` on `.fc-rail`; session tab ids unchanged
- Coverage: four ids still Cooling / Lighting / Profiles / Support

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Icon rail. Session keys unchanged. PWM unchanged.
