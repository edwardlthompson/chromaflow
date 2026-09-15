<script>
  import { createEventDispatcher } from "svelte";
  import t from "../locales/en.json";

  export let label = "R";
  export let tip = "";
  export let value = 0;
  export let min = 0;
  export let max = 255;
  export let maxlength = 3;
  export let chanClass = "";

  const dispatch = createEventDispatcher();

  function clamp(n) {
    const v = Math.round(Number(n));
    if (!Number.isFinite(v)) return Math.round(value);
    return Math.max(min, Math.min(max, v));
  }

  function send(n, commit) {
    const v = clamp(n);
    dispatch("input", v);
    if (commit) dispatch("commit");
  }

  function onType(event) {
    const raw = String(event.target.value || "").replace(/[^\d]/g, "");
    event.target.value = raw;
    if (raw === "") return;
    const n = Number(raw);
    if (!Number.isFinite(n)) return;
    if (n > max) {
      send(max, false);
      return;
    }
    if (raw.length < String(min).length && n < min) return;
    send(n, false);
  }

  function onBlur(event) {
    send(event.target.value === "" ? value : event.target.value, true);
  }

  function onKey(event) {
    if (event.key === "ArrowUp") {
      event.preventDefault();
      send(value + 1, true);
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      send(value - 1, true);
    }
  }
</script>

<div class="slider-row" title={tip}>
  <span>{label}</span>
  <input
    type="range"
    {min}
    {max}
    {value}
    aria-label={label}
    class={chanClass}
    on:input={(e) => send(e.target.value, false)}
    on:change={() => dispatch("commit")}
  />
  <input
    type="text"
    inputmode="numeric"
    pattern="[0-9]*"
    {maxlength}
    spellcheck="false"
    aria-label="{label} {t['lighting.value']}"
    value={Math.round(value)}
    on:input={onType}
    on:blur={onBlur}
    on:keydown={onKey}
  />
  <span class="stepper">
    <button type="button" class="nudge" aria-label="{label} {t['lighting.up']}" on:click={() => send(value + 1, true)}>+</button>
    <button type="button" class="nudge" aria-label="{label} {t['lighting.down']}" on:click={() => send(value - 1, true)}>−</button>
  </span>
</div>
