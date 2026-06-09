/** Promise-based confirmation dialog (Svelte 5 runes).
 *
 * Usage:
 *   import { confirmDialog } from '$lib/stores/confirm.svelte';
 *   if (!(await confirmDialog.ask({ message: 'Delete this?', danger: true }))) return;
 */
export interface ConfirmOptions {
  title?: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  /** Style the confirm button as destructive (red). */
  danger?: boolean;
}

interface PendingConfirm extends ConfirmOptions {
  resolve: (ok: boolean) => void;
}

class ConfirmState {
  current = $state<PendingConfirm | null>(null);

  ask(opts: ConfirmOptions): Promise<boolean> {
    // If a dialog is already open, resolve it as cancelled first.
    this.current?.resolve(false);
    return new Promise<boolean>((resolve) => {
      this.current = { ...opts, resolve };
    });
  }

  private settle(ok: boolean) {
    const cur = this.current;
    this.current = null;
    cur?.resolve(ok);
  }
  confirm() {
    this.settle(true);
  }
  cancel() {
    this.settle(false);
  }
}

export const confirmDialog = new ConfirmState();
