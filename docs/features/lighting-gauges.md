# Feature: lighting-gauges

> Host Direct green–yellow–red meters from CPU, GPU, combined, RAM, and disk. No OpenRGB plugin. Checklist: 🔲 open · ✅ done · ❌ blocked.

## Acceptance criteria

- ✅ User-visible behavior: Lighting shows a picker-sized hardware cube (CPU, GPU, Combined, RAM, Disk) with name/temp/usage and a spectrum bar; Temp/Usage toggle drives LED color; effect names are those hardware labels (not “temp”); `Hardware gauges` paints motherboard from CPU, GPU AIO from GPU, keyboard and speakers from combined; meters share Cooling’s 400 ms `hardware_gauges` tick (including during Cycle All)
- ✅ Offline/error behavior: missing GPU uses cached `nvidia-smi` (2 s) else green + status; missing CPU uses GPU for combined; missing load stays last ratio else green; bad meminfo/`df` stay last ratio else green
- ✅ Accessibility: Temp/Usage is a labelled toggle group; each row is a button; Speed tooltip explains gauges ignore it
- ✅ i18n: `lighting.gaugeMetric`, `lighting.gaugeTemp`, `lighting.gaugeUsage`, `lighting.gaugesRoute` in `apps/desktop/src/locales/en.json`

## Smoke scenario

1. _Given_ native OpenRGB on `127.0.0.1:6742` and host Direct
2. _When_ the user applies `Hardware gauges`
3. _Then_ Aorus follows CPU, 4090 AIO follows GPU, Keychron and Arena follow the average of CPU and GPU, PWM is not written, and Cinnamon does not hitch from inventory rebuilds on the Lighting tab

## Container map

| Layer | Path |
|-------|------|
| Logic | `crates/chromaflow-core/src/gauges.rs`, `nvidia_smi.rs`, `apps/desktop/src/lib/gauges.js`, `gaugesTick.js`, `effectQmk.js` |
| View | `apps/desktop/src/pages/Lighting.svelte`, `EffectList.svelte`, `GaugeMeter.svelte`, `App.svelte` |
| Tests | `gauges.rs` / `nvidia_smi.rs` unit tests + `tests/test_chromaflow_lighting.py` |
| Wiring | Tauri `hardware_gauges` on `allow-inventory`; App `startGaugeTick` (400 ms) feeds Cooling and Lighting |
## Tests

- Automated: yes — lerp stops, max(cpu,gpu), empty meminfo, nvidia-smi parse, classify route/cpu/gpu, gaugeLane 4090/Aorus/Keychron/Arena, skip identical paint, 400 ms shared poll, usage hist includes `cpu_load`
- Coverage: hwmon GPU preferred over smi; `df -P` parse fixture; `scan_temps` skips PWM; disk TTL 30 s; smi TTL 2 s

## Fallback validation

- Command: `python3 scripts/agent-run.py watch-agent-gates --once --autofix --scope auto`

## Definition of Done

Gauges are host Direct frames (ADR-0017). No Hardware Sync plugin. PWM remains ADR-0010.
