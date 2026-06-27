<script lang="ts">
  export let rows: Array<{ id: number; name: string; [key: string]: unknown }>;
  export let valueKey: string;
  export let unitLabel = 'exalts';
  export let save: (row: any) => Promise<void>;
  export let move: ((row: any, direction: -1 | 1) => Promise<void>) | null = null;
  export let remove: ((row: any) => Promise<void>) | null = null;
  export let protectedNames: string[] = [];
  export let divineMode = false;
  export let divineRate = 0;
  export let exaltOnlyNames: string[] = [];

  function showsDivines(row: any) {
    return divineMode && divineRate > 0 && !exaltOnlyNames.includes(row.name);
  }

  // Recompute each row's displayed value reactively. The unit props
  // (divineMode, divineRate, exaltOnlyNames) are referenced directly here so
  // Svelte re-runs this when the unit toggles. A bare displayValue(row) call in
  // the markup would hide those dependencies and the inputs would not refresh.
  $: displayRows = rows.map((row) => ({
    row,
    value:
      divineMode && divineRate > 0 && !exaltOnlyNames.includes(row.name)
        ? Number(((Number(row[valueKey]) || 0) / divineRate).toFixed(1))
        : Number(row[valueKey])
  }));

  async function commit(row: any, value: string) {
    const entered = Number(value) || 0;
    row[valueKey] = showsDivines(row) ? entered * divineRate : entered;
    await save(row);
  }

  $: hasActions = Boolean(move) || Boolean(remove);
</script>

<table>
  <thead>
    <tr><th>Name</th><th>Value in {unitLabel}</th>{#if hasActions}<th></th>{/if}</tr>
  </thead>
  <tbody>
    {#if rows.length === 0}
      <tr><td class="empty" colspan={hasActions ? 3 : 2}>No entries yet.</td></tr>
    {:else}
      {#each displayRows as { row, value }, index}
        <tr>
          <td>{row.name}</td>
          <td>
            <input
              type="number"
              min="0"
              step={divineMode ? '0.1' : 'any'}
              {value}
              on:change={(event) => commit(row, event.currentTarget.value)}
            />
          </td>
          {#if hasActions}
            <td class="row-actions">
              {#if move}
                <button class="ghost" disabled={index === 0} on:click={() => move?.(row, -1)}>Up</button>
                <button class="ghost" disabled={index === displayRows.length - 1} on:click={() => move?.(row, 1)}>Down</button>
              {/if}
              {#if remove && !protectedNames.includes(row.name)}
                <button class="ghost danger-button" on:click={() => remove?.(row)}>Remove</button>
              {/if}
            </td>
          {/if}
        </tr>
      {/each}
    {/if}
  </tbody>
</table>
