<script lang="ts">
  import { NodeViewWrapper } from 'svelte-tiptap';
  import type { NodeViewProps } from '@tiptap/core';
  import { fetchAttachmentObjectUrl } from '$lib/api';
  import { mediaViewer } from '$lib/stores/media.svelte';

  let { node }: NodeViewProps = $props();

  // A node view instance is bound to one node, so these are stable for its lifetime.
  const isVideo = $derived(node.type.name === 'video');
  const name = $derived<string>(node.attrs.title ?? node.attrs.alt ?? '');
  // Stored src looks like /api/attachments/{uuid}
  const attachmentId = $derived(
    (node.attrs.src ?? '').match(/\/api\/attachments\/([0-9a-fA-F-]+)/)?.[1] ?? null
  );

  let url = $state<string | null>(null);
  let failed = $state(false);

  // Fetch (with auth) into an object URL. Driven by $effect rather than onMount
  // because svelte-tiptap mounts node views via Svelte's mount(), where $effect
  // is the reliable post-mount hook; it also re-runs if the node's src changes.
  $effect(() => {
    const id = attachmentId;
    url = null;
    failed = false;
    if (!id) {
      failed = true;
      return;
    }
    let cancelled = false;
    let local: string | null = null;
    fetchAttachmentObjectUrl(id)
      .then((u) => {
        if (cancelled) {
          URL.revokeObjectURL(u);
          return;
        }
        local = u;
        url = u;
      })
      .catch(() => {
        if (!cancelled) failed = true;
      });
    return () => {
      cancelled = true;
      if (local) URL.revokeObjectURL(local);
    };
  });

  function openViewer() {
    if (!attachmentId || !url) return;
    mediaViewer.open({ kind: isVideo ? 'video' : 'image', attachmentId, name });
  }
</script>

<NodeViewWrapper class="my-2 inline-block max-w-full align-top">
  {#if failed}
    <span class="inline-flex items-center gap-1 rounded bg-slate-100 px-2 py-1 text-xs text-slate-500">
      ⚠ media unavailable
    </span>
  {:else if !url}
    <span class="inline-flex items-center rounded bg-slate-100 px-2 py-1 text-xs text-slate-400">
      Loading…
    </span>
  {:else if isVideo}
    <button
      type="button"
      class="group relative block overflow-hidden rounded-md border border-slate-200"
      onclick={openViewer}
      title={name || 'Play video'}
    >
      <video src={url} class="max-h-56 max-w-full" muted preload="metadata"></video>
      <span
        class="pointer-events-none absolute inset-0 grid place-items-center bg-black/20 text-4xl text-white/90 transition group-hover:bg-black/30"
        >▶</span
      >
    </button>
  {:else}
    <button type="button" class="block" onclick={openViewer} title={name || 'View image'}>
      <img
        src={url}
        alt={name}
        class="max-h-72 max-w-full cursor-zoom-in rounded-md border border-slate-200"
      />
    </button>
  {/if}
</NodeViewWrapper>
