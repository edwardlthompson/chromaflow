<script>
  import { createEventDispatcher } from "svelte";
  import ColorWheel from "./ColorWheel.svelte";
  import GaugeMeter from "./GaugeMeter.svelte";
  import EffectList from "./EffectList.svelte";
  import LedGrid from "./LedGrid.svelte";
  import { deviceKey, hasLedLayout, liveSwatch, normalizeHex, sizeLabel } from "./lighting.js";
  import { HOST_EFFECTS, layoutColors } from "./effectViz.js";
  import { mosaicDevice } from "./keyboard.js";
  import { ui } from "./ui.js";
  import t from "../locales/en.json";

  export let devices = [];
  export let busy = false;
  export let gauges = {};

  const dispatch = createEventDispatcher();
  let openId = "all";
  let allHex = "#0052ff";
  let rowHex = "#0052ff";
  let boards = {};

  $: totalLeds = devices.reduce((n, d) => n + (Number(d.leds) || 0), 0);
  $: allSize = totalLeds ? `${totalLeds} LEDs` : t["lighting.sizeUnknown"];
  $: allProtocol = devices.length
    ? [...new Set(devices.map((d) => d.protocol).filter(Boolean))].join(" · ")
    : t["lighting.empty"];
  $: allModes = HOST_EFFECTS;
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

  function openAll() {
    openId = "all";
  }

  function openDevice(d) {
    openId = deviceKey(d);
    rowHex = normalizeHex(d.color) || normalizeHex((d.led_colors || [])[0]) || rowHex;
  }

  function applyDevice(d) {
    const hex = openId === deviceKey(d) ? rowHex : allHex;
    dispatch("apply", { backend: d.backend, device: d.name, color: hex });
  }

  function applyAll() {
    dispatch("applyAll", { color: allHex });
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
      <span class="device-meta">{allProtocol}</span>
      <span class="device-meta">{allSize}</span>
    </div>
    {#if openId === "all"}
      <div class="picker-flow">
        <ColorWheel bind:hex={allHex} wheelLabel={t["lighting.color"]}>
          <EffectList modes={allModes} selected={sharedMode} hex={allHex} {busy} on:mode={(ev) => dispatch("modeAll", ev.detail)} />
          <button type="button" class="tools-apply" on:click={applyAll} disabled={busy}>{t["lighting.applyAll"]}</button>
        </ColorWheel>
        <GaugeMeter {gauges} on:apply={(ev) => dispatch("gauge", ev.detail)} />
        {#if hasLedLayout(allBoard)}
          <button
            type="button"
            class="layout-toggle"
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
        <span class="device-meta">{d.protocol}</span>
        <span class="device-meta">{sizeLabel(d)}</span>
      </div>
      {#if openId === deviceKey(d)}
        <div class="picker-flow">
          <ColorWheel bind:hex={rowHex} wheelLabel={t["lighting.color"]}>
            <EffectList modes={HOST_EFFECTS} selected={d.mode} hex={rowHex} {busy} on:mode={(ev) => dispatch("mode", { backend: d.backend, device: d.name, ...ev.detail })} />
            <button type="button" class="tools-apply" on:click={() => applyDevice(d)} disabled={busy}>{t["lighting.apply"]}</button>
          </ColorWheel>
          <GaugeMeter {gauges} on:apply={(ev) => dispatch("gauge", ev.detail)} />
          {#if hasLedLayout(d)}
            <button
              type="button"
              class="layout-toggle"
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
      {/if}
    </li>
  {/each}
</ul>
