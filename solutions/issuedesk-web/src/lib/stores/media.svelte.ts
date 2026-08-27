/** Global media lightbox / video popup state (Svelte 5 runes). */
export interface MediaAttachment {
  kind: 'image' | 'video';
  /** Attachment id; the viewer fetches its own object URL so its lifetime is
      decoupled from whatever thumbnail triggered it. */
  attachmentId: string;
  name?: string;
}

/** An inline diagram (mermaid) — already-rendered, already-sanitized SVG markup,
    so the viewer has nothing to fetch. */
export interface MediaDiagram {
  kind: 'diagram';
  svg: string;
  name?: string;
}

export type MediaItem = MediaAttachment | MediaDiagram;

class MediaViewerState {
  current = $state<MediaItem | null>(null);

  open(item: MediaItem) {
    this.current = item;
  }
  close() {
    this.current = null;
  }
}

export const mediaViewer = new MediaViewerState();
