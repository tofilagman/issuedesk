<script lang="ts">
  import { mediaViewer } from '$lib/stores/media.svelte';
  import { fetchAttachmentObjectUrl } from '$lib/api';

  let url = $state<string | null>(null);
  let loadError = $state(false);

  // Fetch a fresh object URL whenever the viewed item changes; revoke on close.
  // Diagrams carry their own markup, so there is nothing to fetch for those.
  $effect(() => {
    const cur = mediaViewer.current;
    url = null;
    loadError = false;
    if (!cur || cur.kind === 'diagram') return;
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

  // ---- diagram zoom / pan ----
  const MIN = 0.2;
  const MAX = 8;
  const clamp = (n: number) => Math.min(MAX, Math.max(MIN, n));

  let stageEl = $state<HTMLDivElement | null>(null);
  let contentEl = $state<HTMLDivElement | null>(null);
  let scale = $state(1);
  let tx = $state(0);
  let ty = $state(0);
  let dragging = $state(false);

  const diagram = $derived(mediaViewer.current?.kind === 'diagram' ? mediaViewer.current : null);

  /** Size the SVG to its natural dimensions, then scale it to fill the stage. */
  function fitToStage() {
    const svg = contentEl?.querySelector('svg');
    if (!svg || !stageEl) return;
    const box = svg.viewBox?.baseVal;
    const w = box?.width || svg.clientWidth || 1;
    const h = box?.height || svg.clientHeight || 1;
    // mermaid caps its output with an inline max-width; drop it so the diagram
    // can grow past the size it had in the document.
    svg.style.maxWidth = 'none';
    svg.style.width = `${w}px`;
    svg.style.height = `${h}px`;
    const rect = stageEl.getBoundingClientRect();
    scale = clamp(Math.min((rect.width * 0.92) / w, (rect.height * 0.92) / h));
    tx = 0;
    ty = 0;
  }

  // Fit once the stage and the {@html} SVG are in the DOM.
  $effect(() => {
    if (diagram && contentEl && stageEl) fitToStage();
  });

  /** Zoom by `factor`, keeping the point at stage-centre offset (cx, cy) fixed. */
  function zoomAt(factor: number, cx: number, cy: number) {
    const next = clamp(scale * factor);
    const k = next / scale;
    tx = cx - k * (cx - tx);
    ty = cy - k * (cy - ty);
    scale = next;
  }

  function onWheel(e: WheelEvent) {
    if (!stageEl) return;
    e.preventDefault();
    const rect = stageEl.getBoundingClientRect();
    zoomAt(
      Math.exp(-e.deltaY * 0.0015),
      e.clientX - rect.left - rect.width / 2,
      e.clientY - rect.top - rect.height / 2
    );
  }

  // Deltas are tracked from clientX/Y rather than movementX/Y, which Safari
  // doesn't reliably populate on PointerEvent.
  let lastX = 0;
  let lastY = 0;

  function capture(e: PointerEvent, on: boolean) {
    const el = e.currentTarget as HTMLElement;
    try {
      if (on) el.setPointerCapture(e.pointerId);
      else el.releasePointerCapture(e.pointerId);
    } catch {
      // Capture is a nicety; dragging still works without it.
    }
  }

  function onPointerDown(e: PointerEvent) {
    dragging = true;
    lastX = e.clientX;
    lastY = e.clientY;
    capture(e, true);
  }
  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    tx += e.clientX - lastX;
    ty += e.clientY - lastY;
    lastX = e.clientX;
    lastY = e.clientY;
  }
  function onPointerUp(e: PointerEvent) {
    dragging = false;
    capture(e, false);
  }

  // Offer the diagram as a downloadable .svg.
  let svgUrl = $state<string | null>(null);
  $effect(() => {
    const d = diagram;
    if (!d) {
      svgUrl = null;
      return;
    }
    const u = URL.createObjectURL(new Blob([d.svg], { type: 'image/svg+xml' }));
    svgUrl = u;
    return () => {
      URL.revokeObjectURL(u);
      svgUrl = null;
    };
  });

  const downloadUrl = $derived(diagram ? svgUrl : url);
  const downloadName = $derived(
    diagram ? `${mediaViewer.current?.name || 'diagram'}.svg` : mediaViewer.current?.name || 'download'
  );

  function onKey(e: KeyboardEvent) {
    if (!mediaViewer.current) return;
    if (e.key === 'Escape') {
      mediaViewer.close();
      return;
    }
    if (!diagram || !stageEl) return;
    if (e.key === '+' || e.key === '=') zoomAt(1.25, 0, 0);
    else if (e.key === '-') zoomAt(0.8, 0, 0);
    else if (e.key === '0') fitToStage();
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
    <div class="absolute right-3 top-3 z-10 flex items-center gap-3">
      {#if diagram}
        <div class="flex items-center gap-1 rounded-md bg-white/10 px-1 py-1 text-white/90">
          <button
            class="rounded px-2 py-0.5 text-sm hover:bg-white/20"
            aria-label="Zoom out"
            onclick={() => zoomAt(0.8, 0, 0)}>−</button
          >
          <span class="w-12 text-center text-xs tabular-nums">{Math.round(scale * 100)}%</span>
          <button
            class="rounded px-2 py-0.5 text-sm hover:bg-white/20"
            aria-label="Zoom in"
            onclick={() => zoomAt(1.25, 0, 0)}>+</button
          >
          <button class="rounded px-2 py-0.5 text-xs hover:bg-white/20" onclick={fitToStage}>Fit</button>
        </div>
      {/if}
      {#if downloadUrl}
        <a
          href={downloadUrl}
          download={downloadName}
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

    {#if diagram}
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        bind:this={stageEl}
        class="diagram-stage relative flex h-[92vh] w-[94vw] items-center justify-center overflow-hidden rounded-lg bg-white"
        class:cursor-grabbing={dragging}
        class:cursor-grab={!dragging}
        onwheel={onWheel}
        onpointerdown={onPointerDown}
        onpointermove={onPointerMove}
        onpointerup={onPointerUp}
        onpointercancel={onPointerUp}
        ondblclick={fitToStage}
      >
        <div
          bind:this={contentEl}
          class="diagram-content origin-center select-none"
          style="transform: translate({tx}px, {ty}px) scale({scale});"
        >
          <!-- mermaid sanitizes its own output (securityLevel: 'strict') -->
          {@html diagram.svg}
        </div>
      </div>
      <p class="absolute bottom-4 text-xs text-white/50">
        Scroll to zoom · drag to pan · double-click or Fit to reset · Esc to close
      </p>
    {:else if loadError}
      <p class="text-sm text-white/80">Couldn't load this media.</p>
    {:else if !url}
      <p class="text-sm text-white/70">Loading…</p>
    {:else if mediaViewer.current.kind === 'image'}
      <img
        src={url}
        alt={mediaViewer.current.name ?? ''}
        class="max-h-[90vh] max-w-[90vw] rounded shadow-2xl"
      />
    {:else}
      <!-- svelte-ignore a11y_media_has_caption -->
      <video src={url} controls autoplay class="max-h-[90vh] max-w-[90vw] rounded shadow-2xl"></video>
    {/if}
  </div>
{/if}
