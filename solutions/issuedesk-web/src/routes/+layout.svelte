<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { auth } from '$lib/stores/auth.svelte';
  import { theme } from '$lib/stores/theme.svelte';
  import { toasts } from '$lib/stores/toast.svelte';
  import { api, loadToken, clearToken } from '$lib/api';
  import MediaViewer from '$lib/components/MediaViewer.svelte';
  import ApiEndpointDialog from '$lib/components/ApiEndpointDialog.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import Avatar from '$lib/components/Avatar.svelte';
  import type { User } from '$lib/types';

  let { children } = $props();

  let showApi = $state(false);

  // Adopt the stored/system scheme and follow later OS changes.
  onMount(() => theme.init());

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

  // Customers have no business on admin, settings, or dashboard pages — the
  // server 403s them anyway; this just keeps the UI coherent.
  $effect(() => {
    const id = page.route.id ?? '';
    if (
      auth.ready &&
      auth.isCustomer &&
      (id.startsWith('/admin') || id.includes('/settings') || id.includes('/dashboard'))
    ) {
      goto('/');
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
    <header class="border-b border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-900">
      <div class="mx-auto flex max-w-7xl items-center gap-4 px-4 py-2.5">
        <a href="/" class="flex items-center gap-2 font-semibold text-indigo-700">
          <span class="grid h-7 w-7 place-items-center rounded bg-indigo-600 text-sm text-white">id</span>
          issuedesk
        </a>
        <nav class="flex items-center gap-1 text-sm">
          <a class="btn-ghost" href="/">Projects</a>
          {#if !auth.isCustomer}
            <a class="btn-ghost" href="/dashboard">Dashboard</a>
          {/if}
          {#if auth.isAdmin}
            <a class="btn-ghost" href="/admin/users">Users</a>
            <a class="btn-ghost" href="/admin/groups">Groups</a>
            <a class="btn-ghost" href="/admin/keys">API Keys</a>
          {/if}
        </nav>
        <div class="ml-auto flex items-center gap-3 text-sm">
          <button
            class="btn-ghost !px-2"
            title="API endpoint"
            aria-label="Show API endpoint"
            onclick={() => (showApi = true)}
          >
            <svg viewBox="0 0 16 16" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M6 11.5 4 8l2-3.5M10 4.5 12 8l-2 3.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>
          <button
            class="btn-ghost !px-2"
            title={theme.resolved === 'dark' ? 'Switch to light theme' : 'Switch to dark theme'}
            aria-label="Toggle colour theme"
            onclick={() => theme.toggle()}
          >
            {#if theme.resolved === 'dark'}
              <!-- sun -->
              <svg viewBox="0 0 16 16" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="8" cy="8" r="3" />
                <path d="M8 1v1.5M8 13.5V15M1 8h1.5M13.5 8H15M3.05 3.05l1.06 1.06M11.89 11.89l1.06 1.06M12.95 3.05l-1.06 1.06M4.11 11.89l-1.06 1.06" stroke-linecap="round" />
              </svg>
            {:else}
              <!-- moon -->
              <svg viewBox="0 0 16 16" class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="1.5">
                <path d="M13.5 9.5A5.5 5.5 0 0 1 6.5 2.5a5.5 5.5 0 1 0 7 7Z" stroke-linejoin="round" />
              </svg>
            {/if}
          </button>
          <span class="flex items-center gap-2 text-slate-600 dark:text-slate-300">
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
      <div class="py-20 text-center text-slate-400 dark:text-slate-500">Loading…</div>
    {/if}
  </main>
</div>

<!-- Fullscreen image lightbox / video popup -->
<MediaViewer />

<!-- API endpoint reference -->
<ApiEndpointDialog bind:open={showApi} />

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
