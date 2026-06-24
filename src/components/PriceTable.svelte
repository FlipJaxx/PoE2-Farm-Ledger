<script lang="ts">
  export let rows: Array<{ id: number; name: string; [key: string]: unknown }>;
  export let valueKey: string;
  export let save: (row: any) => Promise<void>;
</script>

<table>
  <thead><tr><th>Name</th><th>Value in exalts</th><th></th></tr></thead>
  <tbody>
    {#each rows as row}
      <tr>
        <td>{row.name}</td>
        <td>
          <input
            type="number"
            min="0"
            step="0.01"
            value={Number(row[valueKey])}
            on:change={(event) => (row[valueKey] = Number(event.currentTarget.value))}
          />
        </td>
        <td><button on:click={() => save(row)}>Save</button></td>
      </tr>
    {/each}
  </tbody>
</table>
