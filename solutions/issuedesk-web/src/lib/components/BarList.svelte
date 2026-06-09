<script lang="ts">
  let {
    items
  }: {
    items: { label: string; value: number; color: string }[];
  } = $props();

  const max = $derived(Math.max(1, ...items.map((i) => i.value)));
</script>

<ul class="space-y-2.5">
  {#each items as it (it.label)}
    <li>
      <div class="mb-1 flex items-center justify-between text-xs">
        <span class="text-slate-600">{it.label}</span>
        <span class="font-medium text-slate-500">{it.value}</span>
      </div>
      <div class="h-2 w-full overflow-hidden rounded-full bg-slate-100">
        <div
          class="h-full rounded-full transition-all"
          style="width:{(it.value / max) * 100}%;background:{it.color}"
        ></div>
      </div>
    </li>
  {/each}
</ul>
