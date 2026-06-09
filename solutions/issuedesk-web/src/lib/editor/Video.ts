import { Node, mergeAttributes } from '@tiptap/core';

export interface VideoOptions {
  HTMLAttributes: Record<string, unknown>;
}

declare module '@tiptap/core' {
  interface Commands<ReturnType> {
    video: {
      /** Insert a video node pointing at an attachment URL. */
      setVideo: (options: { src: string; title?: string | null }) => ReturnType;
    };
  }
}

/**
 * A block-level, atomic video node. There is no markdown syntax for video, so
 * it round-trips through tiptap-markdown as a raw `<video src=…>` HTML tag
 * (the Markdown extension is configured with `html: true`, and `parseHTML`
 * below re-hydrates it on the way back in).
 */
export const Video = Node.create<VideoOptions>({
  name: 'video',
  group: 'block',
  atom: true,
  draggable: true,
  selectable: true,

  addOptions() {
    return { HTMLAttributes: {} };
  },

  addAttributes() {
    return {
      src: { default: null },
      title: { default: null }
    };
  },

  parseHTML() {
    return [{ tag: 'video[src]' }];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      'video',
      mergeAttributes({ controls: 'true' }, this.options.HTMLAttributes, HTMLAttributes)
    ];
  },

  addCommands() {
    return {
      setVideo:
        (options) =>
        ({ commands }) =>
          commands.insertContent({ type: this.name, attrs: options })
    };
  },

  // Consumed by tiptap-markdown for (de)serialization.
  addStorage() {
    return {
      markdown: {
        serialize(state: { write: (s: string) => void; closeBlock: (n: unknown) => void }, node: { attrs: { src: string; title?: string | null } }) {
          const title = node.attrs.title ? ` title="${node.attrs.title}"` : '';
          state.write(`<video src="${node.attrs.src}"${title} controls></video>`);
          state.closeBlock(node);
        },
        parse: {}
      }
    };
  }
});
