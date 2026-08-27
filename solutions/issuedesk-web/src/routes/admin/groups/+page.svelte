<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { toasts } from '$lib/stores/toast.svelte';
  import { confirmDialog } from '$lib/stores/confirm.svelte';
  import Avatar from '$lib/components/Avatar.svelte';
  import { ROLE_LABELS, type Group, type Member, type User } from '$lib/types';

  let groups = $state<Group[]>([]);
  let allUsers = $state<User[]>([]);
  let loading = $state(true);
  let showNew = $state(false);
  let nf = $state({ name: '', description: '' });
  let busy = $state(false);

  // Expanded group + its roster.
  let openId = $state('');
  let members = $state<Member[]>([]);
  let addUserId = $state('');

  async function load() {
    loading = true;
    try {
      [groups, allUsers] = await Promise.all([
        api.get<Group[]>('/api/groups'),
        api.get<User[]>('/api/users')
      ]);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Failed to load groups');
    } finally {
      loading = false;
    }
  }
  onMount(load);

  async function create(e: Event) {
    e.preventDefault();
    busy = true;
    try {
      await api.post('/api/groups', { name: nf.name, description: nf.description || null });
      toasts.success('Group created');
      showNew = false;
      nf = { name: '', description: '' };
      await load();
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Create failed');
    } finally {
      busy = false;
    }
  }

  async function deleteGroup(g: Group) {
    if (!(await confirmDialog.ask({ title: 'Delete group', message: `Delete the “${g.name}” group? Its members will no longer see each other's tickets.`, confirmText: 'Delete', danger: true }))) return;
    try {
      await api.del(`/api/groups/${g.id}`);
      groups = groups.filter((x) => x.id !== g.id);
      if (openId === g.id) openId = '';
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Delete failed');
    }
  }

  async function toggle(g: Group) {
    if (openId === g.id) {
      openId = '';
      return;
    }
    openId = g.id;
    members = [];
    addUserId = '';
    try {
      members = await api.get<Member[]>(`/api/groups/${g.id}/members`);
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Failed to load members');
    }
  }

  async function addMember(e: Event) {
    e.preventDefault();
    if (!addUserId || !openId) return;
    try {
      members = await api.post<Member[]>(`/api/groups/${openId}/members`, { userId: addUserId });
      addUserId = '';
      groups = groups.map((g) => (g.id === openId ? { ...g, memberCount: members.length } : g));
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Add failed');
    }
  }

  async function removeMember(m: Member) {
    try {
      await api.del(`/api/groups/${openId}/members/${m.userId}`);
      members = members.filter((x) => x.userId !== m.userId);
      groups = groups.map((g) => (g.id === openId ? { ...g, memberCount: members.length } : g));
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Remove failed');
    }
  }

  const memberIds = $derived(new Set(members.map((m) => m.userId)));
  const candidates = $derived(allUsers.filter((u) => !memberIds.has(u.id)));
</script>

<div class="mb-4 flex items-center">
  <div>
    <h1 class="text-xl font-semibold">Groups</h1>
    <p class="text-xs text-slate-500 dark:text-slate-400">Customers in the same group can see each other's tickets.</p>
  </div>
  <button class="btn-primary ml-auto" onclick={() => (showNew = !showNew)}>New group</button>
</div>

{#if showNew}
  <form onsubmit={create} class="card mb-5 grid gap-3 p-4 sm:grid-cols-3 sm:items-end">
    <div><label class="mb-1 block text-xs text-slate-500 dark:text-slate-400" for="gn">Name</label><input id="gn" class="input" bind:value={nf.name} required /></div>
    <div><label class="mb-1 block text-xs text-slate-500 dark:text-slate-400" for="gd">Description</label><input id="gd" class="input" bind:value={nf.description} /></div>
    <div><button class="btn-primary" disabled={busy}>Add</button></div>
  </form>
{/if}

<div class="card overflow-hidden">
  <table class="w-full text-sm">
    <thead class="border-b border-slate-200 bg-slate-50 text-left text-xs uppercase text-slate-500 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-400">
      <tr><th class="px-3 py-2">Group</th><th class="px-3 py-2">Description</th><th class="px-3 py-2">Members</th><th class="px-3 py-2"></th></tr>
    </thead>
    <tbody>
      {#each groups as g (g.id)}
        <tr class="cursor-pointer border-b border-slate-100 hover:bg-slate-50 dark:border-slate-800 dark:hover:bg-slate-900" onclick={() => toggle(g)}>
          <td class="px-3 py-2 font-medium">{g.name}</td>
          <td class="px-3 py-2 text-slate-600 dark:text-slate-300">{g.description ?? ''}</td>
          <td class="px-3 py-2 text-slate-600 dark:text-slate-300">{g.memberCount}</td>
          <td class="px-3 py-2 text-right">
            <button class="btn-ghost !text-xs text-rose-600" onclick={(e) => { e.stopPropagation(); deleteGroup(g); }}>Delete</button>
          </td>
        </tr>
        {#if openId === g.id}
          <tr class="border-b border-slate-100 bg-slate-50/60 dark:border-slate-800">
            <td colspan="4" class="px-3 py-3">
              <ul class="divide-y divide-slate-100 text-sm">
                {#each members as m (m.userId)}
                  <li class="flex items-center gap-2 py-2">
                    <Avatar seed={m.userId} name={m.displayName} size={26} />
                    <span class="font-medium">{m.displayName}</span>
                    <span class="text-slate-400 dark:text-slate-500">@{m.userName}</span>
                    {#if m.role === 2}<span class="rounded bg-sky-100 px-1.5 text-xs text-sky-700">{ROLE_LABELS[2]}</span>{/if}
                    <button class="btn-ghost ml-auto !text-xs text-rose-600" onclick={() => removeMember(m)}>Remove</button>
                  </li>
                {:else}
                  <li class="py-2 text-xs text-slate-400 dark:text-slate-500">No members yet.</li>
                {/each}
              </ul>
              <form onsubmit={addMember} class="mt-3 flex max-w-md gap-2">
                <select class="input" bind:value={addUserId}>
                  <option value="">Add member…</option>
                  {#each candidates as u}<option value={u.id}>{u.displayName} (@{u.userName})</option>{/each}
                </select>
                <button class="btn-primary">Add</button>
              </form>
            </td>
          </tr>
        {/if}
      {/each}
    </tbody>
  </table>
  {#if loading}
    <p class="p-4 text-center text-slate-400 dark:text-slate-500">Loading…</p>
  {:else if groups.length === 0}
    <p class="p-4 text-center text-slate-400 dark:text-slate-500">No groups yet.</p>
  {/if}
</div>
