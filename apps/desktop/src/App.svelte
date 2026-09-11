<script>
  import Cooling from "./pages/Cooling.svelte";
  import Lighting from "./pages/Lighting.svelte";
  import Profiles from "./pages/Profiles.svelte";
  import Support from "./pages/Support.svelte";
  import inventory from "./fixtures/inventory.json";

  const tabs = ["Cooling", "Lighting", "Profiles", "Support"];
  let tab = "Cooling";
</script>

<p class="banner" role="status">
  English UI. Do not run as root. This build cannot write PWM. Conflicts:
  {inventory.conflicts.length ? inventory.conflicts.join(", ") : "none detected"}.
</p>
<nav aria-label="Primary">
  {#each tabs as name}
    <button type="button" aria-current={tab === name ? "page" : undefined} on:click={() => (tab = name)}>
      {name}
    </button>
  {/each}
</nav>
<main>
  {#if tab === "Cooling"}
    <Cooling {inventory} />
  {:else if tab === "Lighting"}
    <Lighting {inventory} />
  {:else if tab === "Profiles"}
    <Profiles />
  {:else}
    <Support />
  {/if}
</main>
