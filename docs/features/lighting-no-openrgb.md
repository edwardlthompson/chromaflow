# Feature: lighting-no-openrgb

> Stop the OpenRGB sibling. Lighting uses native HID + liquidctl Fusion. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Lighting lists Keychron, Arena, Prime, and Fusion without 127.0.0.1:6742. Apply color uses those backends. OpenRGB is not started.
- ✅ Offline/error behavior: missing hidraw/liquidctl is a typed error; GPU row errors if I2C `0x68` is missing; `CHROMAFLOW_OPENRGB_SDK=1` restores the sibling
- ✅ Accessibility: existing Lighting alerts; no new unlabeled controls
- ✅ i18n: `lighting.localhost` / Fusion copy say native paths, not “start OpenRGB”

## Smoke scenario

1. _Given_ OpenRGB is not listening on 6742 and `chromaflowd` is still the PWM watchdog
2. _When_ the user Applys a color on Fusion or Keychron
3. _Then_ the device lights, no OpenRGB process appears, and PWM is not written

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/{lighting_port,gpu_apply,liquidctl_apply,openrgb_spawn}.rs` |
| View | `apps/desktop/src/lib/lighting.js` `pages/Lighting.svelte` |
| Tests | `tests/test_chromaflow_lighting.py` `gpu_apply` unit tests |
| Decision | `docs/adr/0021-native-lighting-without-openrgb.md` |

## Tests

- Automated: yes — spawn off unless `CHROMAFLOW_OPENRGB_SDK=1`; Fusion listed without SDK; PCI GPU identity; no `set_pwm`
- Coverage: empty color refused; Fusion apply does not call `ensure_sdk`

## Fallback validation

- Live: `liquidctl -m Fusion set sync color fixed FF00AA` with 6742 closed
- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

OpenRGB dead by default. Native Lighting for USB + Fusion. GPU shroud writes I2C `0x68` when present. ADR-0021.
