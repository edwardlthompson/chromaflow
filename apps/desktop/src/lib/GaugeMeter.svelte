<script>
  import { createEventDispatcher } from "svelte";
  import {
    combinedCelsius,
    combinedUsageRatio,
    fmtPct,
    fmtTemp,
    laneRatio,
  } from "./gauges.js";
  import { persistSession } from "./session.js";
  import { ui } from "./ui.js";
  import t from "../locales/en.json";

  export let gauges = {
    cpu_c: null,
    gpu_c: null,
    ram: null,
    disk: null,
    ram_c: null,
    disk_c: null,
    cpu_load: null,
    gpu_load: null,
    note: "",
  };

  const dispatch = createEventDispatcher();
  let metric = ui.gaugeMetric === "usage" ? "usage" : "temp";
  let palette = ui.gaugePalette === "green" ? "green" : "blue";

  function setMetric(v) {
    metric = v === "usage" ? "usage" : "temp";
    ui.gaugeMetric = metric;
    persistSession();
  }

  function setPalette(v) {
    palette = v === "green" ? "green" : "blue";
    ui.gaugePalette = palette;
    persistSession();
  }

  $: g = gauges || {};
  $: rows = [
    { mode: "CPU", label: t["lighting.gaugeCpu"], temp: fmtTemp(g.cpu_c), usage: fmtPct(g.cpu_load), ratio: laneRatio(g, "gauge_cpu", metric) },
    { mode: "GPU", label: t["lighting.gaugeGpu"], temp: fmtTemp(g.gpu_c), usage: fmtPct(g.gpu_load), ratio: laneRatio(g, "gauge_gpu", metric) },
    { mode: "Combined", label: t["lighting.gaugeCombined"], temp: fmtTemp(combinedCelsius(g.cpu_c, g.gpu_c)), usage: fmtPct(combinedUsageRatio(g.cpu_load, g.gpu_load)), ratio: laneRatio(g, "gauge_temp", metric) },
    { mode: "RAM", label: t["lighting.gaugeRam"], temp: fmtTemp(g.ram_c), usage: fmtPct(g.ram), ratio: laneRatio(g, "gauge_ram", metric) },
    { mode: "Disk", label: t["lighting.gaugeDisk"], temp: fmtTemp(g.disk_c), usage: fmtPct(g.disk), ratio: laneRatio(g, "gauge_disk", metric) },
  ];
</script>

<section class="picker-card picker-square gauge-meter" aria-label={t["lighting.gauges"]} title={t["lighting.gaugesRoute"]}>
  <div class="gauge-metric">
    <button
      type="button"
      class="gauge-switch"
      role="switch"
      aria-checked={metric === "usage"}
      aria-label={t["lighting.gaugeMetric"]}
      on:click={() => setMetric(metric === "usage" ? "temp" : "usage")}
    >
      <span class:is-on={metric === "temp"}>{t["lighting.gaugeTemp"]}</span>
      <span class="gauge-knob" aria-hidden="true"></span>
      <span class:is-on={metric === "usage"}>{t["lighting.gaugeUsage"]}</span>
    </button>
    <button
      type="button"
      class="gauge-switch"
      role="switch"
      aria-checked={palette === "green"}
      aria-label={t["lighting.gaugePalette"]}
      on:click={() => setPalette(palette === "green" ? "blue" : "green")}
    >
      <span class:is-on={palette === "blue"}>{t["lighting.gaugeBlue"]}</span>
      <span class="gauge-knob" aria-hidden="true"></span>
      <span class:is-on={palette === "green"}>{t["lighting.gaugeGreen"]}</span>
    </button>
  </div>
  {#each rows as row}
    <button type="button" class="gauge-row" title={t["lighting.gaugeSpectrum"]} on:click={() => dispatch("apply", { mode: row.mode })}>
      <span class="gauge-row-head">
        <strong>{row.label}</strong>
        <span>{row.temp}</span>
        <span>{row.usage}</span>
      </span>
      <span class="gauge-spectrum" class:gauge-blue={palette === "blue"}>
        <span class="gauge-marker" style="left:{Math.max(0, Math.min(1, row.ratio)) * 100}%"></span>
      </span>
    </button>
  {/each}
</section>
