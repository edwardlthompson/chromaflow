<script>
  import { onMount } from "svelte";
  import {
    lightingDevices,
    mergePreview,
    normalizeHex,
    deviceKey,
    researchDevices,
  } from "../lib/lighting.js";
  import { preferMode, hostOpenRgbFrames, stampHostFrames } from "../lib/effectViz.js";
  import { startLightingTick } from "../lib/lightingTick.js";
  import { gaugeLane, mergeGauges, pushSample } from "../lib/gauges.js";
  import DeviceList from "../lib/DeviceList.svelte";
  import ResearchList from "../lib/ResearchList.svelte";
  import { isTauri, invoke } from "../lib/tauri.js";
  import { loadSession, persistSession } from "../lib/session.js";
  import { ui } from "../lib/ui.js";
  import t from "../locales/en.json";

  export let inventory;

  let error = "";
  let busy = false;
  let applyMsg = "";
  let lastColor = { ...ui.lastColor };
  let lastMode = { ...ui.lastMode };
  let preview = [];
  let gauges = ui.gauges;
  let hist = ui.gaugeHist || [];

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
  let histKey = "";
  $: {
    const v = viewGauges;
    const key = `${v.cpu_c}|${v.gpu_c}|${v.ram}|${v.disk}`;
    if (key !== histKey) {
      histKey = key;
      hist = pushSample(hist, v);
    }
  }

  function remember() {
    persistSession(ui.page);
  }

  onMount(() => {
    loadSession().then((s) => {
      lastMode = { ...s.lastMode };
      lastColor = { ...s.lastColor };
    });
    return startLightingTick({
      getDevices: () => devices,
      getPreview: () => preview,
      setPreview: (rows) => {
        preview = rows;
      },
      setGauges: (next, nextHist) => {
        if (ui.page !== "Lighting") return;
        gauges = next;
        hist = nextHist || [];
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
    if (mode) lastMode = { ...lastMode, [key]: mode };
    ui.lastColor = lastColor;
    ui.lastMode = lastMode;
    remember();
  }

  function pushHostNow() {
    const frames = hostOpenRgbFrames(devices, ui.lastMode, ui.lastColor, performance.now(), ui.effectSpeed);
    if (frames.length) preview = stampHostFrames(preview, frames);
    if (!isTauri() || !frames.length) return;
    invoke("lighting_sync", { frames }).catch(() => {});
  }

  async function applyColor(backend, device, nextHex) {
    busy = true;
    ui.pausePoll = true;
    error = "";
    applyMsg = t["lighting.applying"];
    try {
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
    const key = `${backend}:${device}`;
    const use = normalizeHex(nextHex);
    if (use) {
      lastColor = { ...lastColor, [key]: use };
      ui.lastColor = lastColor;
    }
    lastMode = { ...lastMode, [key]: mode };
    ui.lastMode = lastMode;
    remember();
    applyMsg = isTauri() ? t["lighting.applyOk"] : t["lighting.applyNeedGui"];
    pushHostNow();
  }

  async function applyLed(backend, device, led, nextHex) {
    busy = true;
    ui.pausePoll = true;
    error = "";
    applyMsg = t["lighting.applying"];
    try {
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
    ui.pausePoll = true;
    error = "";
    applyMsg = t["lighting.applying"];
    try {
      if (!devices.length) {
        applyMsg = t["lighting.empty"];
        return;
      }
      const notes = [];
      for (const d of devices) {
        await applyOne(d.backend, d.name, nextHex);
        notes.push(applyMsg);
      }
      applyMsg = notes.filter(Boolean).join(" · ");
    } catch (err) {
      error = String(err);
      applyMsg = "";
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
      if (d.backend !== "openrgb" && d.backend !== "arena" && d.backend !== "prime" && d.backend !== "liquidctl") {
        continue;
      }
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
    if (n) pushHostNow();
  }

  async function applyGaugeMode(mode) {
    const name = String(mode || "");
    if (name === "Hardware gauges" || name === "RAM" || name === "Disk") {
      return applyModeAll(name);
    }
    const want = name === "GPU" ? "gpu" : name === "CPU" ? "cpu" : name === "Combined" ? "combined" : "";
    if (!want) return applyModeAll(name);
    const next = { ...lastMode };
    let n = 0;
    for (const d of devices) {
      if (d.backend !== "openrgb" && d.backend !== "arena" && d.backend !== "prime" && d.backend !== "liquidctl") {
        continue;
      }
      if (gaugeLane(d) !== want) continue;
      next[deviceKey(d)] = name;
      n += 1;
    }
    lastMode = next;
    ui.lastMode = next;
    remember();
    applyMsg = n ? (isTauri() ? t["lighting.applyOk"] : t["lighting.applyNeedGui"]) : t["lighting.noEffects"];
    if (n) pushHostNow();
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
  {:else if orgb.status !== "reachable"}
    <p>{t["lighting.none"]}</p>
  {:else}
    <p>{t["lighting.empty"]}</p>
  {/if}
</div>

<div class="card">
  <h2>{t["lighting.research"]}</h2>
  <ResearchList devices={research} kernel={(inventory && inventory.kernel_release) || ""} {busy} />
</div>
