<script>
  import { createEventDispatcher } from "svelte";
  import { extraFailNote } from "./lighting.js";
  import { failLines } from "./failLines.js";
  import t from "../locales/en.json";

  export let extras = [];
  export let extraResults = [];
  export let busy = false;

  const dispatch = createEventDispatcher();

  function note(item) {
    return failLines(extraFailNote(extraResults, item.id, item.present));
  }
</script>

<ul class="extras">
  {#each extras as item}
    {@const fail = note(item)}
    <li data-present={item.present ? "true" : "false"}>
      <span class="extras-row">
        <span class="mark">{item.present ? t["support.ready"] : t["support.needed"]}</span>
        {t[`lighting.extras.${item.id}`] || item.label || item.id}
        {#if item.label && item.id === "linux-modules-extra"}
          <span class="path">{item.label}</span>
        {/if}
        <button type="button" class="btn-secondary" on:click={() => dispatch("item", item.id)} disabled={busy}>
          {t["support.installItem"]}
        </button>
      </span>
      {#if fail.first}
        <p class="fail" role="alert">{fail.first}</p>
        {#if fail.rest}
          <details>
            <summary>{t["lighting.rowDetails"]}</summary>
            <p class="fail">{fail.rest}</p>
          </details>
        {/if}
      {/if}
    </li>
  {/each}
</ul>
