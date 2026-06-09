<script lang="ts">
  import { getContext } from 'svelte';
  import { api } from '$lib/api';
  import { toasts } from '$lib/stores/toast.svelte';
  import Donut from '$lib/components/Donut.svelte';
  import BarList from '$lib/components/BarList.svelte';
  import {
    STATUS_LABELS,
    TYPE_LABELS,
    PRIORITY_LABELS,
    STATUS_COLORS,
    TYPE_COLORS,
    PRIORITY_COLORS,
    type ProjectStats,
    type Project
  } from '$lib/types';

  const ctx = getContext<{ project: Project | null }>('project');
  const projectId = $derived(ctx.project?.id ?? '');

  let stats = $state<ProjectStats | null>(null);
  let loadedFor = $state('');

  $effect(() => {
    if (projectId && loadedFor !== projectId) {
      loadedFor = projectId;
      void load();
    }
  });

  async function load() {
    try {
      stats = await api.get<ProjectStats>(`/api/projects/${projectId}/stats`);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Failed to load dashboard');
    }
  }

  const completion = $derived(stats && stats.total > 0 ? Math.round((stats.done / stats.total) * 100) : 0);
  const statusSeg = $derived(
    stats ? STATUS_LABELS.map((label, i) => ({ label, value: stats!.byStatus[i] ?? 0, color: STATUS_COLORS[i] })) : []
  );
  const prioBars = $derived(
    stats ? PRIORITY_LABELS.map((label, i) => ({ label, value: stats!.byPriority[i] ?? 0, color: PRIORITY_COLORS[i] })) : []
  );
  const typeBars = $derived(
    stats ? TYPE_LABELS.map((label, i) => ({ label, value: stats!.byType[i] ?? 0, color: TYPE_COLORS[i] })) : []
  );
  const assigneeBars = $derived(
    stats ? stats.byAssignee.map((a) => ({ label: a.displayName, value: a.count, color: '#6366f1' })) : []
  );
</script>

{#snippet tile(label: string, value: number | string, accent: string)}
  <div class="card p-4">
    <div class="text-2xl font-semibold {accent}">{value}</div>
    <div class="mt-0.5 text-xs text-slate-500">{label}</div>
  </div>
{/snippet}

{#if stats}
  <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
    {@render tile('Total issues', stats.total, 'text-slate-800')}
    {@render tile('Open', stats.open, 'text-sky-600')}
    {@render tile('Done', stats.done, 'text-emerald-600')}
    {@render tile('Completion', `${completion}%`, 'text-indigo-600')}
    {@render tile('Created (7d)', stats.createdLast7, 'text-slate-800')}
    {@render tile('Resolved (7d)', stats.resolvedLast7, 'text-emerald-600')}
  </div>

  <div class="mt-3 h-2 w-full overflow-hidden rounded-full bg-slate-100">
    <div class="h-full rounded-full bg-emerald-500 transition-all" style="width:{completion}%"></div>
  </div>

  <div class="mt-5 grid gap-5 lg:grid-cols-3">
    <div class="card p-5">
      <h3 class="mb-4 text-sm font-semibold text-slate-600">By status</h3>
      <Donut segments={statusSeg} centerLabel="issues" />
    </div>
    <div class="card p-5">
      <h3 class="mb-4 text-sm font-semibold text-slate-600">By priority</h3>
      <BarList items={prioBars} />
    </div>
    <div class="card p-5">
      <h3 class="mb-4 text-sm font-semibold text-slate-600">By type</h3>
      <BarList items={typeBars} />
    </div>
  </div>

  <div class="mt-5 grid gap-5 lg:grid-cols-2">
    <div class="card p-5">
      <div class="mb-4 flex items-center">
        <h3 class="text-sm font-semibold text-slate-600">Workload by assignee</h3>
        <span class="ml-auto text-xs text-slate-400">{stats.unassigned} unassigned</span>
      </div>
      {#if assigneeBars.length === 0}
        <p class="text-xs text-slate-400">No assigned issues yet.</p>
      {:else}
        <BarList items={assigneeBars} />
      {/if}
    </div>
    <div class="card flex flex-col justify-center p-5">
      <h3 class="mb-3 text-sm font-semibold text-slate-600">At a glance</h3>
      <p class="text-sm text-slate-600">
        <span class="font-semibold text-slate-800">{stats.open}</span> open ·
        <span class="font-semibold text-emerald-600">{stats.done}</span> done ·
        <span class="font-semibold text-amber-600">{stats.unassigned}</span> unassigned
      </p>
      <p class="mt-2 text-sm text-slate-600">
        {stats.createdLast7} created and {stats.resolvedLast7} resolved in the last 7 days.
      </p>
    </div>
  </div>
{:else}
  <p class="text-slate-400">Loading dashboard…</p>
{/if}
