<script>
  import t from "../locales/en.json";
  import { isTauri, invoke } from "../lib/tauri.js";
  import { confirmTakeover } from "../lib/pwm.js";
  import { pwmOn } from "../lib/ui.js";
  import { curveLabel, PRESETS } from "../lib/coolingCurves.js";
  import {
    canApplyNames,
    curveOptions,
    needsZeroAsk,
    rgbLook,
    rgbOptions,
    takeoverFile,
  } from "../lib/profilesApply.js";

  export let inventory = {};

  let file = { schema: 1, active: null, profiles: [] };
  let name = "quiet";
  let curve_set = "quiet";
  let rgb = "off";
  let error = "";
  let saved = "";
  let busy = false;

  function pick(row) {
    name = row.name;
    curve_set = row.curve_set;
    rgb = row.rgb;
  }

  function curveText(id) {
    return curveLabel(PRESETS.find((p) => p.id === id) || { id }, t);
  }

  function rgbText(id) {
    return t[`profiles.rgb.${id}`] || id;
  }

  async function load() {
    error = "";
    try {
      if (isTauri()) {
        file = await invoke("profiles_load");
        const active = (file.profiles || []).find((row) => row.name === file.active);
        if (active) pick(active);
      }
    } catch (err) {
      error = String(err);
    }
  }

  async function save() {
    error = "";
    saved = "";
    const rest = (file.profiles || []).filter((row) => row.name !== name);
    const next = {
      schema: 1,
      active: name,
      profiles: rest.concat({ name, curve_set, rgb }),
    };
    try {
      if (isTauri()) {
        file = await invoke("profiles_save", { file: next });
      } else {
        file = next;
      }
      saved = t["profiles.saved"];
    } catch (err) {
      error = String(err);
    }
  }

  async function applyRow(row) {
    error = "";
    saved = "";
    if (!canApplyNames(row.curve_set, row.rgb)) return;
    if (!isTauri()) {
      error = t["profiles.applyNeedGui"];
      return;
    }
    const conflicts = (inventory && inventory.conflicts) || [];
    if (conflicts.length) {
      error = t["cooling.noTakeover"];
      return;
    }
    const look = rgbLook(row.rgb);
    let recipe;
    try {
      recipe = takeoverFile(await invoke("pwm_load"), inventory, row.curve_set);
    } catch (err) {
      error = String(err);
      return;
    }
    const fans = (recipe.channels || []).some((ch) => ch.enabled);
    if (fans) {
      if (!(await confirmTakeover(t["cooling.takeoverAsk"]))) return;
      if (needsZeroAsk(recipe) && !(await confirmTakeover(t["cooling.zeroAsk"]))) return;
    }
    busy = true;
    try {
      if ((recipe.channels || []).some((ch) => ch.enabled)) {
        await invoke("pwm_takeover", { file: recipe });
        pwmOn.set(true);
      }
      await invoke("lighting_broadcast", { color: look.color, mode: look.mode });
      saved = t["profiles.applied"];
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  load();
</script>

<h1 class="visually-hidden">{t["profiles.title"]}</h1>
<p class="fc-note">{t["profiles.help"]}</p>
<div class="card fc-form">
  <label class="fc-field">{t["profiles.name"]} <input bind:value={name} /></label>
  <label class="fc-field">
    {t["profiles.curve"]}
    <select bind:value={curve_set}>
      {#each curveOptions(curve_set) as id}
        <option value={id}>{curveText(id)}</option>
      {/each}
    </select>
  </label>
  <label class="fc-field">
    {t["profiles.rgb"]}
    <select bind:value={rgb}>
      {#each rgbOptions(rgb) as id}
        <option value={id}>{rgbText(id)}</option>
      {/each}
    </select>
  </label>
  <p class="fc-form-actions">
    <button type="button" on:click={save} disabled={busy}>{t["profiles.save"]}</button>
    <button type="button" on:click={() => applyRow({ name, curve_set, rgb })} disabled={busy || !canApplyNames(curve_set, rgb)}>
      {t["profiles.apply"]}
    </button>
  </p>
  {#if saved}
    <p role="status">{saved}</p>
  {/if}
  {#if error}
    <p role="alert">{error}</p>
  {/if}
</div>
<div class="card">
  <h2>{t["profiles.savedList"]}</h2>
  {#if (file.profiles || []).length}
    <ul class="fc-profile-list">
      {#each file.profiles as row}
        <li>
          <button type="button" on:click={() => pick(row)}>{row.name}</button>
          <span class="device-meta">{curveText(row.curve_set)} · {rgbText(row.rgb)}</span>
          <button type="button" on:click={() => applyRow(row)} disabled={busy || !canApplyNames(row.curve_set, row.rgb)}>
            {t["profiles.apply"]}
          </button>
        </li>
      {/each}
    </ul>
  {:else}
    <p class="fc-note">{t["profiles.empty"]}</p>
  {/if}
</div>
