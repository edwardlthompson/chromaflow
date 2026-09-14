<script>
  import { createEventDispatcher } from "svelte";
  import ChannelRow from "./ChannelRow.svelte";
  import { groupedModes } from "./effectCatalog.js";
  import { classify } from "./effectQmk.js";
  import { persistSession } from "./session.js";
  import { ui } from "./ui.js";
  import t from "../locales/en.json";

  export let modes = [];
  export let hex = "#0052ff";
  export let busy = false;
  export let selected = "";

  const dispatch = createEventDispatcher();
  let mode = "";
  let seen = "";
  $: speed = Math.max(1, Math.min(255, Math.round(Number(ui.effectSpeed) || 128)));

  function firstMode(list) {
    const skip = new Set(["off"]);
    const idle = new Set(["off", "direct", "custom"]);
    const names = list || [];
    return (
      names.find((m) => !idle.has(String(m).toLowerCase())) ||
      names.find((m) => !skip.has(String(m).toLowerCase())) ||
      names[0] ||
      ""
    );
  }

  function setSpeed(v) {
    ui.effectSpeed = Math.max(1, Math.min(255, Math.round(Number(v) || 128)));
    persistSession();
  }

  $: if (selected && selected !== seen && modes.includes(selected)) {
    seen = selected;
    mode = selected;
  }
  $: if (modes.length && !modes.includes(mode)) mode = firstMode(modes);
  $: if (!modes.length) mode = "";
  $: groups = groupedModes(modes);
  $: gauge = String(classify(mode)).startsWith("gauge");
</script>

{#if modes.length}
  <div class="effect-row">
    <select bind:value={mode} data-applied={selected} disabled={busy} title={t["lighting.effectHint"]} aria-label={t["lighting.effect"]} on:change={() => { if (mode && mode !== selected) dispatch("mode", { mode, color: hex, speed }); }}>
      {#each groups as group}
        <optgroup label={t[`lighting.group.${group.id}`]}>
          {#each group.modes as name}
            <option value={name}>{name}</option>
          {/each}
        </optgroup>
      {/each}
    </select>
    <ChannelRow
      label={t["lighting.speed"]}
      tip={gauge ? t["lighting.tip.gaugeSpeed"] : t["lighting.tip.speed"]}
      value={speed}
      min={1}
      max={255}
      chanClass="chan-speed"
      on:input={(e) => setSpeed(e.detail)}
    />
    <button type="button" disabled={busy || !mode} on:click={() => dispatch("mode", { mode, color: hex, speed })}>
      {t["lighting.applyEffect"]}
    </button>
  </div>
{/if}
