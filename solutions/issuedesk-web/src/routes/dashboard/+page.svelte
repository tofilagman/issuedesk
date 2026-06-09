<script lang="ts">
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
    type SystemStats
  } from '$lib/types';

  let stats = $state<SystemStats | null>(null);

  $effect(() => {
    void load();
  });

  async function load() {
    try {
      stats = await api.get<SystemStats>('/api/stats');
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Failed to load dashboard');
    }
  }

  const completion = $derived(stats && stats.issues > 0 ? Math.round((stats.done / stats.issues) * 100) : 0);
  const statusSeg = $derived(
    stats ? STATUS_LABELS.map((label, i) => ({ label, value: stats!.byStatus[i] ?? 0, color: STATUS_COLORS[i] })) : []
  );
  const prioBars = $derived(
    stats ? PRIORITY_LABELS.map((label, i) => ({ label, value: stats!.byPriority[i] ?? 0, color: PRIORITY_COLORS[i] })) : []
  );
  const typeBars = $derived(
    stats ? TYPE_LABELS.map((label, i) => ({ label, value: stats!.byType[i] ?? 0, color: TYPE_COLORS[i] })) : []
  );
  function pct(done: number, total: number) {
    return total > 0 ? Math.round((done / total) * 100) : 0;
  }
</script>

{#snippet tile(label: string, value: number | string, accent: string)}
  <div class="card p-4">
    <div class="text-2xl font-semibold {accent}">{value}</div>
    <div class="mt-0.5 text-xs text-slate-500">{label}</div>
  </div>
{/snippet}

<h1 class="mb-4 text-xl font-semibold">Dashboard</h1>

{#if stats}
  <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-6">
    {@render tile('Projects', stats.projects, 'text-slate-800')}
    {@render tile('Issues', stats.issues, 'text-slate-800')}
    {@render tile('Open', stats.open, 'text-sky-600')}
    {@render tile('Done', stats.done, 'text-emerald-600')}
    {@render tile('Completion', `${completion}%`, 'text-indigo-600')}
    {@render tile('Active users', stats.users, 'text-slate-800')}
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

  <div class="mt-5 card p-5">
    <div class="mb-3 flex items-center">
      <h3 class="text-sm font-semibold text-slate-600">Top projects</h3>
      <span class="ml-auto text-xs text-slate-400">{stats.createdLast7} issues created in the last 7 days</span>
    </div>
    {#if stats.topProjects.length === 0}
      <p class="text-xs text-slate-400">No projects yet.</p>
    {:else}
      <ul class="space-y-3">
        {#each stats.topProjects as p (p.key)}
          <li>
            <div class="mb-1 flex items-center gap-2 text-sm">
              <span class="rounded bg-indigo-100 px-1.5 py-0.5 font-mono text-xs font-bold text-indigo-700">{p.key}</span>
              <a href={`/p/${p.key}/dashboard`} class="truncate text-slate-700 hover:underline">{p.name}</a>
              <span class="ml-auto text-xs text-slate-500">{p.done}/{p.total} done</span>
            </div>
            <div class="h-2 w-full overflow-hidden rounded-full bg-slate-100">
              <div class="h-full rounded-full bg-emerald-500" style="width:{pct(p.done, p.total)}%"></div>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{:else}
  <p class="text-slate-400">Loading dashboard…</p>
{/if}
