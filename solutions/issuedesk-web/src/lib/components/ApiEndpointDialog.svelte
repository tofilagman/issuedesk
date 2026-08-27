<script lang="ts">
  import { toasts } from '$lib/stores/toast.svelte';

  let { open = $bindable(false) }: { open?: boolean } = $props();

  interface Operation {
    method: string;
    path: string;
    summary: string;
  }

  // Same-origin API: the SPA is served by the backend it talks to.
  const origin = $derived(typeof window !== 'undefined' ? window.location.origin : '');
  const base = $derived(`${origin}/api`);

  const curl = $derived(`curl -H "X-API-Key: $ISSUEDESK_KEY" \\\n  ${base}/tickets/WAT-1?format=md`);

  // Read the endpoint list from the shipped OpenAPI spec rather than repeating
  // it here, so this panel can't drift from the actual API surface.
  let ops = $state<Operation[] | null>(null);
  let specError = $state(false);
  let authHeader = $state('X-API-Key');

  const ORDER = ['get', 'post', 'patch', 'put', 'delete'];

  $effect(() => {
    if (!open || ops || specError) return;
    fetch('/openapi.json')
      .then((r) => (r.ok ? r.json() : Promise.reject(new Error(String(r.status)))))
      .then((spec) => {
        const apiKey = spec?.components?.securitySchemes?.apiKeyAuth;
        if (apiKey?.name) authHeader = apiKey.name;
        const list: Operation[] = [];
        for (const [path, item] of Object.entries(spec?.paths ?? {})) {
          for (const [method, op] of Object.entries(item as Record<string, unknown>)) {
            if (!ORDER.includes(method)) continue;
            list.push({
              method: method.toUpperCase(),
              path,
              summary: (op as { summary?: string })?.summary ?? ''
            });
          }
        }
        ops = list;
      })
      .catch(() => (specError = true));
  });

  async function copy(text: string, msg: string) {
    try {
      await navigator.clipboard.writeText(text);
      toasts.success(msg);
    } catch {
      toasts.error('Copy failed');
    }
  }

  const METHOD_COLOR: Record<string, string> = {
    GET: 'text-emerald-600 dark:text-emerald-400',
    POST: 'text-indigo-600 dark:text-indigo-400',
    PATCH: 'text-amber-600 dark:text-amber-400',
    PUT: 'text-amber-600 dark:text-amber-400',
    DELETE: 'text-rose-600 dark:text-rose-400'
  };

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') open = false;
  }
</script>

<svelte:window onkeydown={onKey} />

{#if open}
  <div
    class="fixed inset-0 z-[70] flex items-start justify-center overflow-y-auto bg-black/50 p-4 py-10"
    role="presentation"
    onclick={(e) => {
      if (e.target === e.currentTarget) open = false;
    }}
  >
    <div class="card w-full max-w-2xl p-5">
      <div class="flex items-start gap-3">
        <div>
          <h2 class="text-base font-semibold text-slate-900 dark:text-slate-100">API endpoint</h2>
          <p class="mt-0.5 text-xs text-slate-500 dark:text-slate-400">
            Integrate with issuedesk using a key from
            <a class="text-indigo-600 underline dark:text-indigo-400" href="/admin/keys">API Keys</a>.
          </p>
        </div>
        <button
          class="ml-auto text-2xl leading-none text-slate-400 hover:text-slate-600 dark:text-slate-500 dark:hover:text-slate-300"
          aria-label="Close"
          onclick={() => (open = false)}>×</button
        >
      </div>

      <!-- Base URL -->
      <div class="mt-4">
        <div class="text-xs font-medium text-slate-500 dark:text-slate-400">Base URL</div>
        <div class="mt-1 flex items-center gap-2">
          <code
            class="flex-1 overflow-x-auto rounded border border-slate-200 bg-slate-50 px-3 py-2 font-mono text-xs text-slate-700 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200"
            >{base}</code
          >
          <button class="btn-ghost shrink-0 !text-xs" onclick={() => copy(base, 'Base URL copied')}
            >Copy</button
          >
        </div>
      </div>

      <!-- Auth -->
      <div class="mt-4">
        <div class="text-xs font-medium text-slate-500 dark:text-slate-400">Authentication</div>
        <div class="mt-1 flex items-center gap-2">
          <code
            class="flex-1 overflow-x-auto rounded border border-slate-200 bg-slate-50 px-3 py-2 font-mono text-xs text-slate-700 dark:border-slate-700 dark:bg-slate-800 dark:text-slate-200"
            >{authHeader}: &lt;your key&gt;</code
          >
          <button
            class="btn-ghost shrink-0 !text-xs"
            onclick={() => copy(`${authHeader}: `, 'Header copied')}>Copy</button
          >
        </div>
      </div>

      <!-- Endpoints, straight from the OpenAPI spec -->
      <div class="mt-4">
        <div class="flex items-baseline gap-2">
          <span class="text-xs font-medium text-slate-500 dark:text-slate-400">Endpoints</span>
          <a
            class="text-xs text-indigo-600 underline dark:text-indigo-400"
            href="/api-docs.html"
            target="_blank"
            rel="noopener">Open full API docs →</a
          >
        </div>
        {#if specError}
          <p class="mt-1 text-xs text-slate-400 dark:text-slate-500">
            Couldn't load the API spec. It's available at
            <a class="text-indigo-600 underline dark:text-indigo-400" href="/openapi.json">/openapi.json</a>.
          </p>
        {:else if !ops}
          <p class="mt-1 text-xs text-slate-400 dark:text-slate-500">Loading…</p>
        {:else}
          <ul
            class="mt-1 max-h-72 divide-y divide-slate-100 overflow-y-auto rounded border border-slate-200 dark:divide-slate-800 dark:border-slate-700"
          >
            {#each ops as op (op.method + op.path)}
              <li class="flex items-center gap-2 px-3 py-1.5">
                <span
                  class="w-14 shrink-0 font-mono text-[0.65rem] font-semibold {METHOD_COLOR[
                    op.method
                  ] ?? 'text-slate-500'}">{op.method}</span
                >
                <div class="min-w-0 flex-1">
                  <code class="block truncate font-mono text-xs text-slate-700 dark:text-slate-200"
                    >{op.path}</code
                  >
                  {#if op.summary}
                    <span class="block truncate text-[0.7rem] text-slate-400 dark:text-slate-500"
                      >{op.summary}</span
                    >
                  {/if}
                </div>
                <button
                  class="btn-ghost shrink-0 !text-xs"
                  onclick={() => copy(origin + op.path, 'Endpoint copied')}>Copy</button
                >
              </li>
            {/each}
          </ul>
        {/if}
      </div>

      <!-- Example -->
      <div class="mt-4">
        <div class="text-xs font-medium text-slate-500 dark:text-slate-400">Example</div>
        <div class="mt-1 flex items-start gap-2">
          <pre
            class="flex-1 overflow-x-auto rounded bg-slate-800 px-3 py-2 font-mono text-xs text-slate-100 dark:bg-slate-950">{curl}</pre>
          <button class="btn-ghost shrink-0 !text-xs" onclick={() => copy(curl, 'Command copied')}
            >Copy</button
          >
        </div>
      </div>
    </div>
  </div>
{/if}
