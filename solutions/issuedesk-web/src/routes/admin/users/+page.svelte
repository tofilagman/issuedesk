<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { auth } from '$lib/stores/auth.svelte';
  import { toasts } from '$lib/stores/toast.svelte';
  import Avatar from '$lib/components/Avatar.svelte';
  import type { User } from '$lib/types';

  let users = $state<User[]>([]);
  let loading = $state(true);
  let showNew = $state(false);
  let nf = $state({ userName: '', email: '', displayName: '', password: '', role: 0 });
  let busy = $state(false);

  async function load() {
    loading = true;
    try {
      users = await api.get<User[]>('/api/users');
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Failed to load users');
    } finally {
      loading = false;
    }
  }
  onMount(load);

  async function create(e: Event) {
    e.preventDefault();
    busy = true;
    try {
      await api.post('/api/users', nf);
      toasts.success('User created');
      showNew = false;
      nf = { userName: '', email: '', displayName: '', password: '', role: 0 };
      await load();
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Create failed');
    } finally {
      busy = false;
    }
  }

  async function update(u: User, body: Record<string, unknown>) {
    try {
      await api.patch(`/api/users/${u.id}`, body);
      await load();
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Update failed');
    }
  }

  async function resetPassword(u: User) {
    const password = prompt(`New password for ${u.userName}:`);
    if (!password) return;
    try {
      await api.post(`/api/users/${u.id}/password`, { password });
      toasts.success('Password reset');
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Reset failed');
    }
  }
</script>

<div class="mb-4 flex items-center">
  <h1 class="text-xl font-semibold">Users</h1>
  <button class="btn-primary ml-auto" onclick={() => (showNew = !showNew)}>New user</button>
</div>

{#if showNew}
  <form onsubmit={create} class="card mb-5 grid gap-3 p-4 sm:grid-cols-5 sm:items-end">
    <div><label class="mb-1 block text-xs text-slate-500" for="un">Username</label><input id="un" class="input" bind:value={nf.userName} required /></div>
    <div><label class="mb-1 block text-xs text-slate-500" for="em">Email</label><input id="em" class="input" type="email" bind:value={nf.email} required /></div>
    <div><label class="mb-1 block text-xs text-slate-500" for="dn">Display name</label><input id="dn" class="input" bind:value={nf.displayName} required /></div>
    <div><label class="mb-1 block text-xs text-slate-500" for="pw">Password</label><input id="pw" class="input" type="password" bind:value={nf.password} required /></div>
    <div class="flex gap-2">
      <select class="input" bind:value={nf.role}><option value={0}>Member</option><option value={1}>Admin</option></select>
      <button class="btn-primary" disabled={busy}>Add</button>
    </div>
  </form>
{/if}

<div class="card overflow-hidden">
  <table class="w-full text-sm">
    <thead class="border-b border-slate-200 bg-slate-50 text-left text-xs uppercase text-slate-500">
      <tr><th class="px-3 py-2">User</th><th class="px-3 py-2">Email</th><th class="px-3 py-2">Role</th><th class="px-3 py-2">Active</th><th class="px-3 py-2"></th></tr>
    </thead>
    <tbody>
      {#each users as u (u.id)}
        <tr class="border-b border-slate-100">
          <td class="px-3 py-2">
            <div class="flex items-center gap-2">
              <Avatar seed={u.id} name={u.displayName} size={26} />
              <span class="font-medium">{u.displayName}</span>
              <span class="text-slate-400">@{u.userName}</span>
            </div>
          </td>
          <td class="px-3 py-2 text-slate-600">{u.email}</td>
          <td class="px-3 py-2">
            <select class="input max-w-[8rem]" value={u.role} onchange={(e) => update(u, { role: Number((e.target as HTMLSelectElement).value) })}>
              <option value={0}>Member</option><option value={1}>Admin</option>
            </select>
          </td>
          <td class="px-3 py-2">
            <input type="checkbox" checked={u.isActive} onchange={(e) => update(u, { isActive: (e.target as HTMLInputElement).checked })} />
          </td>
          <td class="px-3 py-2 text-right"><button class="btn-ghost !text-xs" onclick={() => resetPassword(u)}>Reset password</button></td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if loading}<p class="p-4 text-center text-slate-400">Loading…</p>{/if}
</div>
