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
  import { startGaugeTick } from "./lib/gaugesTick.js";
  import RailNav from "./lib/RailNav.svelte";
  import ConfirmDialog from "./lib/ConfirmDialog.svelte";
  import t from "./locales/en.json";

  const titles = {
    Cooling: t["cooling.title"],
    Lighting: t["lighting.title"],
    Profiles: t["profiles.title"],
    Support: t["support.title"],
  };
  const boot = applySession(loadLocal());
  let tab = boot.tab;
  let inventory = fixture;
  let lightInv = fixture;
  let source = "sample";
  let loadError = "";
  let forceLoad = false;
  let lightReady = false;
  let runLoad = async () => {};
  let gauges = ui.gauges;
  let hist = ui.gaugeHist || [];

  $: ui.page = tab;

  function setTab(name) {
    if (name === tab) return;
    tab = name;
    persistSession(name);
    if (name !== "Cooling") ui.scrolling = false;
    if (name === "Lighting" && !lightReady) {
      forceLoad = true;
      runLoad();
    } else if (name === "Cooling") {
      runLoad();
    }
  }

  function skipPoll() {
    if (forceLoad) return false;
    if (ui.pausePoll) return true;
    if (tab === "Lighting") {
      const n = (((lightInv && lightInv.openrgb) || {}).controllers || []).length;
      if (n > 0 && researchDevices(lightInv).length === 0) return true;
    } else if ((tab === "Profiles" || tab === "Support") && source === "this machine") return true;
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
      const light = tab === "Lighting";
      try {
        if (isTauri()) {
          inventory = await invoke("inventory", { light });
          if (light) {
            lightInv = inventory;
            lightReady = true;
          }
          source = "this machine";
          loadError = "";
          return;
        }
        const res = await fetch("/live-inventory.json", { cache: "no-store" });
        if (!res.ok) return;
        const data = await res.json();
        if (data && Array.isArray(data.hwmon)) {
          inventory = data;
          if (light) {
            lightInv = data;
            lightReady = true;
          }
          source = "this machine";
        }
      } catch (err) {
        loadError = String(err);
      }
    };
    runLoad = load;
    loadSession().then((s) => {
      tab = s.tab;
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
      const wait = tab === "Cooling" ? 400 : tab === "Lighting" ? 15000 : ui.pollMs || 4000;
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
    <span class="fc-crumb">{titles[tab] || tab}</span>
    <div class="fc-pills">
      <span class="fc-pill">{source === "this machine" ? t["app.sourceLive"] : t["app.sourceSample"]}</span>
      <span class="fc-pill" class:on={$pwmOn} title={$pwmOn ? t["app.pwmOn"] : t["app.pwmOff"]}>
        {$pwmOn ? t["app.watchdog"] : t["app.firmware"]}
      </span>
      {#if loadError}
        <span class="fc-pill" role="alert" title={loadError}>{t["app.error"]}</span>
      {/if}
    </div>
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
      <div class:tab-hidden={tab !== "Cooling"} aria-hidden={tab !== "Cooling"} inert={tab !== "Cooling"}>
        <Cooling {inventory} live={source === "this machine"} {gauges} {hist} />
      </div>
      <div class:tab-hidden={tab !== "Lighting"} aria-hidden={tab !== "Lighting"} inert={tab !== "Lighting"}>
        <Lighting inventory={lightInv} {gauges} />
      </div>
      <div class:tab-hidden={tab !== "Profiles"} aria-hidden={tab !== "Profiles"} inert={tab !== "Profiles"}>
        <Profiles {inventory} />
      </div>
      <div class:tab-hidden={tab !== "Support"} aria-hidden={tab !== "Support"} inert={tab !== "Support"}>
        <Support {inventory} active={tab === "Support"} />
      </div>
    </main>
  </div>
</div>
<ConfirmDialog />
