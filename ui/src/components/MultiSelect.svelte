<script lang="ts">
  type Option = { id: number; name: string; group?: string };
  let {
    label,
    options,
    selected,
    onToggle,
  }: { label: string; options: Option[]; selected: number[]; onToggle: (id: number) => void } = $props();

  const groups = $derived([...new Set(options.map((o) => o.group ?? ''))]);
</script>

<details class="multi">
  <summary class:active={selected.length > 0}>
    {label}{selected.length > 0 ? ` (${selected.length})` : ''} ▾
  </summary>
  <div class="panel">
    {#if options.length === 0}
      <div class="muted">Keine Einträge vorhanden</div>
    {/if}
    {#each groups as group (group)}
      {#if group}<div class="group">{group}</div>{/if}
      {#each options.filter((o) => (o.group ?? '') === group) as option (option.id)}
        <label>
          <input
            type="checkbox"
            checked={selected.includes(option.id)}
            onchange={() => onToggle(option.id)}
          />
          {option.name}
        </label>
      {/each}
    {/each}
  </div>
</details>

<style>
  .multi { position: relative; }
  summary { list-style: none; cursor: pointer; border: 1px solid var(--border); border-radius: 6px; padding: 0.3rem 0.7rem; user-select: none; }
  summary::-webkit-details-marker { display: none; }
  summary.active { border-color: var(--accent); color: var(--accent); font-weight: 600; }
  .panel { position: absolute; z-index: 10; top: calc(100% + 4px); left: 0; min-width: 220px; max-height: 320px; overflow: auto; background: var(--bg); border: 1px solid var(--border); border-radius: 8px; padding: 0.5rem; box-shadow: 0 6px 20px rgb(0 0 0 / 0.12); }
  label { display: flex; gap: 0.4rem; align-items: center; padding: 0.15rem 0; white-space: nowrap; }
  .group { font-size: 0.8rem; color: var(--text-muted); margin-top: 0.4rem; }
</style>
