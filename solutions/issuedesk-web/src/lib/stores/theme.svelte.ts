/** Colour-scheme preference (Svelte 5 runes). */
export type ThemeChoice = 'light' | 'dark' | 'system';

const KEY = 'issuedesk-theme';

function systemPrefersDark(): boolean {
  return typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches;
}

function readStored(): ThemeChoice {
  if (typeof localStorage === 'undefined') return 'system';
  const v = localStorage.getItem(KEY);
  return v === 'light' || v === 'dark' || v === 'system' ? v : 'system';
}

/**
 * Applies the resolved scheme to <html>, which is what every `dark:` utility
 * keys off (see the @custom-variant in app.css). The same logic runs as an
 * inline script in app.html so the first paint is already correct.
 */
function apply(resolved: 'light' | 'dark') {
  if (typeof document === 'undefined') return;
  document.documentElement.dataset.theme = resolved;
  document.documentElement.style.colorScheme = resolved;
}

class ThemeState {
  /** What the user picked; 'system' follows the OS. */
  choice = $state<ThemeChoice>('system');
  /** What that currently resolves to. */
  resolved = $state<'light' | 'dark'>('light');

  /** Called once on mount; returns a teardown for the OS-preference listener. */
  init(): () => void {
    this.choice = readStored();
    this.refresh();

    const mq = window.matchMedia('(prefers-color-scheme: dark)');
    const onChange = () => {
      if (this.choice === 'system') this.refresh();
    };
    mq.addEventListener('change', onChange);
    return () => mq.removeEventListener('change', onChange);
  }

  private refresh() {
    this.resolved = this.choice === 'system' ? (systemPrefersDark() ? 'dark' : 'light') : this.choice;
    apply(this.resolved);
  }

  set(choice: ThemeChoice) {
    this.choice = choice;
    localStorage.setItem(KEY, choice);
    this.refresh();
  }

  /** Flip between light and dark, leaving 'system' behind. */
  toggle() {
    this.set(this.resolved === 'dark' ? 'light' : 'dark');
  }
}

export const theme = new ThemeState();
