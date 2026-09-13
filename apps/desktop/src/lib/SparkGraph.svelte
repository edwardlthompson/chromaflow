<script>
  import { areaPath, GRID_XS, GRID_YS, SPARK_H, SPARK_W } from "./spark.js";

  export let values = [];
  export let gid = "g";
  export let kind = "temp";
  export let label = "";

  $: path = areaPath(values, SPARK_W, SPARK_H);
</script>

<svg class="fc-spark" viewBox="0 0 {SPARK_W} {SPARK_H}" preserveAspectRatio="none" role="img" aria-label={label}>
  <defs>
    <linearGradient id={gid} x1="0" y1="1" x2="0" y2="0">
      {#if kind === "usage"}
        <stop offset="0%" stop-color="#00c040" />
        <stop offset="50%" stop-color="#f0d000" />
        <stop offset="100%" stop-color="#ff2020" />
      {:else}
        <stop offset="0%" stop-color="#2060ff" />
        <stop offset="50%" stop-color="#c040ff" />
        <stop offset="100%" stop-color="#ff2020" />
      {/if}
    </linearGradient>
  </defs>
  <rect width={SPARK_W} height={SPARK_H} fill="#0b1d38" />
  {#each GRID_YS as y}
    <line x1="0" y1={y} x2={SPARK_W} y2={y} stroke="#1e3a66" stroke-width="1" />
  {/each}
  {#each GRID_XS as x}
    <line x1={x} y1="0" x2={x} y2={SPARK_H} stroke="#1e3a66" stroke-width="1" />
  {/each}
  {#if path}
    <path d={path} fill={`url(#${gid})`} />
  {/if}
</svg>
