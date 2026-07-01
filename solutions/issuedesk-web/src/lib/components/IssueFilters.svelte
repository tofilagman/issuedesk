<script lang="ts">
  import type { Snippet } from 'svelte';
  import { TYPE_LABELS, PRIORITY_LABELS, type Label, type Member } from '$lib/types';

  export interface IssueFilterState {
    q: string;
    type: string;
    priority: string;
    assignee: string;
    label: string;
  }

  let {
    filters = $bindable(),
    members,
    labels,
    onchange,
    children
  }: {
    filters: IssueFilterState;
    members: Member[];
    labels: Label[];
    /** Fired after any filter changes; parent reloads. */
    onchange: () => void;
    /** Optional trailing content (e.g. a "New issue" button), right-aligned. */
    children?: Snippet;
  } = $props();
</script>

<div class="mb-4 flex flex-wrap items-center gap-2">
  <input class="input max-w-xs" placeholder="Search title or key…" bind:value={filters.q} oninput={onchange} />
  <select class="input max-w-[10rem]" bind:value={filters.type} onchange={onchange}>
    <option value="">All types</option>
    {#each TYPE_LABELS as t, i}<option value={String(i)}>{t}</option>{/each}
  </select>
  <select class="input max-w-[10rem]" bind:value={filters.priority} onchange={onchange}>
    <option value="">All priorities</option>
    {#each PRIORITY_LABELS as p, i}<option value={String(i)}>{p}</option>{/each}
  </select>
  <select class="input max-w-[12rem]" bind:value={filters.assignee} onchange={onchange}>
    <option value="">All assignees</option>
    {#each members as m}<option value={m.userId}>{m.displayName}</option>{/each}
  </select>
  <select class="input max-w-[10rem]" bind:value={filters.label} onchange={onchange}>
    <option value="">All labels</option>
    {#each labels as l}<option value={l.id}>{l.name}</option>{/each}
  </select>
  {#if children}
    <div class="ml-auto">{@render children()}</div>
  {/if}
</div>
