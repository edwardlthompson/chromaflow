# Feature: desktop-tokens

> Product desktop CSS uses generated design tokens, not raw hex (G-I18N). Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: `apps/desktop` colors/spacing come from `design-tokens/design-tokens.json` (synced CSS variables); Fan Control navy + yellow look stays
- ✅ Offline/error behavior: `scripts/sync-design-tokens.py` remains the regen path; no network
- ✅ Accessibility: contrast of text on cards and rail still meets the current dark theme
- ✅ i18n: still no user copy in CSS

## Smoke scenario

1. _Given_ tokens define `--fc-card` / `--fc-yellow` (or mapped `--gp-*`)
2. _When_ `app.css` is grepped for `#` color literals in rules
3. _Then_ chrome/cards/rail use `var(--…)` only (gradients in the color wheel may keep stops)

## Container map

| Layer | Path |
|-------|------|
| Logic | `design-tokens/design-tokens.json` `scripts/sync-design-tokens.py` |
| View | `apps/desktop/src/app.css` |
| Tests | `tests/test_chromaflow_tauri.py` or a desktop token check |
| Docs | `docs/DESIGN_GUIDE.md` `docs/PRODUCT_GAPS.md` G-I18N |

## Tests

- Automated: yes — `app.css` chrome/card/rail rules use `var(--` ; wheel gradients documented if exempt
- Coverage: `sync-design-tokens` still writes web/Android outputs

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

G-I18N tokens for product CSS. PWM unchanged. Do not bump gtk-rs.
