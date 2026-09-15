<script>
  import { createEventDispatcher } from "svelte";
  import { fanDutyPct } from "./cooling.js";
  import { defaultKind, defaultName, defaultTempId } from "./coolingBoard.js";
  import { PRESETS, curveLabel } from "./coolingCurves.js";
  import { canControl } from "./pwm.js";
  import t from "../locales/en.json";

  export let cards = [];
  export let names = {};
  export let taken = {};
  export let kinds = {};
  export let curveId = {};
  export let tempId = {};
  export let sources = [];
  export let curves = PRESETS;
  export let conflicts = [];
  export let busy = false;
  export let units = {};

  const dispatch = createEventDispatcher();

  function set(id, field, value) {
    dispatch("change", { id, field, value });
  }
</script>

<div class="fc-board">
  {#each cards as card (card.id)}
    {@const pct = fanDutyPct(card)}
    <article class="fc-tile fc-ctrl">
      <header class="fc-tile-head">
        <label>
          {t["cooling.rename"]}
          <input
            class="fc-name"
            value={names[card.id] || defaultName(card)}
            on:change={(e) => set(card.id, "name", e.currentTarget.value)}
          />
        </label>
        <button type="button" class="fc-hide btn-secondary" on:click={() => set(card.id, "hidden", true)}>{t["cooling.hide"]}</button>
      </header>
      <label class="fc-switch">
        <input
          type="checkbox"
          autocomplete="off"
          aria-label={t["cooling.takeover"]}
          disabled={!canControl(card, conflicts) || busy}
          checked={!!taken[card.id]}
          on:change={(e) => dispatch("take", { card, checked: e.currentTarget.checked })}
        />
        <span>{t["cooling.curve"]}</span>
        <select value={curveId[card.id] || "balanced"} on:change={(e) => set(card.id, "curve", e.currentTarget.value)}>
          {#each curves as preset}
            <option value={preset.id}>{curveLabel(preset, t)}</option>
          {/each}
        </select>
      </label>
      <label>
        {t["cooling.tempSource"]}
        <select value={tempId[card.id] || defaultTempId(sources)} on:change={(e) => set(card.id, "temp", e.currentTarget.value)}>
          {#each sources as src}
            <option value={src.id}>{src.label}</option>
          {/each}
        </select>
      </label>
      <label>
        {t["cooling.kind"]}
        <select value={kinds[card.id] || defaultKind(card)} on:change={(e) => set(card.id, "kind", e.currentTarget.value)}>
          <option value="fan">{t["cooling.kindFan"]}</option>
          <option value="pump">{t["cooling.kindPump"]}</option>
        </select>
      </label>
      <label class="fc-switch">
        <input type="checkbox" checked={!!units[card.id]} on:change={(e) => set(card.id, "unit", e.currentTarget.checked)} />
        <span>{t["cooling.aioUnit"]}</span>
      </label>
      <div class="fc-readout">
        <strong>{pct == null ? "—" : pct + " %"}</strong>
        <span>{card.fan ? card.fan.value + " RPM" : t["cooling.noRpm"]}</span>
      </div>
    </article>
  {/each}
</div>
