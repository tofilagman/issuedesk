<script lang="ts">
  import { getContext } from 'svelte';
  import { api } from '$lib/api';
  import { auth } from '$lib/stores/auth.svelte';
  import { toasts } from '$lib/stores/toast.svelte';
  import type { Label, Member, Project, User } from '$lib/types';

  const ctx = getContext<{ project: Project | null }>('project');
  let projectId = $derived(ctx.project?.id ?? '');

  let members = $state<Member[]>([]);
  let labels = $state<Label[]>([]);
  let allUsers = $state<User[]>([]);
  let loadedFor = $state('');

  let addUserId = $state('');
  let nl = $state({ name: '', color: '#3b82f6' });

  $effect(() => {
    if (projectId && loadedFor !== projectId) {
      loadedFor = projectId;
      void load();
    }
  });

  async function load() {
    try {
      const tasks: [Promise<Member[]>, Promise<Label[]>] = [
        api.get<Member[]>(`/api/projects/${projectId}/members`),
        api.get<Label[]>(`/api/projects/${projectId}/labels`)
      ];
      [members, labels] = await Promise.all(tasks);
      if (auth.isAdmin) allUsers = await api.get<User[]>('/api/users');
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Failed to load settings');
    }
  }

  async function addMember(e: Event) {
    e.preventDefault();
    if (!addUserId) return;
    try {
      members = await api.post<Member[]>(`/api/projects/${projectId}/members`, { userId: addUserId });
      addUserId = '';
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Add failed');
    }
  }

  async function removeMember(m: Member) {
    try {
      await api.del(`/api/projects/${projectId}/members/${m.userId}`);
      members = members.filter((x) => x.userId !== m.userId);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Remove failed');
    }
  }

  async function addLabel(e: Event) {
    e.preventDefault();
    try {
      const l = await api.post<Label>(`/api/projects/${projectId}/labels`, nl);
      labels = [...labels, l];
      nl = { name: '', color: '#3b82f6' };
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Create label failed');
    }
  }

  async function deleteLabel(l: Label) {
    try {
      await api.del(`/api/labels/${l.id}`);
      labels = labels.filter((x) => x.id !== l.id);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Delete failed');
    }
  }

  const memberIds = $derived(new Set(members.map((m) => m.userId)));
  const candidates = $derived(allUsers.filter((u) => !memberIds.has(u.id)));
</script>

<div class="grid gap-5 lg:grid-cols-2">
  <!-- Members -->
  <div class="card p-5">
    <h3 class="mb-3 font-semibold text-slate-700">Members</h3>
    <ul class="divide-y divide-slate-100 text-sm">
      {#each members as m (m.userId)}
        <li class="flex items-center gap-2 py-2">
          <span class="font-medium">{m.displayName}</span>
          <span class="text-slate-400">@{m.userName}</span>
          {#if m.role === 1}<span class="rounded bg-amber-100 px-1.5 text-xs text-amber-700">lead</span>{/if}
          <button class="btn-ghost ml-auto !text-xs text-rose-600" onclick={() => removeMember(m)}>Remove</button>
        </li>
      {/each}
    </ul>
    {#if auth.isAdmin}
      <form onsubmit={addMember} class="mt-3 flex gap-2">
        <select class="input" bind:value={addUserId}>
          <option value="">Add member…</option>
          {#each candidates as u}<option value={u.id}>{u.displayName} (@{u.userName})</option>{/each}
        </select>
        <button class="btn-primary">Add</button>
      </form>
    {/if}
  </div>

  <!-- Labels -->
  <div class="card p-5">
    <h3 class="mb-3 font-semibold text-slate-700">Labels</h3>
    <div class="flex flex-wrap gap-2">
      {#each labels as l (l.id)}
        <span class="label-chip gap-1" style={`background-color:${l.color}`}>
          {l.name}
          <button class="opacity-70 hover:opacity-100" onclick={() => deleteLabel(l)}>✕</button>
        </span>
      {/each}
      {#if labels.length === 0}<p class="text-xs text-slate-400">No labels yet.</p>{/if}
    </div>
    <form onsubmit={addLabel} class="mt-4 flex items-end gap-2">
      <div class="flex-1">
        <label class="mb-1 block text-xs text-slate-500" for="ln">Name</label>
        <input id="ln" class="input" bind:value={nl.name} required />
      </div>
      <div>
        <label class="mb-1 block text-xs text-slate-500" for="lc">Color</label>
        <input id="lc" type="color" class="h-9 w-12 rounded border border-slate-300" bind:value={nl.color} />
      </div>
      <button class="btn-primary">Add</button>
    </form>
  </div>
</div>
