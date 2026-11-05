# Build Plan

<!-- remaining-tally -->
**Remaining:** AGENT 1 · LOCAL 1 · CLOUD 0 · AUTO 15 · HUMAN 19 · ADB 0 · **35 open**
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
- 🔲 [AUTO] Merge Dependabot [#84](https://github.com/edwardlthompson/chromaflow/pull/84) (chore(deps-dev): bump ip-address from 10.5.0 to 10.7.6 in /examples/web)
- 🔲 [AUTO] Merge Dependabot [#83](https://github.com/edwardlthompson/chromaflow/pull/83) (chore(deps-dev): bump brace-expansion from 5.0.9 to 5.0.12 in /examples/web)
- 🔲 [AUTO] Merge Dependabot [#81](https://github.com/edwardlthompson/chromaflow/pull/81) (chore(deps-dev): bump source-map-js from 1.2.1 to 1.2.2 in /examples/web)
- 🔲 [AUTO] Merge Dependabot [#80](https://github.com/edwardlthompson/chromaflow/pull/80) (chore(deps): bump source-map-js from 1.2.1 to 1.2.2 in /apps/desktop)
- 🔲 [AUTO] Merge Dependabot [#77](https://github.com/edwardlthompson/chromaflow/pull/77) (chore(deps): bump the chromaflow-rust group across 1 directory with 4 updates)
- 🔲 [AUTO] Merge Dependabot [#73](https://github.com/edwardlthompson/chromaflow/pull/73) (chore(deps-dev): bump vite from 8.3.0 to 8.3.5 in /examples/web)
- 🔲 [AUTO] Merge Dependabot [#70](https://github.com/edwardlthompson/chromaflow/pull/70) (chore(deps-dev): bump vite from 8.3.0 to 8.3.5 in /apps/desktop)
- 🔲 [AUTO] Merge Dependabot [#69](https://github.com/edwardlthompson/chromaflow/pull/69) (chore(deps-dev): bump js-yaml from 5.2.3 to 5.5.0 in /examples/web)
- 🔲 [AUTO] Merge Dependabot [#66](https://github.com/edwardlthompson/chromaflow/pull/66) (chore(deps-dev): bump undici from 7.29.0 to 7.30.1 in /examples/web)
- 🔲 [AUTO] Merge Dependabot [#63](https://github.com/edwardlthompson/chromaflow/pull/63) (chore(deps): bump the github-actions group across 1 directory with 4 updates)
- 🔲 [AUTO] Merge Dependabot [#57](https://github.com/edwardlthompson/chromaflow/pull/57) (chore(deps-dev): bump the web-dependencies group across 1 directory with 8 updates)
- 🔲 [AUTO] Merge Dependabot [#49](https://github.com/edwardlthompson/chromaflow/pull/49) (chore(deps-dev): bump compression from 1.8.1 to 1.8.2 in /examples/web)
- 🔲 [AUTO] Merge Dependabot [#47](https://github.com/edwardlthompson/chromaflow/pull/47) (chore(deps-dev): bump proxy-addr from 2.0.7 to 2.0.8 in /examples/web)
- 🔲 [AUTO] Merge Dependabot [#38](https://github.com/edwardlthompson/chromaflow/pull/38) (chore(deps): bump the chromaflow-desktop group across 1 directory with 5 updates)
- 🔲 [AUTO] Merge Dependabot [#30](https://github.com/edwardlthompson/chromaflow/pull/30) (chore(deps): bump devalue from 5.9.2 to 5.9.4 in /apps/desktop)
- 🔲 [AGENT][LOCAL] Merge release [#10](https://github.com/edwardlthompson/chromaflow/pull/10) (chore(main): release 1.9.0) — scope: .
<!-- open-prs-sync:end -->

### Template gaps (synced)

> Auto-managed. Do not hand-edit inside markers.

<!-- template-gaps-sync:begin -->
_No template gaps; .template-version matches upstream (or template maintainer N/A)._
<!-- template-gaps-sync:end -->

### Sacred files (human)

These stay open. Do not copy them from the parent template. See `.agent/HUMAN_BACKLOG.md`.

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

### Sprint 41 — Windows SuperIO FC-parity (GPL-3)

Keep GPL-3.0-or-later (ADR-0023). No proprietary relicense. No reboot. Do not paste Fan Control / LHM sources. Official PawnIO stays a separate install (winget / PawnIO.Setup) — GPL-3 must not embed the proprietary driver binary in our package.

### Sequential

- ✅ [AGENT][LOCAL] Windows: suppress EasyTune/Fan Control autostart (services + tasks + Run) in seize + Install All probe — scope: `crates/chromaflow-core/src/windows/conflicts.rs`, `support.rs`, tests
- ✅ [AGENT][LOCAL] Windows: Gigabyte EC firmware-disable via IsaBridgeEC MMIO (offset 0x947) then re-probe — scope: `crates/chromaflow-lpc/src/isa_bridge.rs`, `server.rs`, docs
- ✅ [AGENT][LOCAL] Windows: first-run Install All keeps winget PawnIO + LPC/chromaflowd autostart; Cooling banner Install all CTA — scope: `support.rs`, `docs/features/windows-lpc-bridge.md`, `docs/HARDWARE.md`

### Parallel

<!-- parallel_exception: Sequential-only Windows SuperIO sprint; all AGENT rows complete -->
_None until Sequential lock._

### Sprint 42 — FC live SuperIO parity (ECIO)

Fan Control UI shows saved `/lpc/it8689e` + `/lpc/it87952e` cards; FC log still reports those sensors missing. Same empty probe as ChromaFlow. Implement LHM-style ECIO Gigabyte fallback (original Rust, no paste). GPL-3. No reboot.

### Sequential

1. ✅ [AGENT][LOCAL] Windows: probe-debug stages (LpcIO IDs, IsaBridge find, 0x947, ECIO) + Cooling/docs honesty vs FC ghost cards — scope: `crates/chromaflow-lpc/src/probe_debug.rs`, `docs/features/windows-lpc-bridge.md`, `docs/HARDWARE.md`, `apps/desktop/src/locales/en.json`
2. ✅ [AGENT][LOCAL] Windows: ECIO Gigabyte firmware-disable fallback when IsaBridge latch is 0xFF / IDs 0xFFFF, then retry ITE probe — scope: `crates/chromaflow-lpc/src/isa_bridge.rs`, `ecio.rs`, `server.rs`, tests
3. ✅ [AGENT][LOCAL] Windows: redeploy LPC helper, verify pipe probe / docs; keep EasyTune/FC autostart suppressed — scope: install path, `windows-lpc-bridge.md`

### Parallel

<!-- parallel_exception: Sequential-only ECIO sprint; all AGENT rows complete -->
_None until Sequential lock._

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
