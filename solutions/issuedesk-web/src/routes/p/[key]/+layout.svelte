<script lang="ts">
  import { setContext } from 'svelte';
  import { page } from '$app/state';
  import { getProjectByKey } from '$lib/projects';
  import { toasts } from '$lib/stores/toast.svelte';
  import type { Project } from '$lib/types';

  let { children } = $props();

  const key = $derived(page.params.key);

  // Shared, reactive project holder placed in context for child routes.
  const ctx = $state<{ project: Project | null }>({ project: null });
  setContext('project', ctx);

  let loadingKey = $state('');
  $effect(() => {
    if (key && key !== loadingKey) {
      loadingKey = key;
      getProjectByKey(key)
        .then((p) => (ctx.project = p))
        .catch((e) => toasts.error(e instanceof Error ? e.message : 'Project not found'));
    }
  });

  const tab = $derived(
    page.route.id?.includes('/list')
      ? 'list'
      : page.route.id?.includes('/settings')
        ? 'settings'
        : page.route.id?.includes('/issue/')
          ? ''
          : 'board'
  );
</script>

{#if ctx.project}
  <div class="mb-4">
    <div class="flex items-center gap-2">
      <span class="rounded bg-indigo-100 px-2 py-0.5 text-sm font-bold text-indigo-700">
        {ctx.project.key}
      </span>
      <h1 class="text-xl font-semibold">{ctx.project.name}</h1>
    </div>
    <nav class="mt-3 flex gap-1 border-b border-slate-200 text-sm">
      <a
        href={`/p/${ctx.project.key}`}
        class="border-b-2 px-3 py-2 {tab === 'board'
          ? 'border-indigo-600 font-medium text-indigo-700'
          : 'border-transparent text-slate-500'}">Board</a
      >
      <a
        href={`/p/${ctx.project.key}/list`}
        class="border-b-2 px-3 py-2 {tab === 'list'
          ? 'border-indigo-600 font-medium text-indigo-700'
          : 'border-transparent text-slate-500'}">List</a
      >
      <a
        href={`/p/${ctx.project.key}/settings`}
        class="border-b-2 px-3 py-2 {tab === 'settings'
          ? 'border-indigo-600 font-medium text-indigo-700'
          : 'border-transparent text-slate-500'}">Settings</a
      >
    </nav>
  </div>
  {@render children()}
{:else}
  <p class="text-slate-400">Loading project…</p>
{/if}
