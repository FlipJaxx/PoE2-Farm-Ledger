<script lang="ts">
  import type { SessionLine } from '../types';

  export let rows: SessionLine[];
  export let updateInvestment: (line: SessionLine) => Promise<void>;
  export let fmt: (value: number, digits?: number) => string;
</script>

<table>
  <thead>
    <tr><th>Type</th><th>Item</th><th>Count</th><th>Value/ex</th><th>Total</th></tr>
  </thead>
  <tbody>
    {#if rows.length === 0}
      <tr><td class="empty" colspan="5">No data yet.</td></tr>
    {:else}
      {#each rows as line}
      <tr>
        <td><input bind:value={line.investment_type} on:change={() => updateInvestment(line)} /></td>
        <td>{line.item_name}</td>
        <td><input type="number" min="0" step="0.01" bind:value={line.count} on:change={() => updateInvestment(line)} /></td>
        <td><input type="number" min="0" step="0.01" bind:value={line.value_in_exalts_snapshot} on:change={() => updateInvestment(line)} /></td>
        <td>{fmt(line.total_value_exalts)} ex</td>
      </tr>
      {/each}
    {/if}
  </tbody>
</table>
