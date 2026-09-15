<script>
  import { createEventDispatcher } from "svelte";
  import ColorWheel from "./ColorWheel.svelte";
  import GaugeMeter from "./GaugeMeter.svelte";
  import EffectList from "./EffectList.svelte";
  import LedGrid from "./LedGrid.svelte";
  import { deviceKey, hasLedLayout, liveSwatch, normalizeHex, sizeLabel } from "./lighting.js";
  import { SHARED_MODES, deviceModes } from "./effectCatalog.js";
  import { layoutColors } from "./effectViz.js";
  import { mosaicDevice } from "./keyboard.js";
  import { ui } from "./ui.js";
  import t from "../locales/en.json";

  export let devices = [];
  export let busy = false;
  export let busyKey = "";
  export let gauges = {};

  const dispatch = createEventDispatcher();
  let openId = "all";
  let allHex = "#0052ff";
  let rowHex = "#0052ff";
  let boards = {};
  let protoOpen = {};

  $: totalLeds = devices.reduce((n, d) => n + (Number(d.leds) || 0), 0);
  $: allSize = totalLeds ? `${totalLeds} LEDs` : t["lighting.sizeUnknown"];
  $: allModes = SHARED_MODES;
  $: sharedMode = devices.length && devices.every((d) => d.mode === devices[0].mode) ? devices[0].mode : "";
  $: allBoard = mosaicDevice(devices.map((d) => ({ ...d, led_colors: layoutColors(d) })));
  $: ui.liveBoard =
    (openId === "all" && Boolean(boards.all)) ||
    devices.some((d) => openId === deviceKey(d) && Boolean(boards[deviceKey(d)]));

  function allLive() {
    const cols = devices.flatMap((d) => layoutColors(d));
    return liveSwatch({ led_colors: cols, color: allHex });
  }

  function toggle(id) {
    boards = { ...boards, [id]: !boards[id] };
  }

  function toggleProto(id) {
    protoOpen = { ...protoOpen, [id]: !protoOpen[id] };
  }

  function openAll() {
    openId = "all";
  }

  function openDevice(d) {
    openId = deviceKey(d);
    rowHex = normalizeHex(d.color) || normalizeHex((d.led_colors || [])[0]) || rowHex;
  }

  function commitAll(ev) {
    const hex = (ev && ev.detail && ev.detail.hex) || allHex;
    dispatch("applyAll", { color: hex });
  }

  function commitDevice(d, ev) {
    const hex = (ev && ev.detail && ev.detail.hex) || rowHex;
    dispatch("apply", { backend: d.backend, device: d.name, color: hex, mode: "Solid Color" });
  }
</script>

<ul class="devices">
  <li>
    <div class="device-row">
      <button
        type="button"
        class="swatch"
        style="background:{allLive()}"
        aria-label={t["lighting.openPicker"]}
        aria-pressed={openId === "all"}
        on:click={openAll}
      ></button>
      <strong>{t["lighting.allDevices"]}</strong>
      <span class="device-meta">{allSize}</span>
    </div>
    {#if openId === "all"}
      <div class="picker-slot is-open">
        <div class="picker-slot-inner">
      <div class="picker-flow picker-wide">
        <ColorWheel bind:hex={allHex} wheelLabel={t["lighting.color"]} busy={busy || busyKey === "all"} on:commit={commitAll}>
          <EffectList modes={allModes} selected={sharedMode} hex={allHex} busy={busy || busyKey === "all"} on:mode={(ev) => dispatch("modeAll", ev.detail)} />
        </ColorWheel>
        <GaugeMeter {gauges} on:apply={(ev) => dispatch("gauge", ev.detail)} />
        {#if hasLedLayout(allBoard)}
          <button
            type="button"
            class="layout-toggle btn-secondary"
            aria-pressed={Boolean(boards.all)}
            aria-expanded={Boolean(boards.all)}
            aria-label={boards.all ? t["lighting.hideLayout"] : t["lighting.showLayout"]}
            on:click={() => toggle("all")}
          ><span class="chevron"></span></button>
          {#if boards.all}
            <LedGrid device={allBoard} color={allHex} {busy} interactive={false} />
          {/if}
        {/if}
      </div>
        </div>
      </div>
    {/if}
  </li>
  {#each devices as d}
    <li>
      <div class="device-row">
        <button
          type="button"
          class="swatch"
          style="background:{liveSwatch({ ...d, led_colors: layoutColors(d) })}"
          aria-label="{d.name} {t['lighting.openPicker']}"
          aria-pressed={openId === deviceKey(d)}
          on:click={() => openDevice(d)}
        ></button>
        <strong>{d.name}</strong>
        <span class="device-meta">{sizeLabel(d)}</span>
        {#if d.protocol}
          <button
            type="button"
            class="row-details btn-secondary"
            aria-expanded={Boolean(protoOpen[deviceKey(d)])}
            aria-controls="proto-{deviceKey(d)}"
            aria-label={t["lighting.rowDetails"]}
            on:click={() => toggleProto(deviceKey(d))}
          >{t["lighting.rowDetails"]}</button>
        {/if}
        {#if protoOpen[deviceKey(d)]}
          <span id="proto-{deviceKey(d)}" class="device-meta path">{d.protocol}</span>
        {/if}
      </div>
      {#if openId === deviceKey(d)}
        <div class="picker-slot is-open">
          <div class="picker-slot-inner">
        <div class="picker-flow">
          <ColorWheel bind:hex={rowHex} wheelLabel={t["lighting.color"]} busy={busy || busyKey === deviceKey(d)} on:commit={(ev) => commitDevice(d, ev)}>
            <EffectList modes={deviceModes(d)} selected={d.mode} hex={rowHex} busy={busy || busyKey === deviceKey(d)} on:mode={(ev) => dispatch("mode", { backend: d.backend, device: d.name, ...ev.detail })} />
          </ColorWheel>
          <GaugeMeter {gauges} on:apply={(ev) => dispatch("gauge", ev.detail)} />
          {#if hasLedLayout(d)}
            <button
              type="button"
              class="layout-toggle btn-secondary"
              aria-pressed={Boolean(boards[deviceKey(d)])}
              aria-expanded={Boolean(boards[deviceKey(d)])}
              aria-label={boards[deviceKey(d)] ? t["lighting.hideLayout"] : t["lighting.showLayout"]}
              on:click={() => toggle(deviceKey(d))}
            ><span class="chevron"></span></button>
            {#if boards[deviceKey(d)]}
              <LedGrid device={d} color={rowHex} {busy} on:led={(ev) => dispatch("led", { backend: d.backend, device: d.name, ...ev.detail })} />
            {/if}
          {/if}
        </div>
          </div>
        </div>
      {/if}
    </li>
  {/each}
</ul>
