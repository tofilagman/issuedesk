<script lang="ts">
  import { getContext } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { api, loadToken } from '$lib/api';
  import { auth } from '$lib/stores/auth.svelte';
  import { toasts } from '$lib/stores/toast.svelte';
  import {
    STATUS_LABELS,
    TYPE_LABELS,
    PRIORITY_LABELS,
    PRIORITY_META,
    type Activity,
    type Attachment,
    type Comment,
    type IssueDetail,
    type Label,
    type Member,
    type Project
  } from '$lib/types';

  const ctx = getContext<{ project: Project | null }>('project');
  const number = $derived(Number(page.params.number));

  let issue = $state<IssueDetail | null>(null);
  let comments = $state<Comment[]>([]);
  let attachments = $state<Attachment[]>([]);
  let activity = $state<Activity[]>([]);
  let labels = $state<Label[]>([]);
  let members = $state<Member[]>([]);
  let newComment = $state('');
  let loadedKey = $state('');

  let projectId = $derived(ctx.project?.id ?? '');

  $effect(() => {
    const k = `${projectId}:${number}`;
    if (projectId && number && k !== loadedKey) {
      loadedKey = k;
      void loadAll();
    }
  });

  async function loadAll() {
    try {
      issue = await api.get<IssueDetail>(`/api/projects/${projectId}/issues/${number}`);
      const id = issue.id;
      [comments, attachments, activity, labels, members] = await Promise.all([
        api.get<Comment[]>(`/api/issues/${id}/comments`),
        api.get<Attachment[]>(`/api/issues/${id}/attachments`),
        api.get<Activity[]>(`/api/issues/${id}/activity`),
        api.get<Label[]>(`/api/projects/${projectId}/labels`),
        api.get<Member[]>(`/api/projects/${projectId}/members`)
      ]);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Failed to load issue');
    }
  }

  async function patch(body: Record<string, unknown>) {
    if (!issue) return;
    try {
      issue = await api.patch<IssueDetail>(`/api/issues/${issue.id}`, body);
      activity = await api.get<Activity[]>(`/api/issues/${issue.id}/activity`);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Update failed');
    }
  }

  async function addComment(e: Event) {
    e.preventDefault();
    if (!issue || !newComment.trim()) return;
    try {
      await api.post(`/api/issues/${issue.id}/comments`, { body: newComment });
      newComment = '';
      [comments, activity] = await Promise.all([
        api.get<Comment[]>(`/api/issues/${issue.id}/comments`),
        api.get<Activity[]>(`/api/issues/${issue.id}/activity`)
      ]);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Comment failed');
    }
  }

  const labelIdsOnIssue = $derived(new Set(issue?.labels.map((l) => l.id) ?? []));
  async function toggleLabel(l: Label) {
    if (!issue) return;
    try {
      if (labelIdsOnIssue.has(l.id)) {
        await api.del(`/api/issues/${issue.id}/labels/${l.id}`);
      } else {
        await api.post(`/api/issues/${issue.id}/labels`, { labelId: l.id });
      }
      issue = await api.get<IssueDetail>(`/api/issues/${issue.id}`);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Label update failed');
    }
  }

  let uploading = $state(false);
  async function uploadFile(e: Event) {
    const input = e.target as HTMLInputElement;
    if (!issue || !input.files?.length) return;
    const fd = new FormData();
    fd.append('file', input.files[0]);
    uploading = true;
    try {
      await api.upload(`/api/issues/${issue.id}/attachments`, fd);
      [attachments, activity] = await Promise.all([
        api.get<Attachment[]>(`/api/issues/${issue.id}/attachments`),
        api.get<Activity[]>(`/api/issues/${issue.id}/activity`)
      ]);
      input.value = '';
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Upload failed');
    } finally {
      uploading = false;
    }
  }

  // Token can't ride in a plain <a href>; fetch as a blob then trigger download.
  async function download(att: Attachment) {
    try {
      const res = await fetch(api.downloadUrl(att.id), {
        headers: { Authorization: `Bearer ${loadToken()}`, 'Sec-Fetch-Site': 'same-origin' }
      });
      if (!res.ok) throw new Error('download failed');
      const blob = await res.blob();
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = att.filename;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Download failed');
    }
  }

  async function deleteAttachment(att: Attachment) {
    if (!issue) return;
    try {
      await api.del(`/api/attachments/${att.id}`);
      attachments = await api.get<Attachment[]>(`/api/issues/${issue.id}/attachments`);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Delete failed');
    }
  }

  async function deleteIssue() {
    if (!issue || !confirm('Delete this issue permanently?')) return;
    try {
      await api.del(`/api/issues/${issue.id}`);
      toasts.success('Issue deleted');
      goto(`/p/${ctx.project?.key}`);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Delete failed');
    }
  }

  function actionText(a: Activity): string {
    switch (a.action) {
      case 0: return 'created this issue';
      case 1: return `changed status ${STATUS_LABELS[Number(a.oldValue)]} → ${STATUS_LABELS[Number(a.newValue)]}`;
      case 2: return 'changed the assignee';
      case 3: return 'commented';
      case 4: return 'added a label';
      case 5: return 'removed a label';
      case 6: return `attached ${a.newValue}`;
      case 7: return `removed attachment ${a.oldValue}`;
      case 8: return `changed priority ${PRIORITY_LABELS[Number(a.oldValue)]} → ${PRIORITY_LABELS[Number(a.newValue)]}`;
      case 9: return `changed type ${TYPE_LABELS[Number(a.oldValue)]} → ${TYPE_LABELS[Number(a.newValue)]}`;
      case 10: return 'edited the title';
      default: return 'updated the issue';
    }
  }

  function fmt(ts: string) {
    return new Date(ts).toLocaleString();
  }
  function fmtSize(n: number) {
    return n < 1024 ? `${n} B` : n < 1048576 ? `${(n / 1024).toFixed(1)} KB` : `${(n / 1048576).toFixed(1)} MB`;
  }
</script>

{#if issue}
  <a href={`/p/${ctx.project?.key}`} class="text-sm text-indigo-600 hover:underline">← Board</a>

  <div class="mt-2 grid gap-5 lg:grid-cols-[1fr_300px]">
    <!-- Main column -->
    <div class="space-y-5">
      <div class="card p-5">
        <div class="flex items-center gap-2 text-sm text-slate-400">
          <span class="font-mono">{issue.key}</span>
          <button class="btn-danger ml-auto !py-1 !text-xs" onclick={deleteIssue}>Delete</button>
        </div>
        <h1 class="mt-1 text-2xl font-semibold">{issue.title}</h1>
        <p class="mt-3 whitespace-pre-wrap text-sm text-slate-700">
          {issue.description || 'No description.'}
        </p>
      </div>

      <!-- Labels -->
      <div class="card p-5">
        <h3 class="mb-2 text-sm font-semibold text-slate-600">Labels</h3>
        {#if labels.length === 0}
          <p class="text-xs text-slate-400">No labels defined for this project yet.</p>
        {:else}
          <div class="flex flex-wrap gap-2">
            {#each labels as l (l.id)}
              <button
                class="label-chip {labelIdsOnIssue.has(l.id) ? '' : 'opacity-40'}"
                style={`background-color:${l.color}`}
                onclick={() => toggleLabel(l)}
              >
                {l.name}
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <!-- Attachments -->
      <div class="card p-5">
        <div class="mb-2 flex items-center">
          <h3 class="text-sm font-semibold text-slate-600">Attachments</h3>
          <label class="btn-ghost ml-auto cursor-pointer text-xs">
            {uploading ? 'Uploading…' : '+ Upload'}
            <input type="file" class="hidden" onchange={uploadFile} disabled={uploading} />
          </label>
        </div>
        {#if attachments.length === 0}
          <p class="text-xs text-slate-400">No attachments.</p>
        {:else}
          <ul class="divide-y divide-slate-100 text-sm">
            {#each attachments as a (a.id)}
              <li class="flex items-center gap-2 py-1.5">
                <button class="text-indigo-600 hover:underline" onclick={() => download(a)}>{a.filename}</button>
                <span class="text-xs text-slate-400">{fmtSize(a.sizeBytes)}</span>
                <button class="btn-ghost ml-auto !py-0.5 !text-xs text-rose-600" onclick={() => deleteAttachment(a)}>✕</button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>

      <!-- Comments -->
      <div class="card p-5">
        <h3 class="mb-3 text-sm font-semibold text-slate-600">Comments</h3>
        <div class="space-y-3">
          {#each comments as c (c.id)}
            <div class="rounded-md bg-slate-50 p-3">
              <div class="flex items-center gap-2 text-xs text-slate-500">
                <span class="font-medium text-slate-700">{c.authorName}</span>
                <span>{fmt(c.createdAt)}</span>
              </div>
              <p class="mt-1 whitespace-pre-wrap text-sm">{c.body}</p>
            </div>
          {/each}
          {#if comments.length === 0}
            <p class="text-xs text-slate-400">No comments yet.</p>
          {/if}
        </div>
        <form onsubmit={addComment} class="mt-3">
          <textarea class="input min-h-20" placeholder="Add a comment…" bind:value={newComment}></textarea>
          <button class="btn-primary mt-2">Comment</button>
        </form>
      </div>
    </div>

    <!-- Sidebar: fields + activity -->
    <div class="space-y-5">
      <div class="card space-y-3 p-4 text-sm">
        <div>
          <span class="mb-1 block text-xs font-medium text-slate-500">Status</span>
          <select class="input" value={issue.status} onchange={(e) => patch({ status: Number((e.target as HTMLSelectElement).value) })}>
            {#each STATUS_LABELS as s, i}<option value={i}>{s}</option>{/each}
          </select>
        </div>
        <div>
          <span class="mb-1 block text-xs font-medium text-slate-500">Assignee</span>
          <select class="input" value={issue.assigneeId ?? ''} onchange={(e) => patch({ assigneeId: (e.target as HTMLSelectElement).value || null })}>
            <option value="">Unassigned</option>
            {#each members as m}<option value={m.userId}>{m.displayName}</option>{/each}
          </select>
        </div>
        <div>
          <span class="mb-1 block text-xs font-medium text-slate-500">Priority</span>
          <select class="input" value={issue.priority} onchange={(e) => patch({ priority: Number((e.target as HTMLSelectElement).value) })}>
            {#each PRIORITY_LABELS as p, i}<option value={i}>{p}</option>{/each}
          </select>
        </div>
        <div>
          <span class="mb-1 block text-xs font-medium text-slate-500">Type</span>
          <select class="input" value={issue.type} onchange={(e) => patch({ type: Number((e.target as HTMLSelectElement).value) })}>
            {#each TYPE_LABELS as t, i}<option value={i}>{t}</option>{/each}
          </select>
        </div>
        <div class="border-t border-slate-100 pt-2 text-xs text-slate-500">
          Reporter: {issue.reporterName}<br />
          Created: {fmt(issue.createdAt)}
        </div>
      </div>

      <div class="card p-4">
        <h3 class="mb-3 text-sm font-semibold text-slate-600">Activity</h3>
        <ol class="space-y-3 text-xs">
          {#each activity as a (a.id)}
            <li class="flex gap-2">
              <span class="mt-0.5 h-1.5 w-1.5 shrink-0 rounded-full bg-indigo-400"></span>
              <div>
                <span class="font-medium text-slate-700">{a.actorName}</span>
                <span class="text-slate-500"> {actionText(a)}</span>
                <div class="text-slate-400">{fmt(a.createdAt)}</div>
              </div>
            </li>
          {/each}
        </ol>
      </div>
    </div>
  </div>
{:else}
  <p class="text-slate-400">Loading issue…</p>
{/if}
