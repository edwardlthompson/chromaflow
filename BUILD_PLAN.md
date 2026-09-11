# Build Plan

<!-- remaining-tally -->
**Remaining:** AGENT 0 · AUTO 1 · HUMAN 0 · ADB 0 · **1 open**
<!-- /remaining-tally -->

Live board for ChromaFlow. Finished work: [`COMPLETED_TASKS.md`](COMPLETED_TASKS.md).

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

### Sprint 0 — Customize

<!-- parallel_exception: init must finish before overlay -->

1. ✅ [AGENT] Run `scripts/init-project.sh` (web, FOSS, MIT, ChromaFlow)
2. ✅ [AGENT] Fill `branding/product.json` (`mode: product`); NOTICE; product docs
3. ✅ [HUMAN] Create GitHub repo and run `scripts/setup-github-repo.sh` (`gh` admin)
4. 🔲 [AUTO] Sprint 0 sign-off on `main` after first push: `validate-bootstrap --quick` · `feature-gate --stack web` · `check-github-ci --wait 300`
5. ✅ [HUMAN] FOSS tier chosen at init
6. ✅ [HUMAN] Enable Dependabot alerts and private vulnerability reporting
7. ✅ [HUMAN] Bookmark `docs/help/BATCH_COMMANDS.md`

### Sprint 1 — Inventory and Support dry-run

<!-- parallel_exception: public JSON locked in Sequential then overlay landed together -->

1. ✅ [AGENT] Lock inventory/support JSON + ADRs 0001/0006–0010
2. ✅ [AGENT] YAML allowlist + `install-support.sh --dry-run` + polkit stub
3. ✅ [AGENT] `chromaflow` CLI (sensors/devices/rescan/support) with fixture tests
4. ✅ [AGENT] Tauri/Svelte stub pages (display-only curve; Support dry-run)
5. ✅ [HUMAN] Approve ADRs and smoke on a Mint 21/22 Cinnamon machine

### Waiting on a person

1. ✅ [HUMAN] Live pkexec `--apply` on a VM after `.deb` helper exists (not this sprint)

### Open PRs (synced)

> Auto-managed. Do not hand-edit rows inside the markers.

<!-- open-prs-sync:begin -->
_No open Dependabot or Release Please PRs._
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

Older sprints: [`COMPLETED_TASKS.md`](COMPLETED_TASKS.md).
