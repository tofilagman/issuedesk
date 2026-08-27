/**
 * Lazily loads and configures mermaid, so its (large) bundle only ships to
 * readers who actually open content containing a diagram.
 */
type MermaidModule = typeof import('mermaid').default;
export type DiagramTheme = 'light' | 'dark';

let loading: Promise<MermaidModule> | null = null;
/** The scheme mermaid is currently initialized for. */
let initialized: DiagramTheme | null = null;

function loadMermaid(): Promise<MermaidModule> {
  loading ??= import('mermaid').then(({ default: mermaid }) => mermaid);
  return loading;
}

/**
 * mermaid keeps one global config, so switching schemes means re-initializing.
 * Callers re-render their diagrams when the app theme flips.
 */
function configure(mermaid: MermaidModule, mode: DiagramTheme) {
  if (initialized === mode) return;
  mermaid.initialize({
    startOnLoad: false,
    // Diagram source is user-authored content: sanitize, and never allow
    // click bindings or inline scripts through.
    securityLevel: 'strict',
    theme: mode === 'dark' ? 'dark' : 'default',
    // Render our own error UI instead of mermaid's built-in error diagram.
    suppressErrorRendering: true,
    fontFamily: 'ui-sans-serif, system-ui, sans-serif'
  });
  initialized = mode;
}

let seq = 0;

/** Render diagram source to sanitized SVG markup. Rejects on a syntax error. */
export async function renderMermaid(code: string, mode: DiagramTheme = 'light'): Promise<string> {
  const mermaid = await loadMermaid();
  configure(mermaid, mode);
  // The id must be unique and a valid CSS selector — mermaid uses it to key the
  // temporary element it renders into.
  const id = `mermaid-${++seq}`;
  try {
    const { svg } = await mermaid.render(id, code);
    return svg;
  } finally {
    // With suppressErrorRendering, a failed render can leave its scratch node behind.
    document.getElementById(`d${id}`)?.remove();
  }
}
