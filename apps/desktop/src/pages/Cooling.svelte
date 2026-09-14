<script>
  import { onMount } from "svelte";
  import FansList from "../lib/FansList.svelte";
  import PumpsList from "../lib/PumpsList.svelte";
  import TempsList from "../lib/TempsList.svelte";
  import CurvesList from "../lib/CurvesList.svelte";
  import { controlCards, inHiddenRow, onBoard } from "../lib/cooling.js";
  import { canControl, channelOf, confirmTakeover } from "../lib/pwm.js";
  import { aioMemberIds, defaultKind, defaultName, defaultParams, defaultTempId, expandUnit } from "../lib/coolingBoard.js";
  import { allCurves } from "../lib/coolingCurves.js";
  import { applyPreset, canApplyRecipe, extrasOf, fileFromState, stateFromFile } from "../lib/coolingPersist.js";
  import { pickList } from "../lib/coolingMix.js";
  import { claimQueue, doneMsg, sweepEach, sweepQueue } from "../lib/coolingSweep.js";
  import { isTauri, invoke } from "../lib/tauri.js";
  import { ui, pwmOn } from "../lib/ui.js";
  import t from "../locales/en.json";

  export let inventory;
  export let live = false;
  export let gauges = {};
  export let hist = [];
  let names = {};
  let taken = {};
  let kinds = {};
  let curveId = {};
  let tempId = {};
  let params = {};
  let hidden = {};
  let shown = {};
  let mixes = [];
  let custom = [];
  let calibration = {};
  let busy = false;
  let applyMsg = "";
  let units = {};
  let unitSeeded = false;
  let recipe = null;
  let hydrated = false;
  let persistTimer = 0;
  let toastTimer = 0;
  let toastMsg = "";

  $: chips = inventory.hwmon || [];
  $: conflicts = (inventory && inventory.conflicts) || [];
  $: gaps = (inventory && inventory.gaps) || [];
  $: allCards = controlCards(chips, inventory.gpu_fans);
  $: sources = pickList(mixes);
  $: curves = allCurves(custom);
  $: fans = allCards.filter((c) => (kinds[c.id] || defaultKind(c)) !== "pump" && onBoard(c, hidden, shown));
  $: pumps = allCards.filter((c) => (kinds[c.id] || defaultKind(c)) === "pump" && onBoard(c, hidden, shown));
  $: tucked = allCards.filter((c) => inHiddenRow(c, hidden, shown));
  $: anyTaken = Object.values(taken).some(Boolean);
  $: pwmOn.set(anyTaken);
  $: if (!unitSeeded && allCards.length) {
    units = Object.fromEntries(aioMemberIds(allCards).map((id) => [id, true]));
    unitSeeded = true;
  }
  $: if (!live) hydrated = false;
  $: if (canApplyRecipe(live, recipe, allCards, hydrated)) {
    applyLoaded(recipe);
    hydrated = true;
  }

  function extras(card) {
    return extrasOf(card, curveId, tempId, kinds, params, sources);
  }

  function sourceFor(card) {
    const id = tempId[card.id] || defaultTempId(sources);
    return sources.find((s) => s.id === id) || sources[0] || null;
  }

  function fileOf() {
    const allowZero = Object.values(params).some((p) => Number(p && p.min_pct) === 0);
    return fileFromState(allCards, taken, extras, channelOf, sourceFor, mixes, names, calibration, allowZero, units, custom, hidden, shown);
  }

  function savedToast() {
    toastMsg = t["cooling.saved"];
    if (typeof window === "undefined") return;
    window.clearTimeout(toastTimer);
    toastTimer = window.setTimeout(() => {
      toastMsg = "";
    }, 2200);
  }

  async function storeFile(showToast) {
    if (!isTauri() || !live || !hydrated) return false;
    try {
      await invoke("pwm_store", { file: fileOf() });
      if (showToast) savedToast();
      return true;
    } catch (err) {
      applyMsg = String(err);
      return false;
    }
  }

  function persist() {
    if (!isTauri() || !live || !hydrated) return;
    if (typeof window !== "undefined") window.clearTimeout(persistTimer);
    persistTimer = window.setTimeout(() => {
      storeFile(true);
    }, 200);
  }

  async function onSave() {
    if (!live || !hydrated) return;
    if (typeof window !== "undefined") window.clearTimeout(persistTimer);
    if (anyTaken) await pushCurves();
    else await storeFile(true);
  }

  async function pushCurves(keepBusy) {
    if (!live || !hydrated) return false;
    const file = fileOf();
    await storeFile(false);
    if (!isTauri()) {
      applyMsg = t["cooling.needsNative"];
      return false;
    }
    if (!keepBusy) busy = true;
    try {
      const out = await invoke("pwm_takeover", { file });
      savedToast();
      if (!keepBusy) applyMsg = (out && out.detail) || t["cooling.applied"];
      return true;
    } catch (err) {
      applyMsg = String(err);
      return false;
    } finally {
      if (!keepBusy) busy = false;
    }
  }

  async function onTake(ev) {
    const peers = expandUnit(ev.detail.card, allCards, units);
    const checked = ev.detail.checked;
    if (checked) {
      if (!peers.some((c) => canControl(c, conflicts))) return;
      if (!Object.values(taken).some(Boolean) && !confirmTakeover(t["cooling.takeoverAsk"])) return;
      if (peers.some((c) => Number((params[c.id] || defaultParams()).min_pct) === 0) && !confirmTakeover(t["cooling.zeroAsk"])) return;
    }
    let next = { ...taken };
    for (const card of peers) {
      if (checked && !canControl(card, conflicts)) continue;
      if (checked) {
        if (!tempId[card.id]) tempId = { ...tempId, [card.id]: defaultTempId(sources) };
        if (!curveId[card.id]) curveId = { ...curveId, [card.id]: "balanced" };
      }
      next[card.id] = checked;
    }
    taken = next;
    persist();
    if (Object.values(taken).some(Boolean)) await pushCurves();
    else if (isTauri()) {
      try {
        await invoke("pwm_release");
        applyMsg = t["cooling.released"];
      } catch (err) {
        applyMsg = String(err);
      }
    }
  }

  async function onApplyAll(ev) {
    const id = ev.detail && ev.detail.id;
    if (!id || !live || !hydrated) return;
    const queue = allCards.filter((c) => c.pwm && onBoard(c, hidden, shown) && canControl(c, conflicts));
    if (!queue.length) {
      applyMsg = t["cooling.calibrateNone"];
      return;
    }
    if (!anyTaken && !confirmTakeover(t["cooling.takeoverAsk"])) return;
    const next = applyPreset(queue, curveId, taken, tempId, defaultTempId(sources), id);
    curveId = next.curveId;
    taken = next.taken;
    tempId = next.tempId;
    persist();
    await pushCurves();
  }

  async function onCalibrateAll() {
    if (!isTauri()) {
      applyMsg = t["cooling.needsNative"];
      return;
    }
    const queue = sweepQueue(allCards, kinds, hidden, conflicts);
    if (!queue.length) {
      applyMsg = t["cooling.calibrateNone"];
      return;
    }
    const fresh = queue.filter((c) => !taken[c.id]);
    if (fresh.length && !Object.values(taken).some(Boolean) && !confirmTakeover(t["cooling.takeoverAsk"])) return;
    if (queue.some((c) => Number((params[c.id] || defaultParams()).min_pct) === 0) && !confirmTakeover(t["cooling.zeroAsk"])) return;
    const claimed = claimQueue(queue, taken, tempId, curveId, defaultTempId, sources);
    taken = claimed.taken;
    tempId = claimed.tempId;
    curveId = claimed.curveId;
    busy = true;
    ui.pausePoll = true;
    try {
      if (!(await pushCurves(true))) return;
      const { nextCal, silent } = await sweepEach(queue, invoke, channelOf, sourceFor, extras, names, defaultName, t, (m) => {
        applyMsg = m;
      });
      calibration = { ...calibration, ...nextCal };
      persist();
      await pushCurves(true);
      applyMsg = doneMsg(t, queue.length, silent);
    } catch (err) {
      applyMsg = String(err);
      try { await pushCurves(true); } catch { /* keep calibrate error */ }
    } finally {
      ui.pausePoll = false;
      busy = false;
    }
  }

  function onChange(ev) {
    const { id, field, value } = ev.detail;
    if (field === "name") names = { ...names, [id]: value };
    else if (field === "kind") kinds = { ...kinds, [id]: value };
    else if (field === "curve") curveId = { ...curveId, [id]: value };
    else if (field === "temp") tempId = { ...tempId, [id]: value };
    else if (field === "hidden") {
      hidden = { ...hidden, [id]: true };
      shown = { ...shown, [id]: false };
    } else if (field === "unit") units = { ...units, [id]: Boolean(value) };
    else if (field === "param") {
      const cur = params[id] || defaultParams();
      params = { ...params, [id]: { ...cur, [value.key]: value.value } };
    }
    persist();
    if ((field === "curve" || field === "temp" || field === "param") && taken[id]) pushCurves();
  }

  function onMix(ev) {
    const d = ev.detail;
    const src = d.sources && d.sources.length ? d.sources : ["gauge:cpu", "gauge:gpu"];
    mixes = mixes.concat({
      id: `mix-${Date.now()}`,
      label: `${d.op} mix`,
      op: d.op,
      sources: src,
      offset: Number(d.offset) || 0,
    });
    persist();
  }

  function reveal(id) {
    hidden = { ...hidden, [id]: false };
    shown = { ...shown, [id]: true };
    persist();
  }

  function applyLoaded(file) {
    const s = stateFromFile(file, allCards);
    names = s.names;
    mixes = s.mixes;
    custom = s.custom;
    calibration = s.calibration;
    hidden = s.hidden;
    shown = s.shown;
    taken = s.taken;
    kinds = s.kinds;
    curveId = s.curveId;
    tempId = s.tempId;
    params = s.params;
    if (s.units) {
      units = s.units;
      unitSeeded = true;
    }
  }

  onMount(() => {
    if (isTauri()) invoke("pwm_load").then((file) => { recipe = file; }).catch(() => {});
  });
</script>

<div class="fc-shell">
  {#if conflicts.length}
    <p class="banner" role="alert">
      {t["cooling.conflict"].replace("{names}", conflicts.join(", "))}
    </p>
  {/if}
  <div class="fc-toolbar">
    <div class="fc-toolbar-copy">
      <p class="fc-note">{t["cooling.readOnly"]}</p>
      {#if conflicts.length}
        <p class="fc-note">{t["cooling.noTakeover"]}</p>
      {:else}
        <p>{anyTaken ? t["cooling.applied"] : t["cooling.notApplied"]}</p>
      {/if}
      {#if applyMsg}
        <p class="status" role="status">{applyMsg}</p>
      {/if}
    </div>
    <div class="fc-toolbar-actions">
      <button type="button" class="fc-cal-all" disabled={busy || conflicts.length} on:click={onCalibrateAll}>
        {t["cooling.calibrate"]}
      </button>
      <button type="button" class="fc-save" disabled={busy} on:click={onSave}>{t["cooling.save"]}</button>
    </div>
  </div>
  <FansList cards={fans} {names} {taken} {kinds} {curveId} {tempId} {sources} {curves} {conflicts} {busy} {units} on:take={onTake} on:change={onChange} />
  <PumpsList cards={pumps} {names} {taken} {kinds} {curveId} {tempId} {sources} {curves} {conflicts} {busy} {units} liquidctl={inventory.liquidctl} on:take={onTake} on:change={onChange} />
  <TempsList {chips} {mixes} {gauges} {hist} on:mix={onMix} />
  <CurvesList bind:custom on:persist={persist} on:applyAll={onApplyAll} />
  {#if tucked.length}
    <p class="path">
      {t["cooling.hidden"]}
      {#each tucked as card}
        <button type="button" class="fc-hide" on:click={() => reveal(card.id)}>
          {names[card.id] || defaultName(card)}
        </button>
      {/each}
    </p>
  {/if}
  {#if !chips.length}
    <div class="fc-tile"><h2>{t["cooling.noChips"]}</h2></div>
  {/if}
  {#each gaps as gap}
    <div class="fc-tile">
      <strong>{gap.id}</strong>
      <p>{gap.detail}</p>
    </div>
  {/each}
  {#if toastMsg}
    <p class="fc-toast" role="status">{toastMsg}</p>
  {/if}
</div>
