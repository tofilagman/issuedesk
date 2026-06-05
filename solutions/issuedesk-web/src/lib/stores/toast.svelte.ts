export interface Toast {
  id: number;
  kind: 'success' | 'error' | 'info';
  message: string;
}

class ToastState {
  items = $state<Toast[]>([]);
  private seq = 0;

  push(kind: Toast['kind'], message: string) {
    const id = ++this.seq;
    this.items = [...this.items, { id, kind, message }];
    setTimeout(() => this.dismiss(id), 4000);
  }
  success(m: string) {
    this.push('success', m);
  }
  error(m: string) {
    this.push('error', m);
  }
  info(m: string) {
    this.push('info', m);
  }
  dismiss(id: number) {
    this.items = this.items.filter((t) => t.id !== id);
  }
}

export const toasts = new ToastState();
