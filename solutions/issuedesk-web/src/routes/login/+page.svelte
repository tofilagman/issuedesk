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

<!-- Full-bleed animated gradient backdrop (sits behind the layout's centered card) -->
<div class="pointer-events-none fixed inset-0 -z-10 overflow-hidden bg-gradient-to-br from-indigo-700 via-violet-600 to-sky-500">
  <div class="absolute -left-24 -top-24 h-96 w-96 rounded-full bg-sky-300/30 blur-3xl"></div>
  <div class="absolute -bottom-32 -right-16 h-[28rem] w-[28rem] rounded-full bg-fuchsia-400/30 blur-3xl"></div>
  <div class="absolute left-1/3 top-1/2 h-72 w-72 rounded-full bg-indigo-300/20 blur-3xl"></div>

  <!-- Issue-card silhouettes linked by connectors, drifting slowly -->
  <svg class="absolute inset-0 h-full w-full" viewBox="0 0 1440 900" preserveAspectRatio="xMidYMid slice" aria-hidden="true">
    <defs>
      <!-- generic issue card -->
      <g id="lg-card">
        <rect width="150" height="94" rx="10" fill="rgba(255,255,255,0.05)" stroke="rgba(255,255,255,0.35)" stroke-width="1.5" />
        <circle cx="21" cy="23" r="5" fill="rgba(255,255,255,0.45)" />
        <rect x="35" y="18" width="72" height="9" rx="4.5" fill="rgba(255,255,255,0.4)" />
        <rect x="16" y="43" width="118" height="7" rx="3.5" fill="rgba(255,255,255,0.22)" />
        <rect x="16" y="58" width="86" height="7" rx="3.5" fill="rgba(255,255,255,0.22)" />
        <rect x="16" y="74" width="42" height="9" rx="4.5" fill="rgba(255,255,255,0.3)" />
        <rect x="64" y="74" width="28" height="9" rx="4.5" fill="rgba(255,255,255,0.18)" />
      </g>
      <!-- done card: title + big check -->
      <g id="lg-done">
        <rect width="120" height="76" rx="10" fill="rgba(255,255,255,0.05)" stroke="rgba(255,255,255,0.32)" stroke-width="1.5" />
        <rect x="14" y="15" width="64" height="8" rx="4" fill="rgba(255,255,255,0.35)" />
        <circle cx="60" cy="49" r="13" fill="none" stroke="rgba(255,255,255,0.4)" stroke-width="2" />
        <path d="M53 49 l5 5 l10 -10" fill="none" stroke="rgba(255,255,255,0.5)" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" />
      </g>
    </defs>

    <!-- connectors -->
    <g fill="none" stroke="rgba(255,255,255,0.28)" stroke-width="1.5" stroke-dasharray="5 9" stroke-linecap="round" class="lg-dash">
      <path d="M310 205 C 470 260, 520 380, 470 520" />
      <path d="M470 555 C 520 700, 700 760, 920 745" />
      <path d="M1130 240 C 1000 300, 940 420, 1075 500" />
      <path d="M1180 545 C 1250 630, 1180 700, 1085 735" />
      <path d="M320 160 C 620 90, 900 110, 1120 190" />
    </g>
    <!-- junction nodes -->
    <g fill="rgba(255,255,255,0.45)">
      <circle cx="470" cy="537" r="5" />
      <circle cx="720" cy="105" r="5" />
      <circle cx="1075" cy="500" r="4" />
      <circle cx="500" cy="345" r="4" />
    </g>

    <!-- cards, each drifting at its own pace -->
    <g class="lg-float-a"><use href="#lg-card" transform="translate(160 130) rotate(-4)" /></g>
    <g class="lg-float-b"><use href="#lg-card" transform="translate(360 480) rotate(3) scale(0.85)" /></g>
    <g class="lg-float-c"><use href="#lg-done" transform="translate(910 700) rotate(-3)" /></g>
    <g class="lg-float-b"><use href="#lg-card" transform="translate(1120 165) rotate(4) scale(0.9)" /></g>
    <g class="lg-float-a"><use href="#lg-done" transform="translate(1090 480) rotate(5) scale(0.9)" /></g>
    <g class="lg-float-c"><use href="#lg-card" transform="translate(60 640) rotate(6) scale(0.7)" /></g>
  </svg>
</div>

<div class="grid min-h-[80vh] place-items-center">
  <form onsubmit={submit} class="w-full max-w-sm space-y-4 rounded-xl border border-white/40 bg-white/95 p-6 shadow-2xl backdrop-blur dark:border-white/10 dark:bg-slate-900/90">
    <div class="flex items-center gap-2 text-lg font-semibold text-indigo-700">
      <span class="grid h-8 w-8 place-items-center rounded bg-indigo-600 text-white">id</span>
      issuedesk
    </div>
    <p class="text-sm text-slate-500 dark:text-slate-400">Sign in to your account</p>
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

<style>
  @keyframes lg-drift {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-16px);
    }
  }
  @keyframes lg-flow {
    to {
      stroke-dashoffset: -280;
    }
  }
  .lg-float-a {
    animation: lg-drift 9s ease-in-out infinite;
  }
  .lg-float-b {
    animation: lg-drift 12s ease-in-out 1.5s infinite;
  }
  .lg-float-c {
    animation: lg-drift 10s ease-in-out 3s infinite;
  }
  .lg-dash {
    animation: lg-flow 24s linear infinite;
  }
  @media (prefers-reduced-motion: reduce) {
    .lg-float-a,
    .lg-float-b,
    .lg-float-c,
    .lg-dash {
      animation: none;
    }
  }
</style>
