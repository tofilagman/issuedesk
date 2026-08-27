<script lang="ts">
  import { NodeViewWrapper } from 'svelte-tiptap';
  import type { NodeViewProps } from '@tiptap/core';
  import { renderMermaid } from './mermaidRenderer';

  let { node, updateAttributes, deleteNode, editor, selected }: NodeViewProps = $props();

  const code = $derived<string>(node.attrs.code ?? '');

  // `editor.isEditable` is a plain getter; mirror it into state and refresh on
  // 'update', which setEditable() emits.
  let editable = $state(editor.isEditable);
  $effect(() => {
    const sync = () => (editable = editor.isEditable);
    editor.on('update', sync);
    return () => void editor.off('update', sync);
  });

  // A freshly inserted (empty) diagram opens straight into its source pane.
  let showSource = $state(editor.isEditable && (node.attrs.code ?? '').trim() === '');

  let svg = $state('');
  let error = $state<string | null>(null);

  // Re-render on every source change, debounced so typing stays smooth.
  $effect(() => {
    const src = code;
    if (src.trim() === '') {
      svg = '';
      error = null;
      return;
    }
    let cancelled = false;
    const timer = setTimeout(() => {
      renderMermaid(src)
        .then((out) => {
          if (cancelled) return;
          svg = out;
          error = null;
        })
        .catch((e: unknown) => {
          if (cancelled) return;
          svg = '';
          error = e instanceof Error ? e.message : String(e);
        });
    }, 200);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  });

  const rows = $derived(Math.min(24, Math.max(4, code.split('\n').length + 1)));
</script>

<NodeViewWrapper class="mermaid-node">
  <div class="mmd" class:mmd-selected={editable && selected} contenteditable="false">
    {#if editable}
      <div class="mmd-bar">
        <span class="mmd-grip" data-drag-handle draggable="true" title="Drag to move">⠿</span>
        <span class="mmd-title">Mermaid</span>
        <button
          type="button"
          class="mmd-tab"
          class:mmd-tab-on={!showSource}
          onclick={() => (showSource = false)}>Diagram</button
        >
        <button
          type="button"
          class="mmd-tab"
          class:mmd-tab-on={showSource}
          onclick={() => (showSource = true)}>Source</button
        >
        <button
          type="button"
          class="mmd-tab ml-auto text-rose-600"
          title="Remove diagram"
          onclick={() => deleteNode()}>Remove</button
        >
      </div>
    {/if}

    {#if editable && showSource}
      <!-- svelte-ignore a11y_autofocus -->
      <textarea
        class="mmd-source"
        {rows}
        spellcheck="false"
        placeholder={'graph TD\n  A --> B'}
        value={code}
        onmousedown={(e) => e.stopPropagation()}
        onkeydown={(e) => e.stopPropagation()}
        oninput={(e) => updateAttributes({ code: e.currentTarget.value })}
      ></textarea>
    {/if}

    {#if !editable || !showSource || svg || error}
      <div class="mmd-preview">
        {#if error}
          <div class="mmd-error">
            <span class="font-medium">Diagram error</span>
            <pre>{error}</pre>
          </div>
        {:else if svg}
          <!-- mermaid sanitizes its own output (securityLevel: 'strict') -->
          {@html svg}
        {:else if code.trim() === ''}
          <span class="mmd-empty">Empty diagram</span>
        {:else}
          <span class="mmd-empty">Rendering…</span>
        {/if}
      </div>
    {/if}
  </div>
</NodeViewWrapper>
