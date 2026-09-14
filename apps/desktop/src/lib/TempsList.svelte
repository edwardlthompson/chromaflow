<script>
  import { createEventDispatcher } from "svelte";
  import { MIX_OPS, collectTemps } from "./coolingMix.js";
  import { pick, sparkId } from "./spark.js";
  import SparkGraph from "./SparkGraph.svelte";
  import t from "../locales/en.json";

  export let chips = [];
  export let mixes = [];
  export let gauges = {};
  export let hist = [];

  const dispatch = createEventDispatcher();
  let mixOpen = false;
  let mixOp = "max";
  let mixPick = [];
  let mixOffset = 0;

  $: sources = collectTemps(chips, gauges, mixes);

  function toggle(id) {
    mixPick = mixPick.includes(id) ? mixPick.filter((x) => x !== id) : mixPick.concat(id);
  }

  function dispatchMix() {
    dispatch("mix", { op: mixOp, sources: mixPick, offset: mixOffset });
    mixOpen = false;
    mixPick = [];
  }
</script>

<section class="fc-section">
  <div class="fc-section-head">
    <h2>{t["cooling.temps"]}</h2>
    <button type="button" class="fc-plus" aria-label={t["cooling.addMix"]} on:click={() => (mixOpen = !mixOpen)}><span>+</span></button>
  </div>
  {#if mixOpen}
    <div class="fc-mix">
      <label>
        {t["cooling.mixOp"]}
        <select bind:value={mixOp}>
          {#each MIX_OPS as op}
            <option value={op}>{t[`cooling.mix.${op}`]}</option>
          {/each}
        </select>
      </label>
      <label>
        {t["cooling.mixOffset"]}
        <input type="number" bind:value={mixOffset} />
      </label>
      {#each sources.filter((s) => !String(s.id).startsWith("mix:")) as src}
        <label class="fc-switch">
          <input type="checkbox" checked={mixPick.includes(src.id)} on:change={() => toggle(src.id)} />
          <span>{src.label}</span>
        </label>
      {/each}
      <button type="button" on:click={dispatchMix}>{t["cooling.mixAdd"]}</button>
    </div>
  {/if}
  <div class="fc-board">
    {#each sources as item (item.id)}
      {@const uid = sparkId(item.id)}
      <article class="fc-tile fc-meter">
        <header class="fc-tile-head">
          <h3>{item.label}</h3>
        </header>
        <div class="fc-readout">
          <strong class:invalid={item.temp.invalid}>{item.temp.text}</strong>
          <span class="path">{(item.usage && item.usage.text) || "—"}</span>
        </div>
        <div class="fc-temp-graphs">
          <SparkGraph values={pick(hist, item.id, "temp")} gid={"tg-" + uid} kind="temp" label={t["cooling.tempGraph"]} />
          <SparkGraph values={pick(hist, item.id, "usage")} gid={"ug-" + uid} kind="usage" label={t["cooling.usageGraph"]} />
        </div>
      </article>
    {/each}
  </div>
</section>
