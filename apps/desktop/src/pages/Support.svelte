<script>
  import fixture from "../fixtures/support.json";
  import t from "../locales/en.json";
  import { isTauri, invoke } from "../lib/tauri.js";
  import { extraKernelItems, mergeExtraResults } from "../lib/lighting.js";
  import { failLines } from "../lib/failLines.js";
  import ExtraList from "../lib/ExtrasList.svelte";
  import { confirmTakeover } from "../lib/pwm.js";
  import { checkForUpdate, applyUpdate, statusText } from "../lib/updates.js";
  import { VERSION } from "../lib/version.js";
  import competitorFixture from "../fixtures/competitors.json";

  export let inventory = {};
  export let active = false;

  const VENMO = "https://venmo.com/code?user_id=1857304970395648420";

  let plan = fixture;
  let result = null;
  let extraResults = [];
  let error = "";
  let busy = false;
  let advanced = false;
  let competitorPlan = competitorFixture;
  let competitorMsg = "";
  let primed = false;
  let updateMsg = "";
  let updateBusy = false;

  $: extras = extraKernelItems(plan);
  $: fail = failLines(error);
  $: competitorNames = (competitorPlan && competitorPlan.detected) || [];
  $: competitorPkgs = (competitorPlan && competitorPlan.would_remove) || [];

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

  async function runDryRun() {
    busy = true;
    error = "";
    result = null;
    try {
      await loadPlan();
    } catch (err) {
      error = String(err);
      plan = fixture;
    } finally {
      busy = false;
    }
  }

  $: if (active && !primed) {
    primed = true;
    runDryRun();
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
        error = fails.length ? String(fails[0].error || "") : ((result && result.errors) || []).join(" · ");
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
        const msg = String(err);
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
    if (!(await confirmTakeover(t["support.competitorsConfirm"]))) return;
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

  async function openDonate() {
    if (isTauri()) await invoke("open_url", { url: VENMO });
    else if (typeof window !== "undefined") window.open(VENMO, "_blank", "noopener");
  }

  async function runUpdate() {
    updateBusy = true;
    updateMsg = t["support.updateChecking"];
    try {
      const report = await checkForUpdate();
      const next = report && report.status === "available"
        ? await applyUpdate(report, { confirm: t["support.updateConfirm"] }, true)
        : report;
      updateMsg = statusText(next, t);
    } catch (err) {
      updateMsg = String(err || t["support.updateOffline"]);
    } finally {
      updateBusy = false;
    }
  }
</script>

<h1 class="visually-hidden">{t["support.title"]}</h1>
<p>{t["support.help"]}</p>
<p>{t["support.toolbox"]}</p>
<p>
  <button type="button" on:click={() => runApply()} disabled={busy}>{t["support.installAll"]}</button>
</p>
{#if fail.first}
  <p role="alert">{fail.first}</p>
  {#if fail.rest}
    <details>
      <summary>{t["lighting.rowDetails"]}</summary>
      <p>{fail.rest}</p>
    </details>
  {/if}
{/if}
{#if result}
  <p class="status" role="status">{t["support.installed"]}</p>
  {#if result.warnings && result.warnings.length}
    <p>{result.warnings.join(" · ")}</p>
  {/if}
{/if}
<div class="card">
  <details>
    <summary>{t["lighting.extras"]}</summary>
    <label>
      <input type="checkbox" bind:checked={advanced} />
      {t["support.advanced"]}
    </label>
    <ExtraList {extras} {extraResults} {busy} on:item={(ev) => runApply(ev.detail)} />
  </details>
</div>
<div class="card">
  <details>
    <summary>{t["support.competitors"]}</summary>
    <p>{t["support.competitorsHelp"]}</p>
    {#if competitorNames.length}
      <p>{t["support.competitorsFound"].replace("{names}", competitorNames.join(", "))}</p>
      <p class="path">{competitorPkgs.join(", ")}</p>
    {:else}
      <p>{t["support.competitorsNone"]}</p>
    {/if}
    <p>
      <button type="button" class="btn-secondary" on:click={runRemoveCompetitors} disabled={busy || !competitorNames.length}>
        {t["support.competitorsRemove"]}
      </button>
    </p>
    {#if competitorMsg}
      <p class="status" role="status">{competitorMsg}</p>
    {/if}
  </details>
</div>
<div class="card">
  <p>
    <button type="button" class="btn-secondary" on:click={runUpdate} disabled={updateBusy}>
      {t["support.update"]}
    </button>
  </p>
  {#if updateMsg}
    <p class="status" role="status">{updateMsg}</p>
  {/if}
</div>
<div class="card">
  <details>
    <summary>{t["app.about"]}</summary>
    <p>{t["app.version"]} {VERSION}</p>
    <p class="path">{t["app.license"]}</p>
    <p>
      <button type="button" class="btn-secondary" on:click={openDonate}>{t["app.donate"]}</button>
    </p>
  </details>
</div>
