<script lang="ts">
  import { goto } from '$app/navigation';
  import { api, saveToken } from '$lib/api';
  import { auth } from '$lib/stores/auth.svelte';
  import { toasts } from '$lib/stores/toast.svelte';
  import type { User } from '$lib/types';

  let userName = $state('');
  let password = $state('');
  let busy = $state(false);

  async function submit(e: Event) {
    e.preventDefault();
    busy = true;
    try {
      const res = await api.post<{ token: string; user: User }>('/auth/signIn', {
        userName,
        password
      });
      saveToken(res.token);
      auth.user = res.user;
      goto('/');
    } catch (err) {
      toasts.error(err instanceof Error ? err.message : 'Sign in failed');
    } finally {
      busy = false;
    }
  }
</script>

<div class="grid min-h-[70vh] place-items-center">
  <form onsubmit={submit} class="card w-full max-w-sm space-y-4 p-6">
    <div class="flex items-center gap-2 text-lg font-semibold text-indigo-700">
      <span class="grid h-8 w-8 place-items-center rounded bg-indigo-600 text-white">id</span>
      issuedesk
    </div>
    <p class="text-sm text-slate-500">Sign in to your account</p>
    <div>
      <label class="mb-1 block text-sm font-medium" for="u">Username</label>
      <input id="u" class="input" bind:value={userName} autocomplete="username" required />
    </div>
    <div>
      <label class="mb-1 block text-sm font-medium" for="p">Password</label>
      <input
        id="p"
        class="input"
        type="password"
        bind:value={password}
        autocomplete="current-password"
        required
      />
    </div>
    <button class="btn-primary w-full" disabled={busy}>{busy ? 'Signing in…' : 'Sign in'}</button>
  </form>
</div>
