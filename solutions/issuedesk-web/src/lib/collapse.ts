/** Persist collapse/expand state of UI sections in localStorage, keyed by name.
    Preferences are global (per section), so "always collapse attachments" sticks
    across issues and reloads. */
const PREFIX = 'issuedesk-collapse:';

export function loadCollapse(key: string, defaultOpen = true): boolean {
  if (typeof localStorage === 'undefined') return defaultOpen;
  const v = localStorage.getItem(PREFIX + key);
  return v === null ? defaultOpen : v === '1';
}

export function saveCollapse(key: string, open: boolean) {
  if (typeof localStorage !== 'undefined') localStorage.setItem(PREFIX + key, open ? '1' : '0');
}
