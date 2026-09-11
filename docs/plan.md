# Implementation Plan

Active work: [`BUILD_PLAN.md`](../BUILD_PLAN.md).

## Milestone — Sprint 0–1 scaffold

| Task | Owner | Tests / fallback |
|------|-------|------------------|
| Bootstrap init + rename | AGENT | `validate-bootstrap --quick` |
| NOTICE, ARCHITECTURE, HARDWARE, SUPPORT-SCRIPT, ADRs | AGENT | files exist; encoding check |
| Allowlist YAML + install-support.sh --dry-run | AGENT | ShellCheck; dry-run JSON; no apt/modprobe |
| Inventory CLI | AGENT | `cargo test --workspace --exclude chromaflow-desktop`; fixture sysfs |
| Tauri/Svelte stubs | AGENT | `npm run build` in `apps/desktop`; CI `cargo build -p chromaflow-desktop` with webkit2gtk |

## Next feature (after this milestone)

1. `chromaflowd` watchdog + `pwm*_enable` writes with failsafe
2. Live polkit `--apply` on a Mint VM (`[HUMAN]`)
3. OpenRGB SDK read of controller list (still no vendored C++)
4. `.deb` that installs the pinned helper path
