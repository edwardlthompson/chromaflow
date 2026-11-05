# Feature: windows-desktop

> Tauri GUI + NSIS packaging on Windows beside the Linux Mint path.

## Acceptance criteria

- ✅ Config under `%APPDATA%\chromaflow` via `platform/`
- ✅ Single-instance lock under `%LOCALAPPDATA%\chromaflow`
- ✅ NSIS bundle target + Release `windows-nsis` job
- ✅ Cooling banner honesty when SuperIO probe is empty
- 🔲 Lab: Keychron / Arena / Fusion HID Apply on Windows hardware

## Smoke scenario

1. _Given_ a Windows 10/11 x64 host with WebView2
2. _When_ `bash scripts/smoke-chromaflow.sh` runs
3. _Then_ cargo tests (exclude desktop) and `chromaflow-cli support` succeed

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/platform/windows.rs`, `windows/` |
| View | `apps/desktop` |
| Tests | `tests/test_chromaflow_tauri.py`, crate lib tests |
| Wiring | `.github/workflows/chromaflow.yml` |

## Tests

- Automated: yes — workspace cargo tests exclude desktop host on CI smoke
- Coverage: Windows HID Apply lab rows remain HUMAN

## Fallback validation

- Why tests are not feasible: N/A for automated smoke
- Command: `bash scripts/smoke-chromaflow.sh`
