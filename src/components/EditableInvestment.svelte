<script lang="ts">
  import type { SessionLine } from '../types';

  export let rows: SessionLine[];
  export let updateInvestment: (line: SessionLine) => Promise<void>;
  export let removeInvestment: ((line: SessionLine) => Promise<void>) | null = null;
  export let fmt: (value: number, digits?: number) => string;
</script>

<table>
  <thead>
    <tr><th>Type</th><th>Item</th><th>Count</th><th>Value/ex</th><th>Total</th>{#if removeInvestment}<th></th>{/if}</tr>
  </thead>
  <tbody>
    {#if rows.length === 0}
      <tr><td class="empty" colspan={removeInvestment ? 6 : 5}>No data yet.</td></tr>
    {:else}
      {#each rows as line}
      <tr>
        <td><input bind:value={line.investment_type} on:change={() => updateInvestment(line)} /></td>
        <td>{line.item_name}</td>
        <td><input type="number" min="0" step="1" bind:value={line.count} on:change={() => updateInvestment(line)} /></td>
        <td><input type="number" min="0" step="any" bind:value={line.value_in_exalts_snapshot} on:change={() => updateInvestment(line)} /></td>
        <td>{fmt(line.total_value_exalts)} ex</td>
        {#if removeInvestment}
          <td class="row-actions"><button class="ghost danger-button" on:click={() => removeInvestment?.(line)}>Remove</button></td>
        {/if}
      </tr>
      {/each}
    {/if}
  </tbody>
</table>
