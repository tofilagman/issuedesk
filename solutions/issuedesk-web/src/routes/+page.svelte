<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '$lib/api';
  import { auth } from '$lib/stores/auth.svelte';
  import { toasts } from '$lib/stores/toast.svelte';
  import type { Project } from '$lib/types';

  let projects = $state<Project[]>([]);
  let loading = $state(true);
  let showCreate = $state(false);
  let form = $state({ key: '', name: '', description: '' });
  let busy = $state(false);

  async function load() {
    loading = true;
    try {
      projects = await api.get<Project[]>('/api/projects');
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Failed to load projects');
    } finally {
      loading = false;
    }
  }
  onMount(load);

  async function create(e: Event) {
    e.preventDefault();
    busy = true;
    try {
      await api.post<Project>('/api/projects', {
        key: form.key.toUpperCase(),
        name: form.name,
        description: form.description || null
      });
      toasts.success('Project created');
      showCreate = false;
      form = { key: '', name: '', description: '' };
      await load();
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Create failed');
    } finally {
      busy = false;
    }
  }
</script>

<div class="mb-4 flex items-center">
  <h1 class="text-xl font-semibold">Projects</h1>
  {#if auth.isAdmin}
    <button class="btn-primary ml-auto" onclick={() => (showCreate = !showCreate)}>New project</button>
  {/if}
</div>

{#if showCreate}
  <form onsubmit={create} class="card mb-5 grid gap-3 p-4 sm:grid-cols-[120px_1fr_2fr_auto] sm:items-end">
    <div>
      <label class="mb-1 block text-xs font-medium text-slate-500" for="k">Key</label>
      <input id="k" class="input uppercase" maxlength="10" placeholder="WAT" bind:value={form.key} required />
    </div>
    <div>
      <label class="mb-1 block text-xs font-medium text-slate-500" for="n">Name</label>
      <input id="n" class="input" placeholder="Water Station" bind:value={form.name} required />
    </div>
    <div>
      <label class="mb-1 block text-xs font-medium text-slate-500" for="d">Description</label>
      <input id="d" class="input" bind:value={form.description} />
    </div>
    <button class="btn-primary" disabled={busy}>Create</button>
  </form>
{/if}

{#if loading}
  <p class="text-slate-400">Loading…</p>
{:else if projects.length === 0}
  <div class="card grid place-items-center p-12 text-center text-slate-400">
    No projects yet.{#if auth.isAdmin}<br />Create one to get started.{/if}
  </div>
{:else}
  <div class="grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
    {#each projects as p (p.id)}
      <a href={`/p/${p.key}`} class="card block p-4 transition-shadow hover:shadow-md">
        <div class="flex items-center gap-2">
          <span class="rounded bg-indigo-100 px-2 py-0.5 text-xs font-bold text-indigo-700">{p.key}</span>
          <span class="font-medium">{p.name}</span>
        </div>
        {#if p.description}
          <p class="mt-2 line-clamp-2 text-sm text-slate-500">{p.description}</p>
        {/if}
        <p class="mt-3 text-xs text-slate-400">{p.issueSeq} issues</p>
      </a>
    {/each}
  </div>
{/if}
