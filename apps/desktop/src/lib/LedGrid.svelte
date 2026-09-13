<script>
  import { createEventDispatcher } from "svelte";
  import { boardLayout } from "./keyboard.js";
  import { layoutColors } from "./effectViz.js";
  import t from "../locales/en.json";

  export let device = {};
  export let color = "#0052ff";
  export let busy = false;
  export let interactive = true;

  const dispatch = createEventDispatcher();
  let painted = {};

  $: colors = layoutColors(device);
  $: keys = boardLayout(device);
  $: modeName = String((device && device.mode) || "");

  function paint(idx) {
    if (!interactive || idx < 0 || busy) return;
    painted = { ...painted, [idx]: color };
    dispatch("led", { led: idx, color });
  }

  function bg(idx) {
    return colors[idx] || painted[idx] || device.color || "#222";
  }

  function fg(hex) {
    const h = String(hex || "").replace("#", "");
    if (h.length !== 6) return "#eee";
    const r = parseInt(h.slice(0, 2), 16);
    const g = parseInt(h.slice(2, 4), 16);
    const b = parseInt(h.slice(4, 6), 16);
    return r * 299 + g * 587 + b * 114 >= 140000 ? "#111" : "#eee";
  }
</script>

{#if keys}
  <div
    class="picker-card picker-board"
    title={modeName || (interactive ? t["lighting.ledHelp"] : t["lighting.ledLayout"])}
  >
    <div
      class="led-grid"
      style="grid-template-columns:repeat({keys.w},minmax(0,1fr));grid-template-rows:repeat({keys.h},minmax(0,1fr))"
      role="grid"
      aria-label={modeName ? `${t["lighting.ledLayout"]} ${modeName}` : t["lighting.ledLayout"]}
    >
      {#each keys.cells as cell}
        <button
          type="button"
          class="led-cell"
          style="grid-column:{cell.x + 1} / span {cell.span};grid-row:{cell.y + 1} / span {cell.rowSpan || 1};background:{bg(cell.idx)};color:{fg(bg(cell.idx))}"
          disabled={busy || !interactive}
          aria-label="{cell.label} {cell.idx}"
          title={cell.label}
          on:click={() => paint(cell.idx)}
        >{cell.label}</button>
      {/each}
    </div>
  </div>
{/if}
