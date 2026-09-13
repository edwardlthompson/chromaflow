<script>
  import ControlList from "./ControlList.svelte";
  import t from "../locales/en.json";

  export let cards = [];
  export let names = {};
  export let taken = {};
  export let kinds = {};
  export let curveId = {};
  export let tempId = {};
  export let sources = [];
  export let curves = [];
  export let conflicts = [];
  export let busy = false;
  export let units = {};
  export let liquidctl = null;
</script>

<section class="fc-section">
  <h2>{t["cooling.pumps"]}</h2>
  <ControlList {cards} {names} {taken} {kinds} {curveId} {tempId} {sources} {curves} {conflicts} {busy} {units} on:change on:take />
  {#if liquidctl && liquidctl.detail}
    <p class="path">{t["cooling.liquidctl"]}: {liquidctl.detail}</p>
  {/if}
</section>
