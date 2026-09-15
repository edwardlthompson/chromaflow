<script>
  import { createEventDispatcher } from "svelte";
  import {
    SUGGESTED_HEX,
    hexToHsv,
    hsvToHex,
    hsvToRgb,
    rgbToHex,
    normalizeHex,
    loadRecent,
    pushRecent,
  } from "./color.js";
  import { KELVIN_MAX, KELVIN_MIN, KELVIN_PRESETS, kelvinToHex, kelvinToRgb, rgbToKelvin } from "./kelvin.js";
  import ChannelRow from "./ChannelRow.svelte";
  import t from "../locales/en.json";

  export let hex = "#0052ff";
  export let wheelLabel = "Color";
  export let busy = false;

  const dispatch = createEventDispatcher();

  let hue = 221;
  let sat = 1;
  let val = 1;
  let ringEl;
  let squareEl;
  let drag = "";
  let kelvinHeld = null;
  let recent = loadRecent();
  const chips = SUGGESTED_HEX;

  $: {
    const hsv = hexToHsv(hex);
    if (hsv && normalizeHex(hex) !== hsvToHex(hue, sat, val)) {
      hue = hsv.h;
      sat = hsv.s;
      val = hsv.v;
    }
  }
  $: rgb = hsvToRgb(hue, sat, val);
  $: kelvin = kelvinHeld == null ? rgbToKelvin(Math.round(rgb.r), Math.round(rgb.g), Math.round(rgb.b)) : kelvinHeld;
  $: hueCss = hsvToHex(hue, 1, 1);
  $: ringKnob = ringPos(hue);
  $: sqLeft = `${sat * 100}%`;
  $: sqTop = `${(1 - val) * 100}%`;

  function ringPos(h) {
    const rad = (h * Math.PI) / 180;
    return {
      left: `${50 + Math.cos(rad) * 46}%`,
      top: `${50 + Math.sin(rad) * 46}%`,
    };
  }

  function emit(h, s, v) {
    kelvinHeld = null;
    hue = h;
    sat = s;
    val = v;
    hex = hsvToHex(h, s, v);
  }

  function remember() {
    recent = pushRecent(hex);
  }

  function commit() {
    const use = normalizeHex(hex);
    if (!use) return;
    dispatch("commit", { hex: use });
  }

  function pickHex(next) {
    const hsv = hexToHsv(next);
    if (!hsv) return;
    emit(hsv.h, hsv.s, hsv.v);
    remember();
    commit();
  }

  function onRing(event) {
    if (!ringEl) return;
    const box = ringEl.getBoundingClientRect();
    const x = event.clientX - box.left - box.width / 2;
    const y = event.clientY - box.top - box.height / 2;
    const nextHue = (((Math.atan2(y, x) * 180) / Math.PI) + 360) % 360;
    emit(nextHue, sat, val);
  }

  function onSquare(event) {
    if (!squareEl) return;
    const box = squareEl.getBoundingClientRect();
    const s = Math.max(0, Math.min(1, (event.clientX - box.left) / box.width));
    const v = Math.max(0, Math.min(1, 1 - (event.clientY - box.top) / box.height));
    emit(hue, s, v);
  }

  function inSquare(event) {
    if (!squareEl) return false;
    const box = squareEl.getBoundingClientRect();
    return (
      event.clientX >= box.left &&
      event.clientX <= box.right &&
      event.clientY >= box.top &&
      event.clientY <= box.bottom
    );
  }

  function down(event) {
    drag = inSquare(event) ? "square" : "ring";
    if (ringEl && ringEl.setPointerCapture) ringEl.setPointerCapture(event.pointerId);
    if (drag === "square") onSquare(event);
    else onRing(event);
  }

  function move(event) {
    if (drag === "ring") onRing(event);
    if (drag === "square") onSquare(event);
  }

  function up() {
    if (drag) {
      remember();
      commit();
    }
    drag = "";
  }

  function setRgb(channel, n) {
    kelvinHeld = null;
    const next = { r: rgb.r, g: rgb.g, b: rgb.b, [channel]: n };
    hex = rgbToHex(next.r, next.g, next.b);
  }

  function setHsv(channel, n) {
    if (channel === "h") emit(n, sat, val);
    if (channel === "s") emit(hue, n / 255, val);
    if (channel === "v") emit(hue, sat, n / 255);
  }

  function setKelvin(k) {
    const v = Math.max(KELVIN_MIN, Math.min(KELVIN_MAX, Math.round(Number(k) || 0)));
    kelvinHeld = v;
    const next = kelvinToRgb(v);
    hex = rgbToHex(next.r, next.g, next.b);
  }
</script>

<div class="picker-cubes" style="--hue:{hueCss}">
  <div class="picker-card picker-square picker-wheel">
    <div
      bind:this={ringEl}
      class="wheel-stack"
      role="slider"
      aria-label={wheelLabel}
      aria-valuemin="0"
      aria-valuemax="360"
      aria-valuenow={Math.round(hue)}
      tabindex="0"
      on:pointerdown={down}
      on:pointermove={move}
      on:pointerup={up}
      on:pointercancel={up}
    >
      <div class="hue-ring" aria-hidden="true"></div>
      <span class="wheel-knob ring-knob" style="left:{ringKnob.left};top:{ringKnob.top};background:{hueCss}"></span>
      <div bind:this={squareEl} class="sv-square" role="img" aria-label={t["lighting.sv"]}>
        <span class="wheel-knob" style="left:{sqLeft};top:{sqTop};background:{hex}"></span>
      </div>
    </div>
  </div>
  <div class="picker-card picker-square picker-sliders">
    <ChannelRow label={t["lighting.r"]} tip={t["lighting.tip.r"]} value={Math.round(rgb.r)} chanClass="chan-r" on:input={(e) => setRgb("r", e.detail)} on:commit={() => { remember(); commit(); }} />
    <ChannelRow label={t["lighting.g"]} tip={t["lighting.tip.g"]} value={Math.round(rgb.g)} chanClass="chan-g" on:input={(e) => setRgb("g", e.detail)} on:commit={() => { remember(); commit(); }} />
    <ChannelRow label={t["lighting.b"]} tip={t["lighting.tip.b"]} value={Math.round(rgb.b)} chanClass="chan-b" on:input={(e) => setRgb("b", e.detail)} on:commit={() => { remember(); commit(); }} />
    <ChannelRow label={t["lighting.h"]} tip={t["lighting.tip.h"]} value={Math.round(hue)} max={360} chanClass="chan-h" on:input={(e) => setHsv("h", e.detail)} on:commit={() => { remember(); commit(); }} />
    <ChannelRow label={t["lighting.s"]} tip={t["lighting.tip.s"]} value={Math.round(sat * 255)} chanClass="chan-s" on:input={(e) => setHsv("s", e.detail)} on:commit={() => { remember(); commit(); }} />
    <ChannelRow label={t["lighting.v"]} tip={t["lighting.tip.v"]} value={Math.round(val * 255)} chanClass="chan-v" on:input={(e) => setHsv("v", e.detail)} on:commit={() => { remember(); commit(); }} />
    <ChannelRow
      label={t["lighting.k"]}
      tip={t["lighting.tip.k"]}
      value={kelvin}
      min={KELVIN_MIN}
      max={KELVIN_MAX}
      maxlength={4}
      chanClass="chan-k"
      on:input={(e) => setKelvin(e.detail)}
      on:commit={() => { remember(); commit(); }}
    />
  </div>
  <div class="picker-card picker-square picker-tools" aria-busy={busy}>
    <p class="hex-row" title={t["lighting.tip.hex"]}>
      <label>{t["lighting.hex"]} <input type="text" bind:value={hex} maxlength="7" spellcheck="false" title={t["lighting.tip.hex"]} on:change={() => { kelvinHeld = null; remember(); commit(); }} /></label>
      <span class="tools-preview" style="background:{hex}" aria-hidden="true"></span>
    </p>
    <div class="swatch-row" role="group" aria-label={t["lighting.suggested"]}>
      {#each chips as c}
        <button type="button" class="chip" style="background:{c}" aria-label={c} on:click={() => pickHex(c)}></button>
      {/each}
    </div>
    <div class="swatch-row swatch-more" role="group" aria-label={t["lighting.moreSwatches"]}>
      {#each KELVIN_PRESETS as k}
        <button
          type="button"
          class="chip"
          style="background:{kelvinToHex(k)}"
          aria-label="{k} K"
          title="{k} K"
          on:click={() => pickHex(kelvinToHex(k))}
        ></button>
      {/each}
      <span class="swatch-gap" aria-hidden="true"></span>
      {#each [0, 1, 2, 3] as i}
        <button
          type="button"
          class="chip recent"
          style="background:{recent[i] || "transparent"}"
          aria-label="{t['lighting.recent']} {recent[i] || ''}"
          disabled={!recent[i]}
          on:click={() => pickHex(recent[i])}
        ></button>
      {/each}
    </div>
    <div class="tools-rest">
      <slot />
    </div>
  </div>
</div>
