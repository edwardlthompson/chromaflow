<script>
  import { onMount } from "svelte";
  import Cooling from "./pages/Cooling.svelte";
  import Lighting from "./pages/Lighting.svelte";
  import Profiles from "./pages/Profiles.svelte";
  import Support from "./pages/Support.svelte";
  import fixture from "./fixtures/inventory.json";
  import { isTauri, invoke } from "./lib/tauri.js";
  import { ui, pwmOn } from "./lib/ui.js";
  import { applySession, loadLocal, loadSession, persistSession } from "./lib/session.js";
  import { researchDevices } from "./lib/research.js";
  import { setHostCycle, wantsCycle } from "./lib/lightingTick.js";
  import { startGaugeTick } from "./lib/gaugesTick.js";
  import RailNav from "./lib/RailNav.svelte";
  import t from "./locales/en.json";

  const boot = applySession(loadLocal());
  let tab = boot.tab;
  let inventory = fixture;
  let lightInv = fixture;
  let source = "sample";
  let loadError = "";
  let forceLoad = false;
  let runLoad = async () => {};
  let gauges = ui.gauges;
  let hist = ui.gaugeHist || [];

  $: conflicts = (inventory && inventory.conflicts) || [];
  $: ui.page = tab;

  function setTab(name) {
    tab = name;
    persistSession(name);
    if (name !== "Cooling") ui.scrolling = false;
    forceLoad = true;
    runLoad();
  }

  function skipPoll() {
    if (forceLoad) return false;
    if (ui.pausePoll) return true;
    if (tab === "Lighting") {
      const n = (((lightInv && lightInv.openrgb) || {}).controllers || []).length;
      if (n > 0 && researchDevices(lightInv).length === 0) return true;
    } else if (tab === "Profiles" && source === "this machine") return true;
    if (tab === "Cooling") return ui.scrolling;
    const ae = typeof document !== "undefined" ? document.activeElement : null;
    const tag = ae && ae.tagName;
    return tag === "INPUT" || tag === "SELECT" || tag === "TEXTAREA";
  }

  onMount(() => {
    let timer = 0;
    let stop = false;
    const load = async () => {
      if (skipPoll() && source === "this machine") return;
      forceLoad = false;
      const light = tab === "Lighting" || tab === "Support";
      try {
        if (isTauri()) {
          inventory = await invoke("inventory", { light });
          if (light) lightInv = inventory;
          source = "this machine";
          loadError = "";
          return;
        }
        const res = await fetch("/live-inventory.json", { cache: "no-store" });
        if (!res.ok) return;
        const data = await res.json();
        if (data && Array.isArray(data.hwmon)) {
          inventory = data;
          if (light) lightInv = data;
          source = "this machine";
        }
      } catch (err) {
        loadError = String(err);
      }
    };
    runLoad = load;
    loadSession().then((s) => {
      tab = s.tab;
      if (wantsCycle(s.lastMode)) setHostCycle(true, invoke, isTauri);
      forceLoad = true;
      runLoad();
    });
    const onLeave = () => persistSession(tab);
    window.addEventListener("pagehide", onLeave);
    window.addEventListener("beforeunload", onLeave);
    const tick = async () => {
      if (stop) return;
      await load();
      if (stop) return;
      const wait =
        tab === "Cooling" ? 400 : tab === "Lighting" ? (ui.cycleOn ? 15000 : 4000) : ui.pollMs || 4000;
      timer = window.setTimeout(tick, wait);
    };
    tick();
    const stopGauges = startGaugeTick({
      invoke,
      isTauri,
      onSample: (next, nextHist) => {
        gauges = next;
        hist = nextHist || [];
      },
    });
    return () => {
      stop = true;
      window.clearTimeout(timer);
      stopGauges();
      window.removeEventListener("pagehide", onLeave);
      window.removeEventListener("beforeunload", onLeave);
    };
  });
</script>

<div class="app-shell">
  <header class="fc-top">
    <span class="brand">ChromaFlow</span>
    <p class="status" role="status">
      Showing {source}. {$pwmOn ? t["app.pwmOn"] : t["app.pwmOff"]} Conflicts: {conflicts.length ? conflicts.join(", ") : "none"}.
      {#if loadError} {loadError}{/if}
    </p>
  </header>
  <div class="fc-body">
    <RailNav {tab} {setTab} />
    <main
      class:page={tab !== "Cooling"}
      on:scroll={() => {
        ui.scrolling = true;
        window.clearTimeout(ui.scrollTimer);
        ui.scrollTimer = window.setTimeout(() => {
          ui.scrolling = false;
        }, 450);
      }}
    >
      {#if tab === "Cooling"}
        <Cooling {inventory} live={source === "this machine"} {gauges} {hist} />
      {:else if tab === "Lighting"}
        <Lighting inventory={lightInv} {gauges} />
      {:else if tab === "Profiles"}
        <Profiles />
      {:else}
        <Support {inventory} />
      {/if}
    </main>
  </div>
</div>
