<script lang="ts">
  import { getContext } from 'svelte';
  import { api } from '$lib/api';
  import { auth } from '$lib/stores/auth.svelte';
  import { toasts } from '$lib/stores/toast.svelte';
  import { confirmDialog } from '$lib/stores/confirm.svelte';
  import Avatar from '$lib/components/Avatar.svelte';
  import type { Group, Label, Member, Project, User } from '$lib/types';

  const ctx = getContext<{ project: Project | null }>('project');
  let projectId = $derived(ctx.project?.id ?? '');

  let members = $state<Member[]>([]);
  let labels = $state<Label[]>([]);
  let groups = $state<Group[]>([]);
  let allUsers = $state<User[]>([]);
  let allGroups = $state<Group[]>([]);
  let loadedFor = $state('');

  let addUserId = $state('');
  let addGroupId = $state('');
  let nl = $state({ name: '', color: '#3b82f6' });

  $effect(() => {
    if (projectId && loadedFor !== projectId) {
      loadedFor = projectId;
      void load();
    }
  });

  async function load() {
    try {
      const tasks: [Promise<Member[]>, Promise<Label[]>, Promise<Group[]>] = [
        api.get<Member[]>(`/api/projects/${projectId}/members`),
        api.get<Label[]>(`/api/projects/${projectId}/labels`),
        api.get<Group[]>(`/api/projects/${projectId}/groups`)
      ];
      [members, labels, groups] = await Promise.all(tasks);
      if (auth.isAdmin) {
        [allUsers, allGroups] = await Promise.all([
          api.get<User[]>('/api/users'),
          api.get<Group[]>('/api/groups')
        ]);
      }
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
    if (!(await confirmDialog.ask({ title: 'Remove member', message: `Remove ${m.displayName} from this project?`, confirmText: 'Remove', danger: true }))) return;
    try {
      await api.del(`/api/projects/${projectId}/members/${m.userId}`);
      members = members.filter((x) => x.userId !== m.userId);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Remove failed');
    }
  }

  async function addGroup(e: Event) {
    e.preventDefault();
    if (!addGroupId) return;
    try {
      groups = await api.post<Group[]>(`/api/projects/${projectId}/groups`, { groupId: addGroupId });
      addGroupId = '';
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Add failed');
    }
  }

  async function removeGroup(g: Group) {
    if (!(await confirmDialog.ask({ title: 'Remove group', message: `Remove “${g.name}” from this project? Its ${g.memberCount} member(s) lose access unless they are direct members.`, confirmText: 'Remove', danger: true }))) return;
    try {
      await api.del(`/api/projects/${projectId}/groups/${g.id}`);
      groups = groups.filter((x) => x.id !== g.id);
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
    if (!(await confirmDialog.ask({ title: 'Delete label', message: `Delete the “${l.name}” label? It will be removed from all issues.`, confirmText: 'Delete', danger: true }))) return;
    try {
      await api.del(`/api/labels/${l.id}`);
      labels = labels.filter((x) => x.id !== l.id);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Delete failed');
    }
  }

  const memberIds = $derived(new Set(members.map((m) => m.userId)));
  const candidates = $derived(allUsers.filter((u) => !memberIds.has(u.id)));
  const linkedGroupIds = $derived(new Set(groups.map((g) => g.id)));
  const groupCandidates = $derived(allGroups.filter((g) => !linkedGroupIds.has(g.id)));
</script>

<div class="grid gap-5 lg:grid-cols-2">
  <!-- Members -->
  <div class="card p-5">
    <h3 class="mb-3 font-semibold text-slate-700">Members</h3>
    <ul class="divide-y divide-slate-100 text-sm">
      {#each members as m (m.userId)}
        <li class="flex items-center gap-2 py-2">
          <Avatar seed={m.userId} name={m.displayName} size={26} />
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

  <!-- Groups (their members all get project access) -->
  <div class="card p-5">
    <h3 class="mb-1 font-semibold text-slate-700">Groups</h3>
    <p class="mb-3 text-xs text-slate-400">Everyone in a linked group can access this project without being added individually.</p>
    <ul class="divide-y divide-slate-100 text-sm">
      {#each groups as g (g.id)}
        <li class="flex items-center gap-2 py-2">
          <span class="grid h-6 w-6 place-items-center rounded bg-indigo-100 text-xs font-semibold text-indigo-700">{g.name.slice(0, 1).toUpperCase()}</span>
          <span class="font-medium">{g.name}</span>
          <span class="text-xs text-slate-400">{g.memberCount} member{g.memberCount === 1 ? '' : 's'}</span>
          <button class="btn-ghost ml-auto !text-xs text-rose-600" onclick={() => removeGroup(g)}>Remove</button>
        </li>
      {:else}
        <li class="py-2 text-xs text-slate-400">No groups linked.</li>
      {/each}
    </ul>
    {#if auth.isAdmin}
      <form onsubmit={addGroup} class="mt-3 flex gap-2">
        <select class="input" bind:value={addGroupId}>
          <option value="">Add group…</option>
          {#each groupCandidates as g}<option value={g.id}>{g.name} ({g.memberCount})</option>{/each}
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
