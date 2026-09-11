<script>
  import fixture from "../fixtures/support.json";

  let plan = null;
  let error = "";
  let busy = false;

  function isTauri() {
    return typeof window !== "undefined" && Boolean(window.__TAURI_INTERNALS__);
  }

  async function runDryRun() {
    busy = true;
    error = "";
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        plan = await invoke("support_dry_run", { advanced: false });
      } else {
        plan = fixture;
      }
    } catch (err) {
      error = String(err);
      plan = fixture;
    } finally {
      busy = false;
    }
  }
</script>

<h1>Support</h1>
<p>
  Install detection support + rescan is the default path for missing fans or LEDs.
  This button is <strong>dry-run only</strong> (no apt, modprobe, or udev writes).
</p>
<button type="button" on:click={runDryRun} disabled={busy}>
  Install detection support + rescan (dry-run)
</button>
{#if error}
  <p role="alert">{error}</p>
{/if}
{#if plan}
  <div class="card">
    <p>Log out of Cinnamon after udev/group changes: {plan.logout_required ? "yes" : "no"}.</p>
    <p>Reboot required: {plan.reboot_required ? "yes" : "no"}.</p>
    <p>Would load: {(plan.would_load || []).join(", ") || "(none)"}</p>
    <p>Skipped experimental: {(plan.skipped_experimental || []).join(", ") || "(none)"}</p>
    <pre>{JSON.stringify(plan, null, 2)}</pre>
  </div>
{/if}
