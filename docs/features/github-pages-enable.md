# Feature: github-pages-enable

> The README Golden Path demo URL must resolve via GitHub Actions Pages, not a 404. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: repo Pages `build_type` is `workflow`; `pages.yml` can deploy `examples/web/dist`
- ✅ Offline/error behavior: `scripts/enable-github-pages.sh` is idempotent when Pages is already on; `gh` auth failures stay HUMAN
- ✅ Accessibility: N/A (static demo)
- ✅ i18n: N/A (Golden Path English)

## Smoke scenario

1. _Given_ `gh api repos/{owner}/{repo}/pages` was 404
2. _When_ `python3 scripts/agent-run.py enable-github-pages` (or the shell wrapper) runs
3. _Then_ Pages exists with workflow source; dispatch `pages.yml` if the site was just created

## Container map

| Layer | Path |
|-------|------|
| Logic | `scripts/lib/github_pages_enable.py` |
| Tests | `tests/test_github_pages_enable.py` |
| Wiring | `scripts/enable-github-pages.sh` `.github/workflows/pages.yml` `docs/WEB_PROJECT_LAYOUT.md` |

## Tests

- Automated: yes — POST payload uses `build_type=workflow`; already-enabled GET is success; README still links the expected github.io URL
- Coverage: mock `gh`; do not require a live HTTP 200 in unit tests

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Do not publish `docs/` as Pages. PWM unchanged.
