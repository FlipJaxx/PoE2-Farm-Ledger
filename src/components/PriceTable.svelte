<script lang="ts">
  export let rows: Array<{ id: number; name: string; [key: string]: unknown }>;
  export let valueKey: string;
  export let unitLabel = 'exalts';
  export let save: (row: any) => Promise<void>;
  export let move: ((row: any, direction: -1 | 1) => Promise<void>) | null = null;
  export let remove: ((row: any) => Promise<void>) | null = null;
  export let protectedNames: string[] = [];

  async function commit(row: any, value: string) {
    row[valueKey] = Number(value);
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
      {#each rows as row, index}
        <tr>
          <td>{row.name}</td>
          <td>
            <input
              type="number"
              min="0"
              step="any"
              value={Number(row[valueKey])}
              on:change={(event) => commit(row, event.currentTarget.value)}
            />
          </td>
          {#if hasActions}
            <td class="row-actions">
              {#if move}
                <button class="ghost" disabled={index === 0} on:click={() => move?.(row, -1)}>Up</button>
                <button class="ghost" disabled={index === rows.length - 1} on:click={() => move?.(row, 1)}>Down</button>
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
