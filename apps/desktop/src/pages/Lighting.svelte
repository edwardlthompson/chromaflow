<script>
  import { onMount } from "svelte";
  import {
    lightingDevices,
    mergePreview,
    normalizeHex,
    deviceKey,
    researchDevices,
  } from "../lib/lighting.js";
  import { preferMode, hostOpenRgbFrames, stampHostFrames, classify } from "../lib/effectViz.js";
  import { startLightingTick, pushHidNow, pushFusionNow, bumpPaint, uniformMode, sharedHex, setHostCycle, wantsCycle } from "../lib/lightingTick.js";
  import { gaugeLane, mergeGauges } from "../lib/gauges.js";
  import DeviceList from "../lib/DeviceList.svelte";
  import ResearchList from "../lib/ResearchList.svelte";
  import { isTauri, invoke } from "../lib/tauri.js";
  import { loadSession, persistSession } from "../lib/session.js";
  import { ui } from "../lib/ui.js";
  import t from "../locales/en.json";

  export let inventory;
  export let gauges = {};

  let error = "";
  let busy = false;
  let applyMsg = "";
  let lastColor = { ...ui.lastColor };
  let lastMode = { ...ui.lastMode };
  let preview = [];

  $: orgb = (inventory && inventory.openrgb) || { status: "unreachable", detail: "", controllers: [] };
  $: devices = mergePreview(
    lightingDevices(inventory).map((d) => {
      const key = deviceKey(d);
      return { ...d, color: normalizeHex(d.color) || lastColor[key] || "" };
    }),
    preview,
  ).map((d) => ({
    ...d,
    mode: preferMode(d.mode, lastMode[deviceKey(d)]),
  }));
  $: research = researchDevices(inventory);
  $: viewGauges = mergeGauges(gauges, inventory && inventory.hwmon);

  function remember() {
    persistSession(ui.page);
  }

  onMount(() => {
    loadSession().then((s) => {
      lastMode = { ...s.lastMode };
      lastColor = { ...s.lastColor };
      if (wantsCycle(lastMode)) setHostCycle(true, invoke, isTauri);
    });
    return startLightingTick({
      getDevices: () => devices,
      getPreview: () => preview,
      setPreview: (rows) => {
        preview = rows;
      },
      invoke,
      isTauri,
    });
  });

  async function applyOne(backend, device, nextHex, mode, led) {
    const use = normalizeHex(nextHex);
    if (!use) {
      applyMsg = t["lighting.applyNeedColor"];
      return;
    }
    if (!isTauri()) {
      applyMsg = t["lighting.applyNeedGui"];
      return;
    }
    const res = await invoke("lighting_apply", {
      backend,
      device,
      color: use,
      mode: mode || null,
      led: led == null ? null : led,
    });
    applyMsg = (res && res.detail) || t["lighting.applyOk"];
    const key = `${backend}:${device}`;
    lastColor = { ...lastColor, [key]: use };
    lastMode = { ...lastMode, [key]: mode || "Direct" };
    ui.lastColor = lastColor;
    ui.lastMode = lastMode;
    remember();
  }

  function hostOk(d) {
    return ["openrgb", "arena", "prime", "liquidctl", "keychron", "msi_gpu"].includes(d.backend);
  }

  function pushHostNow(now) {
    const t = now == null ? performance.now() : now;
    const frames = hostOpenRgbFrames(devices, ui.lastMode, ui.lastColor, t, ui.effectSpeed);
    if (frames.length) preview = stampHostFrames(preview, frames);
    if (!isTauri() || !frames.length) return;
    invoke("lighting_sync", { frames }).catch(() => {});
  }

  async function applyColor(backend, device, nextHex) {
    busy = true;
    error = "";
    applyMsg = t["lighting.applying"];
    try {
      await bumpPaint();
      setHostCycle(false, invoke, isTauri);
      await applyOne(backend, device, nextHex);
    } catch (err) {
      error = String(err);
      applyMsg = "";
    } finally {
      busy = false;
      ui.pausePoll = false;
    }
  }

  async function applyMode(backend, device, mode, nextHex) {
    await bumpPaint();
    try {
      const key = `${backend}:${device}`;
      const use = normalizeHex(nextHex);
      if (use) {
        lastColor = { ...lastColor, [key]: use };
        ui.lastColor = lastColor;
      }
      lastMode = { ...lastMode, [key]: mode };
      ui.lastMode = lastMode;
      remember();
      if (classify(mode) === "cycle_all") {
        setHostCycle(true, invoke, isTauri);
        applyMsg = isTauri() ? t["lighting.applyOk"] : t["lighting.applyNeedGui"];
        return;
      }
      if (!wantsCycle(lastMode)) setHostCycle(false, invoke, isTauri);
      applyMsg = isTauri() ? t["lighting.applyOk"] : t["lighting.applyNeedGui"];
      const t0 = performance.now();
      pushHostNow(t0);
      if (isTauri()) {
        pushFusionNow(devices, lastMode, lastColor, invoke);
        pushHidNow(devices, lastMode, lastColor, invoke, t0);
      }
    } finally {
      ui.pausePoll = false;
    }
  }

  async function applyLed(backend, device, led, nextHex) {
    busy = true;
    error = "";
    applyMsg = t["lighting.applying"];
    try {
      await bumpPaint();
      setHostCycle(false, invoke, isTauri);
      await applyOne(backend, device, nextHex, null, led);
    } catch (err) {
      error = String(err);
      applyMsg = "";
    } finally {
      busy = false;
      ui.pausePoll = false;
    }
  }

  async function applyAll(nextHex) {
    busy = true;
    error = "";
    applyMsg = t["lighting.applying"];
    try {
      if (!devices.length) {
        applyMsg = t["lighting.empty"];
        return;
      }
      const use = normalizeHex(nextHex);
      if (!use) {
        applyMsg = t["lighting.applyNeedColor"];
        return;
      }
      await bumpPaint();
      const colors = { ...lastColor };
      const modes = { ...lastMode };
      for (const d of devices) {
        const key = `${d.backend}:${d.name}`;
        colors[key] = use;
        modes[key] = "Direct";
      }
      lastColor = colors;
      lastMode = modes;
      ui.lastColor = colors;
      ui.lastMode = modes;
      remember();
      setHostCycle(false, invoke, isTauri);
      if (!isTauri()) {
        applyMsg = t["lighting.applyNeedGui"];
        return;
      }
      const res = await invoke("lighting_broadcast", { color: use, mode: "Solid Color" });
      applyMsg = (res && res.detail) || t["lighting.applyOk"];
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
      ui.pausePoll = false;
    }
  }

  async function applyModeAll(mode, nextHex) {
    const use = normalizeHex(nextHex);
    const next = { ...lastMode };
    const colors = { ...lastColor };
    let n = 0;
    for (const d of devices) {
      if (!hostOk(d)) continue;
      const key = deviceKey(d);
      next[key] = mode;
      if (use) colors[key] = use;
      n += 1;
    }
    lastMode = next;
    lastColor = colors;
    ui.lastMode = next;
    ui.lastColor = colors;
    remember();
    applyMsg = n ? (isTauri() ? t["lighting.applyOk"] : t["lighting.applyNeedGui"]) : t["lighting.noEffects"];
    if (!n) return;
    await bumpPaint();
    try {
      if (classify(mode) === "cycle_all") {
        setHostCycle(true, invoke, isTauri);
        return;
      }
      setHostCycle(false, invoke, isTauri);
      const t0 = performance.now();
      const hex = sharedHex(devices, lastMode, lastColor, t0);
      if (isTauri() && uniformMode(devices, lastMode) && hex) {
        preview = stampHostFrames(
          preview,
          devices.map((d) => ({ name: d.name, colors: [hex] })),
        );
        invoke("lighting_broadcast", { color: hex, mode }).catch(() => {});
      } else {
        pushHostNow(t0);
        if (isTauri()) {
          pushFusionNow(devices, lastMode, lastColor, invoke);
          pushHidNow(devices, lastMode, lastColor, invoke, t0);
        }
      }
    } finally {
      ui.pausePoll = false;
    }
  }

  async function applyGaugeMode(mode) {
    const name = String(mode || "");
    if (name === "Hardware gauges" || name === "RAM" || name === "Disk" || name === "Combined") {
      return applyModeAll(name);
    }
    const want = name === "GPU" ? "gpu" : name === "CPU" ? "cpu" : name === "Combined" ? "combined" : "";
    if (!want) return applyModeAll(name);
    const next = { ...lastMode };
    let n = 0;
    for (const d of devices) {
      if (!hostOk(d)) continue;
      if (gaugeLane(d) !== want) continue;
      next[deviceKey(d)] = name;
      n += 1;
    }
    lastMode = next;
    ui.lastMode = next;
    remember();
    applyMsg = n ? (isTauri() ? t["lighting.applyOk"] : t["lighting.applyNeedGui"]) : t["lighting.noEffects"];
    if (n) {
      if (!wantsCycle(lastMode)) setHostCycle(false, invoke, isTauri);
      const t0 = performance.now();
      pushHostNow(t0);
      if (isTauri()) pushHidNow(devices, lastMode, lastColor, invoke, t0);
    }
  }

  async function installEngine() {
    busy = true;
    error = "";
    applyMsg = t["lighting.installEngine"];
    try {
      if (!isTauri()) {
        applyMsg = t["lighting.applyNeedGui"];
        return;
      }
      const res = await invoke("lighting_engine_install");
      applyMsg = (res && res.path) || t["lighting.engineInstalled"];
    } catch (err) {
      error = String(err);
      applyMsg = "";
    } finally {
      busy = false;
    }
  }
</script>

<h1>{t["lighting.title"]}</h1>
<div class="card">
  <h2>{t["lighting.devices"]}</h2>
  {#if orgb.sandboxed}
    <p class="banner" role="alert">{t["lighting.sandbox"]}</p>
  {/if}
  {#if orgb.engine_missing}
    <p class="banner" role="alert">{t["lighting.engineMissing"]}</p>
    <p>
      <button type="button" on:click={installEngine} disabled={busy}>{t["lighting.installEngine"]}</button>
    </p>
  {/if}
  <p class="status" role="status">{applyMsg}{error ? ` ${error}` : ""}</p>
  {#if devices.length}
    <DeviceList
      {devices}
      {busy}
      gauges={viewGauges}
      on:apply={(ev) => applyColor(ev.detail.backend, ev.detail.device, ev.detail.color)}
      on:applyAll={(ev) => applyAll(ev.detail.color)}
      on:mode={(ev) => applyMode(ev.detail.backend, ev.detail.device, ev.detail.mode, ev.detail.color)}
      on:modeAll={(ev) => applyModeAll(ev.detail.mode, ev.detail.color)}
      on:gauge={(ev) => applyGaugeMode(ev.detail.mode)}
      on:led={(ev) => applyLed(ev.detail.backend, ev.detail.device, ev.detail.led, ev.detail.color)}
    />
  {:else}
    <p>{t["lighting.none"]}</p>
  {/if}
</div>

<div class="card">
  <h2>{t["lighting.research"]}</h2>
  <ResearchList devices={research} kernel={(inventory && inventory.kernel_release) || ""} {busy} />
</div>
