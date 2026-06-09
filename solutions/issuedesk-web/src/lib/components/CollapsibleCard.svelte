<script lang="ts">
  import type { Snippet } from 'svelte';
  import { loadCollapse, saveCollapse } from '$lib/collapse';

  let {
    title,
    count = undefined,
    storageKey,
    defaultOpen = true,
    actions = undefined,
    children
  }: {
    title: string;
    count?: number;
    storageKey: string;
    defaultOpen?: boolean;
    /** Optional header-right content (e.g. an Upload button); does not toggle. */
    actions?: Snippet;
    children: Snippet;
  } = $props();

  let open = $state(loadCollapse(storageKey, defaultOpen));
  function toggle() {
    open = !open;
    saveCollapse(storageKey, open);
  }
</script>

<div class="card">
  <div class="flex items-center gap-2 p-4">
    <button
      type="button"
      class="-m-1 flex flex-1 items-center gap-2 rounded p-1 text-left hover:bg-slate-50"
      onclick={toggle}
      aria-expanded={open}
    >
      <svg
        viewBox="0 0 20 20"
        fill="currentColor"
        class="h-4 w-4 shrink-0 text-slate-400 transition-transform duration-150 {open ? '' : '-rotate-90'}"
      >
        <path
          fill-rule="evenodd"
          d="M5.23 7.21a.75.75 0 011.06.02L10 11.17l3.71-3.94a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z"
          clip-rule="evenodd"
        />
      </svg>
      <h3 class="text-sm font-semibold text-slate-600">{title}</h3>
      {#if count !== undefined}
        <span class="rounded-full bg-slate-100 px-2 text-xs font-medium text-slate-500">{count}</span>
      {/if}
    </button>
    {#if actions}
      <div class="shrink-0">{@render actions()}</div>
    {/if}
  </div>
  {#if open}
    <div class="border-t border-slate-100 p-4">
      {@render children()}
    </div>
  {/if}
</div>
