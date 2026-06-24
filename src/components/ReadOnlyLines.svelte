<script lang="ts">
  import type { SessionLine } from '../types';

  export let title: string;
  export let rows: SessionLine[];
  export let fmt: (value: number, digits?: number) => string;
</script>

<section class="panel">
  <h2>{title}</h2>
  <table>
    <thead><tr><th>Type</th><th>Name</th><th>Count</th><th>Value/ex</th><th>Total</th></tr></thead>
    <tbody>
      {#each rows as line}
        <tr>
          <td>{line.item_type || line.investment_type}</td>
          <td>{line.item_name}</td>
          <td>{fmt(line.count, 2)}</td>
          <td>{fmt(line.value_in_exalts_snapshot, 2)}</td>
          <td>{fmt(line.total_value_exalts, 2)} ex</td>
        </tr>
      {/each}
    </tbody>
  </table>
</section>
