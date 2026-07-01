/** Svelte action: call `callback` whenever the node scrolls into view (with a
    little margin, for infinite-scroll "load more" sentinels). */
export function onVisible(node: HTMLElement, callback: () => void) {
  let current = callback;
  const obs = new IntersectionObserver(
    (entries) => {
      if (entries.some((e) => e.isIntersecting)) current();
    },
    { rootMargin: '150px' }
  );
  obs.observe(node);
  return {
    update(next: () => void) {
      current = next;
    },
    destroy() {
      obs.disconnect();
    }
  };
}
