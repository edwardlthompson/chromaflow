<script>
  import { onMount } from "svelte";
  import fixture from "../fixtures/support.json";
  import t from "../locales/en.json";
  import { isTauri, invoke } from "../lib/tauri.js";
  import { extraKernelItems, mergeExtraResults } from "../lib/lighting.js";
  import ExtraList from "../lib/ExtrasList.svelte";
  import competitorFixture from "../fixtures/competitors.json";

  export let inventory = {};

  let plan = fixture;
  let result = null;
  let extraResults = [];
  let error = "";
  let busy = false;
  let advanced = true;
  let competitorPlan = competitorFixture;
  let competitorMsg = "";

  $: extras = extraKernelItems(plan);

  function brief(msg) {
    const text = String(msg || "").trim();
    return text.length > 180 ? `${text.slice(0, 180)}…` : text;
  }

  function markPresentOk() {
    const live = extraKernelItems(plan);
    extraResults = extraResults.map((row) => {
      const item = live.find((entry) => entry.id === row.id);
      return item && item.present ? { id: row.id, ok: true, error: "" } : row;
    });
  }

  async function loadPlan() {
    if (isTauri()) {
      plan = await invoke("support_dry_run", { advanced });
      competitorPlan = await invoke("competitors_plan");
      return;
    }
    const res = await fetch(advanced ? "/live-support-advanced.json" : "/live-support.json", { cache: "no-store" });
    plan = res.ok ? await res.json() : fixture;
  }
  $: competitorNames = (competitorPlan && competitorPlan.detected) || [];
  $: competitorPkgs = (competitorPlan && competitorPlan.would_remove) || [];

  async function runDryRun() {
    busy = true;
    error = "";
    result = null;
    try {
      await loadPlan();
    } catch (err) {
      error = brief(err);
      plan = fixture;
    } finally {
      busy = false;
    }
  }

  async function runApply(extra) {
    busy = true;
    error = "";
    result = null;
    try {
      if (!isTauri()) {
        const msg = t["support.applyNeedGui"];
        error = msg;
        extraResults = extra
          ? mergeExtraResults(extraResults, [{ id: extra, ok: false, error: msg }])
          : extras.map((row) => ({ id: row.id, ok: false, error: msg }));
        return;
      }
      result = await invoke("support_apply", { advanced, extra: extra || null });
      const next = (result && result.extra_results) || [];
      extraResults = extra ? mergeExtraResults(extraResults, next) : next;
      if (result && result.extras) {
        plan = { ...plan, extras: result.extras };
      }
      try {
        await loadPlan();
      } catch (_) {
        /* keep apply extras */
      }
      markPresentOk();
      const live = extraKernelItems(plan);
      const target = extra && live.find((row) => row.id === extra);
      if (target && target.present) {
        error = "";
      } else if (result && result.ok) {
        error = "";
      } else {
        const fails = extraResults.filter((row) => row.ok === false);
        error = fails.length ? brief(fails[0].error) : brief(((result && result.errors) || []).join(" · "));
      }
    } catch (err) {
      try {
        await loadPlan();
      } catch (_) {
        /* keep prior extras */
      }
      markPresentOk();
      const live = extraKernelItems(plan);
      const target = extra && live.find((row) => row.id === extra);
      if (target && target.present) {
        extraResults = mergeExtraResults(extraResults, [{ id: extra, ok: true, error: "" }]);
        error = "";
      } else {
        const msg = brief(err);
        error = msg;
        extraResults = extra
          ? mergeExtraResults(extraResults, [{ id: extra, ok: false, error: msg }])
          : extras.map((row) => ({ id: row.id, ok: false, error: msg }));
      }
    } finally {
      busy = false;
    }
  }

  async function runRemoveCompetitors() {
    if (typeof window !== "undefined" && !window.confirm(t["support.competitorsConfirm"])) return;
    busy = true;
    error = "";
    competitorMsg = "";
    try {
      if (!isTauri()) {
        error = t["support.applyNeedGui"];
        return;
      }
      const res = await invoke("competitors_remove");
      competitorPlan = await invoke("competitors_plan");
      const errs = (res && res.errors) || [];
      if (errs.length) {
        error = errs.join(" · ");
        return;
      }
      competitorMsg = t["support.competitorsRemoved"];
    } catch (err) {
      error = String(err);
    } finally {
      busy = false;
    }
  }

  onMount(() => {
    runDryRun();
  });
</script>

<h1>{t["support.title"]}</h1>
<p>{t["support.help"]}</p>
{#if error}
  <p role="alert">{error}</p>
{/if}
{#if result}
  <p class="status" role="status">{t["support.installed"]}</p>
  {#if result.warnings && result.warnings.length}
    <p>{result.warnings.join(" · ")}</p>
  {/if}
{/if}
<div class="card">
  <h2>{t["lighting.extras"]}</h2>
  <label>
    <input type="checkbox" bind:checked={advanced} />
    {t["support.advanced"]}
  </label>
  <ExtraList
    {extras}
    {extraResults}
    {busy}
    on:all={() => runApply()}
    on:item={(ev) => runApply(ev.detail)}
  />
</div>
<div class="card">
  <h2>{t["support.competitors"]}</h2>
  <p>{t["support.competitorsHelp"]}</p>
  {#if competitorNames.length}
    <p>{t["support.competitorsFound"].replace("{names}", competitorNames.join(", "))}</p>
    <p class="path">{competitorPkgs.join(", ")}</p>
  {:else}
    <p>{t["support.competitorsNone"]}</p>
  {/if}
  <p>
    <button type="button" on:click={runRemoveCompetitors} disabled={busy || !competitorNames.length}>
      {t["support.competitorsRemove"]}
    </button>
  </p>
  {#if competitorMsg}
    <p class="status" role="status">{competitorMsg}</p>
  {/if}
</div>

