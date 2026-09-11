<script>
  export let inventory;
  const chip = inventory.hwmon[0];
  const points = "0,80 40,70 80,40 120,30 160,25 200,20";
</script>

<h1>Cooling</h1>
<p>
  Inventory is read-only. The curve below is a placeholder and is not applied to
  hardware.
</p>
{#if inventory.conflicts.length}
  <p class="banner" role="alert">
    Another fan daemon is present: {inventory.conflicts.join(", ")}. ChromaFlow will
    not take PWM without an explicit confirm in a later release.
  </p>
{/if}
<div class="card">
  <h2>{chip ? chip.name : "No hwmon chips"}</h2>
  {#if chip}
    <ul>
      {#each chip.temps as t}
        <li>Temp {t.label}: {t.value}</li>
      {/each}
      {#each chip.fans as f}
        <li>Fan {f.label}: {f.value}</li>
      {/each}
      {#each chip.pwms as p}
        <li>
          {p.name}={p.value} enable={p.enable_exists ? "yes" : "no"} writable={p.writable
            ? "yes"
            : "no (report only)"}
        </li>
      {/each}
    </ul>
  {/if}
  <svg viewBox="0 0 200 100" width="320" height="160" role="img" aria-label="Placeholder fan curve">
    <rect width="200" height="100" fill="#0f172a" />
    <polyline fill="none" stroke="#2dd4bf" stroke-width="3" points={points} />
  </svg>
</div>
{#each inventory.gaps as gap}
  <div class="card">
    <strong>{gap.id}</strong>
    <p>{gap.detail}</p>
  </div>
{/each}
