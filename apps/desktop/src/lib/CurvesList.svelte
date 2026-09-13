<script>
  import { createEventDispatcher } from "svelte";
  import { PRESETS, cloneCurve, presetPoints } from "./coolingCurves.js";
  import CurveGraph from "./CurveGraph.svelte";
  import t from "../locales/en.json";

  export let custom = [];
  const dispatch = createEventDispatcher();

  function persist(next) {
    custom = next;
    dispatch("persist");
  }

  function clone(src) {
    const n = custom.length + 1;
    const label = `${(src.label || t[`cooling.${src.id}`] || t["cooling.custom"])} ${n}`;
    persist(custom.concat([cloneCurve(src, label)]));
  }

  function add() {
    clone({ id: "balanced", points: presetPoints("balanced") });
  }

  function setLabel(id, label) {
    persist(custom.map((row) => (row.id === id ? { ...row, label } : row)));
  }

  function setPt(id, i, which, n) {
    persist(
      custom.map((row) => {
        if (row.id !== id) return row;
        const points = row.points.map((pt, j) => (j === i ? (which === 0 ? [n, pt[1]] : [pt[0], n]) : pt));
        return { ...row, points };
      }),
    );
  }
</script>

<section class="fc-section">
  <div class="fc-section-head">
    <h2>{t["cooling.curves"]}</h2>
    <button type="button" class="fc-plus" aria-label={t["cooling.addCurve"]} on:click={add}><span>+</span></button>
  </div>
  <div class="fc-board">
    {#each PRESETS as preset (preset.id)}
      <article class="fc-tile fc-curve">
        <header class="fc-tile-head fc-curve-head">
          <h3>{t[`cooling.${preset.id}`]}</h3>
          <div class="fc-curve-actions">
            <button type="button" class="fc-clone" on:click={() => dispatch("applyAll", { id: preset.id })}>{t["cooling.applyAll"]}</button>
            <button type="button" class="fc-clone" on:click={() => clone(preset)}>{t["cooling.clone"]}</button>
          </div>
        </header>
        <p class="path">{t[`cooling.${preset.id}Help`]}</p>
        <div class="fc-fill"><CurveGraph points={preset.points} /></div>
      </article>
    {/each}
    {#each custom as row (row.id)}
      <article class="fc-tile fc-curve">
        <header class="fc-tile-head">
          <input class="fc-name" value={row.label} on:change={(e) => setLabel(row.id, e.currentTarget.value)} />
        </header>
        <p class="path">{t["cooling.customHelp"]}</p>
        <div class="fc-fill"><CurveGraph points={row.points} /></div>
        <dl class="fc-params">
          {#each row.points as pt, i}
            <div>
              <dt>°C</dt>
              <dd><input type="number" min="20" max="100" value={pt[0]} on:change={(e) => setPt(row.id, i, 0, Number(e.currentTarget.value))} /></dd>
            </div>
            <div>
              <dt>%</dt>
              <dd><input type="number" min="0" max="100" value={pt[1]} on:change={(e) => setPt(row.id, i, 1, Number(e.currentTarget.value))} /></dd>
            </div>
          {/each}
        </dl>
      </article>
    {/each}
  </div>
</section>
