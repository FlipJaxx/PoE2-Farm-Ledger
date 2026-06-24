<script lang="ts">
  export let rows: Array<{ id: number; name: string; [key: string]: unknown }>;
  export let valueKey: string;
  export let save: (row: any) => Promise<void>;
  export let move: ((row: any, direction: -1 | 1) => Promise<void>) | null = null;

  async function commit(row: any, value: string) {
    row[valueKey] = Number(value);
    await save(row);
  }
</script>

<table>
  <thead>
    <tr><th>Name</th><th>Value in exalts</th>{#if move}<th>Order</th>{/if}</tr>
  </thead>
  <tbody>
    {#if rows.length === 0}
      <tr><td class="empty" colspan={move ? 3 : 2}>No entries yet.</td></tr>
    {:else}
      {#each rows as row, index}
        <tr>
          <td>{row.name}</td>
          <td>
            <input
              type="number"
              min="0"
              step="0.01"
              value={Number(row[valueKey])}
              on:change={(event) => commit(row, event.currentTarget.value)}
            />
          </td>
          {#if move}
            <td class="row-actions">
              <button class="ghost" disabled={index === 0} on:click={() => move?.(row, -1)}>Up</button>
              <button class="ghost" disabled={index === rows.length - 1} on:click={() => move?.(row, 1)}>Down</button>
            </td>
          {/if}
        </tr>
      {/each}
    {/if}
  </tbody>
</table>
