<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { toasts } from '$lib/stores/toast.svelte';
  import { confirmDialog } from '$lib/stores/confirm.svelte';
  import type { ApiKey, ApiKeyCreated } from '$lib/types';

  let keys = $state<ApiKey[]>([]);
  let loading = $state(true);
  let showNew = $state(false);
  let name = $state('');
  let busy = $state(false);
  // The freshly-created key — its plaintext secret is shown exactly once.
  let created = $state<ApiKeyCreated | null>(null);

  const origin = typeof window !== 'undefined' ? window.location.origin : 'https://your-host';

  async function load() {
    loading = true;
    try {
      keys = await api.get<ApiKey[]>('/api/api-keys');
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Failed to load keys');
    } finally {
      loading = false;
    }
  }
  onMount(load);

  async function create(e: Event) {
    e.preventDefault();
    if (!name.trim()) return;
    busy = true;
    try {
      created = await api.post<ApiKeyCreated>('/api/api-keys', { name: name.trim() });
      showNew = false;
      name = '';
      await load();
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Create failed');
    } finally {
      busy = false;
    }
  }

  async function revoke(k: ApiKey) {
    const ok = await confirmDialog.ask({
      title: 'Revoke API key',
      message: `Revoke “${k.name}”? Any client using it will immediately lose access.`,
      confirmText: 'Revoke',
      danger: true
    });
    if (!ok) return;
    try {
      await api.del(`/api/api-keys/${k.id}`);
      toasts.success('Key revoked');
      await load();
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Revoke failed');
    }
  }

  async function copy(text: string, label = 'Copied') {
    try {
      await navigator.clipboard.writeText(text);
      toasts.success(label);
    } catch {
      toasts.error('Copy failed — select and copy manually');
    }
  }

  function fmt(ts?: string | null) {
    if (!ts) return '—';
    return new Date(ts).toLocaleString();
  }
</script>

<div class="mb-4 flex items-center">
  <div>
    <h1 class="text-xl font-semibold">API Keys</h1>
    <p class="text-sm text-slate-500 dark:text-slate-400">
      Read-only credentials for relaying tickets to external clients (e.g. Claude).
    </p>
  </div>
  <div class="ml-auto flex items-center gap-2">
    <a class="btn-ghost" href="/api-docs.html" target="_blank" rel="noopener">API docs</a>
    <button class="btn-primary" onclick={() => (showNew = !showNew)}>New key</button>
  </div>
</div>

{#if showNew}
  <form onsubmit={create} class="card mb-5 flex flex-wrap items-end gap-3 p-4">
    <div class="flex-1 min-w-[14rem]">
      <label class="mb-1 block text-xs text-slate-500 dark:text-slate-400" for="kn">Key name</label>
      <input id="kn" class="input w-full" placeholder="e.g. claude-relay" bind:value={name} required />
    </div>
    <button class="btn-primary" disabled={busy}>Generate</button>
  </form>
{/if}

{#if created}
  <div class="card mb-5 border-emerald-300 bg-emerald-50/60 p-4">
    <div class="mb-2 flex items-center gap-2">
      <span class="h-2 w-2 rounded-full bg-emerald-500"></span>
      <h2 class="font-semibold text-emerald-900">Key “{created.name}” created</h2>
      <button class="btn-ghost ml-auto !text-xs" onclick={() => (created = null)}>Dismiss</button>
    </div>
    <p class="mb-2 text-sm text-emerald-900">
      Copy it now — this is the only time the full secret is shown.
    </p>
    <div class="mb-3 flex items-center gap-2">
      <code class="flex-1 overflow-x-auto rounded bg-white px-3 py-2 font-mono text-sm text-slate-800 ring-1 ring-emerald-200 dark:bg-slate-900 dark:text-slate-100">{created.secret}</code>
      <button class="btn-primary !py-2" onclick={() => copy(created!.secret, 'Key copied')}>Copy</button>
    </div>
    <p class="mb-1 text-xs font-medium uppercase tracking-wide text-emerald-800">Relay a ticket to Claude</p>
    <div class="flex items-center gap-2">
      <code class="flex-1 overflow-x-auto rounded bg-white px-3 py-2 font-mono text-xs text-slate-700 ring-1 ring-emerald-200 dark:bg-slate-900 dark:text-slate-200">curl -H "X-API-Key: {created.secret}" {origin}/api/tickets/WAT-1?format=md</code>
      <button
        class="btn-ghost !text-xs"
        onclick={() => copy(`curl -H "X-API-Key: ${created!.secret}" ${origin}/api/tickets/WAT-1?format=md`, 'Command copied')}
      >Copy</button>
    </div>
    <p class="mt-2 text-xs text-emerald-800">
      Swap <code class="font-mono">WAT-1</code> for the key from any ticket URL. Drop
      <code class="font-mono">?format=md</code> for JSON.
    </p>
  </div>
{/if}

<div class="card overflow-hidden">
  <table class="w-full text-sm">
    <thead class="border-b border-slate-200 bg-slate-50 text-left text-xs uppercase text-slate-500 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-400">
      <tr>
        <th class="px-3 py-2">Name</th>
        <th class="px-3 py-2">Key</th>
        <th class="px-3 py-2">Last used</th>
        <th class="px-3 py-2">Created</th>
        <th class="px-3 py-2"></th>
      </tr>
    </thead>
    <tbody>
      {#each keys as k (k.id)}
        <tr class="border-b border-slate-100 dark:border-slate-800">
          <td class="px-3 py-2 font-medium">{k.name}</td>
          <td class="px-3 py-2"><code class="font-mono text-slate-500 dark:text-slate-400">{k.prefix}…</code></td>
          <td class="px-3 py-2 text-slate-600 dark:text-slate-300">{fmt(k.lastUsedAt)}</td>
          <td class="px-3 py-2 text-slate-600 dark:text-slate-300">{fmt(k.createdAt)}</td>
          <td class="px-3 py-2 text-right">
            <button class="btn-ghost !text-xs !text-rose-600" onclick={() => revoke(k)}>Revoke</button>
          </td>
        </tr>
      {/each}
      {#if !loading && keys.length === 0}
        <tr><td colspan="5" class="px-3 py-6 text-center text-slate-400 dark:text-slate-500">No API keys yet.</td></tr>
      {/if}
    </tbody>
  </table>
  {#if loading}<p class="p-4 text-center text-slate-400 dark:text-slate-500">Loading…</p>{/if}
</div>
