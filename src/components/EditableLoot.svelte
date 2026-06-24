<script lang="ts">
  import type { SessionLine } from '../types';

  export let title: string;
  export let rows: SessionLine[];
  export let updateLoot: (line: SessionLine) => Promise<void>;
  export let fmt: (value: number, digits?: number) => string;
  export let embedded = false;
</script>

<section class:panel={!embedded} class:embedded={embedded}>
  <h2>{title}</h2>
  <table>
    <thead>
      <tr><th>Name</th><th>Count</th><th>Value/ex</th><th>Total</th></tr>
    </thead>
    <tbody>
      {#if rows.length === 0}
        <tr><td class="empty" colspan="4">Ingen data ennå.</td></tr>
      {:else}
        {#each rows as line}
        <tr>
          <td>{line.item_name}</td>
          <td><input type="number" min="0" step="0.01" bind:value={line.count} on:change={() => updateLoot(line)} /></td>
          <td><input type="number" min="0" step="0.01" bind:value={line.value_in_exalts_snapshot} on:change={() => updateLoot(line)} /></td>
          <td>{fmt(line.total_value_exalts)} ex</td>
        </tr>
        {/each}
      {/if}
    </tbody>
  </table>
</section>
