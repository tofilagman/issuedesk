import { auth } from './stores/auth.svelte';

const TOKEN_KEY = 'issuedesk-token';

export function loadToken(): string | null {
  if (typeof localStorage === 'undefined') return null;
  return localStorage.getItem(TOKEN_KEY);
}
export function saveToken(token: string) {
  localStorage.setItem(TOKEN_KEY, token);
}
export function clearToken() {
  localStorage.removeItem(TOKEN_KEY);
}

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
  }
}

interface RequestOptions {
  method?: string;
  body?: unknown;
  /** raw body (e.g. FormData) — skips JSON encoding */
  raw?: BodyInit;
}

async function request<T>(path: string, opts: RequestOptions = {}): Promise<T> {
  const headers: Record<string, string> = {
    // Satisfies the backend browser-lock; the browser also sets this on real
    // navigations, but we set it explicitly so fetch() calls pass too.
    'Sec-Fetch-Site': 'same-origin'
  };
  const token = loadToken();
  if (token) headers['Authorization'] = `Bearer ${token}`;

  let body: BodyInit | undefined;
  if (opts.raw !== undefined) {
    body = opts.raw;
  } else if (opts.body !== undefined) {
    headers['Content-Type'] = 'application/json';
    body = JSON.stringify(opts.body);
  }

  const res = await fetch(path, { method: opts.method ?? 'GET', headers, body });

  if (res.status === 401) {
    clearToken();
    auth.user = null;
    if (typeof window !== 'undefined' && window.location.pathname !== '/login') {
      window.location.assign('/login');
    }
    throw new ApiError(401, 'unauthorized');
  }

  if (!res.ok) {
    let message = res.statusText;
    try {
      const j = await res.json();
      message = j.message || j.error || message;
    } catch {
      /* non-JSON error */
    }
    throw new ApiError(res.status, message);
  }

  if (res.status === 204) return undefined as T;
  const ct = res.headers.get('content-type') ?? '';
  if (ct.includes('application/json')) return (await res.json()) as T;
  return (await res.text()) as unknown as T;
}

export const api = {
  get: <T>(p: string) => request<T>(p),
  post: <T>(p: string, body?: unknown) => request<T>(p, { method: 'POST', body }),
  patch: <T>(p: string, body?: unknown) => request<T>(p, { method: 'PATCH', body }),
  del: <T>(p: string) => request<T>(p, { method: 'DELETE' }),
  upload: <T>(p: string, form: FormData) => request<T>(p, { method: 'POST', raw: form }),
  /** URL for a download link (token can't ride in a plain <a>, so callers fetch). */
  downloadUrl: (id: string) => `/api/attachments/${id}`
};

/**
 * Fetch an attachment's bytes with the auth header and return an object URL.
 * Used for inline media (img/video) and the lightbox — a bare <img src> can't
 * carry the Bearer token, and we must never persist the token into stored
 * markdown. Callers own the returned URL and must `URL.revokeObjectURL` it.
 */
export async function fetchAttachmentObjectUrl(id: string): Promise<string> {
  const res = await fetch(api.downloadUrl(id), {
    headers: { Authorization: `Bearer ${loadToken()}`, 'Sec-Fetch-Site': 'same-origin' }
  });
  if (!res.ok) throw new ApiError(res.status, 'failed to load attachment');
  const blob = await res.blob();
  return URL.createObjectURL(blob);
}
