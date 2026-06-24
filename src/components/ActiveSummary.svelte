<script lang="ts">
  import type { SessionDetail } from '../types';

  export let detail: SessionDetail;
  export let runningSeconds: number;
  export let fmt: (value: number, digits?: number) => string;
  export let duration: (seconds: number) => string;

  $: hours = Math.max(0, runningSeconds) / 3600;
  $: liveProfitPerHour = hours > 0 ? detail.session.net_profit_exalts / hours : 0;
  $: liveMapsPerHour = hours > 0 ? detail.session.maps_run / hours : 0;
  $: liveDivinePerHour =
    detail.session.divine_value_exalts_snapshot > 0 ? liveProfitPerHour / detail.session.divine_value_exalts_snapshot : 0;
</script>

<section class="metric-grid">
  <article>
    <span>Net profit</span>
    <strong class:profit={detail.session.net_profit_exalts >= 0} class:loss={detail.session.net_profit_exalts < 0}>{fmt(detail.session.net_profit_exalts)} ex</strong>
    <small>{fmt(liveDivinePerHour, 2)} div/hour</small>
  </article>
  <article><span>Loot</span><strong>{fmt(detail.session.total_loot_value_exalts)} ex</strong><small>gross</small></article>
  <article><span>Investment</span><strong>{fmt(detail.session.total_investment_value_exalts)} ex</strong><small>cost</small></article>
  <article><span>Profit/hour</span><strong>{fmt(liveProfitPerHour)} ex</strong><small>{duration(runningSeconds)}</small></article>
  <article><span>Profit/map</span><strong>{fmt(detail.session.profit_per_map_exalts)} ex</strong><small>{detail.session.maps_run} maps</small></article>
  <article><span>Maps/hour</span><strong>{fmt(liveMapsPerHour)}</strong><small>{detail.session.mechanic_name}</small></article>
</section>
