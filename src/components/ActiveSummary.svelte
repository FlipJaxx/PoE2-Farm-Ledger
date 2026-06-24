<script lang="ts">
  import type { SessionDetail } from '../types';
  import { fmtNumber, fmtValue } from '../format';

  export let detail: SessionDetail;
  export let runningSeconds: number;
  export let duration: (seconds: number) => string;

  $: rate = detail.session.divine_value_exalts_snapshot;
  $: hours = Math.max(0, runningSeconds) / 3600;
  $: liveProfitPerHour = hours > 0 ? detail.session.net_profit_exalts / hours : 0;
  $: liveMapsPerHour = hours > 0 ? detail.session.maps_run / hours : 0;
  $: liveDivinePerHour = rate > 0 ? liveProfitPerHour / rate : 0;
</script>

<section class="metric-grid">
  <article>
    <span>Net profit</span>
    <strong class:profit={detail.session.net_profit_exalts >= 0} class:loss={detail.session.net_profit_exalts < 0}>{fmtValue(detail.session.net_profit_exalts, rate)}</strong>
    <small>{fmtNumber(liveDivinePerHour, 2)} div/hour</small>
  </article>
  <article><span>Loot</span><strong>{fmtValue(detail.session.total_loot_value_exalts, rate)}</strong><small>gross</small></article>
  <article><span>Investment</span><strong>{fmtValue(detail.session.total_investment_value_exalts, rate)}</strong><small>cost</small></article>
  <article><span>Profit/hour</span><strong>{fmtValue(liveProfitPerHour, rate)}</strong><small>{duration(runningSeconds)}</small></article>
  <article><span>Profit/map</span><strong>{fmtValue(detail.session.profit_per_map_exalts, rate)}</strong><small>{detail.session.maps_run} maps</small></article>
  <article><span>Maps/hour</span><strong>{fmtNumber(liveMapsPerHour)}</strong><small>{detail.session.mechanic_name}</small></article>
</section>
