# Feature: cooling-conflicts

> Cooling and Support list the same live fan/RGB daemons. Leftover unit files do not block PWM. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Cooling conflict banner and Support competitor list use active/enabled units or `pidof`, not leftover `.service` files
- ✅ Offline/error behavior: `CHROMAFLOW_CONFLICTS=none` is empty; missing systemctl falls back to pidof
- ✅ Accessibility: existing `role="alert"` banner
- ✅ i18n: existing `cooling.conflict`

## Smoke scenario

1. _Given_ fancontrol is purged and coolercontrold is masked
2. _When_ inventory is collected
3. _Then_ `conflicts[]` is empty and Cooling take-over can enable

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/conflicts.rs` `scripts/lib/chromaflow_competitors.py` |
| View | Cooling banner / Support competitor card (no new UI) |
| Tests | `tests/test_chromaflow_support.py` |
| Wiring | `collect_inventory` |

## Tests

- Automated: yes — source asserts `is-active`/`is-enabled`; env override; no unit-dir scan
- Coverage: masked leftover is not a conflict

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Cooling and Support agree. Leftover unit files do not block take-over.

## Notes

- After each AGENT step: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`
