<script lang="ts">
  import type { ReportRow } from '../types';

  export let rows: ReportRow[];
  export let compact = false;

  function fmt(value: number, digits = 1) {
    return Number.isFinite(value) ? value.toLocaleString(undefined, { maximumFractionDigits: digits }) : '0';
  }

  function duration(seconds: number) {
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    return `${h}h ${m}m`;
  }
</script>

<table>
  <thead>
    <tr>
      <th>Name</th><th>Sessions</th><th>Avg ex/h</th><th>Avg ex/map</th><th>Maps</th>{#if !compact}<th>Time</th><th>Total</th><th>Best</th><th>Worst</th>{/if}
    </tr>
  </thead>
  <tbody>
    {#each rows as row}
      <tr>
        <td>{row.group_name}</td>
        <td>{row.sessions}</td>
        <td>{fmt(row.average_profit_per_hour)}</td>
        <td>{fmt(row.average_profit_per_map)}</td>
        <td>{row.total_maps}</td>
        {#if !compact}
          <td>{duration(row.total_time_seconds)}</td>
          <td class:profit={row.total_net_profit >= 0} class:loss={row.total_net_profit < 0}>{fmt(row.total_net_profit)} ex</td>
          <td>{fmt(row.best_session_profit)} ex</td>
          <td>{fmt(row.worst_session_profit)} ex</td>
        {/if}
      </tr>
    {/each}
  </tbody>
</table>
