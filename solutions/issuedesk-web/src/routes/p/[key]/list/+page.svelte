<script lang="ts">
  import { getContext } from 'svelte';
  import { api } from '$lib/api';
  import { toasts } from '$lib/stores/toast.svelte';
  import { onVisible } from '$lib/actions/onVisible';
  import IssueFilters, { type IssueFilterState } from '$lib/components/IssueFilters.svelte';
  import {
    STATUS_LABELS,
    TYPE_META,
    PRIORITY_META,
    type IssueListItem,
    type IssueListResponse,
    type Label,
    type Member,
    type Project
  } from '$lib/types';

  const ctx = getContext<{ project: Project | null }>('project');
  const PAGE = 30;

  let issues = $state<IssueListItem[]>([]);
  let total = $state(0);
  let page = $state(0);
  let loading = $state(false);
  let labels = $state<Label[]>([]);
  let members = $state<Member[]>([]);
  let filters = $state<IssueFilterState>({ q: '', type: '', priority: '', assignee: '', label: '' });

  let projectId = $derived(ctx.project?.id ?? '');
  let hasMore = $derived(page === 0 || issues.length < total);

  function buildParams(): URLSearchParams {
    const p = new URLSearchParams();
    p.set('page', String(page + 1));
    p.set('pageSize', String(PAGE));
    if (filters.assignee) p.set('assigneeId', filters.assignee);
    if (filters.type) p.set('type', filters.type);
    if (filters.priority) p.set('priority', filters.priority);
    if (filters.label) p.set('labelId', filters.label);
    if (filters.q) p.set('q', filters.q);
    return p;
  }

  async function load(reset = false) {
    if (loading || !projectId) return;
    if (reset) {
      page = 0;
      issues = [];
      total = 0;
    } else if (page > 0 && issues.length >= total) {
      return;
    }
    loading = true;
    try {
      const res = await api.get<IssueListResponse>(`/api/projects/${projectId}/issues?${buildParams()}`);
      issues = [...issues, ...res.items];
      total = res.total;
      page += 1;
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Failed to load');
    } finally {
      loading = false;
    }
  }

  let debounce: ReturnType<typeof setTimeout>;
  function onFiltersChange() {
    clearTimeout(debounce);
    debounce = setTimeout(() => load(true), 250);
  }

  let loadedFor = $state('');
  $effect(() => {
    if (projectId && loadedFor !== projectId) {
      loadedFor = projectId;
      void Promise.all([
        api.get<Label[]>(`/api/projects/${projectId}/labels`).then((l) => (labels = l)),
        api.get<Member[]>(`/api/projects/${projectId}/members`).then((m) => (members = m))
      ]);
      void load(true);
    }
  });
</script>

<IssueFilters bind:filters {members} {labels} onchange={onFiltersChange} />

<div class="card overflow-hidden">
  <table class="w-full text-sm">
    <thead class="border-b border-slate-200 bg-slate-50 text-left text-xs uppercase text-slate-500 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-400">
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
        <tr class="border-b border-slate-100 hover:bg-slate-50 dark:border-slate-800 dark:hover:bg-slate-900">
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
          <td class="px-3 py-2 text-slate-600 dark:text-slate-300">{i.assigneeName ?? '—'}</td>
        </tr>
      {/each}
    </tbody>
  </table>

  {#if loading}
    <p class="p-4 text-center text-slate-400 dark:text-slate-500">Loading…</p>
  {:else if issues.length === 0}
    <p class="p-8 text-center text-slate-400 dark:text-slate-500">No issues.</p>
  {/if}

  <!-- Infinite-scroll sentinel: loads the next page when it nears the viewport. -->
  {#if hasMore && issues.length > 0}
    <div use:onVisible={() => load()} class="h-1"></div>
  {/if}
</div>

{#if total > 0}
  <p class="mt-2 text-center text-xs text-slate-400 dark:text-slate-500">Showing {issues.length} of {total}</p>
{/if}
