<script lang="ts">
  type DefaultInvestmentRow = {
    investment_type: string;
    item_name: string;
    count: number;
    value_in_exalts: number;
  };

  export let value = '[]';

  let rows: DefaultInvestmentRow[] = [];
  let lastValue = '';

  function parseRows(raw: string): DefaultInvestmentRow[] {
    try {
      const parsed = JSON.parse(raw || '[]') as Array<Record<string, unknown>>;
      if (!Array.isArray(parsed)) return [];
      return parsed.map((row) => ({
        investment_type: String(row.investment_type || 'Maps'),
        item_name: String(row.item_name || ''),
        count: Math.max(0, Number(row.count) || 0),
        value_in_exalts: Math.max(0, Number(row.value_in_exalts) || 0)
      }));
    } catch {
      return [];
    }
  }

  function syncValue() {
    const validRows = rows
      .map((row) => ({
        investment_type: row.investment_type.trim() || 'Misc',
        item_name: row.item_name.trim(),
        count: Math.max(0, Number(row.count) || 0),
        value_in_exalts: Math.max(0, Number(row.value_in_exalts) || 0)
      }))
      .filter((row) => row.item_name);
    value = JSON.stringify(validRows);
    lastValue = value;
  }

  function addRow() {
    rows = [...rows, { investment_type: 'Maps', item_name: '', count: 1, value_in_exalts: 0 }];
    syncValue();
  }

  function removeRow(index: number) {
    rows = rows.filter((_, rowIndex) => rowIndex !== index);
    syncValue();
  }

  $: if (value !== lastValue) {
    rows = parseRows(value);
    lastValue = value;
  }
</script>

<div class="table-editor">
  <table>
    <thead>
      <tr><th>Type</th><th>Item</th><th>Count</th><th>Value/ex</th><th></th></tr>
    </thead>
    <tbody>
      {#if rows.length === 0}
        <tr><td class="empty" colspan="5">No default investments.</td></tr>
      {:else}
        {#each rows as row, index}
          <tr>
            <td><input bind:value={row.investment_type} on:change={syncValue} /></td>
            <td><input bind:value={row.item_name} required on:change={syncValue} /></td>
            <td><input type="number" min="0" step="1" bind:value={row.count} on:change={syncValue} /></td>
            <td><input type="number" min="0" step="any" bind:value={row.value_in_exalts} on:change={syncValue} /></td>
            <td class="row-actions"><button class="ghost danger-button" on:click={() => removeRow(index)}>Remove</button></td>
          </tr>
        {/each}
      {/if}
    </tbody>
  </table>
  <button class="ghost" on:click={addRow}>Add investment</button>
</div>
