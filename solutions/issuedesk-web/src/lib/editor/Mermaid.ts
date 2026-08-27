import { Node, mergeAttributes, nodeInputRule } from '@tiptap/core';

export interface MermaidOptions {
  HTMLAttributes: Record<string, unknown>;
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    mermaid: {
      /** Insert a mermaid diagram block (defaults to a small starter graph). */
      setMermaid: (options?: { code?: string }) => ReturnType;
    };
  }
}

export const DEFAULT_MERMAID = `graph TD
  A[Start] --> B{Works?}
  B -- yes --> C[Ship it]
  B -- no --> D[Fix it]
  D --> B`;

/** Matches the `<code>` class markdown-it emits for a ```mermaid fence. */
const MERMAID_CLASS = /(^|\s)language-mermaid(\s|$)/;

function mermaidSource(element: HTMLElement): string | null {
  const code = element.querySelector('code');
  if (!code || !MERMAID_CLASS.test(code.className)) return null;
  // markdown-it keeps a trailing newline inside the fence; drop it so the
  // source round-trips unchanged.
  return (code.textContent ?? '').replace(/\n$/, '');
}

/**
 * A block-level, atomic mermaid diagram. The diagram source lives in the `code`
 * attribute rather than as editable node content: the node view renders the SVG
 * and offers a textarea for the source, which is simpler than keeping a live
 * preview beside an editable code block.
 *
 * It round-trips through tiptap-markdown as a plain ```mermaid fence, so the
 * stored markdown stays readable (and renders as a diagram on GitHub et al).
 *
 * `priority` is raised so both the ```mermaid input rule and the `<pre>` parse
 * rule are tried before StarterKit's codeBlock, which would otherwise claim them.
 */
export const Mermaid = Node.create<MermaidOptions>({
  name: 'mermaid',
  group: 'block',
  atom: true,
  draggable: true,
  selectable: true,
  priority: 200,

  addOptions() {
    return { HTMLAttributes: {} };
  },

  addAttributes() {
    return {
      code: {
        default: '',
        rendered: false,
        parseHTML: (element) => mermaidSource(element as HTMLElement) ?? ''
      }
    };
  },

  parseHTML() {
    return [
      {
        tag: 'pre',
        priority: 60,
        // `false` falls through to codeBlock's own `pre` rule.
        getAttrs: (element) => (mermaidSource(element as HTMLElement) === null ? false : {})
      }
    ];
  },

  renderHTML({ node, HTMLAttributes }) {
    return [
      'pre',
      mergeAttributes(this.options.HTMLAttributes, HTMLAttributes),
      ['code', { class: 'language-mermaid' }, node.attrs.code ?? '']
    ];
  },

  addCommands() {
    return {
      setMermaid:
        (options) =>
        ({ commands }) =>
          commands.insertContent({
            type: this.name,
            attrs: { code: options?.code ?? DEFAULT_MERMAID }
          })
    };
  },

  addInputRules() {
    return [
      nodeInputRule({
        find: /^```mermaid[\s\n]$/,
        type: this.type,
        getAttributes: () => ({ code: '' })
      })
    ];
  },

  // Consumed by tiptap-markdown for (de)serialization.
  addStorage() {
    return {
      markdown: {
        serialize(
          state: {
            write: (s: string) => void;
            text: (s: string, escape?: boolean) => void;
            ensureNewLine: () => void;
            closeBlock: (n: unknown) => void;
          },
          node: { attrs: { code?: string } }
        ) {
          state.write('```mermaid\n');
          state.text(node.attrs.code ?? '', false);
          state.ensureNewLine();
          state.write('```');
          state.closeBlock(node);
        },
        parse: {}
      }
    };
  }
});
