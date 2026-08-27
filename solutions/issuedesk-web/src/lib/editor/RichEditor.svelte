<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Editor } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import Image from '@tiptap/extension-image';
  import Placeholder from '@tiptap/extension-placeholder';
  import { TableKit } from '@tiptap/extension-table/kit';
  import { Markdown } from 'tiptap-markdown';
  import { SvelteNodeViewRenderer } from 'svelte-tiptap';
  import { Video } from './Video';
  import { Mermaid } from './Mermaid';
  import MediaNodeView from './MediaNodeView.svelte';
  import MermaidNodeView from './MermaidNodeView.svelte';
  import { api } from '$lib/api';
  import { toasts } from '$lib/stores/toast.svelte';

  let {
    value = '',
    editable = false,
    placeholder = '',
    /** When set, media (drag/paste/pick) is uploaded as an attachment of this issue. */
    issueId = null,
    onChange,
    /** Fired after a media file is uploaded + inserted, so parents can refresh
        the issue's attachment list. */
    onMediaAdded
  }: {
    value?: string;
    editable?: boolean;
    placeholder?: string;
    issueId?: string | null;
    onChange?: (markdown: string) => void;
    onMediaAdded?: () => void;
  } = $props();

  let element: HTMLDivElement;
  let fileInput = $state<HTMLInputElement | null>(null);
  // `$state.raw`: track reassignment (null -> Editor) for reactivity, but DON'T
  // deeply proxy the Editor. A plain `$state` proxy would make any effect that
  // reads `editor` re-run on every internal mutation (e.g. setContent), which
  // creates an infinite loop with the value-sync effect below.
  let editor = $state.raw<Editor | null>(null);
  let uploading = $state(false);
  // Bumped on every transaction so toolbar active-states stay reactive.
  let tick = $state(0);
  // Media pasted/dropped/picked before the issue exists (creation flow) is held
  // here as `blobUrl -> File` and shown via a local preview; it is uploaded for
  // real once the issue is saved (see uploadPending). Not reactive — read
  // imperatively by the parent via bind:this.
  const pending = new Map<string, File>();

  // tiptap-markdown augments editor.storage at runtime but not in the types.
  function mdStorage(e: Editor): { getMarkdown(): string } {
    return (e.storage as unknown as Record<string, { getMarkdown(): string }>).markdown;
  }

  // ---- public instance API (used via bind:this) ----
  export function getMarkdown(): string {
    return editor ? (mdStorage(editor)?.getMarkdown() ?? '') : '';
  }
  export function clear() {
    editor?.commands.clearContent(true);
  }
  export function focusEditor() {
    editor?.commands.focus();
  }
  /** True if media was buffered before the issue existed (creation flow). */
  export function hasPendingMedia(): boolean {
    return pending.size > 0;
  }
  /**
   * Upload everything buffered during creation against the now-saved issue, and
   * rewrite each placeholder blob URL in the document to the real attachment
   * URL. After this resolves, `getMarkdown()` returns persistable content.
   */
  export async function uploadPending(newIssueId: string): Promise<void> {
    if (!editor) return;
    for (const [blobUrl, file] of pending) {
      const fd = new FormData();
      fd.append('file', file);
      try {
        const att = await api.upload<{ id: string; mimeType: string; filename: string }>(
          `/api/issues/${newIssueId}/attachments`,
          fd
        );
        const realUrl = api.downloadUrl(att.id);
        const tr = editor.state.tr;
        editor.state.doc.descendants((n, pos) => {
          if ((n.type.name === 'image' || n.type.name === 'video') && n.attrs.src === blobUrl) {
            tr.setNodeMarkup(pos, undefined, { ...n.attrs, src: realUrl });
          }
        });
        if (tr.docChanged) editor.view.dispatch(tr);
        onMediaAdded?.();
      } catch (e) {
        toasts.error(e instanceof Error ? e.message : 'Upload failed');
      } finally {
        URL.revokeObjectURL(blobUrl);
      }
    }
    pending.clear();
    onChange?.(getMarkdown());
  }

  onMount(() => {
    const ImageWithView = Image.extend({
      addNodeView() {
        return SvelteNodeViewRenderer(MediaNodeView);
      }
    }).configure({ inline: false });

    const VideoWithView = Video.extend({
      addNodeView() {
        return SvelteNodeViewRenderer(MediaNodeView);
      }
    });

    const MermaidWithView = Mermaid.extend({
      addNodeView() {
        return SvelteNodeViewRenderer(MermaidNodeView);
      }
    });

    editor = new Editor({
      element,
      editable,
      content: value,
      extensions: [
        StarterKit,
        ImageWithView,
        VideoWithView,
        MermaidWithView,
        Placeholder.configure({ placeholder }),
        TableKit.configure({ table: { resizable: true } }),
        Markdown.configure({ html: true, transformPastedText: true, linkify: true })
      ],
      editorProps: {
        attributes: { class: 'tiptap focus:outline-none' },
        handlePaste: (_view, event) => handleFiles(event.clipboardData, event),
        handleDrop: (_view, event) => handleFiles((event as DragEvent).dataTransfer, event)
      },
      onUpdate: () => {
        onChange?.(getMarkdown());
      },
      onTransaction: () => {
        tick++;
      }
    });
  });

  onDestroy(() => {
    // Free any previews never flushed (e.g. issue creation cancelled).
    for (const blobUrl of pending.keys()) URL.revokeObjectURL(blobUrl);
    editor?.destroy();
  });

  // Toggle editable reactively without rebuilding the editor.
  $effect(() => {
    editor?.setEditable(editable);
  });

  // Keep read-only viewers in sync if the source value changes (e.g. after a
  // save/edit reloads the content). The editor is already created with the
  // initial `value`, so we only re-set on a genuine change — never on mount and
  // never while the user is typing.
  let lastSynced = value;
  $effect(() => {
    const v = value;
    if (editor && !editable && !editor.isFocused && v !== lastSynced) {
      lastSynced = v;
      editor.commands.setContent(v, { emitUpdate: false });
    }
  });

  // ---- toolbar active states ----
  const active = $derived.by(() => {
    tick; // dependency: recompute after each transaction
    const e = editor;
    return {
      bold: e?.isActive('bold') ?? false,
      italic: e?.isActive('italic') ?? false,
      strike: e?.isActive('strike') ?? false,
      code: e?.isActive('code') ?? false,
      h1: e?.isActive('heading', { level: 1 }) ?? false,
      h2: e?.isActive('heading', { level: 2 }) ?? false,
      bullet: e?.isActive('bulletList') ?? false,
      ordered: e?.isActive('orderedList') ?? false,
      quote: e?.isActive('blockquote') ?? false,
      link: e?.isActive('link') ?? false,
      table: e?.isActive('table') ?? false,
      mermaid: e?.isActive('mermaid') ?? false
    };
  });

  function setLink() {
    if (!editor) return;
    const prev = (editor.getAttributes('link').href as string) ?? '';
    const url = window.prompt('Link URL', prev);
    if (url === null) return;
    if (url === '') {
      editor.chain().focus().extendMarkRange('link').unsetLink().run();
    } else {
      editor.chain().focus().extendMarkRange('link').setLink({ href: url }).run();
    }
  }

  // ---- media upload ----
  function insertMediaNode(file: File, src: string, name: string) {
    if (!editor) return;
    if (file.type.startsWith('video/')) {
      editor.chain().focus().setVideo({ src, title: name }).run();
    } else {
      editor.chain().focus().setImage({ src, alt: name }).run();
    }
  }

  async function uploadAndInsert(file: File) {
    if (!editor) return;
    // No issue yet (creation flow): buffer the file and show a local preview;
    // the real upload happens on save via uploadPending().
    if (!issueId) {
      const blobUrl = URL.createObjectURL(file);
      pending.set(blobUrl, file);
      insertMediaNode(file, blobUrl, file.name);
      return;
    }
    const fd = new FormData();
    fd.append('file', file);
    uploading = true;
    try {
      const att = await api.upload<{ id: string; mimeType: string; filename: string }>(
        `/api/issues/${issueId}/attachments`,
        fd
      );
      insertMediaNode(file, api.downloadUrl(att.id), att.filename);
      onMediaAdded?.();
    } catch (e) {
      toasts.error(e instanceof Error ? e.message : 'Upload failed');
    } finally {
      uploading = false;
    }
  }

  /** Pull image/video files out of a paste/drop event; returns true if handled. */
  function handleFiles(dt: DataTransfer | null, event: Event): boolean {
    if (!editable || !dt) return false;
    const files = Array.from(dt.files).filter(
      (f) => f.type.startsWith('image/') || f.type.startsWith('video/')
    );
    if (files.length === 0) return false;
    event.preventDefault();
    files.forEach((f) => void uploadAndInsert(f));
    return true;
  }

  function onPick(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files) Array.from(input.files).forEach((f) => void uploadAndInsert(f));
    input.value = '';
  }
</script>

<div class="rich-editor" class:rich-editable={editable}>
  {#if editable}
    <div class="flex flex-wrap items-center gap-0.5 border-b border-slate-200 px-1.5 py-1">
      <button type="button" class="te-btn" class:te-on={active.bold} title="Bold"
        onclick={() => editor?.chain().focus().toggleBold().run()}><b>B</b></button>
      <button type="button" class="te-btn" class:te-on={active.italic} title="Italic"
        onclick={() => editor?.chain().focus().toggleItalic().run()}><i>I</i></button>
      <button type="button" class="te-btn" class:te-on={active.strike} title="Strikethrough"
        onclick={() => editor?.chain().focus().toggleStrike().run()}><s>S</s></button>
      <span class="te-sep"></span>
      <button type="button" class="te-btn" class:te-on={active.h1} title="Heading 1"
        onclick={() => editor?.chain().focus().toggleHeading({ level: 1 }).run()}>H1</button>
      <button type="button" class="te-btn" class:te-on={active.h2} title="Heading 2"
        onclick={() => editor?.chain().focus().toggleHeading({ level: 2 }).run()}>H2</button>
      <span class="te-sep"></span>
      <button type="button" class="te-btn" class:te-on={active.bullet} title="Bullet list"
        onclick={() => editor?.chain().focus().toggleBulletList().run()}>•</button>
      <button type="button" class="te-btn" class:te-on={active.ordered} title="Numbered list"
        onclick={() => editor?.chain().focus().toggleOrderedList().run()}>1.</button>
      <button type="button" class="te-btn" class:te-on={active.quote} title="Quote"
        onclick={() => editor?.chain().focus().toggleBlockquote().run()}>❝</button>
      <button type="button" class="te-btn" class:te-on={active.code} title="Inline code"
        onclick={() => editor?.chain().focus().toggleCode().run()}>{'</>'}</button>
      <span class="te-sep"></span>
      <button type="button" class="te-btn" class:te-on={active.link} title="Link" onclick={setLink}>🔗</button>
      <button
        type="button"
        class="te-btn"
        class:te-on={active.table}
        title="Insert table"
        onclick={() =>
          editor?.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()}>
        <svg viewBox="0 0 16 16" class="h-3.5 w-3.5" fill="none" stroke="currentColor" stroke-width="1.4">
          <rect x="1.5" y="2.5" width="13" height="11" rx="1" />
          <line x1="1.5" y1="6" x2="14.5" y2="6" />
          <line x1="1.5" y1="10" x2="14.5" y2="10" />
          <line x1="6" y1="2.5" x2="6" y2="13.5" />
          <line x1="10.5" y1="2.5" x2="10.5" y2="13.5" />
        </svg>
      </button>
      <button
        type="button"
        class="te-btn"
        title="Insert image or video"
        disabled={uploading}
        onclick={() => fileInput?.click()}>{uploading ? '…' : '📷'}</button>
      <button
        type="button"
        class="te-btn"
        class:te-on={active.mermaid}
        title="Insert mermaid diagram"
        onclick={() => editor?.chain().focus().setMermaid().run()}>
        <svg viewBox="0 0 16 16" class="h-3.5 w-3.5" fill="none" stroke="currentColor" stroke-width="1.4">
          <rect x="5" y="1.5" width="6" height="3.5" rx="0.8" />
          <rect x="1" y="10.5" width="5.5" height="3.5" rx="0.8" />
          <rect x="9.5" y="10.5" width="5.5" height="3.5" rx="0.8" />
          <path d="M8 5v2.5M8 7.5H3.75v3M8 7.5h4.5v3" />
        </svg>
      </button>
      {#if active.table}
        <span class="te-sep"></span>
        <button type="button" class="te-btn" title="Add column"
          onclick={() => editor?.chain().focus().addColumnAfter().run()}>+Col</button>
        <button type="button" class="te-btn" title="Add row"
          onclick={() => editor?.chain().focus().addRowAfter().run()}>+Row</button>
        <button type="button" class="te-btn" title="Delete column"
          onclick={() => editor?.chain().focus().deleteColumn().run()}>−Col</button>
        <button type="button" class="te-btn" title="Delete row"
          onclick={() => editor?.chain().focus().deleteRow().run()}>−Row</button>
        <button type="button" class="te-btn" title="Delete table"
          onclick={() => editor?.chain().focus().deleteTable().run()}>🗑</button>
      {/if}
      <input
        bind:this={fileInput}
        type="file"
        accept="image/*,video/*"
        multiple
        class="hidden"
        onchange={onPick}
      />
    </div>
  {/if}
  <div bind:this={element} class:te-content={editable}></div>
</div>
