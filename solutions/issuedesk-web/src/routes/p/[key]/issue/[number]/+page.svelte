<script lang="ts">
  import { getContext } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import { api, loadToken } from '$lib/api';
  import { auth } from '$lib/stores/auth.svelte';
  import { toasts } from '$lib/stores/toast.svelte';
  import { confirmDialog } from '$lib/stores/confirm.svelte';
  import RichEditor from '$lib/editor/RichEditor.svelte';
  import AttachmentThumb from '$lib/components/AttachmentThumb.svelte';
  import CollapsibleCard from '$lib/components/CollapsibleCard.svelte';
  import Avatar from '$lib/components/Avatar.svelte';
  import { loadCollapse, saveCollapse } from '$lib/collapse';
  import {
    STATUS_LABELS,
    TYPE_LABELS,
    PRIORITY_LABELS,
    PRIORITY_META,
    LINK_TYPE_LABELS,
    LINK_TYPE_ORDER,
    type Activity,
    type Attachment,
    type Comment,
    type IssueDetail,
    type IssueLink,
    type IssueListItem,
    type IssueListResponse,
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
  let links = $state<IssueLink[]>([]);
  let loadedKey = $state('');

  // Link-issue modal
  let showLink = $state(false);
  let linkType = $state(1); // default: Blocks
  let linkQuery = $state('');
  let linkResults = $state<IssueListItem[]>([]);
  let searching = $state(false);

  // Inline title edit
  let editingTitle = $state(false);
  let titleDraft = $state('');

  // Description edit
  let editingDesc = $state(false);
  let descDraft = $state('');

  // New comment
  let newComment = $state('');
  let commentKey = $state(0); // bump to remount (clear) the comment editor
  let posting = $state(false);

  // Comment edit
  let editingCommentId = $state<string | null>(null);
  let editCommentDraft = $state('');

  // Description collapse (persisted)
  let descOpen = $state(loadCollapse('description', true));
  function toggleDesc() {
    descOpen = !descOpen;
    saveCollapse('description', descOpen);
  }

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
      [comments, attachments, activity, labels, members, links] = await Promise.all([
        api.get<Comment[]>(`/api/issues/${id}/comments`),
        api.get<Attachment[]>(`/api/issues/${id}/attachments`),
        api.get<Activity[]>(`/api/issues/${id}/activity`),
        api.get<Label[]>(`/api/projects/${projectId}/labels`),
        api.get<Member[]>(`/api/projects/${projectId}/members`),
        api.get<IssueLink[]>(`/api/issues/${id}/links`)
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

  async function refreshAttachments() {
    if (!issue) return;
    attachments = await api.get<Attachment[]>(`/api/issues/${issue.id}/attachments`);
  }

  // ---- title ----
  function startEditTitle() {
    if (!issue) return;
    titleDraft = issue.title;
    editingTitle = true;
  }
  async function saveTitle() {
    if (!issue) return;
    const t = titleDraft.trim();
    editingTitle = false;
    if (!t || t === issue.title) return;
    await patch({ title: t });
  }

  // ---- description ----
  function startEditDesc() {
    if (!issue) return;
    descDraft = issue.description ?? '';
    editingDesc = true;
    descOpen = true; // editing implies expanded
    saveCollapse('description', true);
  }
  async function saveDesc() {
    if (!issue) return;
    await patch({ description: descDraft });
    await refreshAttachments();
    editingDesc = false;
  }

  // ---- comments ----
  async function addComment() {
    if (!issue || !newComment.trim() || posting) return;
    posting = true;
    try {
      await api.post(`/api/issues/${issue.id}/comments`, { body: newComment });
      newComment = '';
      commentKey++; // remount editor to clear it
      [comments, activity] = await Promise.all([
        api.get<Comment[]>(`/api/issues/${issue.id}/comments`),
        api.get<Activity[]>(`/api/issues/${issue.id}/activity`)
      ]);
      await refreshAttachments();
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Comment failed');
    } finally {
      posting = false;
    }
  }

  function canEditComment(c: Comment): boolean {
    return auth.isAdmin || c.authorId === auth.user?.id;
  }
  function startEditComment(c: Comment) {
    editingCommentId = c.id;
    editCommentDraft = c.body;
  }
  async function saveComment(c: Comment) {
    if (!issue) return;
    try {
      await api.patch(`/api/comments/${c.id}`, { body: editCommentDraft });
      editingCommentId = null;
      comments = await api.get<Comment[]>(`/api/issues/${issue.id}/comments`);
      await refreshAttachments();
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Update failed');
    }
  }
  async function deleteComment(c: Comment) {
    if (!issue) return;
    if (!(await confirmDialog.ask({ title: 'Delete comment', message: 'Delete this comment? This cannot be undone.', confirmText: 'Delete', danger: true }))) return;
    try {
      await api.del(`/api/comments/${c.id}`);
      [comments, activity] = await Promise.all([
        api.get<Comment[]>(`/api/issues/${issue.id}/comments`),
        api.get<Activity[]>(`/api/issues/${issue.id}/activity`)
      ]);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Delete failed');
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
    if (!(await confirmDialog.ask({ title: 'Delete attachment', message: `Delete “${att.filename}”?`, confirmText: 'Delete', danger: true }))) return;
    try {
      await api.del(`/api/attachments/${att.id}`);
      attachments = await api.get<Attachment[]>(`/api/issues/${issue.id}/attachments`);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Delete failed');
    }
  }

  async function deleteIssue() {
    if (!issue) return;
    if (!(await confirmDialog.ask({ title: 'Delete issue', message: `Permanently delete ${issue.key}? This removes its comments and attachments too.`, confirmText: 'Delete', danger: true }))) return;
    try {
      await api.del(`/api/issues/${issue.id}`);
      toasts.success('Issue deleted');
      goto(`/p/${ctx.project?.key}`);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Delete failed');
    }
  }

  // ---- linked issues ----
  const groupedLinks = $derived(
    LINK_TYPE_ORDER.map((t) => ({ type: t, items: links.filter((l) => l.linkType === t) })).filter(
      (g) => g.items.length > 0
    )
  );

  let searchTimer: ReturnType<typeof setTimeout> | undefined;
  function onLinkQuery() {
    clearTimeout(searchTimer);
    searchTimer = setTimeout(searchIssues, 250);
  }
  async function searchIssues() {
    if (!issue || !linkQuery.trim()) {
      linkResults = [];
      return;
    }
    searching = true;
    try {
      const params = new URLSearchParams({ q: linkQuery.trim(), pageSize: '20' });
      const res = await api.get<IssueListResponse>(`/api/projects/${projectId}/issues?${params}`);
      const linkedIds = new Set(links.map((l) => l.issueId));
      linkResults = res.items.filter((i) => i.id !== issue!.id && !linkedIds.has(i.id));
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Search failed');
    } finally {
      searching = false;
    }
  }
  async function addLink(target: IssueListItem) {
    if (!issue) return;
    try {
      links = await api.post<IssueLink[]>(`/api/issues/${issue.id}/links`, {
        targetIssueId: target.id,
        linkType
      });
      activity = await api.get<Activity[]>(`/api/issues/${issue.id}/activity`);
      linkResults = linkResults.filter((i) => i.id !== target.id);
      toasts.success(`Linked ${target.key}`);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Link failed');
    }
  }
  async function removeLink(lk: IssueLink) {
    if (!issue) return;
    if (!(await confirmDialog.ask({ title: 'Remove link', message: `Remove the link to ${lk.key}?`, confirmText: 'Remove', danger: true }))) return;
    try {
      await api.del(`/api/issue-links/${lk.id}`);
      [links, activity] = await Promise.all([
        api.get<IssueLink[]>(`/api/issues/${issue.id}/links`),
        api.get<Activity[]>(`/api/issues/${issue.id}/activity`)
      ]);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Remove failed');
    }
  }
  function closeLinkModal() {
    showLink = false;
    linkQuery = '';
    linkResults = [];
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
      case 11: return `linked ${a.newValue}`;
      case 12: return `removed link to ${a.newValue}`;
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
        {#if editingTitle}
          <!-- svelte-ignore a11y_autofocus -->
          <input
            class="input mt-1 text-2xl font-semibold"
            bind:value={titleDraft}
            autofocus
            onkeydown={(e) => {
              if (e.key === 'Enter') saveTitle();
              if (e.key === 'Escape') editingTitle = false;
            }}
            onblur={saveTitle}
          />
        {:else}
          <h1
            class="group mt-1 flex items-start gap-2 text-2xl font-semibold"
          >
            <span>{issue.title}</span>
            <button
              class="btn-ghost mt-1 !py-0.5 !text-xs opacity-0 transition group-hover:opacity-100"
              onclick={startEditTitle}>Edit</button
            >
          </h1>
        {/if}

        <div class="mt-3">
          <div class="mb-1 flex items-center gap-1">
            <button
              type="button"
              class="-m-1 flex flex-1 items-center gap-1.5 rounded p-1 text-left hover:bg-slate-50"
              onclick={toggleDesc}
              aria-expanded={descOpen}
            >
              <svg
                viewBox="0 0 20 20"
                fill="currentColor"
                class="h-3.5 w-3.5 shrink-0 text-slate-400 transition-transform duration-150 {descOpen ? '' : '-rotate-90'}"
              >
                <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.17l3.71-3.94a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
              </svg>
              <h3 class="text-xs font-semibold text-slate-500">Description</h3>
            </button>
            {#if !editingDesc}
              <button class="btn-ghost shrink-0 !py-0.5 !text-xs" onclick={startEditDesc}>Edit</button>
            {/if}
          </div>
          {#if descOpen}
            {#if editingDesc}
              <RichEditor
                value={issue.description ?? ''}
                editable
                placeholder="Describe the issue… drag, paste, or use 📷 to add images/videos"
                issueId={issue.id}
                onChange={(md) => (descDraft = md)}
                onMediaAdded={refreshAttachments}
              />
              <div class="mt-2 flex gap-2">
                <button class="btn-primary !py-1 !text-xs" onclick={saveDesc}>Save</button>
                <button class="btn-ghost !py-1 !text-xs" onclick={() => (editingDesc = false)}>Cancel</button>
              </div>
            {:else if issue.description}
              <RichEditor value={issue.description} />
            {:else}
              <p class="text-sm text-slate-400">No description.</p>
            {/if}
          {/if}
        </div>
      </div>

      <!-- Labels -->
      <CollapsibleCard title="Labels" storageKey="labels">
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
      </CollapsibleCard>

      <!-- Linked issues -->
      <CollapsibleCard title="Linked issues" storageKey="links" count={links.length}>
        {#snippet actions()}
          <button class="btn-ghost cursor-pointer text-xs" onclick={() => (showLink = true)}>+ Link issue</button>
        {/snippet}
        {#if links.length === 0}
          <p class="text-xs text-slate-400">No linked issues.</p>
        {:else}
          <div class="space-y-3">
            {#each groupedLinks as group (group.type)}
              <div>
                <div class="mb-1 text-xs font-medium uppercase tracking-wide text-slate-400">
                  {LINK_TYPE_LABELS[group.type]}
                </div>
                <ul class="space-y-1">
                  {#each group.items as lk (lk.id)}
                    <li class="flex items-center gap-2 text-sm">
                      <span class="shrink-0 rounded bg-slate-100 px-1.5 py-0.5 text-[10px] font-medium text-slate-500">
                        {STATUS_LABELS[lk.status]}
                      </span>
                      <a href={`/p/${lk.projectKey}/issue/${lk.number}`} class="flex min-w-0 items-center gap-1.5 hover:underline">
                        <span class="font-mono text-xs text-slate-400">{lk.key}</span>
                        <span class="truncate text-slate-700">{lk.title}</span>
                      </a>
                      <button class="btn-ghost ml-auto !py-0.5 !text-xs text-rose-600" title="Remove link" onclick={() => removeLink(lk)}>✕</button>
                    </li>
                  {/each}
                </ul>
              </div>
            {/each}
          </div>
        {/if}
      </CollapsibleCard>

      <!-- Attachments -->
      <CollapsibleCard title="Attachments" storageKey="attachments" count={attachments.length}>
        {#snippet actions()}
          <label class="btn-ghost cursor-pointer text-xs">
            {uploading ? 'Uploading…' : '+ Upload'}
            <input type="file" class="hidden" onchange={uploadFile} disabled={uploading} />
          </label>
        {/snippet}
        {#if attachments.length === 0}
          <p class="text-xs text-slate-400">No attachments.</p>
        {:else}
          <ul class="divide-y divide-slate-100 text-sm">
            {#each attachments as a (a.id)}
              <li class="flex items-center gap-2 py-1.5">
                <AttachmentThumb att={a} />
                <button class="truncate text-indigo-600 hover:underline" onclick={() => download(a)}>{a.filename}</button>
                <span class="shrink-0 text-xs text-slate-400">{fmtSize(a.sizeBytes)}</span>
                <button class="btn-ghost ml-auto !py-0.5 !text-xs text-rose-600" onclick={() => deleteAttachment(a)}>✕</button>
              </li>
            {/each}
          </ul>
        {/if}
      </CollapsibleCard>

      <!-- Comments -->
      <CollapsibleCard title="Comments" storageKey="comments" count={comments.length}>
        <div class="space-y-3">
          {#each comments as c (c.id)}
            <div class="rounded-md bg-slate-50 p-3">
              <div class="flex items-center gap-2 text-xs text-slate-500">
                <Avatar seed={c.authorId} name={c.authorName} size={22} />
                <span class="font-medium text-slate-700">{c.authorName}</span>
                <span>{fmt(c.createdAt)}</span>
                {#if c.updatedAt !== c.createdAt}<span class="text-slate-400">(edited)</span>{/if}
                {#if canEditComment(c) && editingCommentId !== c.id}
                  <div class="ml-auto flex gap-1">
                    <button class="btn-ghost !py-0.5 !text-xs" onclick={() => startEditComment(c)}>Edit</button>
                    <button class="btn-ghost !py-0.5 !text-xs text-rose-600" onclick={() => deleteComment(c)}>Delete</button>
                  </div>
                {/if}
              </div>
              {#if editingCommentId === c.id}
                <div class="mt-2">
                  <RichEditor
                    value={c.body}
                    editable
                    issueId={issue.id}
                    onChange={(md) => (editCommentDraft = md)}
                    onMediaAdded={refreshAttachments}
                  />
                  <div class="mt-2 flex gap-2">
                    <button class="btn-primary !py-1 !text-xs" onclick={() => saveComment(c)}>Save</button>
                    <button class="btn-ghost !py-1 !text-xs" onclick={() => (editingCommentId = null)}>Cancel</button>
                  </div>
                </div>
              {:else}
                <div class="mt-1"><RichEditor value={c.body} /></div>
              {/if}
            </div>
          {/each}
          {#if comments.length === 0}
            <p class="text-xs text-slate-400">No comments yet.</p>
          {/if}
        </div>
        <div class="mt-3">
          {#key commentKey}
            <RichEditor
              editable
              placeholder="Add a comment… drag, paste, or use 📷 to add images/videos"
              issueId={issue.id}
              onChange={(md) => (newComment = md)}
              onMediaAdded={refreshAttachments}
            />
          {/key}
          <button class="btn-primary mt-2" disabled={posting || !newComment.trim()} onclick={addComment}>
            {posting ? 'Posting…' : 'Comment'}
          </button>
        </div>
      </CollapsibleCard>
    </div>

    <!-- Sidebar: fields + activity -->
    <div class="space-y-5">
      <CollapsibleCard title="Details" storageKey="details">
        <div class="space-y-3 text-sm">
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
          <div class="flex items-center gap-2">
            <Avatar seed={issue.reporterId} name={issue.reporterName} size={20} />
            <span>Reporter: {issue.reporterName}</span>
          </div>
          <div class="mt-1">Created: {fmt(issue.createdAt)}</div>
        </div>
        </div>
      </CollapsibleCard>

      <CollapsibleCard title="Activity" storageKey="activity" count={activity.length}>
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
          {#if activity.length === 0}
            <li class="text-slate-400">No activity yet.</li>
          {/if}
        </ol>
      </CollapsibleCard>
    </div>
  </div>

  <!-- Link issue modal -->
  {#if showLink}
    <div
      class="fixed inset-0 z-[55] flex items-start justify-center bg-black/40 p-4 pt-24"
      role="presentation"
      onclick={(e) => {
        if (e.target === e.currentTarget) closeLinkModal();
      }}
    >
      <div class="card w-full max-w-lg p-5">
        <div class="mb-3 flex items-center">
          <h2 class="text-base font-semibold text-slate-800">Link an issue</h2>
          <button class="btn-ghost ml-auto !py-0.5" aria-label="Close" onclick={closeLinkModal}>✕</button>
        </div>
        <div class="flex gap-2">
          <select class="input max-w-[12rem]" bind:value={linkType}>
            {#each LINK_TYPE_LABELS as label, i}<option value={i}>{label}</option>{/each}
          </select>
          <!-- svelte-ignore a11y_autofocus -->
          <input
            class="input flex-1"
            placeholder="Search issues by key or title…"
            bind:value={linkQuery}
            oninput={onLinkQuery}
            autofocus
          />
        </div>
        <div class="mt-3 max-h-72 overflow-y-auto">
          {#if searching}
            <p class="text-xs text-slate-400">Searching…</p>
          {:else if linkQuery.trim() && linkResults.length === 0}
            <p class="text-xs text-slate-400">No matching issues.</p>
          {:else}
            <ul class="divide-y divide-slate-100">
              {#each linkResults as r (r.id)}
                <li>
                  <button
                    class="flex w-full items-center gap-2 py-2 text-left text-sm hover:bg-slate-50"
                    onclick={() => addLink(r)}
                  >
                    <span class="shrink-0 rounded bg-slate-100 px-1.5 py-0.5 text-[10px] font-medium text-slate-500">
                      {STATUS_LABELS[r.status]}
                    </span>
                    <span class="font-mono text-xs text-slate-400">{r.key}</span>
                    <span class="truncate text-slate-700">{r.title}</span>
                    <span class="ml-auto text-xs text-indigo-600">+ Link</span>
                  </button>
                </li>
              {/each}
            </ul>
          {/if}
        </div>
        <p class="mt-3 text-[11px] text-slate-400">Links connect issues within this project.</p>
      </div>
    </div>
  {/if}
{:else}
  <p class="text-slate-400">Loading issue…</p>
{/if}
