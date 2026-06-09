<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { auth } from '$lib/stores/auth.svelte';
  import { toasts } from '$lib/stores/toast.svelte';
  import { api, loadToken, clearToken } from '$lib/api';
  import MediaViewer from '$lib/components/MediaViewer.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import Avatar from '$lib/components/Avatar.svelte';
  import type { User } from '$lib/types';

  let { children } = $props();

  onMount(async () => {
    const token = loadToken();
    if (token) {
      try {
        auth.user = await api.get<User>('/api/users/me');
      } catch {
        clearToken();
      }
    }
    auth.ready = true;

    // Guard: bounce to login when unauthenticated (except on the login route).
    if (!auth.user && page.route.id !== '/login') {
      goto('/login');
    }
  });

  function logout() {
    clearToken();
    auth.user = null;
    goto('/login');
  }

  const isLogin = $derived(page.route.id === '/login');
</script>

<div class="flex min-h-screen flex-col">
  {#if auth.user && !isLogin}
    <header class="border-b border-slate-200 bg-white">
      <div class="mx-auto flex max-w-7xl items-center gap-4 px-4 py-2.5">
        <a href="/" class="flex items-center gap-2 font-semibold text-indigo-700">
          <span class="grid h-7 w-7 place-items-center rounded bg-indigo-600 text-sm text-white">id</span>
          issuedesk
        </a>
        <nav class="flex items-center gap-1 text-sm">
          <a class="btn-ghost" href="/">Projects</a>
          <a class="btn-ghost" href="/dashboard">Dashboard</a>
          {#if auth.isAdmin}
            <a class="btn-ghost" href="/admin/users">Users</a>
          {/if}
        </nav>
        <div class="ml-auto flex items-center gap-3 text-sm">
          <span class="flex items-center gap-2 text-slate-600">
            <Avatar seed={auth.user.id} name={auth.user.displayName} size={26} />
            {auth.user.displayName}
          </span>
          <button class="btn-ghost" onclick={logout}>Sign out</button>
        </div>
      </div>
    </header>
  {/if}

  <main class="mx-auto w-full max-w-7xl flex-1 px-4 py-5">
    {#if auth.ready}
      {@render children()}
    {:else}
      <div class="py-20 text-center text-slate-400">Loading…</div>
    {/if}
  </main>
</div>

<!-- Fullscreen image lightbox / video popup -->
<MediaViewer />

<!-- Confirmation dialog -->
<ConfirmDialog />

<!-- Toasts -->
<div class="fixed bottom-4 right-4 z-50 flex flex-col gap-2">
  {#each toasts.items as t (t.id)}
    <div
      class="card flex items-center gap-3 px-4 py-2.5 text-sm shadow-md"
      class:border-emerald-300={t.kind === 'success'}
      class:border-rose-300={t.kind === 'error'}
    >
      <span
        class="h-2 w-2 rounded-full"
        class:bg-emerald-500={t.kind === 'success'}
        class:bg-rose-500={t.kind === 'error'}
        class:bg-sky-500={t.kind === 'info'}
      ></span>
      {t.message}
    </div>
  {/each}
</div>
