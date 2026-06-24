<script lang="ts">
  import type { ChaseItem } from '../types';

  export let value = '[]';
  export let chaseItems: ChaseItem[] = [];

  let selected = new Set<string>();
  let lastValue = '';

  function parseValue(raw: string) {
    try {
      const rows = JSON.parse(raw || '[]') as Array<unknown>;
      if (!Array.isArray(rows)) return new Set<string>();
      return new Set(
        rows
          .map((row) => {
            if (typeof row === 'string') return row.trim();
            if (row && typeof row === 'object') {
              const objectRow = row as Record<string, unknown>;
              return String(objectRow.name || objectRow.item_name || '').trim();
            }
            return '';
          })
          .filter(Boolean)
      );
    } catch {
      return new Set<string>();
    }
  }

  function syncValue() {
    value = JSON.stringify([...selected]);
    lastValue = value;
  }

  function toggle(name: string, checked: boolean) {
    selected = new Set(selected);
    if (checked) selected.add(name);
    else selected.delete(name);
    syncValue();
  }

  $: if (value !== lastValue) {
    selected = parseValue(value);
    lastValue = value;
  }
</script>

<div class="selector-grid">
  {#if chaseItems.length === 0}
    <p class="muted">No chase items configured.</p>
  {:else}
    {#each chaseItems as item}
      <label class="check-row">
        <input
          type="checkbox"
          checked={selected.has(item.name)}
          on:change={(event) => toggle(item.name, event.currentTarget.checked)}
        />
        <span>{item.name}</span>
      </label>
    {/each}
  {/if}
</div>
