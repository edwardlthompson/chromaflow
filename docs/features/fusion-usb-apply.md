# Feature: fusion-usb-apply

> Motherboard Fusion and CPU AIO stay listed from USB `048d:5702`. Apply must still paint analog + AIO when hidraw and `liquidctl list` are empty. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Apply on Motherboard Fusion and CPU AIO changes those lamps while USB `048d:5702` is present, even if `/sys/class/hidraw` has no ITE node and `liquidctl list` prints nothing
- ✅ Offline/error behavior: typed error if the USB device is gone; never dump NVIDIA I2C; never silent PWM
- ✅ Accessibility: existing Lighting alerts; no new unlabeled controls
- ✅ i18n: reuse `lighting.applyOk` / apply error strings

## Smoke scenario

1. _Given_ `lsusb` shows `048d:5702` and Lighting lists Motherboard Fusion and CPU AIO
2. _When_ the user Applys a solid color on each Fusion row, or CPU usage while other lamps Cycle All
3. _Then_ analog motherboard zones and CPU AIO headers show that color; PWM is not written

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/liquidctl_apply.rs` `fusion_hid.rs` `lighting.rs` `lighting_cycle.rs` |
| View | `apps/desktop/src/lib/lighting.js` `lightingTick.js` `apps/desktop/src/pages/Lighting.svelte` |
| Tests | `tests/test_chromaflow_lighting.py` `crates/chromaflow-core` |

## Tests

- Automated: yes — USB `048d:5702` still lists two Fusion rows; apply refuses empty color; Cycle All is host `fixed`; HID kind per row; cycle thread does not call `fusion_hid`; no `set_pwm`; no `i2cdump`
- Coverage: missing USB is a typed error

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Fusion Apply works from USB identity via `fusion-hid.py` serve. OpenRGB stays opt-in. PWM unchanged.
