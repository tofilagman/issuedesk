<script lang="ts">
  import { getContext } from 'svelte';
  import { api } from '$lib/api';
  import { toasts } from '$lib/stores/toast.svelte';
  import {
    STATUS_COLUMNS,
    TYPE_META,
    PRIORITY_META,
    TYPE_LABELS,
    PRIORITY_LABELS,
    type IssueListItem,
    type IssueListResponse,
    type Label,
    type Member,
    type Project
  } from '$lib/types';

  const ctx = getContext<{ project: Project | null }>('project');

  let issues = $state<IssueListItem[]>([]);
  let labels = $state<Label[]>([]);
  let members = $state<Member[]>([]);
  let loading = $state(true);

  // filters
  let fAssignee = $state('');
  let fType = $state('');
  let fPriority = $state('');
  let fLabel = $state('');
  let fQ = $state('');

  // new issue
  let showNew = $state(false);
  let nf = $state({ title: '', type: 1, priority: 1, assigneeId: '', description: '' });
  let creating = $state(false);

  let dragId = $state<string | null>(null);

  let projectId = $derived(ctx.project?.id ?? '');

  async function load() {
    if (!projectId) return;
    loading = true;
    try {
      const params = new URLSearchParams();
      if (fAssignee) params.set('assigneeId', fAssignee);
      if (fType) params.set('type', fType);
      if (fPriority) params.set('priority', fPriority);
      if (fLabel) params.set('labelId', fLabel);
      if (fQ) params.set('q', fQ);
      params.set('pageSize', '500');
      const res = await api.get<IssueListResponse>(`/api/projects/${projectId}/issues?${params}`);
      issues = res.items;
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Failed to load issues');
    } finally {
      loading = false;
    }
  }

  async function loadMeta() {
    if (!projectId) return;
    [labels, members] = await Promise.all([
      api.get<Label[]>(`/api/projects/${projectId}/labels`),
      api.get<Member[]>(`/api/projects/${projectId}/members`)
    ]);
  }

  // Reload whenever the project resolves or a filter changes.
  $effect(() => {
    if (projectId) {
      void load();
    }
  });
  let metaLoaded = $state('');
  $effect(() => {
    if (projectId && metaLoaded !== projectId) {
      metaLoaded = projectId;
      void loadMeta();
    }
  });

  function byStatus(s: number) {
    return issues.filter((i) => i.status === s);
  }

  async function onDrop(status: number) {
    const id = dragId;
    dragId = null;
    if (!id) return;
    const issue = issues.find((i) => i.id === id);
    if (!issue || issue.status === status) return;
    const prev = issue.status;
    issue.status = status; // optimistic
    try {
      await api.patch(`/api/issues/${id}`, { status });
    } catch (e) {
      issue.status = prev;
      toasts.error(e instanceof Error ? e.message : 'Move failed');
    }
  }

  async function createIssue(e: Event) {
    e.preventDefault();
    creating = true;
    try {
      await api.post(`/api/projects/${projectId}/issues`, {
        title: nf.title,
        type: nf.type,
        priority: nf.priority,
        assigneeId: nf.assigneeId || null,
        description: nf.description || null
      });
      toasts.success('Issue created');
      showNew = false;
      nf = { title: '', type: 1, priority: 1, assigneeId: '', description: '' };
      await load();
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Create failed');
    } finally {
      creating = false;
    }
  }
</script>

<!-- Filters -->
<div class="mb-4 flex flex-wrap items-center gap-2">
  <input class="input max-w-xs" placeholder="Search title…" bind:value={fQ} oninput={load} />
  <select class="input max-w-[10rem]" bind:value={fType} onchange={load}>
    <option value="">All types</option>
    {#each TYPE_LABELS as t, i}<option value={i}>{t}</option>{/each}
  </select>
  <select class="input max-w-[10rem]" bind:value={fPriority} onchange={load}>
    <option value="">All priorities</option>
    {#each PRIORITY_LABELS as p, i}<option value={i}>{p}</option>{/each}
  </select>
  <select class="input max-w-[12rem]" bind:value={fAssignee} onchange={load}>
    <option value="">All assignees</option>
    {#each members as m}<option value={m.userId}>{m.displayName}</option>{/each}
  </select>
  <select class="input max-w-[10rem]" bind:value={fLabel} onchange={load}>
    <option value="">All labels</option>
    {#each labels as l}<option value={l.id}>{l.name}</option>{/each}
  </select>
  <button class="btn-primary ml-auto" onclick={() => (showNew = true)}>New issue</button>
</div>

<!-- Board -->
<div class="grid grid-cols-1 gap-3 md:grid-cols-2 xl:grid-cols-4">
  {#each STATUS_COLUMNS as col (col.value)}
    <div
      class="flex flex-col rounded-lg bg-slate-200/60 p-2"
      role="group"
      aria-label={col.label}
      ondragover={(e) => e.preventDefault()}
      ondrop={() => onDrop(col.value)}
    >
      <div class="mb-2 flex items-center justify-between px-1 text-sm font-semibold text-slate-600">
        <span>{col.label}</span>
        <span class="rounded-full bg-white px-2 text-xs text-slate-500">{byStatus(col.value).length}</span>
      </div>
      <div class="flex min-h-[3rem] flex-col gap-2">
        {#each byStatus(col.value) as issue (issue.id)}
          <a
            href={`/p/${ctx.project?.key}/issue/${issue.number}`}
            class="card block cursor-grab p-3 active:cursor-grabbing"
            draggable="true"
            ondragstart={() => (dragId = issue.id)}
          >
            <div class="flex items-center gap-2 text-xs text-slate-400">
              <span class={TYPE_META[issue.type].color} title={TYPE_META[issue.type].label}>
                {TYPE_META[issue.type].icon}
              </span>
              <span class="font-mono">{issue.key}</span>
              <span
                class="ml-auto rounded px-1.5 py-0.5 text-[10px] font-medium {PRIORITY_META[issue.priority].color}"
              >
                {PRIORITY_META[issue.priority].label}
              </span>
            </div>
            <p class="mt-1.5 text-sm font-medium text-slate-800">{issue.title}</p>
            {#if issue.labels.length}
              <div class="mt-2 flex flex-wrap gap-1">
                {#each issue.labels as l}
                  <span class="label-chip" style={`background-color:${l.color}`}>{l.name}</span>
                {/each}
              </div>
            {/if}
            {#if issue.assigneeName}
              <p class="mt-2 text-xs text-slate-500">👤 {issue.assigneeName}</p>
            {/if}
          </a>
        {/each}
      </div>
    </div>
  {/each}
</div>

{#if loading}
  <p class="mt-4 text-center text-slate-400">Loading…</p>
{/if}

<!-- New issue slide-over -->
{#if showNew}
  <div class="fixed inset-0 z-40 bg-black/30" role="presentation" onclick={() => (showNew = false)}></div>
  <div class="fixed right-0 top-0 z-50 h-full w-full max-w-md overflow-y-auto bg-white p-5 shadow-xl">
    <h2 class="mb-4 text-lg font-semibold">New issue</h2>
    <form onsubmit={createIssue} class="space-y-3">
      <div>
        <label class="mb-1 block text-sm font-medium" for="t">Title</label>
        <input id="t" class="input" bind:value={nf.title} required />
      </div>
      <div class="grid grid-cols-2 gap-3">
        <div>
          <label class="mb-1 block text-sm font-medium" for="ty">Type</label>
          <select id="ty" class="input" bind:value={nf.type}>
            {#each TYPE_LABELS as t, i}<option value={i}>{t}</option>{/each}
          </select>
        </div>
        <div>
          <label class="mb-1 block text-sm font-medium" for="pr">Priority</label>
          <select id="pr" class="input" bind:value={nf.priority}>
            {#each PRIORITY_LABELS as p, i}<option value={i}>{p}</option>{/each}
          </select>
        </div>
      </div>
      <div>
        <label class="mb-1 block text-sm font-medium" for="as">Assignee</label>
        <select id="as" class="input" bind:value={nf.assigneeId}>
          <option value="">Unassigned</option>
          {#each members as m}<option value={m.userId}>{m.displayName}</option>{/each}
        </select>
      </div>
      <div>
        <label class="mb-1 block text-sm font-medium" for="de">Description</label>
        <textarea id="de" class="input min-h-28" bind:value={nf.description}></textarea>
      </div>
      <div class="flex gap-2 pt-2">
        <button class="btn-primary" disabled={creating}>Create</button>
        <button type="button" class="btn-ghost" onclick={() => (showNew = false)}>Cancel</button>
      </div>
    </form>
  </div>
{/if}
