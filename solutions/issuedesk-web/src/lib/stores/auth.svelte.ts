import type { User } from '../types';

/** Reactive auth state (Svelte 5 runes). */
class AuthState {
  user = $state<User | null>(null);
  ready = $state(false); // becomes true once we've checked the stored token

  get isAdmin() {
    return this.user?.role === 1;
  }

  get isCustomer() {
    return this.user?.role === 2;
  }
}

export const auth = new AuthState();
