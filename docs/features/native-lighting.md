# Feature: native-lighting

> MIT native HID ports plus OpenRGB identity gap catalog. OpenRGB is opt-in (`CHROMAFLOW_OPENRGB_SDK=1`). Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Lighting lists Keychron Q6 HE as `keychron` from hidraw even if OpenRGB names are thin; Apply color uses native VIA rgb_matrix SOLID (effect 1, speed 0, HSV) for uniform fills and retries until VIA GET effect matches; per-key host effects SET `0x0A` in 9-LED batches and GET_COLOR-sample LED 0; Cycle All is a Rust Direct fill (`lighting_cycle` `set_fill`) that paints all 108 keys from the 16-bit RGB clock ~8 Hz with Keychron per-key type 0 (static HSV, not breathing), with Arena/Fusion on the same 120 ms lamp thread, Prime live `0x62` (save `0x59` only on Apply or cycle stop), and GPU shroud Cycle All as a 500 ms double-buffered snap (RGB1 back buffer, then static+save; first snap still arms with idle Apply); All Devices only offers Solid/Breathing/Cycle All/Combined; each device lists what it can paint, grouped Solid / Rainbow / Rain and splash / Hardware; Arena rainbow uses 4 HID zones; Fusion host CPU/usage applies immediately; Lighting lists Motherboard Fusion and CPU AIO as separate rows; CPU AIO uses header analog plus a D_LED HID fill; Cycle All and Apply All buffer one hex and `lighting_broadcast` paints every native lamp together
- ✅ Offline/error behavior: missing VIA hidraw is a typed error; `CHROMAFLOW_NATIVE_RGB=0` uses OpenRGB only; OpenRGB-index devices without a port stay sdk-fallback (not fake UI rows)
- ✅ Accessibility: existing Lighting alerts; no new unlabeled controls
- ✅ i18n: reuse existing lighting apply strings

## Smoke scenario

1. _Given_ Keychron hidraw is plugdev-readable
2. _When_ the user Applys a color on the Keychron row (or `chromaflow rgb --backend keychron --device "Keychron Q6 HE" --color FF00FF`)
3. _Then_ the keys show that color over hidraw with OpenRGB not required, GET_COLOR matches, PWM is not written, and `chromaflowd` is not restarted

## Container map

| Layer | Path |
|-------|------|
| Index | `data/openrgb-device-index.csv` `data/native-lighting.yaml` `scripts/fetch-openrgb-device-index.sh` |
| Logic | `crates/chromaflow-core/src/lighting_port.rs` `keychron_apply.rs` `lighting_apply.rs` |
| View | `apps/desktop/src/lib/lighting.js` `research.js` `lightingTick.js` |
| Tests | `crates/chromaflow-core` + `tests/test_chromaflow_lighting.py` |
| Decision | `docs/adr/0020-native-lighting-ports.md` |

## Tests

- Automated: yes — CSV header + Keychron + GPU I2C row; overlay parse; SET packet layout; VIA Direct packets; `CHROMAFLOW_NATIVE_RGB=0`; no `set_pwm`; no OpenRGB C++ includes
- Coverage: empty color refused; Keychron listed without SDK name

## Fallback validation

- Live: `CHROMAFLOW_LIVE_RGB=1` GET_COLOR after native SET
- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

G-LED-NATIVE catalog. Native Keychron Direct. ADR-0020 identity CSV. OpenRGB is opt-in (ADR-0021). Fusion is HID + liquidctl. GPU shroud writes I2C `0x68` when present.
