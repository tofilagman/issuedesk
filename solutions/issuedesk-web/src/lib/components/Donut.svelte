<script lang="ts">
  // A dependency-free donut chart built from a CSS conic-gradient with a hole.
  let {
    segments,
    size = 150,
    thickness = 22,
    centerLabel = 'total'
  }: {
    segments: { label: string; value: number; color: string }[];
    size?: number;
    thickness?: number;
    centerLabel?: string;
  } = $props();

  const sum = $derived(segments.reduce((a, s) => a + s.value, 0));
  const gradient = $derived.by(() => {
    if (sum === 0) return '#e2e8f0';
    let acc = 0;
    const parts: string[] = [];
    for (const s of segments) {
      const start = (acc / sum) * 100;
      acc += s.value;
      const end = (acc / sum) * 100;
      parts.push(`${s.color} ${start}% ${end}%`);
    }
    return `conic-gradient(${parts.join(',')})`;
  });
</script>

<div class="flex items-center gap-5">
  <div class="relative shrink-0" style="width:{size}px;height:{size}px">
    <div class="h-full w-full rounded-full" style="background:{gradient}"></div>
    <div
      class="absolute inset-0 m-auto flex flex-col items-center justify-center rounded-full bg-white"
      style="width:{size - thickness * 2}px;height:{size - thickness * 2}px"
    >
      <span class="text-2xl font-semibold text-slate-800">{sum}</span>
      <span class="text-[11px] text-slate-400">{centerLabel}</span>
    </div>
  </div>
  <ul class="space-y-1.5 text-sm">
    {#each segments as s (s.label)}
      <li class="flex items-center gap-2">
        <span class="h-2.5 w-2.5 shrink-0 rounded-sm" style="background:{s.color}"></span>
        <span class="text-slate-600">{s.label}</span>
        <span class="ml-auto pl-3 font-medium text-slate-500">{s.value}</span>
      </li>
    {/each}
  </ul>
</div>
