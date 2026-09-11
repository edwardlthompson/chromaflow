<script>
  export let inventory;
</script>

<h1>Lighting</h1>
<div class="card">
  <p>OpenRGB SDK: <strong>{inventory.openrgb.status}</strong></p>
  <p>{inventory.openrgb.detail}</p>
  <p>
    ChromaFlow probes 127.0.0.1:6742 only. It does not start OpenRGB as root or
    load Windows plugins.
  </p>
  {#if inventory.openrgb.status !== "reachable"}
    <p>
      No device list yet. If a device works in Windows OpenRGB but not Linux, that
      is <strong>No Linux backend yet</strong> — we do not fake control. See
      docs/HARDWARE.md and OpenRGB device issues.
    </p>
  {/if}
</div>
