# Feature: ci-push-or-pr

> Monday health and BUILD_PLAN sync must not fail the job when `main` requires PRs (`GH006`). Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: after a BUILD_PLAN sync commit, CI tries `git push` to the default branch; on rejection it pushes a `chore/ci-sync-*` branch and opens a PR
- ✅ Offline/error behavior: no-op when BUILD_PLAN is unchanged; missing `gh` after a successful branch push still exits 0 only if the branch is on origin
- ✅ Accessibility: N/A (CI)
- ✅ i18n: N/A

## Smoke scenario

1. _Given_ a child repo with protected `main`
2. _When_ weekly health commits a template-gaps-sync change
3. _Then_ the job opens a PR instead of failing on `GH006`

## Container map

| Layer | Path |
|-------|------|
| Logic | `scripts/ci-push-or-pr.sh` |
| Tests | `tests/test_ci_push_or_pr.py` |
| Wiring | `.github/workflows/weekly-health-check.yml` `.github/workflows/sync-open-prs-build-plan.yml` |

## Tests

- Automated: yes — workflows call the helper; weekly job has `pull-requests: write`; helper mentions protected-branch PR
- Coverage: no bare failing `git push` retry in weekly-health after commit

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Do not grant the health-check bot a ruleset bypass. PWM unchanged.
