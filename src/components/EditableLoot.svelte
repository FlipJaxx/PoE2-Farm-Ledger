<script lang="ts">
  import { fmtValue } from '../format';
  import type { SessionLine } from '../types';

  export let title: string;
  export let rows: SessionLine[];
  export let updateLoot: (line: SessionLine) => Promise<void>;
  export let removeLoot: ((line: SessionLine) => Promise<void>) | null = null;
  export let divineRate = 0;
  export let embedded = false;

  $: unit = divineRate > 0 ? 'div' : 'ex';

  function displayValue(line: SessionLine) {
    return divineRate > 0 ? line.value_in_exalts_snapshot / divineRate : line.value_in_exalts_snapshot;
  }

  async function updateValue(line: SessionLine, value: string) {
    const next = Math.max(0, Number(value) || 0);
    line.value_in_exalts_snapshot = divineRate > 0 ? next * divineRate : next;
    await updateLoot(line);
  }
</script>

<section class:panel={!embedded} class:embedded={embedded}>
  <h2>{title}</h2>
  <table>
    <thead>
      <tr><th>Name</th><th>Count</th><th>Value/{unit}</th><th>Total</th>{#if removeLoot}<th></th>{/if}</tr>
    </thead>
    <tbody>
      {#if rows.length === 0}
        <tr><td class="empty" colspan={removeLoot ? 5 : 4}>No entries yet.</td></tr>
      {:else}
        {#each rows as line}
        <tr>
          <td>{line.item_name}</td>
          <td><input type="number" min="0" step="1" bind:value={line.count} on:change={() => updateLoot(line)} /></td>
          <td>
            <input
              type="number"
              min="0"
              step="any"
              value={displayValue(line)}
              on:change={(event) => updateValue(line, event.currentTarget.value)}
            />
          </td>
          <td>{fmtValue(line.total_value_exalts, divineRate || undefined)}</td>
          {#if removeLoot}
            <td class="row-actions">
              {#if line.item_type === 'custom'}
                <button class="ghost danger-button" on:click={() => removeLoot?.(line)}>Remove</button>
              {/if}
            </td>
          {/if}
        </tr>
        {/each}
      {/if}
    </tbody>
  </table>
</section>
