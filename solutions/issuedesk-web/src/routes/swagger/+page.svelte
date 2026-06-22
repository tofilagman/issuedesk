<script lang="ts">
  import { onMount } from 'svelte';
  import 'swagger-ui/dist/swagger-ui.css';

  let container = $state<HTMLDivElement | null>(null);
  let failed = $state(false);

  onMount(async () => {
    try {
      // Dynamic import: swagger-ui is a large, browser-only bundle, so it is
      // code-split into this route's chunk and only fetched when visited.
      const mod = await import('swagger-ui');
      const SwaggerUI = (mod.default ?? mod) as (opts: Record<string, unknown>) => void;
      SwaggerUI({
        domNode: container,
        url: '/openapi.json',
        deepLinking: true,
        docExpansion: 'list',
        defaultModelsExpandDepth: 0,
        persistAuthorization: true,
        tryItOutEnabled: true
      });
    } catch {
      failed = true;
    }
  });
</script>

<svelte:head><title>issuedesk API — Swagger UI</title></svelte:head>

<div class="mb-3 flex items-center gap-3">
  <h1 class="text-xl font-semibold">API reference</h1>
  <a class="btn-ghost !text-xs" href="/openapi.json" target="_blank" rel="noopener">openapi.json</a>
  <a class="btn-ghost ml-auto !text-xs" href="/admin/keys">← API keys</a>
</div>
<p class="mb-4 text-sm text-slate-500">
  Authorize with a <strong>JWT bearer</strong> token (from sign-in) or an
  <strong>X-API-Key</strong>, then “Try it out”. Use the
  <code class="font-mono">tickets</code> endpoints for relay clients.
</p>

{#if failed}
  <p class="rounded bg-rose-50 p-4 text-sm text-rose-700">Failed to load Swagger UI.</p>
{/if}
<div bind:this={container} class="swagger-host"></div>

<style>
  /* Swagger UI ships its own typography/colors; neutralise the page padding it
     assumes and let it span the content column. */
  .swagger-host :global(.swagger-ui .topbar) {
    display: none;
  }
  .swagger-host :global(.swagger-ui .info) {
    margin: 1rem 0;
  }
</style>
