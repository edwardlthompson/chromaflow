<script>
  import { curveDots, curvePoints, plotX, plotY } from "./coolingCurves.js";
  import t from "../locales/en.json";

  export let points = [];

  $: dots = curveDots(points);
</script>

<svg class="fc-graph" viewBox="0 0 200 100" preserveAspectRatio="xMidYMid meet" role="img" aria-label={t["cooling.graph"]}>
  <rect width="200" height="100" fill="#0b1d38" />
  {#each [25, 50, 75] as p}
    <line x1={plotX(20)} y1={plotY(p)} x2={plotX(100)} y2={plotY(p)} stroke="#1e3a66" />
  {/each}
  {#each [40, 60, 80] as c}
    <line x1={plotX(c)} y1={plotY(100)} x2={plotX(c)} y2={plotY(0)} stroke="#1e3a66" />
  {/each}
  <polyline fill="none" stroke="#dce7f8" stroke-width="2" points={curvePoints(points)} />
  {#each dots as dot}
    <circle cx={dot.cx} cy={dot.cy} r="3" fill="#f5c518" />
    <text
      x={dot.cx + (dot.cx > 140 ? -5 : 5)}
      y={Math.min(88, Math.max(16, dot.cy - 5))}
      text-anchor={dot.cx > 140 ? "end" : "start"}
      fill="#f5c518"
      font-size="7"
    >{dot.t}° {dot.p}%</text>
  {/each}
  <text x="2" y="16" fill="#9eb0c9" font-size="7">100%</text>
  <text x="2" y="54" fill="#9eb0c9" font-size="7">50%</text>
  <text x="2" y="92" fill="#9eb0c9" font-size="7">0%</text>
  <text x={plotX(20)} y="98" fill="#9eb0c9" font-size="7">20°C</text>
  <text x={plotX(60)} y="98" text-anchor="middle" fill="#9eb0c9" font-size="7">60°C</text>
  <text x={plotX(100)} y="98" text-anchor="end" fill="#9eb0c9" font-size="7">100°C</text>
</svg>
