<script>
  import { onMount, tick } from "svelte";
  import { bindConfirm } from "./confirmDialog.js";
  import t from "../locales/en.json";

  let open = false;
  let message = "";
  let resolver = null;
  let okBtn;

  onMount(() => {
    bindConfirm((msg) =>
      new Promise((resolve) => {
        message = msg;
        resolver = resolve;
        open = true;
        tick().then(() => {
          if (okBtn) okBtn.focus();
        });
      }),
    );
    return () => bindConfirm(null);
  });

  function finish(ok) {
    open = false;
    const done = resolver;
    resolver = null;
    if (done) done(Boolean(ok));
  }

  function onKey(ev) {
    if (ev.key === "Escape") {
      ev.preventDefault();
      finish(false);
    }
  }
</script>

{#if open}
  <div class="fc-modal">
    <button type="button" class="fc-modal-back" aria-label={t["app.cancel"]} on:click={() => finish(false)}></button>
    <div
      class="fc-dialog"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="fc-confirm-title"
      tabindex="-1"
      on:keydown={onKey}
    >
      <p id="fc-confirm-title">{message}</p>
      <p class="fc-form-actions">
        <button type="button" class="btn-secondary" on:click={() => finish(false)}>{t["app.cancel"]}</button>
        <button type="button" bind:this={okBtn} on:click={() => finish(true)}>{t["app.continue"]}</button>
      </p>
    </div>
  </div>
{/if}
