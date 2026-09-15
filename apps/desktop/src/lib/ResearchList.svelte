<script>
  import { confirmTakeover } from "./pwm.js";
  import { isTauri, invoke } from "./tauri.js";
  import { DEVICE_ISSUES, submitDeviceReport, reportId, wasReported, markReported } from "./deviceReport.js";
  import t from "../locales/en.json";

  export let devices = [];
  export let kernel = "";
  export let busy = false;

  let msg = "";
  let sending = "";
  let reported = {};

  function ids(row) {
    if (row.vendor_id && row.product_id) return `${row.vendor_id}:${row.product_id}`;
    return "";
  }

  function dimmed(row) {
    const id = reportId(row);
    return Boolean(reported[id] || wasReported(id));
  }

  async function openUrl(url) {
    if (isTauri()) await invoke("open_url", { url });
    else if (typeof window !== "undefined") window.open(url, "_blank", "noopener");
  }

  async function report(row) {
    const id = reportId(row);
    if (!id || dimmed(row) || sending) return;
    if (!(await confirmTakeover(t["lighting.reportConfirm"]))) {
      return;
    }
    msg = "";
    sending = id;
    try {
      const pack = await submitDeviceReport(row, { kernel, openUrl });
      markReported(id);
      reported = { ...reported, [id]: true };
      msg = pack.bodyTooLarge ? t["lighting.reportPasted"] : t["lighting.reportOpened"];
    } catch (err) {
      msg = String(err);
    }
    sending = "";
  }
</script>

{#if devices.length}
  <details>
    <summary>{t["lighting.rowDetails"]}</summary>
    <p class="fc-note">{t["lighting.researchHelp"]}</p>
  </details>
  <ul class="extras">
    {#each devices as row}
      <li>
        <span class="extras-row">
          <strong>{row.name}</strong>
          {#if ids(row)}<span class="path">{ids(row)}</span>{/if}
          <span>{t[`lighting.reason.${row.reason}`] || row.reason || ""}</span>
          <button
            type="button"
            class:reported={dimmed(row)}
            disabled={busy || dimmed(row) || sending === reportId(row)}
            on:click={() => report(row)}
          >{dimmed(row) ? t["lighting.reportSent"] : t["lighting.reportDevice"]}</button>
        </span>
      </li>
    {/each}
  </ul>
  <p>
    <button type="button" on:click={() => openUrl(DEVICE_ISSUES)}>{t["lighting.reportInbox"]}</button>
  </p>
{:else}
  <p>{t["lighting.researchEmpty"]}</p>
{/if}
{#if msg}
  <p class="status" role="status">{msg}</p>
{/if}
