// Pure client-rendered SPA: no SSR, no prerendering. Unmatched paths are served
// the index.html fallback (adapter-static), then the path router takes over.
export const ssr = false;
export const prerender = false;
