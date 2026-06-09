<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Editor } from '@tiptap/core';
  import StarterKit from '@tiptap/starter-kit';
  import Image from '@tiptap/extension-image';
  import Placeholder from '@tiptap/extension-placeholder';
  import { Markdown } from 'tiptap-markdown';
  import { SvelteNodeViewRenderer } from 'svelte-tiptap';
  import { Video } from './Video';
  import MediaNodeView from './MediaNodeView.svelte';
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

    editor = new Editor({
      element,
      editable,
      content: value,
      extensions: [
        StarterKit,
        ImageWithView,
        VideoWithView,
        Placeholder.configure({ placeholder }),
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

  onDestroy(() => editor?.destroy());

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
      link: e?.isActive('link') ?? false
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
  async function uploadAndInsert(file: File) {
    if (!editor) return;
    if (!issueId) {
      toasts.error('Save the issue first, then you can add media.');
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
      const url = api.downloadUrl(att.id);
      if (att.mimeType.startsWith('video/')) {
        editor.chain().focus().setVideo({ src: url, title: att.filename }).run();
      } else {
        editor.chain().focus().setImage({ src: url, alt: att.filename }).run();
      }
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
        title={issueId ? 'Insert image or video' : 'Save the issue first to add media'}
        disabled={!issueId || uploading}
        onclick={() => fileInput?.click()}>{uploading ? '…' : '📷'}</button>
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
