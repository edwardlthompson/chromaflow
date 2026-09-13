<script>
  import t from "../locales/en.json";
  import { isTauri, invoke } from "../lib/tauri.js";

  let file = { schema: 1, active: null, profiles: [] };
  let name = "quiet";
  let curve_set = "default";
  let rgb = "off";
  let error = "";
  let saved = "";

  async function load() {
    error = "";
    try {
      if (isTauri()) {
        file = await invoke("profiles_load");
      }
    } catch (err) {
      error = String(err);
    }
  }

  async function save() {
    error = "";
    saved = "";
    const next = {
      schema: 1,
      active: name,
      profiles: [{ name, curve_set, rgb }],
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

  load();
</script>

<h1>{t["profiles.title"]}</h1>
<p>{t["profiles.help"]}</p>
<label>{t["profiles.name"]} <input bind:value={name} /></label>
<label>{t["profiles.curve"]} <input bind:value={curve_set} /></label>
<label>{t["profiles.rgb"]} <input bind:value={rgb} /></label>
<button type="button" on:click={save}>{t["profiles.save"]}</button>
{#if saved}
  <p role="status">{saved}</p>
{/if}
{#if error}
  <p role="alert">{error}</p>
{/if}
<div class="card">
  <pre>{JSON.stringify(file, null, 2)}</pre>
</div>
