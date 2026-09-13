<script>
  import { createEventDispatcher } from "svelte";
  import { extraFailNote } from "./lighting.js";
  import t from "../locales/en.json";

  export let extras = [];
  export let extraResults = [];
  export let busy = false;

  const dispatch = createEventDispatcher();
</script>

<p>
  <button type="button" on:click={() => dispatch("all")} disabled={busy}>{t["support.installAll"]}</button>
</p>
<ul class="extras">
  {#each extras as item}
    <li data-present={item.present ? "true" : "false"}>
      <span class="extras-row">
        <span class="mark" aria-hidden="true">{item.present ? "●" : "○"}</span>
        {t[`lighting.extras.${item.id}`] || item.label || item.id}
        {#if item.label && item.id === "linux-modules-extra"}
          <span class="path">{item.label}</span>
        {/if}
        <button type="button" on:click={() => dispatch("item", item.id)} disabled={busy}>
          {t["support.installItem"]}
        </button>
      </span>
      {#if extraFailNote(extraResults, item.id, item.present)}
        <p class="fail" role="alert">{extraFailNote(extraResults, item.id, item.present)}</p>
      {/if}
    </li>
  {/each}
</ul>
