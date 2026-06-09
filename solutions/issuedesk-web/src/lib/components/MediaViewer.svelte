<script lang="ts">
  import { mediaViewer } from '$lib/stores/media.svelte';
  import { fetchAttachmentObjectUrl } from '$lib/api';

  let url = $state<string | null>(null);
  let loadError = $state(false);

  // Fetch a fresh object URL whenever the viewed item changes; revoke on close.
  $effect(() => {
    const cur = mediaViewer.current;
    url = null;
    loadError = false;
    if (!cur) return;
    let local: string | null = null;
    let cancelled = false;
    fetchAttachmentObjectUrl(cur.attachmentId)
      .then((u) => {
        if (cancelled) {
          URL.revokeObjectURL(u);
          return;
        }
        local = u;
        url = u;
      })
      .catch(() => {
        if (!cancelled) loadError = true;
      });
    return () => {
      cancelled = true;
      if (local) URL.revokeObjectURL(local);
    };
  });

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') mediaViewer.close();
  }
</script>

<svelte:window onkeydown={onKey} />

{#if mediaViewer.current}
  <!-- Close only when the backdrop itself is clicked, so the media needs no
       (a11y-warning-triggering) click handler of its own. -->
  <div
    class="fixed inset-0 z-[60] flex items-center justify-center bg-black/80 p-4"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) mediaViewer.close();
    }}
  >
    <div class="absolute right-3 top-3 flex items-center gap-3">
      {#if url}
        <a
          href={url}
          download={mediaViewer.current.name || 'download'}
          class="rounded-md bg-white/10 px-3 py-1.5 text-sm text-white/90 hover:bg-white/20"
          onclick={(e) => e.stopPropagation()}>Download</a
        >
      {/if}
      <button
        class="text-3xl leading-none text-white/70 hover:text-white"
        aria-label="Close"
        onclick={() => mediaViewer.close()}>×</button
      >
    </div>
    {#if loadError}
      <p class="text-sm text-white/80">Couldn't load this media.</p>
    {:else if !url}
      <p class="text-sm text-white/70">Loading…</p>
    {:else if mediaViewer.current.kind === 'image'}
      <img src={url} alt={mediaViewer.current.name ?? ''} class="max-h-[90vh] max-w-[90vw] rounded shadow-2xl" />
    {:else}
      <!-- svelte-ignore a11y_media_has_caption -->
      <video src={url} controls autoplay class="max-h-[90vh] max-w-[90vw] rounded shadow-2xl"></video>
    {/if}
  </div>
{/if}
