# Implementation Plan

Active work: [`BUILD_PLAN.md`](../BUILD_PLAN.md). Gap catalog: [`docs/PRODUCT_GAPS.md`](PRODUCT_GAPS.md).

## Milestone — Sprint 0–1 scaffold

| Task | Owner | Tests / fallback |
|------|-------|------------------|
| Bootstrap init + rename | AGENT | `validate-bootstrap --quick` |
| NOTICE, ARCHITECTURE, HARDWARE, SUPPORT-SCRIPT, ADRs | AGENT | files exist; encoding check |
| Allowlist YAML + install-support.sh --dry-run | AGENT | ShellCheck; dry-run JSON; no apt/modprobe |
| Inventory CLI | AGENT | `cargo test --workspace --exclude chromaflow-desktop`; fixture sysfs |
| Tauri/Svelte stubs | AGENT | `npm run build` in `apps/desktop`; CI `cargo build -p chromaflow-desktop` with webkit2gtk |
## Next (Sprint 17+)

1. HUMAN live take-over / RPM / close-GUI failsafe (Sprint 16 backlog)
2. Linear/Flat/Sync curve types and File `.sensor` mixes
3. liquidctl AIO pump rows when `liquidctl status` exposes speed
