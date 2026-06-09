<script lang="ts">
  import { confirmDialog } from '$lib/stores/confirm.svelte';

  let confirmBtn = $state<HTMLButtonElement | null>(null);

  // Focus the confirm button when a dialog opens (keyboard-friendly).
  $effect(() => {
    if (confirmDialog.current) confirmBtn?.focus();
  });

  function onKey(e: KeyboardEvent) {
    if (!confirmDialog.current) return;
    if (e.key === 'Escape') confirmDialog.cancel();
  }
</script>

<svelte:window onkeydown={onKey} />

{#if confirmDialog.current}
  {@const c = confirmDialog.current}
  <div
    class="fixed inset-0 z-[70] flex items-center justify-center bg-black/40 p-4"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) confirmDialog.cancel();
    }}
  >
    <div class="card w-full max-w-sm p-5" role="alertdialog" aria-modal="true" aria-label={c.title ?? 'Confirm'}>
      <h2 class="text-base font-semibold text-slate-800">{c.title ?? 'Are you sure?'}</h2>
      <p class="mt-2 text-sm text-slate-600">{c.message}</p>
      <div class="mt-5 flex justify-end gap-2">
        <button class="btn-ghost" onclick={() => confirmDialog.cancel()}>
          {c.cancelText ?? 'Cancel'}
        </button>
        <button
          bind:this={confirmBtn}
          class={c.danger ? 'btn-danger' : 'btn-primary'}
          onclick={() => confirmDialog.confirm()}
        >
          {c.confirmText ?? 'Confirm'}
        </button>
      </div>
    </div>
  </div>
{/if}
