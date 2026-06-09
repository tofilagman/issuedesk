<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { fetchAttachmentObjectUrl } from '$lib/api';
  import { mediaViewer } from '$lib/stores/media.svelte';
  import type { Attachment } from '$lib/types';

  let { att }: { att: Attachment } = $props();

  const isImage = $derived(att.mimeType.startsWith('image/'));
  const isVideo = $derived(att.mimeType.startsWith('video/'));

  let url = $state<string | null>(null);

  onMount(() => {
    if (isImage || isVideo) {
      fetchAttachmentObjectUrl(att.id)
        .then((u) => (url = u))
        .catch(() => {});
    }
  });
  onDestroy(() => {
    if (url) URL.revokeObjectURL(url);
  });

  function open() {
    mediaViewer.open({ kind: isVideo ? 'video' : 'image', attachmentId: att.id, name: att.filename });
  }
</script>

{#if isImage || isVideo}
  <button
    type="button"
    class="group relative h-16 w-16 shrink-0 overflow-hidden rounded-md border border-slate-200 bg-slate-50"
    title={att.filename}
    onclick={open}
  >
    {#if url && isImage}
      <img src={url} alt={att.filename} class="h-full w-full object-cover" />
    {:else if url && isVideo}
      <video src={url} class="h-full w-full object-cover" muted preload="metadata"></video>
      <span class="pointer-events-none absolute inset-0 grid place-items-center bg-black/25 text-white">▶</span>
    {:else}
      <span class="grid h-full w-full place-items-center text-xs text-slate-400">…</span>
    {/if}
  </button>
{/if}
