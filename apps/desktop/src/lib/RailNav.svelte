<script>
  import t from "../locales/en.json";

  export let tab = "Cooling";
  export let setTab = () => {};

  const items = [
    { id: "Cooling", label: t["cooling.title"] },
    { id: "Lighting", label: t["lighting.title"] },
    { id: "Profiles", label: t["profiles.title"] },
    { id: "Support", label: t["support.title"] },
  ];
</script>

<nav class="fc-rail" aria-label="Primary">
  {#each items as item}
    <div class="rail-slot">
      <button
        type="button"
        title={item.label}
        aria-label={item.label}
        aria-current={tab === item.id ? "page" : undefined}
        on:click={() => setTab(item.id)}
      >
        <svg class="rail-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linejoin="round" stroke-linecap="round" aria-hidden="true">
          {#if item.id === "Cooling"}
            <path
              d="M12 3.5c1.7 1 2.8 2.6 2.9 4.4-1.6-.7-3-.4-4.1.6.1-1.9 1-3.6 1.2-5zm0 17c-1.7-1-2.8-2.6-2.9-4.4 1.6.7 3 .4 4.1-.6-.1 1.9-1 3.6-1.2 5zm8.5-8.5c-1 1.7-2.6 2.8-4.4 2.9.7-1.6.4-3-.6-4.1 1.9.1 3.6 1 5 1.2zM3.5 12c1-1.7 2.6-2.8 4.4-2.9-.7 1.6-.4 3 .6 4.1-1.9-.1-3.6-1-5-1.2z"
            />
            <circle cx="12" cy="12" r="2.1" fill="currentColor" stroke="none" />
          {:else if item.id === "Lighting"}
            <path d="M9.2 13.2c0-2.4 1.4-3.7 2.8-5.4 1.4 1.7 2.8 3 2.8 5.4 0 1.5-1.1 2.4-2.2 2.8V18h-1.2v-2c-1.1-.4-2.2-1.3-2.2-2.8z" />
            <path d="M10.2 19.4h3.6M10.6 21h2.8" />
          {:else if item.id === "Profiles"}
            <circle cx="12" cy="8" r="3.1" />
            <path d="M6.2 19.2c.6-3.1 2.9-4.8 5.8-4.8s5.2 1.7 5.8 4.8" />
          {:else}
            <path d="M14.2 6.4a3.4 3.4 0 0 0-4.8 4.8L5 15.6l3.4 3.4 4.4-4.4a3.4 3.4 0 0 0 4.8-4.8l-2.4 2.4-1.6-1.6z" />
          {/if}
        </svg>
        <span class="rail-label">{item.label}</span>
      </button>
    </div>
  {/each}
</nav>
