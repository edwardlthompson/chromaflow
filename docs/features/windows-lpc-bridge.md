# Feature: windows-lpc-bridge

> Elevated `chromaflow-lpc` + PawnIO SuperIO bridge for motherboard PWM on Windows (ADR-0022).

## Acceptance criteria

- ✅ Unelevated GUI talks JSON lines on `\\.\pipe\chromaflow-lpc`
- ✅ ITE dual-chip MVP mirrors into `%LOCALAPPDATA%\chromaflow\run\lpc-live`
- ✅ Duty writes go through `pwm_apply` (never silent 0%)
- ✅ IsaBridge + ECIO firmware-disable fallbacks documented
- 🔲 Lab: audible Auto-calibrate on Gigabyte dual-ITE

## Smoke scenario

1. _Given_ `CHROMAFLOW_LPC_MOCK=1` or a live PawnIO helper
2. _When_ `cargo test -p chromaflow-lpc` and `cargo test -p chromaflow-core --lib`
3. _Then_ pipe protocol and duty reject-zero tests pass

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-lpc/`, `crates/chromaflow-core/src/windows/lpc_client.rs` |
| View | Cooling / Support banners |
| Tests | `crates/chromaflow-lpc/src/proto.rs`, `lpc_client` tests |
| Wiring | `scripts/install-windows-runtime.ps1` |

## Tests

- Automated: yes — unit + mock pipe roundtrip
- Coverage: real ITE board remains HUMAN smoke

## Fallback validation

- Why tests are not feasible: N/A for unit path
- Command: `bash scripts/smoke-chromaflow.sh` or `cargo test -p chromaflow-lpc`
