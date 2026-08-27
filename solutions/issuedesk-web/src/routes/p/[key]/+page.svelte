<script lang="ts">
  import { getContext, onMount } from 'svelte';
  import { api } from '$lib/api';
  import { auth } from '$lib/stores/auth.svelte';
  import { toasts } from '$lib/stores/toast.svelte';
  import RichEditor from '$lib/editor/RichEditor.svelte';
  import Avatar from '$lib/components/Avatar.svelte';
  import IssueFilters, { type IssueFilterState } from '$lib/components/IssueFilters.svelte';
  import { ticketAging } from '$lib/aging';
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

  // How many cards to load per column at a time. The rest stay collapsed until
  // the user scrolls the column (or clicks "Show more").
  const PAGE = 10;

  interface Column {
    status: number;
    label: string;
    items: IssueListItem[];
    total: number;
    page: number; // last page loaded (0 = none yet)
    loading: boolean;
  }

  let cols = $state<Column[]>(
    STATUS_COLUMNS.map((c) => ({ status: c.value, label: c.label, items: [], total: 0, page: 0, loading: false }))
  );

  let labels = $state<Label[]>([]);
  let members = $state<Member[]>([]);
  let filters = $state<IssueFilterState>({ q: '', type: '', priority: '', assignee: '', label: '' });

  // new issue
  let showNew = $state(false);
  let nf = $state({ title: '', type: 1, priority: 1, assigneeId: '', description: '' });
  let creating = $state(false);
  let editorRef = $state<{
    getMarkdown(): string;
    hasPendingMedia(): boolean;
    uploadPending(issueId: string): Promise<void>;
  } | null>(null);

  // Customers can't move tickets — the board is read-only for them.
  const canDrag = $derived(!auth.isCustomer);

  // drag state
  let dragId = $state<string | null>(null);
  let dragFrom = $state<number | null>(null);
  let overCol = $state<number | null>(null);
  let overId = $state<string | null>(null);
  let overHalf = $state<0 | 1>(0); // 0 = insert above the hovered card, 1 = below

  // Card aging clock.
  let now = $state(Date.now());
  onMount(() => {
    const t = setInterval(() => (now = Date.now()), 60_000);
    return () => clearInterval(t);
  });

  let projectId = $derived(ctx.project?.id ?? '');

  function buildParams(col: Column): URLSearchParams {
    const p = new URLSearchParams();
    p.set('status', String(col.status));
    p.set('sort', 'board');
    p.set('page', String(col.page + 1));
    p.set('pageSize', String(PAGE));
    if (filters.assignee) p.set('assigneeId', filters.assignee);
    if (filters.type) p.set('type', filters.type);
    if (filters.priority) p.set('priority', filters.priority);
    if (filters.label) p.set('labelId', filters.label);
    if (filters.q) p.set('q', filters.q);
    return p;
  }

  async function loadColumn(col: Column, reset = false) {
    if (col.loading || !projectId) return;
    if (reset) {
      col.page = 0;
      col.items = [];
      col.total = 0;
    } else if (col.page > 0 && col.items.length >= col.total) {
      return; // fully loaded
    }
    col.loading = true;
    try {
      const res = await api.get<IssueListResponse>(`/api/projects/${projectId}/issues?${buildParams(col)}`);
      col.items = [...col.items, ...res.items];
      col.total = res.total;
      col.page += 1;
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Failed to load issues');
    } finally {
      col.loading = false;
    }
  }

  function reloadAll() {
    for (const col of cols) void loadColumn(col, true);
  }

  // Debounced reload for filter changes (typing in the search box).
  let debounce: ReturnType<typeof setTimeout>;
  function onFiltersChange() {
    clearTimeout(debounce);
    debounce = setTimeout(reloadAll, 250);
  }

  async function loadMeta() {
    if (!projectId) return;
    [labels, members] = await Promise.all([
      api.get<Label[]>(`/api/projects/${projectId}/labels`),
      api.get<Member[]>(`/api/projects/${projectId}/members`)
    ]);
  }

  let loadedFor = $state('');
  $effect(() => {
    if (projectId && loadedFor !== projectId) {
      loadedFor = projectId;
      void loadMeta();
      reloadAll();
    }
  });

  function onColScroll(e: Event, col: Column) {
    const el = e.currentTarget as HTMLElement;
    if (el.scrollHeight - el.scrollTop - el.clientHeight < 80) void loadColumn(col);
  }

  // ---- drag & drop reordering ----
  function onCardOver(e: DragEvent, status: number, id: string) {
    e.preventDefault();
    e.stopPropagation();
    if (!dragId) return;
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    overCol = status;
    overId = id;
    overHalf = e.clientY - rect.top < rect.height / 2 ? 0 : 1;
  }

  function clearDrag() {
    dragId = null;
    dragFrom = null;
    overCol = null;
    overId = null;
  }

  async function commitDrop(status: number) {
    if (!canDrag) return;
    const id = dragId;
    const from = dragFrom;
    const oId = overId;
    const half = overHalf;
    clearDrag();
    if (!id || from === null) return;

    const src = cols.find((c) => c.status === from)!;
    const dst = cols.find((c) => c.status === status)!;
    const moving = src.items.find((i) => i.id === id);
    if (!moving) return;
    if (oId === id) return; // dropped on itself

    // Destination order without the moving card; find where it lands.
    const dest = dst.items.filter((i) => i.id !== id);
    let insertAt = dest.length;
    if (oId) {
      const j = dest.findIndex((i) => i.id === oId);
      if (j >= 0) insertAt = half === 0 ? j : j + 1;
    }
    const beforeId = insertAt > 0 ? dest[insertAt - 1].id : null;
    const afterId = insertAt < dest.length ? dest[insertAt].id : null;

    // Optimistic reorder.
    src.items = src.items.filter((i) => i.id !== id);
    if (from !== status) {
      src.total = Math.max(0, src.total - 1);
      dst.total += 1;
    }
    moving.status = status;
    dest.splice(insertAt, 0, moving);
    dst.items = dest;

    try {
      await api.patch(`/api/issues/${id}/position`, { status, beforeId, afterId });
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Reorder failed');
      await loadColumn(src, true);
      if (dst !== src) await loadColumn(dst, true);
    }
  }

  async function createIssue(e: Event) {
    e.preventDefault();
    creating = true;
    try {
      const hasMedia = editorRef?.hasPendingMedia() ?? false;
      const created = await api.post<{ id: string }>(`/api/projects/${projectId}/issues`, {
        title: nf.title,
        type: nf.type,
        priority: nf.priority,
        assigneeId: nf.assigneeId || null,
        description: hasMedia ? null : nf.description || null
      });
      if (hasMedia && editorRef) {
        await editorRef.uploadPending(created.id);
        await api.patch(`/api/issues/${created.id}`, { description: editorRef.getMarkdown() || null });
      }
      toasts.success('Issue created');
      showNew = false;
      nf = { title: '', type: 1, priority: 1, assigneeId: '', description: '' };
      await loadColumn(cols[0], true); // new issues land at the top of To Do
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Create failed');
    } finally {
      creating = false;
    }
  }
</script>

<IssueFilters bind:filters {members} {labels} onchange={onFiltersChange}>
  <button class="btn-primary" onclick={() => (showNew = true)}>New issue</button>
</IssueFilters>

<!-- Board -->
<div class="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
  {#each cols as col (col.status)}
    <div
      class="flex min-h-[6rem] flex-col rounded-lg bg-slate-200/60 p-2 dark:bg-slate-800/60"
      role="group"
      aria-label={col.label}
      ondragover={(e) => {
        if (canDrag && dragId) {
          e.preventDefault();
          overCol = col.status;
          overId = null;
        }
      }}
      ondrop={(e) => {
        if (!canDrag) return;
        e.preventDefault();
        void commitDrop(col.status);
      }}
    >
      <div class="mb-2 flex items-center justify-between px-1 text-sm font-semibold text-slate-600 dark:text-slate-300">
        <span>{col.label}</span>
        <span class="rounded-full bg-white px-2 text-xs text-slate-500 dark:bg-slate-900 dark:text-slate-400">{col.total}</span>
      </div>

      <div
        class="flex max-h-[calc(100vh-16rem)] min-h-[2rem] flex-col gap-2 overflow-y-auto pr-0.5"
        onscroll={(e) => onColScroll(e, col)}
      >
        {#each col.items as issue (issue.id)}
          {@const aging = ticketAging(issue.statusSince, issue.status, col.label, now)}
          {@const showLine = overCol === col.status && overId === issue.id}
          <a
            href={`/p/${ctx.project?.key}/issue/${issue.number}`}
            class="card block p-3 {canDrag ? 'cursor-grab active:cursor-grabbing' : ''} {dragId === issue.id ? 'opacity-40' : ''} {showLine &&
            overHalf === 0
              ? 'shadow-[inset_0_3px_0_0_theme(colors.indigo.500)]'
              : ''} {showLine && overHalf === 1 ? 'shadow-[inset_0_-3px_0_0_theme(colors.indigo.500)]' : ''}"
            draggable={canDrag}
            ondragstart={() => {
              if (!canDrag) return;
              dragId = issue.id;
              dragFrom = col.status;
            }}
            ondragover={(e) => canDrag && onCardOver(e, col.status, issue.id)}
            ondragend={clearDrag}
          >
            <div class="flex items-center gap-2 text-xs text-slate-400 dark:text-slate-500">
              <span class={TYPE_META[issue.type].color} title={TYPE_META[issue.type].label}>
                {TYPE_META[issue.type].icon}
              </span>
              <span class="font-mono">{issue.key}</span>
              {#if aging}
                <span
                  class="inline-flex items-center gap-0.5 rounded px-1.5 py-0.5 text-[10px] font-medium {aging.level ===
                  'stale'
                    ? 'bg-rose-100 text-rose-700'
                    : 'bg-amber-100 text-amber-700'}"
                  title={aging.title}
                >
                  ⏱ {aging.label}
                </span>
              {/if}
              <span
                class="ml-auto rounded px-1.5 py-0.5 text-[10px] font-medium {PRIORITY_META[issue.priority].color}"
              >
                {PRIORITY_META[issue.priority].label}
              </span>
            </div>
            <p class="mt-1.5 text-sm font-medium text-slate-800 dark:text-slate-100">{issue.title}</p>
            {#if issue.labels.length}
              <div class="mt-2 flex flex-wrap gap-1">
                {#each issue.labels as l}
                  <span class="label-chip" style={`background-color:${l.color}`}>{l.name}</span>
                {/each}
              </div>
            {/if}
            {#if issue.assigneeName}
              <div class="mt-2 flex items-center gap-1.5 text-xs text-slate-500 dark:text-slate-400">
                <Avatar seed={issue.assigneeId ?? issue.assigneeName} name={issue.assigneeName} size={18} />
                {issue.assigneeName}
              </div>
            {/if}
          </a>
        {/each}

        {#if col.loading}
          <p class="py-2 text-center text-xs text-slate-400 dark:text-slate-500">Loading…</p>
        {:else if col.items.length < col.total}
          <button
            type="button"
            class="rounded-md border border-dashed border-slate-300 py-1.5 text-xs font-medium text-slate-500 hover:bg-white/60 dark:border-slate-600 dark:text-slate-400"
            onclick={() => loadColumn(col)}
          >
            Show {col.total - col.items.length} more
          </button>
        {/if}
      </div>
    </div>
  {/each}
</div>

<!-- New issue slide-over -->
{#if showNew}
  <div class="fixed inset-0 z-40 bg-black/30" role="presentation" onclick={() => (showNew = false)}></div>
  <div class="fixed right-0 top-0 z-50 h-full w-full max-w-md overflow-y-auto bg-white p-5 shadow-xl dark:bg-slate-900">
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
      {#if !auth.isCustomer}
        <div>
          <label class="mb-1 block text-sm font-medium" for="as">Assignee</label>
          <select id="as" class="input" bind:value={nf.assigneeId}>
            <option value="">Unassigned</option>
            {#each members as m}<option value={m.userId}>{m.displayName}</option>{/each}
          </select>
        </div>
      {/if}
      <div>
        <span class="mb-1 block text-sm font-medium">Description</span>
        <RichEditor
          bind:this={editorRef}
          editable
          placeholder="Describe the issue… paste or drop images/videos"
          onChange={(md) => (nf.description = md)}
        />
      </div>
      <div class="flex gap-2 pt-2">
        <button class="btn-primary" disabled={creating}>Create</button>
        <button type="button" class="btn-ghost" onclick={() => (showNew = false)}>Cancel</button>
      </div>
    </form>
  </div>
{/if}
