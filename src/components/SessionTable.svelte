<script lang="ts">
  import type { FarmSession } from '../types';

  export let rows: FarmSession[];
  export let go: (path: string) => void;

  function fmt(value: number, digits = 1) {
    return Number.isFinite(value) ? value.toLocaleString(undefined, { maximumFractionDigits: digits }) : '0';
  }
</script>

<table>
  <thead><tr><th>Strategy</th><th>Mechanic</th><th>Maps</th><th>Net</th><th>Status</th></tr></thead>
  <tbody>
    {#if rows.length === 0}
      <tr><td class="empty" colspan="5">Ingen økter ennå.</td></tr>
    {:else}
      {#each rows as row}
      <tr
        class="clickable"
        role="button"
        tabindex="0"
        on:click={() => go(`/sessions/${row.id}`)}
        on:keydown={(event) => {
          if (event.key === 'Enter' || event.key === ' ') {
            event.preventDefault();
            go(`/sessions/${row.id}`);
          }
        }}
      >
        <td>{row.strategy_name}</td>
        <td>{row.mechanic_name}</td>
        <td>{row.maps_run}</td>
        <td class:profit={row.net_profit_exalts >= 0} class:loss={row.net_profit_exalts < 0}>{fmt(row.net_profit_exalts)} ex</td>
        <td>{row.status}</td>
      </tr>
      {/each}
    {/if}
  </tbody>
</table>
