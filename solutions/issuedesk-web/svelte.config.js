import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    // Single-page app: everything is served from index.html and routed
    // client-side via the hash router (matches the other Outsource frontends).
    // SPA mode: every unknown path falls back to index.html. Both the Rust
    // server (ServeDir fallback) and Traefik route deep links to this entry,
    // so path-based client routing works without server route config.
    adapter: adapter({
      fallback: 'index.html',
      precompress: false,
      strict: false
    })
  }
};

export default config;
