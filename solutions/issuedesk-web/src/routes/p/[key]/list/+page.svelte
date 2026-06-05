<script lang="ts">
  import { getContext } from 'svelte';
  import { api } from '$lib/api';
  import { toasts } from '$lib/stores/toast.svelte';
  import {
    STATUS_LABELS,
    TYPE_META,
    PRIORITY_META,
    type IssueListItem,
    type IssueListResponse,
    type Project
  } from '$lib/types';

  const ctx = getContext<{ project: Project | null }>('project');
  let issues = $state<IssueListItem[]>([]);
  let loading = $state(true);
  let projectId = $derived(ctx.project?.id ?? '');

  $effect(() => {
    if (projectId) void load();
  });

  async function load() {
    loading = true;
    try {
      const res = await api.get<IssueListResponse>(
        `/api/projects/${projectId}/issues?pageSize=500`
      );
      issues = res.items;
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Failed to load');
    } finally {
      loading = false;
    }
  }
</script>

<div class="card overflow-hidden">
  <table class="w-full text-sm">
    <thead class="border-b border-slate-200 bg-slate-50 text-left text-xs uppercase text-slate-500">
      <tr>
        <th class="px-3 py-2">Key</th>
        <th class="px-3 py-2">Type</th>
        <th class="px-3 py-2">Title</th>
        <th class="px-3 py-2">Status</th>
        <th class="px-3 py-2">Priority</th>
        <th class="px-3 py-2">Assignee</th>
      </tr>
    </thead>
    <tbody>
      {#each issues as i (i.id)}
        <tr class="border-b border-slate-100 hover:bg-slate-50">
          <td class="px-3 py-2 font-mono text-xs">
            <a class="text-indigo-600 hover:underline" href={`/p/${ctx.project?.key}/issue/${i.number}`}>
              {i.key}
            </a>
          </td>
          <td class="px-3 py-2" title={TYPE_META[i.type].label}>{TYPE_META[i.type].icon}</td>
          <td class="px-3 py-2">
            <a class="hover:underline" href={`/p/${ctx.project?.key}/issue/${i.number}`}>{i.title}</a>
            {#if i.labels.length}
              <span class="ml-2 inline-flex gap-1 align-middle">
                {#each i.labels as l}<span class="label-chip" style={`background-color:${l.color}`}>{l.name}</span>{/each}
              </span>
            {/if}
          </td>
          <td class="px-3 py-2">{STATUS_LABELS[i.status]}</td>
          <td class="px-3 py-2">
            <span class="rounded px-1.5 py-0.5 text-xs {PRIORITY_META[i.priority].color}">
              {PRIORITY_META[i.priority].label}
            </span>
          </td>
          <td class="px-3 py-2 text-slate-600">{i.assigneeName ?? '—'}</td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if loading}
    <p class="p-4 text-center text-slate-400">Loading…</p>
  {:else if issues.length === 0}
    <p class="p-8 text-center text-slate-400">No issues.</p>
  {/if}
</div>
