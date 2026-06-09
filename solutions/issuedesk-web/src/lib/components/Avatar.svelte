<script lang="ts">
  import { beamData, nextMaskId } from '$lib/avatar';

  let {
    seed,
    name = '',
    size = 28,
    title = undefined
  }: {
    /** Stable seed (user id preferred); falls back to name. */
    seed?: string;
    name?: string;
    size?: number;
    title?: string;
  } = $props();

  const d = $derived(beamData(seed || name || 'anon'));
  const maskId = nextMaskId();
</script>

<svg
  viewBox="0 0 36 36"
  fill="none"
  width={size}
  height={size}
  class="shrink-0 rounded-full"
  role="img"
  aria-label={title ?? name}
>
  {#if title ?? name}<title>{title ?? name}</title>{/if}
  <mask id={maskId} maskUnits="userSpaceOnUse" x="0" y="0" width="36" height="36">
    <rect width="36" height="36" rx="72" fill="#fff" />
  </mask>
  <g mask="url(#{maskId})">
    <rect width="36" height="36" fill={d.backgroundColor} />
    <rect
      x="0"
      y="0"
      width="36"
      height="36"
      transform="translate({d.wrapperTranslateX} {d.wrapperTranslateY}) rotate({d.wrapperRotate} 18 18) scale({d.wrapperScale})"
      fill={d.wrapperColor}
      rx={d.isCircle ? 36 : 6}
    />
    <g
      transform="translate({d.faceTranslateX} {d.faceTranslateY}) rotate({d.faceRotate} 18 18)"
    >
      {#if d.isMouthOpen}
        <path
          d="M15 {19 + d.mouthSpread}c2 1 4 1 6 0"
          stroke={d.faceColor}
          fill="none"
          stroke-linecap="round"
        />
      {:else}
        <path d="M13,{19 + d.mouthSpread} a1,0.75 0 0,0 10,0" fill={d.faceColor} />
      {/if}
      <rect x={14 - d.eyeSpread} y="14" width="1.5" height="2" rx="1" fill={d.faceColor} />
      <rect x={20 + d.eyeSpread} y="14" width="1.5" height="2" rx="1" fill={d.faceColor} />
    </g>
  </g>
</svg>
