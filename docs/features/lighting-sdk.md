# Feature: lighting-sdk

> Localhost OpenRGB SDK device list plus color/mode apply. No vendored C++. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Lighting shows protocol, LED count, and current color per device; click the swatch for that device’s color wheel (RGB/HSV/Kelvin); All devices has its own wheel and apply; every device lists the same host effect catalog; LED layouts sit on the picker row as collapsible 4:1 cards (keyboard aspect or packed LEDs) with live Direct pixels from the host engine
- ✅ Offline/error behavior: 200ms connect timeout; failed set is an error; no fake Windows-only devices; Arena/Prime HID host effects use LED 0 (no OpenRGB `UPDATE_MODE`)
- ✅ Accessibility: status text is readable; color picker has a text label; effect select and LED cells are labelled
- ✅ i18n: `lighting.apply` `lighting.color` `lighting.k` `lighting.effect` `lighting.settings` `lighting.showLayout` keys in `apps/desktop/src/locales/en.json`

## Smoke scenario

1. _Given_ native OpenRGB is listening on `127.0.0.1:6742`
2. _When_ the user picks a color and Apply on a listed controller, or applies a host effect, or clicks a key in Per-LED layout
3. _Then_ ChromaFlow sends `SET_CLIENT_NAME` / `SET_CUSTOM_MODE` / `UPDATE_LEDS` (host effect frames via `lighting_sync`, or `UPDATE_SINGLE_LED`) with `data_size == pkt_size` where required, Fusion `D_LED` zone updates, Arena 7 HID if listed, and does not write PWM

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/openrgb.rs`, `openrgb_apply.rs`, `lighting_apply.rs`, `apps/desktop/src/lib/effectQmk.js` |
| View | `apps/desktop/src/pages/Lighting.svelte` |
| Tests | `crates/chromaflow-core` unit tests + `tests/test_chromaflow_lighting.py` |

## Tests

- Automated: yes — UPDATE_LEDS body, LED-count parse, hex reject, no `set_pwm`
- Coverage: empty device/color; unknown backend; udev still per-VID

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto` (or `cargo test --workspace --exclude chromaflow-desktop`)

## Definition of Done

G-LED color apply through OpenRGB, liquidctl Fusion, Arena 7 HID, and Prime Neo hidraw. PWM remains ADR-0010.

## Notes

- Packet IDs from the public OpenRGB SDK wiki (ADR-0011). Do not paste controller C++.
- Firmware matrix effects do not dump animation RAM. Apply effect runs a **host Direct engine** (`effectQmk.js` plus hardware gauges in ADR-0017) on every OpenRGB controller so each LED color is known (ADR-0016). `lighting_sync` writes motion at ~10 Hz and skips unchanged gauge fills; the layout paints those frames locally. Keychron GET_COLOR overlay (ADR-0015) is skipped while those frames are in flight. Apply color sets Direct and stops the engine.
- Fleet: OpenRGB SDK is the auto-discovery path (name, LED count, colors, matrix map) for any controller the local server lists. The UI effect catalog is `HOST_EFFECTS` (same names on every device), not each controller’s firmware `modes[]`. Arena 7 HID `1038:1a00` report `0x06`, Prime Neo `1038:1856` output `0x62`, and Fusion D_LED resize-to-32 are VID/board backends; host effects there use LED 0 at ~30 Hz. ITE Fusion USB `048d:5702` is the same chip as the Aorus SDK row plus liquidctl; leftover HID without a backend stays research-only.
