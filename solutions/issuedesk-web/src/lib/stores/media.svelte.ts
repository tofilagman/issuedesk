/** Global media lightbox / video popup state (Svelte 5 runes). */
export interface MediaItem {
  kind: 'image' | 'video';
  /** Attachment id; the viewer fetches its own object URL so its lifetime is
      decoupled from whatever thumbnail triggered it. */
  attachmentId: string;
  name?: string;
}

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
