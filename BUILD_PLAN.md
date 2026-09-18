# Build Plan

<!-- remaining-tally -->
**Remaining:** AGENT 0 · AUTO 2 · HUMAN 0 · ADB 0 · **2 open**
<!-- /remaining-tally -->

Live board for ChromaFlow. Finished work: [`COMPLETED_TASKS.md`](COMPLETED_TASKS.md). Catalog vs original brief: [`docs/PRODUCT_GAPS.md`](docs/PRODUCT_GAPS.md).

**Who:** `AGENT` code · `HUMAN` person · `ADB` device · `AUTO` CI/scripts
**State:** 🔲 open · ✅ done · ❌ blocked — reason

Format: `🔲 [OWNER] Short task`. Sequential `[AGENT]` first.

## Smoke gate (hard stop)

After every `[AGENT]` row: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

After the last `[AGENT]`/`[AUTO]` row in a sprint is ✅:

```bash
python3 scripts/agent-run.py smoke-sprint --require

```

---

## Product

> **Sprint 39** archived in COMPLETED_TASKS.md @ `e03b3da`.
> **Sprint 38** archived in COMPLETED_TASKS.md.
> **Sprint 37** archived in COMPLETED_TASKS.md.
> **Sprint 36** archived in COMPLETED_TASKS.md.
> **Sprint 30–35** archived in COMPLETED_TASKS.md.

### Open PRs (synced)

> Auto-managed. Do not hand-edit rows inside the markers.

<!-- open-prs-sync:begin -->
- 🔲 [AUTO] Merge Dependabot [#8](https://github.com/edwardlthompson/chromaflow/pull/8) (chore(deps): bump actions/setup-java from 5 to 6 in the github-actions group)
- 🔲 [AUTO] Merge Dependabot [#6](https://github.com/edwardlthompson/chromaflow/pull/6) (chore(deps-dev): bump the web-dependencies group in /examples/web with 2 updates)
<!-- open-prs-sync:end -->

### Template gaps (synced)

> Auto-managed. Do not hand-edit inside markers.

<!-- template-gaps-sync:begin -->
_No template gaps; .template-version matches upstream (or template maintainer N/A)._
<!-- template-gaps-sync:end -->

---

## Ongoing Maintenance

Not a checklist. GitHub Monday cron and `/ship` own recurring chores. Do not put weekly maintenance rows on this board.

## Archive

Finished sprints 0–29: [`COMPLETED_TASKS.md`](COMPLETED_TASKS.md).

### Archived Sprints

| Sprint | Complete | Location |
|--------|----------|----------|
| 0–15 | 2026-09-12 | `COMPLETED_TASKS.md` |
| 16–23 | 2026-09-12 | `COMPLETED_TASKS.md` |
| 24–29 | 2026-09-13 | `COMPLETED_TASKS.md` |
| 30–33 | 2026-09-14 | `COMPLETED_TASKS.md` |
| 34 | 2026-09-14 | `COMPLETED_TASKS.md` |
| 35 | 2026-09-14 | `COMPLETED_TASKS.md` |
| 36 | 2026-09-14 | `COMPLETED_TASKS.md` |
| 37 | 2026-09-14 | `COMPLETED_TASKS.md` |
| 38 | 2026-09-14 | `COMPLETED_TASKS.md` |
| 39 | 2026-09-14 | `COMPLETED_TASKS.md` |
