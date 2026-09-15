"""Lighting lists USB/ARGB candidates and a detection-support prompt."""
from __future__ import annotations

import json
import os
import stat
import subprocess
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


class LightingHidTests(unittest.TestCase):
    def test_ui_lists_hid_and_prompt(self) -> None:
        lighting = (ROOT / "apps/desktop/src/pages/Lighting.svelte").read_text(encoding="utf-8")
        lib = (ROOT / "apps/desktop/src/lib/lighting.js").read_text(encoding="utf-8")
        locales = (ROOT / "apps/desktop/src/locales/en.json").read_text(encoding="utf-8")
        self.assertIn("from \"../lib/lighting.js\"", lighting)
        self.assertNotIn("live-support-advanced.json", lighting)
        self.assertNotIn("support_dry_run", lighting)
        self.assertNotIn("support_apply", lighting)
        self.assertNotIn("class=\"banner\"", lighting.split("<div class=\"card\">")[0])
        self.assertIn("hid_rgb", lib)
        self.assertNotIn("set_pwm", lib)
        self.assertNotIn("lighting.modulesPrompt", locales)
        self.assertNotIn("lighting.localhost", locales)
        self.assertNotIn("lighting.fusionHint", locales)
        spec = (ROOT / "docs/features/lighting-hid.md").read_text(encoding="utf-8")
        self.assertIn("ADR-0012", spec)
        self.assertIn("0x06", spec)
        lighting = (ROOT / "apps/desktop/src/pages/Lighting.svelte").read_text(encoding="utf-8")
        support = (ROOT / "apps/desktop/src/pages/Support.svelte").read_text(encoding="utf-8")
        devices = (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8")
        self.assertIn("lighting_apply", lighting)
        self.assertIn("applyAll", lighting)
        self.assertIn("DeviceList", lighting)
        self.assertIn("ColorWheel", devices)
        self.assertIn("lighting.allDevices", devices)
        self.assertIn("openPicker", devices)
        self.assertIn("extraKernelItems", lib)
        self.assertIn("extraFailNote", lib)
        self.assertIn("if (present) return \"\"", lib)
        self.assertIn("ExtraList", support)
        self.assertNotIn("ExtraList", lighting)
        self.assertIn("runApply", support)
        self.assertIn("lighting.extras", locales)
        self.assertIn("lighting.allDevices", locales)
        self.assertNotIn("lighting.protocol", locales)
        self.assertIn("lighting.rowDetails", locales)
        self.assertNotIn("usbArgb", lighting)
        self.assertIn("SUGGESTED_HEX", (ROOT / "apps/desktop/src/lib/color.js").read_text(encoding="utf-8"))
        self.assertIn("pushRecent", (ROOT / "apps/desktop/src/lib/color.js").read_text(encoding="utf-8"))
        wheel = (ROOT / "apps/desktop/src/lib/ColorWheel.svelte").read_text(encoding="utf-8")
        self.assertIn("hue-ring", wheel)
        self.assertIn("sv-square", wheel)
        self.assertIn("ChannelRow", wheel)
        row = (ROOT / "apps/desktop/src/lib/ChannelRow.svelte").read_text(encoding="utf-8")
        self.assertIn("inputmode", row)
        self.assertIn("nudge", row)
        self.assertIn("[^\\d]", row)
        self.assertNotIn("brightnessLabel", devices)
        self.assertNotIn("luminosityLabel", devices)
        self.assertIn("No uncontrolled LED devices.", locales)
        research = (ROOT / "apps/desktop/src/lib/ResearchList.svelte").read_text(encoding="utf-8")
        self.assertIn("researchHelp", research)
        self.assertIn("<details>", research)
        self.assertIn("lighting.researchEmpty", research)
        self.assertIn("researchDevices", lighting)
        self.assertIn("lightingDevices", lighting)
        self.assertNotIn("radiatorNote", lighting)
        self.assertNotIn("researchHelp", lighting)
        self.assertNotIn("fusionSame", lighting)
        self.assertIn("researchDevices", (ROOT / "apps/desktop/src/lib/lighting.js").read_text(encoding="utf-8"))
        self.assertIn("lighting.engineMissing", locales)
        self.assertNotIn("start a native server yourself", locales)
        self.assertIn("/usr/libexec/chromaflow/OpenRGB.AppImage", (ROOT / "crates/chromaflow-core/src/openrgb_spawn.rs").read_text(encoding="utf-8"))
        self.assertIn("start_user_unit", (ROOT / "crates/chromaflow-core/src/openrgb_engine.rs").read_text(encoding="utf-8"))
        self.assertIn("chromaflow-sdk.service", (ROOT / "crates/chromaflow-core/src/openrgb_engine.rs").read_text(encoding="utf-8"))
        fetch = (ROOT / "scripts/fetch-openrgb-engine.sh").read_text(encoding="utf-8")
        self.assertIn("openrgb-engine.yaml", fetch)
        self.assertIn("sha256", fetch)
        self.assertNotIn("set_pwm", fetch)
        spawn = (ROOT / "crates/chromaflow-core/src/openrgb_spawn.rs").read_text(encoding="utf-8")
        self.assertIn("APPIMAGE_EXTRACT_AND_RUN", spawn)
        self.assertNotIn("set_pwm", spawn)
        self.assertIn("take(32)", (ROOT / "crates/chromaflow-core/src/lighting.rs").read_text(encoding="utf-8"))
        research = (ROOT / "apps/desktop/src/lib/ResearchList.svelte").read_text(encoding="utf-8")
        self.assertIn("lighting.reportDevice", research)
        self.assertIn("lighting.reportConfirm", research)
        self.assertIn("lighting.reportSent", research)
        self.assertIn("class:reported", research)
        self.assertIn("markReported", research)
        self.assertIn("submitDeviceReport", research)
        self.assertNotIn("lighting.apply", research)
        self.assertNotIn("lighting_apply", research)
        self.assertIn("1038", (ROOT / "apps/desktop/src/lib/research.js").read_text(encoding="utf-8"))
        self.assertIn("1856", (ROOT / "apps/desktop/src/lib/research.js").read_text(encoding="utf-8"))
        spawn = (ROOT / "crates/chromaflow-core/src/openrgb_spawn.rs").read_text(encoding="utf-8")
        self.assertIn("CHROMAFLOW_NO_SPAWN", spawn)
        self.assertIn("CHROMAFLOW_OPENRGB_SDK", spawn)
        self.assertIn("--startminimized", spawn)
        self.assertIn("AtomicBool", spawn)
        self.assertNotIn("QT_QPA_PLATFORM", spawn)
        self.assertNotIn("set_pwm", spawn)
        self.assertIn("flatpak", spawn)
        engine = (ROOT / "crates/chromaflow-core/src/openrgb_engine.rs").read_text(encoding="utf-8")
        self.assertIn("sha256", engine)
        self.assertIn(".part", engine)
        self.assertNotIn("set_pwm", engine)
        yaml = (ROOT / "data/openrgb-engine.yaml").read_text(encoding="utf-8")
        self.assertIn("codeberg.org", yaml)
        self.assertIn("sha256:", yaml)
        notice = (ROOT / "NOTICE").read_text(encoding="utf-8")
        self.assertIn("separate process", notice)
        adr = (ROOT / "docs/adr/0013-bundled-openrgb-appimage.md").read_text(encoding="utf-8")
        self.assertIn("AppImage", adr)
        self.assertIn("127.0.0.1", adr)
        lib = (ROOT / "apps/desktop/src/lib/lighting.js").read_text(encoding="utf-8")
        self.assertIn("fusionFallback", lib)
        self.assertIn("arenaTargets", lib)
        self.assertIn("primeTargets", lib)
        self.assertIn("sizeLabel", lib)
        self.assertIn("parse_controller", (ROOT / "crates/chromaflow-core/src/openrgb_parse.rs").read_text(encoding="utf-8"))
        locales = (ROOT / "apps/desktop/src/locales/en.json").read_text(encoding="utf-8")
        self.assertNotIn('"lighting.apply"', locales)
        self.assertNotIn("lighting.applyAria", locales)
        self.assertIn("on:commit", (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8"))
        self.assertIn('"Solid Color"', (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8"))
        self.assertIn("dispatch(\"commit\"", (ROOT / "apps/desktop/src/lib/ColorWheel.svelte").read_text(encoding="utf-8"))
        self.assertNotIn("tools-apply", devices)
        self.assertNotIn("lighting.applyEffectAria", (ROOT / "apps/desktop/src/lib/EffectList.svelte").read_text(encoding="utf-8"))
        self.assertIn("on:change", (ROOT / "apps/desktop/src/lib/EffectList.svelte").read_text(encoding="utf-8"))
        self.assertIn("lighting.r", locales)
        self.assertIn("lighting.h", locales)
        self.assertIn("lighting.suggested", locales)
        self.assertIn("lighting.recent", locales)
        self.assertIn("lighting.tip.k", locales)
        self.assertIn("lighting.tip.hex", locales)
        self.assertNotIn("lighting.fusionSame", locales)
        self.assertNotIn("aria-description", row)
        self.assertNotIn("aria-description", wheel)
        self.assertIn("tools-rest", wheel)
        self.assertIn("--chip", (ROOT / "apps/desktop/src/app.css").read_text(encoding="utf-8"))
        wheel_src = (ROOT / "apps/desktop/src/lib/ColorWheel.svelte").read_text(encoding="utf-8")
        self.assertIn("swatch-more", wheel_src)
        self.assertLess(wheel_src.find("swatch-more"), wheel_src.find("tools-rest"))
        self.assertIn("KELVIN_PRESETS", (ROOT / "apps/desktop/src/lib/ColorWheel.svelte").read_text(encoding="utf-8"))
        self.assertIn("aspect-ratio: 1", (ROOT / "apps/desktop/src/app.css").read_text(encoding="utf-8"))
        self.assertIn("q6heKeys", (ROOT / "apps/desktop/src/lib/keyboard.js").read_text(encoding="utf-8"))
        self.assertIn("tune_mode", (ROOT / "crates/chromaflow-core/src/openrgb_parse.rs").read_text(encoding="utf-8"))
        self.assertIn("CLIENT_PROTO", (ROOT / "crates/chromaflow-core/src/openrgb_proto.rs").read_text(encoding="utf-8"))
        self.assertIn("lighting.effectHint", locales)
        self.assertIn("hidHasBackend", (ROOT / "apps/desktop/src/lib/research.js").read_text(encoding="utf-8"))
        self.assertIn("048d", (ROOT / "apps/desktop/src/lib/research.js").read_text(encoding="utf-8"))
        self.assertIn("fusionUsbListed", (ROOT / "apps/desktop/src/lib/research.js").read_text(encoding="utf-8"))
        self.assertIn("hid_has_linux_backend", (ROOT / "crates/chromaflow-core/src/lighting.rs").read_text(encoding="utf-8"))
        self.assertIn("lighting.effect", locales)
        self.assertIn("lighting.speed", locales)
        self.assertIn("lighting.tip.speed", locales)
        self.assertIn("ChannelRow", (ROOT / "apps/desktop/src/lib/EffectList.svelte").read_text(encoding="utf-8"))
        self.assertIn("chan-speed", (ROOT / "apps/desktop/src/lib/EffectList.svelte").read_text(encoding="utf-8"))
        self.assertIn("export let selected", (ROOT / "apps/desktop/src/lib/EffectList.svelte").read_text(encoding="utf-8"))
        self.assertIn("on:change", (ROOT / "apps/desktop/src/lib/EffectList.svelte").read_text(encoding="utf-8"))
        self.assertIn("selected={d.mode}", (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8"))
        self.assertIn("selected={sharedMode}", (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8"))
        self.assertIn("persistSession", (ROOT / "apps/desktop/src/pages/Lighting.svelte").read_text(encoding="utf-8"))
        self.assertIn("effectSpeed", (ROOT / "apps/desktop/src/lib/ui.js").read_text(encoding="utf-8"))
        self.assertIn("chan-speed", (ROOT / "apps/desktop/src/app.css").read_text(encoding="utf-8"))
        self.assertNotIn("lighting.settings", locales)
        self.assertIn("kelvinToRgb", (ROOT / "apps/desktop/src/lib/kelvin.js").read_text(encoding="utf-8"))
        self.assertIn("kelvinHeld", (ROOT / "apps/desktop/src/lib/ColorWheel.svelte").read_text(encoding="utf-8"))
        self.assertIn("led_colors", lib)
        keys = (ROOT / "apps/desktop/src/lib/keyboard.js").read_text(encoding="utf-8")
        self.assertIn("matrixKeys", keys)
        self.assertIn("keyColSpan", keys)
        self.assertIn("packBoard", keys)
        self.assertIn("mosaicDevice", keys)
        grid = (ROOT / "apps/desktop/src/lib/LedGrid.svelte").read_text(encoding="utf-8")
        self.assertIn("colors[idx] || painted", grid)
        self.assertNotIn("|| color ||", grid)
        self.assertIn("picker-board", grid)
        devices = (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8")
        self.assertIn("layout-toggle btn-secondary", devices)
        self.assertIn("liveSwatch", devices)
        self.assertIn("boards.all", devices)
        self.assertIn("picker-flow", devices)
        self.assertIn("picker-slot", devices)
        self.assertIn("picker-slot-inner", devices)
        self.assertIn("lighting.rowDetails", devices)
        self.assertIn("busyKey", devices)
        self.assertIn("aria-busy", (ROOT / "apps/desktop/src/lib/ColorWheel.svelte").read_text(encoding="utf-8"))
        self.assertIn("busyKey={busyKey}", (ROOT / "apps/desktop/src/pages/Lighting.svelte").read_text(encoding="utf-8"))
        self.assertIn("showToast", (ROOT / "apps/desktop/src/pages/Lighting.svelte").read_text(encoding="utf-8"))
        self.assertIn("fc-toast", (ROOT / "apps/desktop/src/pages/Lighting.svelte").read_text(encoding="utf-8"))
        self.assertIn("aria-expanded", devices)
        self.assertNotIn('<span class="device-meta">{d.protocol}</span>', devices)
        self.assertIn("pollMs", (ROOT / "apps/desktop/src/App.svelte").read_text(encoding="utf-8"))
        self.assertIn("liveBoard", (ROOT / "apps/desktop/src/lib/ui.js").read_text(encoding="utf-8"))
        self.assertIn("lighting_sync", (ROOT / "apps/desktop/src/pages/Lighting.svelte").read_text(encoding="utf-8"))
        self.assertIn("mergePreview", lib)
        self.assertIn("chevron", devices)
        self.assertIn("lighting.showLayout", locales)
        self.assertIn("led_colors", (ROOT / "crates/chromaflow-core/src/types.rs").read_text(encoding="utf-8"))
        self.assertIn("read_colors", (ROOT / "crates/chromaflow-core/src/openrgb_parse.rs").read_text(encoding="utf-8"))
        self.assertIn("chan-k", (ROOT / "apps/desktop/src/lib/ColorWheel.svelte").read_text(encoding="utf-8"))
        self.assertIn("font-size: inherit", (ROOT / "apps/desktop/src/app.css").read_text(encoding="utf-8"))
        self.assertIn("EffectList", (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8"))
        self.assertIn("LedGrid", (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8"))
        self.assertIn("picker-flow", (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8"))
        self.assertIn("picker-square", (ROOT / "apps/desktop/src/lib/ColorWheel.svelte").read_text(encoding="utf-8"))
        self.assertIn("picker-tools", (ROOT / "apps/desktop/src/lib/ColorWheel.svelte").read_text(encoding="utf-8"))
        self.assertIn("tools-preview", (ROOT / "apps/desktop/src/lib/ColorWheel.svelte").read_text(encoding="utf-8"))
        css = (ROOT / "apps/desktop/src/app.css").read_text(encoding="utf-8")
        self.assertIn("3.4rem 1.3rem", css)
        self.assertIn("min-width: 0", css)
        self.assertIn("flex-wrap: wrap", (ROOT / "apps/desktop/src/app.css").read_text(encoding="utf-8"))
        self.assertIn("--picker-size", (ROOT / "apps/desktop/src/app.css").read_text(encoding="utf-8"))
        self.assertIn("picker-cubes", css)
        self.assertIn("picker-wide", css)
        self.assertIn(":not(.gauge-meter)", css)
        self.assertIn("aspect-ratio: 1", css)
        self.assertIn("border-radius: 0", css)
        self.assertIn(".chevron", css)
        self.assertIn(".gauge-spectrum", css)
        self.assertIn(".gauge-meter", css)
        self.assertIn("gauge-metric", css)
        self.assertIn("gauge-switch", css)
        self.assertIn("gauge-blue", css)
        self.assertNotIn(".gauge-spark", css)
        self.assertNotIn("writing-mode: vertical-rl", css)
        self.assertNotIn("tools-apply", (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8"))
        self.assertIn("uniqueModes", lib)
        self.assertIn("UPDATE_MODE", (ROOT / "crates/chromaflow-core/src/openrgb_proto.rs").read_text(encoding="utf-8"))
        self.assertIn("UPDATE_LED", (ROOT / "crates/chromaflow-core/src/openrgb_proto.rs").read_text(encoding="utf-8"))
        self.assertIn("set_mode", (ROOT / "crates/chromaflow-core/src/openrgb_mode.rs").read_text(encoding="utf-8"))
        self.assertNotIn("set_pwm", (ROOT / "crates/chromaflow-core/src/openrgb_mode.rs").read_text(encoding="utf-8"))
        mode_rs = (ROOT / "crates/chromaflow-core/src/openrgb_mode.rs").read_text(encoding="utf-8")
        self.assertIn("drop_session", mode_rs)
        self.assertIn("paints_color", mode_rs)
        self.assertIn("fill_motion", mode_rs)
        self.assertGreaterEqual(mode_rs.count("write_mode("), 2)
        self.assertIn("drop_session", (ROOT / "crates/chromaflow-core/src/openrgb_preview.rs").read_text(encoding="utf-8"))
        self.assertIn("pub mode:", (ROOT / "crates/chromaflow-core/src/openrgb_preview.rs").read_text(encoding="utf-8"))
        self.assertIn("fill_motion", (ROOT / "crates/chromaflow-core/src/openrgb_parse.rs").read_text(encoding="utf-8"))
        self.assertIn("drop_session", (ROOT / "crates/chromaflow-core/src/lighting_apply.rs").read_text(encoding="utf-8"))
        page = (ROOT / "apps/desktop/src/pages/Lighting.svelte").read_text(encoding="utf-8")
        tick = (ROOT / "apps/desktop/src/lib/lightingTick.js").read_text(encoding="utf-8")
        self.assertIn("pausePoll", page)
        self.assertIn("startLightingTick", page)
        self.assertIn("on:gauge", page)
        self.assertIn("GaugeMeter", (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8"))
        self.assertNotIn("GaugeMeter", page)
        self.assertIn("lighting_sync", page)
        self.assertIn("hostOpenRgbFrames", page)
        self.assertIn("stampHostFrames", page)
        self.assertIn("ui.lastMode", page)
        self.assertIn('mode || "Direct"', page)
        self.assertIn("ui.tickId", tick)
        self.assertIn("MOTION_MS = 100", tick)
        self.assertIn("GAUGE_MS = 500", tick)
        self.assertIn("METER_MS = 1000", tick)
        self.assertIn("lightActive", tick)
        self.assertIn("ui.page === \"Lighting\"", tick)
        self.assertIn("setTimeout", tick)
        self.assertNotIn("requestAnimationFrame", tick)
        self.assertNotIn("requestAnimationFrame", page)
        self.assertIn("try_lock_sdk", (ROOT / "crates/chromaflow-core/src/openrgb_proto.rs").read_text(encoding="utf-8"))
        self.assertIn("drop_probe_cache", (ROOT / "crates/chromaflow-core/src/openrgb.rs").read_text(encoding="utf-8"))
        self.assertIn("LIQUID_TTL", (ROOT / "crates/chromaflow-core/src/lighting.rs").read_text(encoding="utf-8"))
        self.assertNotIn("skipped", page)
        self.assertNotIn("/keychron/i", page)
        self.assertNotIn("(d.modes || []).includes(mode)", page)
        viz = (ROOT / "apps/desktop/src/lib/effectViz.js").read_text(encoding="utf-8")
        qmk = (ROOT / "apps/desktop/src/lib/effectQmk.js").read_text(encoding="utf-8")
        self.assertIn("layoutColors", viz)
        self.assertIn("effectFrame", viz)
        catalog = (ROOT / "apps/desktop/src/lib/effectCatalog.js").read_text(encoding="utf-8")
        self.assertIn("HOST_EFFECTS", viz)
        self.assertIn("Hardware gauges", catalog)
        self.assertIn('"CPU"', catalog)
        self.assertIn('"GPU"', catalog)
        self.assertIn('"Combined"', catalog)
        self.assertIn('"RAM"', catalog)
        self.assertIn('"Disk"', catalog)
        self.assertIn("SHARED_MODES", catalog)
        self.assertIn("deviceModes", catalog)
        self.assertNotIn("CPU temp", catalog)
        self.assertNotIn("GPU temp", catalog)
        self.assertNotIn("CPU + GPU temp", catalog)
        self.assertNotIn("RAM used", catalog)
        self.assertNotIn("Disk used", catalog)
        self.assertIn("gauge_route", qmk)
        self.assertIn("gauge_gpu", qmk)
        self.assertIn("gauge_cpu", qmk)
        self.assertIn("gauge_temp", qmk)
        self.assertIn("gaugeHex", (ROOT / "apps/desktop/src/lib/gauges.js").read_text(encoding="utf-8"))
        self.assertIn("gaugeLane", (ROOT / "apps/desktop/src/lib/gauges.js").read_text(encoding="utf-8"))
        gauges_tick = (ROOT / "apps/desktop/src/lib/gaugesTick.js").read_text(encoding="utf-8")
        self.assertIn("hardware_gauges", gauges_tick)
        self.assertIn("GAUGE_POLL_MS = 400", gauges_tick)
        self.assertNotIn("cycleOn", gauges_tick)
        self.assertNotIn("hardware_gauges", tick)
        self.assertNotIn("lastMeter", tick)
        self.assertIn("startGaugeTick", (ROOT / "apps/desktop/src/App.svelte").read_text(encoding="utf-8"))
        self.assertNotIn("histKey", page)
        self.assertNotIn("{hist}", page)
        self.assertIn("hardware_gauges", (ROOT / "apps/desktop/src-tauri/src/main.rs").read_text(encoding="utf-8"))
        self.assertIn("hardware_gauges", (ROOT / "apps/desktop/src-tauri/permissions/chromaflow.toml").read_text(encoding="utf-8"))
        smi = (ROOT / "crates/chromaflow-core/src/nvidia_smi.rs").read_text(encoding="utf-8")
        self.assertIn("nvidia-smi", smi)
        self.assertIn("from_secs(2)", smi)
        self.assertNotIn("nvidia-smi", (ROOT / "crates/chromaflow-core/src/gauges.rs").read_text(encoding="utf-8"))
        self.assertNotIn("nvidia-smi", (ROOT / "crates/chromaflow-core/src/scan.rs").read_text(encoding="utf-8"))
        self.assertNotIn("set_pwm", (ROOT / "crates/chromaflow-core/src/gauges.rs").read_text(encoding="utf-8"))
        self.assertNotIn("set_pwm", smi)
        self.assertIn("pollMs: 1000", (ROOT / "apps/desktop/src/lib/ui.js").read_text(encoding="utf-8"))
        self.assertIn("tab === \"Cooling\"", (ROOT / "apps/desktop/src/App.svelte").read_text(encoding="utf-8"))
        self.assertIn('tab === "Lighting"', (ROOT / "apps/desktop/src/App.svelte").read_text(encoding="utf-8"))
        self.assertIn("lastPaint", tick)
        self.assertIn("applyGaugeMode", page)
        self.assertIn("font-size: 1rem", css)
        self.assertIn("height: 0.9rem", css)
        self.assertIn("background: var(--fc-white)", css)
        self.assertIn("cpu_load", (ROOT / "crates/chromaflow-core/src/gauges.rs").read_text(encoding="utf-8"))
        self.assertIn("disk_c", (ROOT / "crates/chromaflow-core/src/gauges.rs").read_text(encoding="utf-8"))
        self.assertIn("ram_c", (ROOT / "crates/chromaflow-core/src/gauges.rs").read_text(encoding="utf-8"))
        meter = (ROOT / "apps/desktop/src/lib/GaugeMeter.svelte").read_text(encoding="utf-8")
        self.assertIn("picker-square", meter)
        self.assertIn("picker-card", meter)
        self.assertNotIn("gaugesApplyAll", meter)
        self.assertNotIn("gaugeHex", meter)
        self.assertIn("gauge-marker", meter)
        self.assertNotIn("SparkGraph", meter)
        self.assertIn('role="switch"', meter)
        self.assertIn("gaugePalette", meter)
        self.assertIn("lighting.gaugePalette", locales)
        self.assertIn('gaugePalette: "blue"', (ROOT / "apps/desktop/src/lib/ui.js").read_text(encoding="utf-8"))
        self.assertLess(meter.find("lighting.gaugeBlue"), meter.find("lighting.gaugeGreen"))
        self.assertIn('gaugePalette === "green"', (ROOT / "apps/desktop/src/lib/session.js").read_text(encoding="utf-8"))
        self.assertIn("gauge_palette", (ROOT / "apps/desktop/src-tauri/src/session.rs").read_text(encoding="utf-8"))
        self.assertIn('mode: "CPU"', meter)
        self.assertIn('mode: "GPU"', meter)
        self.assertIn('mode: "Combined"', meter)
        self.assertIn('mode: "RAM"', meter)
        self.assertIn('mode: "Disk"', meter)
        self.assertIn("utilization.gpu", smi)
        self.assertIn("lighting.gaugeMetric", locales)
        self.assertIn("lighting.gaugesRoute", locales)
        self.assertIn("ADR-0017", (ROOT / "docs/adr/0017-host-hardware-gauges.md").read_text(encoding="utf-8"))
        self.assertIn("lighting.tip.gaugeSpeed", locales)
        self.assertIn("lighting.gauges", locales)
        self.assertIn("hardware_gauges", gauges_tick)
        self.assertIn("scan_temps", (ROOT / "apps/desktop/src-tauri/src/main.rs").read_text(encoding="utf-8"))
        self.assertIn("DISK_TTL", (ROOT / "crates/chromaflow-core/src/gauges.rs").read_text(encoding="utf-8"))
        self.assertIn("Value::Array", (ROOT / "apps/desktop/src-tauri/src/main.rs").read_text(encoding="utf-8"))
        self.assertIn("stampHostFrames", viz)
        self.assertIn("isHostEffect", viz)
        self.assertIn("lastMode", (ROOT / "apps/desktop/src/lib/ui.js").read_text(encoding="utf-8"))
        self.assertIn("tickId", (ROOT / "apps/desktop/src/lib/ui.js").read_text(encoding="utf-8"))
        self.assertIn("classify", qmk)
        self.assertIn("paintKind", qmk)
        self.assertIn("deviceModes", (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8"))
        self.assertIn("SHARED_MODES", (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8"))
        self.assertIn("optgroup", (ROOT / "apps/desktop/src/lib/EffectList.svelte").read_text(encoding="utf-8"))
        self.assertIn("lighting.group.solid", locales)
        self.assertIn("lighting.group.rainbow", locales)
        self.assertIn("lighting.group.hardware", locales)
        preview_rs = (ROOT / "crates/chromaflow-core/src/openrgb_preview.rs").read_text(encoding="utf-8")
        self.assertIn("use_hid", preview_rs)
        self.assertIn("written_rows", preview_rs)
        self.assertIn("from_millis(400)", preview_rs)
        self.assertIn("overlay(rows, true)", preview_rs)
        self.assertIn("GET_COLOR", (ROOT / "crates/chromaflow-core/src/keychron_preview.rs").read_text(encoding="utf-8"))
        self.assertIn("hsv255", (ROOT / "crates/chromaflow-core/src/keychron_preview.rs").read_text(encoding="utf-8"))
        self.assertIn("lighting_broadcast", (ROOT / "apps/desktop/src-tauri/src/main.rs").read_text(encoding="utf-8"))
        self.assertIn("lighting_cycle", (ROOT / "apps/desktop/src-tauri/src/main.rs").read_text(encoding="utf-8"))
        self.assertIn("lighting_broadcast", (ROOT / "apps/desktop/src-tauri/permissions/chromaflow.toml").read_text(encoding="utf-8"))
        self.assertIn("lighting_cycle", (ROOT / "apps/desktop/src-tauri/permissions/chromaflow.toml").read_text(encoding="utf-8"))
        self.assertIn("lighting_sync", (ROOT / "apps/desktop/src-tauri/src/main.rs").read_text(encoding="utf-8"))
        self.assertIn("lighting_sync", (ROOT / "apps/desktop/src-tauri/permissions/chromaflow.toml").read_text(encoding="utf-8"))
        self.assertIn("update_leds_colors", (ROOT / "crates/chromaflow-core/src/openrgb_apply.rs").read_text(encoding="utf-8"))
        self.assertIn("layoutColors", (ROOT / "apps/desktop/src/lib/LedGrid.svelte").read_text(encoding="utf-8"))
        self.assertIn("layoutColors", (ROOT / "apps/desktop/src/lib/DeviceList.svelte").read_text(encoding="utf-8"))
        self.assertIn("ADR-0015", (ROOT / "docs/adr/0015-keychron-led-readback.md").read_text(encoding="utf-8"))
        self.assertIn("ADR-0016", (ROOT / "docs/adr/0016-host-direct-effects.md").read_text(encoding="utf-8"))
        self.assertIn("HOST_EFFECTS", (ROOT / "docs/adr/0016-host-direct-effects.md").read_text(encoding="utf-8"))
        adr = (ROOT / "docs/adr/0011-openrgb-sdk-write.md").read_text(encoding="utf-8")
        self.assertIn("UPDATE_LEDS", adr)
        self.assertIn("SET_CLIENT_NAME", adr)
        adr12 = (ROOT / "docs/adr/0012-arena7-hid-write.md").read_text(encoding="utf-8")
        self.assertIn("0x06", adr12)
        apply = (ROOT / "crates/chromaflow-core/src/openrgb_apply.rs").read_text(encoding="utf-8")
        self.assertIn("UPDATE_LEDS", apply)
        self.assertIn("body.len()", apply)
        self.assertIn("openrgb_dled", apply)
        self.assertNotIn("let data_size = 2u32 + 4", apply)
        self.assertNotIn("set_pwm", apply)
        core_apply = (ROOT / "crates/chromaflow-core/src/lighting_apply.rs").read_text(encoding="utf-8")
        self.assertNotIn("set_pwm", core_apply)
        self.assertIn("arena", core_apply)
        self.assertIn("prime", core_apply)
        self.assertIn("keychron", core_apply)
        self.assertIn("msi_gpu", core_apply)
        self.assertIn("keychron_apply::set_color", core_apply)
        self.assertIn("rewrite", core_apply)
        self.assertIn("set_device", core_apply)
        port = (ROOT / "crates/chromaflow-core/src/lighting_port.rs").read_text(encoding="utf-8")
        self.assertIn("aorus", port)
        arena = (ROOT / "crates/chromaflow-core/src/arena_apply.rs").read_text(encoding="utf-8")
        self.assertIn("0x06", arena)
        self.assertNotIn("set_pwm", arena)
        prime = (ROOT / "crates/chromaflow-core/src/prime_apply.rs").read_text(encoding="utf-8")
        self.assertIn("0x62", prime)
        self.assertIn("0x59", prime)
        self.assertIn("fn set_live", prime)
        self.assertIn("fn save", prime)
        self.assertNotIn("set_pwm", prime)
        adr14 = (ROOT / "docs/adr/0014-prime-neo-hid-write.md").read_text(encoding="utf-8")
        self.assertIn("0x62", adr14)
        self.assertNotIn("set_pwm", adr14)
        dled = (ROOT / "crates/chromaflow-core/src/openrgb_dled.rs").read_text(encoding="utf-8")
        self.assertIn("UPDATE_ZONE", dled)
        self.assertNotIn("set_pwm", dled)
        fusion = (ROOT / "crates/chromaflow-core/src/liquidctl_apply.rs").read_text(encoding="utf-8")
        self.assertIn("liquidctl", fusion)
        self.assertNotIn("set_pwm", fusion)


    def test_core_scans_sysfs_not_hidraw_open(self) -> None:
        text = (ROOT / "crates/chromaflow-core/src/lighting.rs").read_text(encoding="utf-8")
        self.assertIn("HID_ID", text)
        self.assertIn("CHROMAFLOW_HIDRAW_SYS", text)
        self.assertIn("CHROMAFLOW_USB_SYS", text)
        self.assertIn("scan_usb_fusion", text)
        self.assertNotIn("set_pwm", text)
        self.assertIn("usb_rgb_not_in_openrgb", (ROOT / "crates/chromaflow-core/src/gaps.rs").read_text(encoding="utf-8"))

    def test_advanced_loads_unmatched_i2c_rgb(self) -> None:
        trap = ROOT / "target" / "support-trap-lighting"
        trap.mkdir(parents=True, exist_ok=True)
        for name in ("apt-get", "modprobe", "usermod", "udevadm"):
            path = trap / name
            path.write_text(f"#!/bin/sh\necho ran >> \"{trap / (name + '.ran')}\"\nexit 42\n", encoding="utf-8")
            path.chmod(path.stat().st_mode | stat.S_IEXEC)
            (trap / f"{name}.ran").write_text("", encoding="utf-8")
        env = os.environ.copy()
        env["PATH"] = f"{trap}{os.pathsep}{env.get('PATH', '')}"
        env["CHROMAFLOW_LSPCI"] = "NVIDIA SMBus"
        env["CHROMAFLOW_LSUSB"] = "048d:5702 RGB LED Controller"
        env["CHROMAFLOW_DMI"] = "Gigabyte Technology Co., Ltd. X570S AORUS MASTER"
        env["CHROMAFLOW_LOADED_MODULES"] = "gigabyte_wmi"
        proc = subprocess.run(
            ["bash", str(ROOT / "scripts" / "install-support.sh"), "--dry-run", "--advanced"],
            check=False,
            capture_output=True,
            text=True,
            env=env,
            cwd=str(ROOT),
        )
        self.assertEqual(proc.returncode, 0, proc.stderr)
        plan = json.loads(proc.stdout)
        self.assertIn("gigabyte_wmi", plan["would_load"])
        self.assertIn("nct6775-i2c", plan["would_load"])
        self.assertNotIn("i2c-nct6775", plan["would_load"])
        self.assertIn("i2c-nvidia-gpu", plan["would_load"])
        self.assertNotIn("nouveau", plan["would_load"])
        self.assertIn("nouveau", plan["skipped_experimental"])
        self.assertTrue(any(n.startswith("linux-modules-extra") for n in plan["would_install"]))
        ids = [row["id"] for row in plan["extras"]]
        self.assertEqual(
            ids,
            [
                "it87-dkms",
                "liquidctl",
                "linux-modules-extra",
                "i2c-dev",
                "i2c-piix4",
                "it87",
                "nct6775",
                "k10temp",
                "jc42",
                "spd5118",
                "gigabyte_wmi",
                "udev",
                "group_i2c",
                "group_plugdev",
                "pwm_acl",
                "i2c-nct6775",
                "i2c-nvidia-gpu",
            ],
        )
        self.assertTrue(all("present" in row for row in plan["extras"]))
        self.assertTrue((trap / "modprobe.ran").read_text() == "")
        lib = (ROOT / "apps/desktop/src/lib/lighting.js").read_text(encoding="utf-8")
        self.assertIn("extraKernelItems", lib)
        self.assertIn("pwm_acl", lib)
        color = (ROOT / "apps/desktop/src/lib/color.js").read_text(encoding="utf-8")
        self.assertIn("hsvToHex", color)
        self.assertIn("SUGGESTED_HEX", color)
        self.assertNotIn("hslToHex", color)
        self.assertNotIn("set_pwm", color)

    def test_udev_is_per_vid_not_blanket_hidraw(self) -> None:
        import sys

        sys.path.insert(0, str(ROOT / "scripts" / "lib"))
        from chromaflow_yaml import load_index  # noqa: E402

        udev = (ROOT / "data" / "udev.yaml").read_text(encoding="utf-8")
        self.assertIn("048d", udev)
        self.assertIn("5702", udev)
        self.assertIn("3434", udev)
        self.assertIn("0b60", udev)
        self.assertIn("1038", udev)
        self.assertIn("1856", udev)
        self.assertIn("1a00", udev)
        self.assertNotIn("set_pwm", udev)
        self.assertIn("pwm-acl.sh", udev)
        rules = load_index(ROOT / "data")["udev"]
        hid = [str(r.get("line") or "") for r in rules if "hidraw" in str(r.get("line") or "")]
        self.assertTrue(hid)
        for line in hid:
            self.assertIn("ATTRS{idVendor}", line)
            self.assertIn("GROUP=\"plugdev\"", line)
        self.assertIn("openrgb_sandboxed", (ROOT / "crates/chromaflow-core/src/gaps.rs").read_text(encoding="utf-8"))
        locales = (ROOT / "apps/desktop/src/locales/en.json").read_text(encoding="utf-8")
        self.assertIn("lighting.sandbox", locales)
        self.assertNotIn("lighting.viaHint", locales)
        self.assertNotIn("lighting.noBackend", locales)

    def test_kelvin_roundtrip_and_spacebar_span(self) -> None:
        import subprocess

        kelvin = subprocess.check_output(
            [
                "node",
                "--input-type=module",
                "-e",
                "import { kelvinToRgb, rgbToKelvin, KELVIN_MIN, KELVIN_MAX } from './apps/desktop/src/lib/kelvin.js';"
                "const a = kelvinToRgb(KELVIN_MIN); const b = kelvinToRgb(KELVIN_MAX);"
                "if (Math.abs(rgbToKelvin(a.r,a.g,a.b) - KELVIN_MIN) > 5) throw new Error('min');"
                "if (Math.abs(rgbToKelvin(b.r,b.g,b.b) - KELVIN_MAX) > 5) throw new Error('max');",
            ],
            cwd=ROOT,
            text=True,
        )
        self.assertEqual(kelvin.strip(), "")
        span = subprocess.check_output(
            [
                "node",
                "--input-type=module",
                "-e",
                "import { keyColSpan, matrixKeys, q6heKeys, boardLayout, packBoard } from './apps/desktop/src/lib/keyboard.js';"
                "import { liveSwatch, mergePreview } from './apps/desktop/src/lib/lighting.js';"
                "import { layoutColors, effectFrame, preferMode, classify, hostOpenRgbFrames, stampHostFrames } from './apps/desktop/src/lib/effectViz.js';"
                "import { time8, ledPoints } from './apps/desktop/src/lib/effectQmk.js';"
                "import { rgb16, phase16, periodMs } from './apps/desktop/src/lib/color.js';"
                "import { gaugeHex, combinedTempRatio, tempRatio, combinedCelsius, laneRatio, pushSample, fmtTemp, fromHwmon, mergeGauges, gaugeLane } from './apps/desktop/src/lib/gauges.js';"
                "import { snapshot, GAUGE_POLL_MS, GAUGE_HIST } from './apps/desktop/src/lib/gaugesTick.js';"
                "import { pick } from './apps/desktop/src/lib/spark.js';"
                "import { ui } from './apps/desktop/src/lib/ui.js';"
                "import { hostCadence, frameSig, MOTION_MS, GAUGE_MS, METER_MS, PREVIEW_MS, fusionFirmware, uniformMode, wantsCycle } from './apps/desktop/src/lib/lightingTick.js';"
                "import { SHARED_MODES, deviceModes, groupedModes, HOST_EFFECTS } from './apps/desktop/src/lib/effectCatalog.js';"
                "if (SHARED_MODES.includes('Cycle Left Right')) throw new Error('shared spatial');"
                "if (deviceModes({ backend: 'liquidctl' }).includes('Cycle Left Right')) throw new Error('fusion spatial');"
                "if (!deviceModes({ backend: 'keychron' }).includes('Rainbow Moving Chevron')) throw new Error('key rainbow');"
                "if (groupedModes(SHARED_MODES).map((g) => g.id).join() !== 'solid,rainbow,hardware') throw new Error('shared groups');"
                "if (HOST_EFFECTS[0] !== 'Solid Color') throw new Error('host order');"
                "if (HOST_EFFECTS.includes('Dual Beacon')) throw new Error('dup beacon');"
                "if (!HOST_EFFECTS.includes('Rainbow Beacon')) throw new Error('beacon');"
                "if (keyColSpan('Key: Space') !== 6) throw new Error('space');"
                "if (keyColSpan('Key: Left Shift') !== 2) throw new Error('shift');"
                "const packed = matrixKeys({ grid_w: 7, grid_h: 1, grid: [0,-1,-1,-1,-1,-1,1], led_names: ['Key: Space','Key: A'] });"
                "if (!packed || packed.cells[0].span !== 1) throw new Error('packed');"
                "const k = q6heKeys({ led_names: Array.from({length: 108}, (_,i)=>'Key: '+i), name: 'Keychron Q6 HE', leds: 108, grid_w: 21, grid_h: 6 });"
                "const space = k.cells.find((c)=>c.idx===98);"
                "const enter = k.cells.find((c)=>c.idx===74);"
                "if (!space || space.span !== 25) throw new Error('q6 space '+ (space && space.span));"
                "if (!enter || enter.x + enter.span !== 60) throw new Error('enter align');"
                "const board = boardLayout({ leds: 68, led_names: Array.from({length: 68}, (_,i)=>'LED '+i) });"
                "if (!board || board.w !== 24 || board.h !== 6 || board.cells.length !== 68) throw new Error('aorus board');"
                "const wide = packBoard(177, []);"
                "if (wide.w !== 30 || wide.cells.length !== 177) throw new Error('pack wide');"
                "if (!String(liveSwatch({ led_colors: ['#ff0000','#00ff00','#0000ff'] })).includes('linear-gradient')) throw new Error('swatch');"
                "const merged = mergePreview([{ name: 'K', led_colors: ['#111111'], color: '#111111', mode: 'Direct' }], [{ name: 'K', led_colors: ['#abcdef'], color: '#abcdef', mode: 'Rainbow Wave' }]);"
                "if (merged[0].led_colors[0] !== '#abcdef') throw new Error('preview merge');"
                "if (merged[0].mode !== 'Rainbow Wave') throw new Error('preview mode');"
                "const rain = layoutColors({ mode: 'Rainbow Wave', leds: 8, led_colors: ['#00ff00','#00ff00','#00ff00','#00ff00','#00ff00','#00ff00','#00ff00','#00ff00'] });"
                "if (rain[0] !== '#00ff00') throw new Error('layout uses live pixels');"
                "if (classify('Rainbow Moving Chevron') !== 'chevron') throw new Error('classify chevron');"
                "if (classify('Rainbow Wave') !== 'cycle_lr') throw new Error('classify wave');"
                "if (classify('Hardware gauges') !== 'gauge_route') throw new Error('classify route');"
                "if (classify('CPU') !== 'gauge_cpu') throw new Error('classify cpu');"
                "if (classify('GPU') !== 'gauge_gpu') throw new Error('classify gpu');"
                "if (classify('Combined') !== 'gauge_temp') throw new Error('classify combined');"
                "if (classify('RAM') !== 'gauge_ram') throw new Error('classify ram');"
                "if (classify('Disk') !== 'gauge_disk') throw new Error('classify disk');"
                "if (classify('CPU temp') !== 'gauge_cpu') throw new Error('classify cpu legacy');"
                "if (gaugeHex(0).toLowerCase() !== '#00ff00') throw new Error('green '+gaugeHex(0));"
                "if (gaugeHex(0.5).toLowerCase() !== '#ffff00') throw new Error('yellow '+gaugeHex(0.5));"
                "if (gaugeHex(1).toLowerCase() !== '#ff0000') throw new Error('red '+gaugeHex(1));"
                "if (gaugeHex(0, 'blue').toLowerCase() !== '#0000ff') throw new Error('blue '+gaugeHex(0, 'blue'));"
                "if (gaugeHex(0.5, 'blue').toLowerCase() !== '#ff00ff') throw new Error('magenta '+gaugeHex(0.5, 'blue'));"
                "if (gaugeHex(1, 'blue').toLowerCase() !== '#ff0000') throw new Error('blue red '+gaugeHex(1, 'blue'));"
                "if (tempRatio(25) !== 0) throw new Error('temp floor');"
                "if (tempRatio(90) !== 1) throw new Error('temp ceil');"
                "if (combinedTempRatio(25, 90) !== 0.5) throw new Error('avg gpu');"
                "if (combinedTempRatio(90, null) !== 1) throw new Error('cpu only');"
                "if (combinedTempRatio(null, null) !== 0) throw new Error('empty temp');"
                "if (tempRatio(null) !== null) throw new Error('null temp');"
                "if (combinedCelsius(null, null) !== null) throw new Error('combined empty');"
                "if (combinedCelsius(53, 40) !== 46.5) throw new Error('combined avg');"
                "if (fmtTemp(null) !== '—') throw new Error('fmt temp');"
                "if (hostCadence({ 'openrgb:K': 'Rainbow Moving Chevron' }) !== MOTION_MS) throw new Error('cadence motion');"
                "if (hostCadence({ 'openrgb:K': 'Hardware gauges' }) !== GAUGE_MS) throw new Error('cadence route');"
                "if (hostCadence({ 'openrgb:K': 'Combined' }) !== GAUGE_MS) throw new Error('cadence gauge');"
                "ui.cycleOn = true;"
                "if (hostCadence({ 'openrgb:K': 'Cycle All' }) !== PREVIEW_MS) throw new Error('cadence cycle');"
                "ui.cycleOn = false;"
                "if (hostCadence({}) !== 0) throw new Error('cadence idle');"
                "if (hostCadence({ 'openrgb:K': 'Rainbow Moving Chevron' }, []) !== 0) throw new Error('cadence devices');"
                "if (fusionFirmware('Cycle All')) throw new Error('cycle is host');"
                "if (fusionFirmware('Breathing')) throw new Error('breath is host');"
                "if (!fusionFirmware('Flashing')) throw new Error('flash fw');"
                "const kb = { backend: 'keychron', name: 'Q6' };"
                "const fz = { backend: 'liquidctl', name: 'Fusion' };"
                "if (uniformMode([kb, fz], { 'keychron:Q6': 'Cycle All', 'liquidctl:Fusion': 'Cycle All' }) !== 'Cycle All') throw new Error('uniform');"
                "if (uniformMode([kb, fz], { 'keychron:Q6': 'Cycle All', 'liquidctl:Fusion': 'Direct' }) !== '') throw new Error('mixed');"
                "if (!wantsCycle({ 'keychron:Q6': 'Cycle All' })) throw new Error('wantsCycle');"
                "if (wantsCycle({ 'keychron:Q6': 'Direct' })) throw new Error('idle cycle');"
                "if (wantsCycle({ 'keychron:Q6': 'Solid Color', 'gone:Old': 'Cycle All' }, [{ backend: 'keychron', name: 'Q6' }])) throw new Error('stale cycle');"
                "if (wantsCycle({ 'gone:Old': 'Cycle All' }, [])) throw new Error('empty devices');"
                "const sigA = frameSig([{ name: 'K', colors: ['#00ff00','#00ff00'] }]);"
                "const sigB = frameSig([{ name: 'K', colors: ['#00ff00','#00ff00'] }]);"
                "if (sigA !== sigB) throw new Error('sig');"
                "const hist = pushSample(pushSample([], { cpu_c: 35, gpu_c: 35 }), { cpu_c: 90, gpu_c: 40 });"
                "if (hist.length !== 2) throw new Error('hist');"
                "if (GAUGE_POLL_MS !== 400) throw new Error('gauge poll');"
                "if (GAUGE_HIST !== 150) throw new Error('gauge hist');"
                "if (snapshot({ cpu_load: 0.4 }).cpu_load !== 0.4) throw new Error('snapshot load');"
                "const loadHist = pushSample([], { cpu_c: 40, cpu_load: 0.5, gpu_c: 50, gpu_load: 0.25 }, GAUGE_HIST);"
                "if (pick(loadHist, 'gauge:cpu', 'usage')[0] !== 0.5) throw new Error('cpu usage hist');"
                "if (laneRatio({ cpu_c: 25, cpu_load: 1 }, 'gauge_cpu', 'temp') !== 0) throw new Error('lane temp');"
                "if (laneRatio({ cpu_c: 25, cpu_load: 1 }, 'gauge_cpu', 'usage') !== 1) throw new Error('lane usage');"
                "if (laneRatio({ ram_c: 25, ram: 1 }, 'gauge_ram', 'temp') !== 0) throw new Error('ram temp');"
                "if (laneRatio({ disk_c: 90, disk: 0 }, 'gauge_disk', 'temp') !== 1) throw new Error('disk temp');"
                "if (gaugeLane({ name: 'MSI 4090 AIO' }) !== 'gpu') throw new Error('lane gpu');"
                "if (gaugeLane({ name: 'CPU AIO' }) !== 'cpu') throw new Error('lane aio');"
                "if (gaugeLane({ name: 'X570S AORUS MASTER' }) !== 'cpu') throw new Error('lane cpu');"
                "if (gaugeLane({ name: 'Keychron Q6 HE' }) !== 'combined') throw new Error('lane key');"
                "if (gaugeLane({ backend: 'arena', name: 'Arena 7' }) !== 'combined') throw new Error('lane arena');"
                "const hw = fromHwmon([{ name: 'k10temp', temps: [{ value: '49750' }, { value: '37500' }] }, { name: 'nvme', temps: [{ value: '56850' }] }]);"
                "if (Math.abs(hw.cpu_c - 49.75) > 0.01) throw new Error('fromHwmon');"
                "if (Math.abs(hw.disk_c - 56.85) > 0.01) throw new Error('fromHwmon disk');"
                "if (hw.gpu_c != null) throw new Error('no gpu');"
                "const dimm = fromHwmon([{ name: 'it8688', temps: [{ label: 'DIMM', value: '41000' }, { label: 'temp1_input', value: '80000' }] }]);"
                "if (Math.abs(dimm.ram_c - 41) > 0.01) throw new Error('dimm label');"
                "const ite = fromHwmon([{ name: 'it87952', temps: [{ label: 'temp1_input', value: '51000' }] }]);"
                "if (ite.ram_c != null) throw new Error('ite not ram');"
                "const gmerged = mergeGauges({ ram: 0.4, disk: null }, [{ name: 'k10temp', temps: [{ value: '50000' }] }]);"
                "if (gmerged.cpu_c !== 50) throw new Error('merge cpu');"
                "if (gmerged.ram !== 0.4) throw new Error('merge ram');"
                "const smiLive = mergeGauges({ gpu_c: 46, ram: 0.1, note: '' }, [{ name: 'k10temp', temps: [{ value: '50000' }] }]);"
                "if (smiLive.gpu_c !== 46) throw new Error('merge smi gpu');"
                "if (smiLive.note) throw new Error('stale gpu note '+smiLive.note);"
                "const row = { leds: 8, grid_w: 8, grid_h: 1, grid: [0,1,2,3,4,5,6,7] };"
                "ui.gauges = { cpu_c: 25, gpu_c: 25, ram: 0, disk: 1, note: '' };"
                "ui.gaugePalette = 'green';"
                "const meter = effectFrame({ ...row, mode: 'Combined' }, 0, '#0052ff', 255);"
                "if (new Set(meter).size !== 1) throw new Error('gauge solid');"
                "if (meter[0] !== gaugeHex(0)) throw new Error('gauge green');"
                "const fast = effectFrame({ ...row, mode: 'Combined' }, 8000, '#0052ff', 255);"
                "if (fast.join() !== meter.join()) throw new Error('gauge ignores speed');"
                "ui.gauges = { cpu_c: 25, gpu_c: 90, ram: 0, disk: 0, note: '' };"
                "const routedGpu = effectFrame({ ...row, name: 'MSI 4090 AIO', mode: 'Hardware gauges' }, 0, '#0052ff', 255);"
                "if (routedGpu[0] !== gaugeHex(1)) throw new Error('route gpu '+routedGpu[0]);"
                "const routedCpu = effectFrame({ ...row, name: 'X570S AORUS MASTER', mode: 'Hardware gauges' }, 0, '#0052ff', 255);"
                "if (routedCpu[0] !== gaugeHex(0)) throw new Error('route cpu '+routedCpu[0]);"
                "ui.gaugeMetric = 'usage';"
                "ui.gauges = { cpu_c: 90, gpu_c: 90, cpu_load: 0, gpu_load: 1, ram: 0, disk: 0, note: '' };"
                "const usedGpu = effectFrame({ ...row, name: 'MSI 4090 AIO', mode: 'Hardware gauges' }, 0, '#0052ff', 255);"
                "if (usedGpu[0] !== gaugeHex(1)) throw new Error('usage gpu '+usedGpu[0]);"
                "const usedCpu = effectFrame({ ...row, name: 'X570S AORUS MASTER', mode: 'Hardware gauges' }, 0, '#0052ff', 255);"
                "if (usedCpu[0] !== gaugeHex(0)) throw new Error('usage cpu '+usedCpu[0]);"
                "ui.gaugeMetric = 'temp';"
                "const routedKey = effectFrame({ ...row, name: 'Keychron Q6 HE', mode: 'Hardware gauges' }, 0, '#0052ff', 255);"
                "if (routedKey[0] !== gaugeHex(1)) throw new Error('route key '+routedKey[0]);"
                "ui.gaugePalette = 'blue';"
                "const blueKey = effectFrame({ ...row, name: 'Keychron Q6 HE', mode: 'Hardware gauges' }, 0, '#0052ff', 255);"
                "if (blueKey[0] !== gaugeHex(1, 'blue')) throw new Error('palette blue '+blueKey[0]);"
                "ui.gaugePalette = 'green';"
                "const frame = effectFrame({ ...row, mode: 'Rainbow Wave' }, 0, '#0052ff');"
                "if (new Set(frame).size < 4) throw new Error('direct frame');"
                "const chev = effectFrame({ ...row, grid_w: 4, grid_h: 2, grid: [0,1,2,3,4,5,6,7], mode: 'Rainbow Moving Chevron' }, 0, '#0052ff');"
                "const wave = effectFrame({ ...row, grid_w: 4, grid_h: 2, grid: [0,1,2,3,4,5,6,7], mode: 'Rainbow Wave' }, 0, '#0052ff');"
                "if (chev.join() === wave.join()) throw new Error('chevron vs wave');"
                "const all = effectFrame({ ...row, mode: 'Cycle All' }, 0, '#0052ff');"
                "if (new Set(all).size !== 1) throw new Error('cycle all');"
                "if (all[0] !== '#ff0000') throw new Error('cycle rgb16 start '+all[0]);"
                "const later = effectFrame({ ...row, mode: 'Cycle All' }, 120, '#0052ff', 128);"
                "if (later[0] === all[0]) throw new Error('cycle rgb16 step');"
                "if (periodMs(128) < 16000) throw new Error('period');"
                "if (rgb16(0).join() !== '255,0,0') throw new Error('rgb16 red');"
                "if (rgb16(0).join() === rgb16(4000).join()) throw new Error('rgb16 walk');"
                "if (phase16(0, 128) !== 0) throw new Error('phase0');"
                "const qpts = ledPoints({ name: 'Keychron Q6 HE', leds: 108 });"
                "if (qpts.length < 108) throw new Error('q6 pts');"
                "if (new Set(qpts.map((p) => Math.round(p.y))).size < 4) throw new Error('q6 rows');"
                "const arf = hostOpenRgbFrames([{ backend: 'arena', name: 'Arena', leds: 4 }], { 'arena:Arena': 'Rainbow Wave' }, {}, 0);"
                "if (!arf.length || new Set(arf[0].colors).size < 2) throw new Error('arena rainbow');"
                "const pushed = hostOpenRgbFrames([{ backend: 'openrgb', name: 'Aorus', leds: 4 }, { backend: 'arena', name: 'Arena', leds: 4 }], { 'openrgb:Aorus': 'Rainbow Wave' }, {}, 0);"
                "if (pushed.length !== 1 || pushed[0].name !== 'Aorus') throw new Error('host frames '+JSON.stringify(pushed));"
                "const gpu = { backend: 'openrgb', name: 'NVIDIA GeForce RTX 4090', leds: 4 };"
                "const gauged = hostOpenRgbFrames([gpu], { 'openrgb:NVIDIA GeForce RTX 4090': 'GPU' }, {}, 0);"
                "if (!gauged.length) throw new Error('gpu gauge frames');"
                "const held = hostOpenRgbFrames([gpu], { 'openrgb:NVIDIA GeForce RTX 4090': 'Direct' }, {}, 0);"
                "if (held.length) throw new Error('direct must not host-overwrite');"
                "const black = effectFrame({ ...row, mode: 'Rainbow Wave' }, 0, '#000000');"
                "if (black[0] === '#000000') throw new Error('black picker');"
                "const white = effectFrame({ ...row, mode: 'Rainbow Wave' }, 0, '#ffffff');"
                "if (new Set(white).size < 4) throw new Error('white picker');"
                "const stamped = stampHostFrames([{ name: 'K', led_colors: ['#000000'], color: '#000000' }], [{ name: 'K', colors: ['#ff00aa', '#00ffaa'] }]);"
                "if (stamped[0].led_colors[0] !== '#ff00aa') throw new Error('stamp');"
                "const direct = layoutColors({ mode: 'Direct', leds: 2, led_colors: ['#00ff00','#00ff00'] });"
                "if (direct[0] !== '#00ff00') throw new Error('direct pixels');"
                "if (preferMode('Direct', 'Rainbow Wave') !== 'Rainbow Wave') throw new Error('hint');"
                "if (time8(4096, 32) === time8(4096, 255)) throw new Error('speed');",
            ],
            cwd=ROOT,
            text=True,
        )
        self.assertEqual(span.strip(), "")
        discover = subprocess.check_output(
            [
                "node",
                "--input-type=module",
                "-e",
                "import { researchDevices, hidHasBackend, fusionUsbListed } from './apps/desktop/src/lib/research.js';"
                "const inv = { hid_rgb: ["
                "{ name: 'Gigabyte RGB Fusion 2.0 (ARGB headers)', vendor_id: '048d', product_id: '5702', readable: true },"
                "{ name: 'Generic CS201', hid_name: 'Generic CS201', vendor_id: '0020', product_id: '0b21', readable: true }"
                "], liquidctl_devices: ['Gigabyte RGB Fusion 2.0 5702 Controller'],"
                "openrgb: { controllers: [{ name: 'X570S AORUS MASTER' }] } };"
                "if (!hidHasBackend(inv.hid_rgb[0], inv)) throw new Error('fusion');"
                "if (!fusionUsbListed(inv)) throw new Error('listed');"
                "if (researchDevices(inv).length) throw new Error('leftover '+JSON.stringify(researchDevices(inv)));"
                "const k = { name: 'Q6 HE', vendor_id: '3434', product_id: '0b60', readable: true };"
                "if (!hidHasBackend(k, { openrgb: { controllers: [{ name: 'Q6 HE' }] } })) throw new Error('q6');"
                "if (!hidHasBackend(k, { openrgb: { controllers: [{ name: 'Aorus' }] } })) throw new Error('native');",
            ],
            cwd=ROOT,
            text=True,
        )
        self.assertEqual(discover.strip(), "")

    def test_native_lighting_index_and_port(self) -> None:
        csv = (ROOT / "data/openrgb-device-index.csv").read_text(encoding="utf-8")
        self.assertIn("OpenRGB detector identity", csv)
        self.assertIn("GPL-2.0-or-later", csv)
        self.assertIn("Keychron RGB QMK/ZMK Keyboard", csv)
        self.assertIn("3434", csv)
        self.assertIn("MSI GeForce RTX 4090 Suprim Liquid X", csv)
        self.assertIn("I2C", csv)
        self.assertNotIn("set_pwm", csv)
        overlay = (ROOT / "data/native-lighting.yaml").read_text(encoding="utf-8")
        self.assertIn("3434:0b60", overlay)
        self.assertIn("status: native", overlay)
        self.assertIn("1038:1a00", overlay)
        self.assertIn("status: extra", overlay)
        port = (ROOT / "crates/chromaflow-core/src/lighting_port.rs").read_text(encoding="utf-8")
        self.assertIn("CHROMAFLOW_NATIVE_RGB", port)
        self.assertIn("keychron", port)
        self.assertNotIn("set_pwm", port)
        self.assertNotIn("#include", port)
        apply_k = (ROOT / "crates/chromaflow-core/src/keychron_apply.rs").read_text(encoding="utf-8")
        self.assertIn("SET_COLOR", apply_k)
        self.assertIn("via_direct_packets", apply_k)
        self.assertIn("via_solid_packets", apply_k)
        self.assertIn("via_effect_get", apply_k)
        self.assertIn("first_hsv", apply_k)
        self.assertIn("poll_effect", apply_k)
        self.assertIn("hsv_close", apply_k)
        self.assertNotIn("cols.iter().all", apply_k)
        self.assertIn("const SET_BATCH: u8 = 9", apply_k)
        self.assertIn("set_zones", (ROOT / "crates/chromaflow-core/src/arena_apply.rs").read_text(encoding="utf-8"))
        fusion_rs = (ROOT / "crates/chromaflow-core/src/liquidctl_apply.rs").read_text(encoding="utf-8")
        self.assertIn("set_fusion_sync", fusion_rs)
        self.assertIn("set_device", fusion_rs)
        self.assertIn("hid_kind", fusion_rs)
        self.assertIn('"soft"', fusion_rs)
        self.assertIn("let _ = set_on", fusion_rs)
        self.assertIn("led6", fusion_rs)
        self.assertNotIn('"color-cycle"', fusion_rs)
        self.assertIn("fusion-hid.py", fusion_rs)
        hid_py = (ROOT / "scripts/fusion-hid.py").read_text(encoding="utf-8")
        self.assertIn("digital", hid_py)
        self.assertIn("uniform", hid_py)
        self.assertIn("soft", hid_py)
        self.assertIn("serve", hid_py)
        self.assertIn("digital(dev, rgb)", hid_py)
        self.assertLess(hid_py.find("digital(dev, rgb)"), hid_py.find("analog(dev, rgb)"))
        self.assertIn("0xCC", hid_py)
        self.assertIn("0x28", hid_py)
        self.assertNotIn("set_pwm", hid_py)
        fusion_hid = (ROOT / "crates/chromaflow-core/src/fusion_hid.rs").read_text(encoding="utf-8")
        self.assertIn("serve", fusion_hid)
        self.assertIn("recv_timeout", fusion_hid)
        self.assertIn('"-u"', fusion_hid)
        page = (ROOT / "apps/desktop/src/pages/Lighting.svelte").read_text(encoding="utf-8")
        tick = (ROOT / "apps/desktop/src/lib/lightingTick.js").read_text(encoding="utf-8")
        self.assertIn("pushHidNow", page)
        self.assertIn("wantsCycle(lastMode, devices)", page)
        self.assertIn("cycleWant", page)
        self.assertNotIn("persistSession(tab, true)", (ROOT / "apps/desktop/src/App.svelte").read_text(encoding="utf-8"))
        self.assertIn("persistSession(name)", (ROOT / "apps/desktop/src/App.svelte").read_text(encoding="utf-8"))
        self.assertIn("ui.page === \"Lighting\"", page)
        self.assertIn("setHostCycle(false", page)
        self.assertIn("pushFusionNow", page)
        self.assertIn("lighting_broadcast", page)
        self.assertIn("setHostCycle", page)
        self.assertIn("lighting_cycle", tick)
        self.assertIn("wantsCycle", tick)
        self.assertIn("ui.cycleOn", tick)
        self.assertIn("if (ui.cycleOn) return PREVIEW_MS", tick)
        self.assertIn("lastFusion", tick)
        self.assertIn('!== "liquidctl"', tick)
        self.assertIn("PREVIEW_MS = 32", tick)
        self.assertIn("bumpPaint", page)
        self.assertIn("pushHidNow", tick)
        self.assertIn("fusionFirmware", tick)
        self.assertIn("msi_gpu", tick)
        self.assertIn("lighting_broadcast", tick)
        self.assertIn("uniformMode", tick)
        self.assertIn('device: "sync"', tick)
        self.assertIn("paintGen", tick)
        self.assertIn('name === "Combined"', page)
        preview_k = (ROOT / "crates/chromaflow-core/src/keychron_preview.rs").read_text(
            encoding="utf-8"
        )
        self.assertIn("const SET_COLOR: u8 = 0x0a", preview_k)
        self.assertIn("SET_TYPE", apply_k)
        self.assertIn("TYPE_SOLID", apply_k)
        self.assertIn("const TYPE_SOLID: u8 = 0", apply_k)
        self.assertIn("SET_BATCH", apply_k)
        self.assertIn("set_hue", apply_k)
        self.assertIn("set_fill", apply_k)
        self.assertIn("poll_hue", apply_k)
        cycle_rs = (ROOT / "crates/chromaflow-core/src/lighting_cycle.rs").read_text(encoding="utf-8")
        self.assertIn("hue_at", cycle_rs)
        self.assertIn("rgb16", cycle_rs)
        self.assertIn("period_ms", cycle_rs)
        self.assertIn("current_hue", cycle_rs)
        self.assertIn("run_lamps", cycle_rs)
        self.assertNotIn("fusion_hid", cycle_rs)
        self.assertIn("set_fill", cycle_rs)
        self.assertIn("prime_apply::set_live", cycle_rs)
        self.assertIn("prime_apply::save", cycle_rs)
        self.assertIn("gpu_apply::set_snap", cycle_rs)
        self.assertIn("GPU_MS", cycle_rs)
        self.assertIn("500", cycle_rs)
        self.assertIn("run_gpu", cycle_rs)
        self.assertNotIn("set_rainbow", cycle_rs)
        self.assertNotIn("thread::scope", cycle_rs)
        self.assertIn("set_running", cycle_rs)
        self.assertNotIn("set_hue", cycle_rs)
        self.assertNotIn("set_pwm", cycle_rs)
        cli = (ROOT / "crates/chromaflow-cli/src/main.rs").read_text(encoding="utf-8")
        self.assertIn("--cycle", cli)
        self.assertIn("print_keychron_poll", cli)
        self.assertIn("current_hue", cli)
        self.assertIn("with_via_lock", apply_k)
        self.assertIn("0x07", apply_k)
        self.assertNotIn("RGBController", apply_k)
        self.assertNotIn("set_pwm", apply_k)
        spec = (ROOT / "docs/features/native-lighting.md").read_text(encoding="utf-8")
        self.assertIn("ADR-0020", spec)
        adr = (ROOT / "docs/adr/0020-native-lighting-ports.md").read_text(encoding="utf-8")
        self.assertIn("identity", adr)
        lib = (ROOT / "apps/desktop/src/lib/lighting.js").read_text(encoding="utf-8")
        self.assertIn("keychronTargets", lib)
        self.assertIn("Motherboard Fusion", lib)
        self.assertIn("CPU AIO", lib)
        self.assertIn("product_id === \"0b60\" ? 108", lib)
        self.assertIn("gpuTargets", lib)
        spawn = (ROOT / "crates/chromaflow-core/src/openrgb_spawn.rs").read_text(encoding="utf-8")
        self.assertIn("CHROMAFLOW_OPENRGB_SDK", spawn)
        self.assertIn("sdk_opt_in", spawn)
        gpu = (ROOT / "crates/chromaflow-core/src/gpu_apply.rs").read_text(encoding="utf-8")
        self.assertIn("0x68", gpu)
        self.assertIn("1462", gpu)
        self.assertIn("i2cset", gpu)
        self.assertIn("set_snap", gpu)
        self.assertIn("snap_pairs", gpu)
        self.assertIn("write_next", gpu)
        self.assertIn("prefer_first", gpu)
        self.assertIn("adapter 1 at", gpu)
        self.assertIn("from_millis(20)", gpu)
        self.assertIn("0x2e", gpu)
        self.assertIn("0x08", gpu)
        self.assertIn("0x13", gpu)
        self.assertIn("0x27", gpu)
        self.assertNotIn("i2ctransfer", gpu)
        self.assertNotIn("skip_dummy", gpu)
        self.assertNotIn("rgb_matches", gpu)
        self.assertNotIn("i2cdump", gpu)
        self.assertNotIn("set_pwm", gpu)
        self.assertNotIn("#include", gpu)
        adr21 = (ROOT / "docs/adr/0021-native-lighting-without-openrgb.md").read_text(encoding="utf-8")
        self.assertIn("CHROMAFLOW_OPENRGB_SDK", adr21)
        self.assertIn("048d:5702", overlay)
        self.assertIn("backend: liquidctl", overlay)
        self.assertIn("msi_gpu", overlay)
        fetch = (ROOT / "scripts/fetch-openrgb-device-index.sh").read_text(encoding="utf-8")
        self.assertIn("openrgb-device-index.csv", fetch)
        self.assertNotIn("set_pwm", fetch)
        listed = subprocess.check_output(
            [
                "node",
                "--input-type=module",
                "-e",
                "import { lightingDevices } from './apps/desktop/src/lib/lighting.js';"
                "const rows = lightingDevices({ hid_rgb: ["
                "{ name: 'Keychron Q6 HE', vendor_id: '3434', product_id: '0b60', readable: true }"
                "], openrgb: { controllers: [{ name: 'Keychron Q6 HE', leds: 108 }] } });"
                "if (rows.filter((d) => /keychron|q6/i.test(d.name)).length !== 1) throw new Error('dup '+JSON.stringify(rows));"
                "if (rows[0] && rows.find((d) => d.backend === 'keychron') == null) throw new Error('backend');"
                "const fusion = lightingDevices({ liquidctl_devices: ['Gigabyte RGB Fusion 2.0 5702 Controller'], hid_rgb: [], openrgb: { controllers: [] } });"
                "if (fusion.filter((d) => d.backend === 'liquidctl').length !== 2) throw new Error('fusion split');"
                "if (!fusion.some((d) => d.name === 'CPU AIO')) throw new Error('aio');"
                "if (!fusion.some((d) => d.name === 'Motherboard Fusion')) throw new Error('board');"
                "const hidOnly = lightingDevices({ liquidctl_devices: [], hid_rgb: ["
                "{ name: 'ITE', vendor_id: '048d', product_id: '5702', readable: true }"
                "], openrgb: { controllers: [] } });"
                "if (hidOnly.filter((d) => d.backend === 'liquidctl').length !== 2) throw new Error('hid fusion');",
            ],
            cwd=ROOT,
            text=True,
        )
        self.assertEqual(listed.strip(), "")


if __name__ == "__main__":
    unittest.main()
