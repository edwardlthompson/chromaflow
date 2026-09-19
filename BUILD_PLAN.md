# Build Plan

<!-- remaining-tally -->
**Remaining:** AGENT 0 · LOCAL 0 · CLOUD 0 · AUTO 0 · HUMAN 19 · ADB 0 · **19 open**
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

<!-- product-brief-sync:begin -->
> Read `AGENT.md` before any sprint row.

**One-liner:** Linux Mint app for fan and pump curves and RGB control.
**Do not drift:** hwmon, openrgb, polkit, pwm

**Rules:**
- The GUI never runs as root. The polkit helper is the only root path.
- Never write PWM except through `pwm_apply` or `chromaflow daemon --watchdog`.
- Never stop `chromaflowd`. Failsafe is `pwm*_enable=2`. Never silent 0%.

**First milestone:** Ship fan curves and lighting with no bundled kernel modules.
<!-- product-brief-sync:end -->

> **Sprint 40** archived in COMPLETED_TASKS.md @ `ac4c695`.
> **Template 1.8.0 catch-up** archived in COMPLETED_TASKS.md @ `ac4c695`.
> **Sprint 39** archived in COMPLETED_TASKS.md @ `e03b3da`.
> **Sprint 38** archived in COMPLETED_TASKS.md.
> **Sprint 37** archived in COMPLETED_TASKS.md.
> **Sprint 36** archived in COMPLETED_TASKS.md.
> **Sprint 30–35** archived in COMPLETED_TASKS.md.

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

### Sacred files (human)

These stay open. Do not copy them from the parent template. See `HUMAN_BACKLOG.md`.

- 🔲 [HUMAN] Sacred: AGENTS.md (never blind-overwrite)
- 🔲 [HUMAN] Sacred: docs/INITIALIZATION_PROMPT.md (never blind-overwrite)
- 🔲 [HUMAN] Sacred: docs/spec.md (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/blender/.gitignore (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/blender/README.md (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/blender/blender.toml (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/blender/cli.py (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/blender/cycles_device.py (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/blender/fixtures/icon-manifest.sample.json (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/blender/manifest.py (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/blender/qa.py (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/blender/render_job.py (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/blender/scene.py (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/node/package-lock.json (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/node/package.json (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/python/pyproject.toml (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/python/uv.lock (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/web/package-lock.json (never blind-overwrite)
- 🔲 [HUMAN] Sacred: examples/web/package.json (never blind-overwrite)

### UX & UI inventory

Complete list from construction gaps and `/ux-review`. Status is only planned / in_progress / done. `/build` does not execute these until `/ux-apply UX-NNN`.

<!-- ux-inventory:begin -->
_No UX inventory items._
<!-- ux-inventory:end -->

### Local agent (This Computer)

Standing queue for This Computer. `/build` claims these only.

<!-- local-agent-lane:begin -->
_No local agent items._
<!-- local-agent-lane:end -->

### Cloud agent (Cursor Cloud)

Standing queue for Cursor Cloud. Cloud claims these only.

<!-- cloud-agent-lane:begin -->
_No cloud agent items._
<!-- cloud-agent-lane:end -->

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
| Template 1.8.0 | 2026-09-18 | `COMPLETED_TASKS.md` |
| 40 | 2026-09-18 | `COMPLETED_TASKS.md` |
