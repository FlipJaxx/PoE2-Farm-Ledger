<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from './api';
  import ActiveSummary from './components/ActiveSummary.svelte';
  import EditableInvestment from './components/EditableInvestment.svelte';
  import EditableLoot from './components/EditableLoot.svelte';
  import PriceTable from './components/PriceTable.svelte';
  import ReadOnlyLines from './components/ReadOnlyLines.svelte';
  import ReportTable from './components/ReportTable.svelte';
  import SessionTable from './components/SessionTable.svelte';
  import type {
    ChaseItem,
    Currency,
    DashboardData,
    FarmSession,
    Mechanic,
    ReportsData,
    SessionDetail,
    SessionLine,
    Strategy
  } from './types';

  let path = window.location.pathname;
  let ready = false;
  let error = '';
  let dashboard: DashboardData | null = null;
  let active: SessionDetail | null = null;
  let selectedSession: SessionDetail | null = null;
  let sessions: FarmSession[] = [];
  let currencies: Currency[] = [];
  let chaseItems: ChaseItem[] = [];
  let mechanics: Mechanic[] = [];
  let strategies: Strategy[] = [];
  let reports: ReportsData | null = null;
  let tick = Date.now();

  let newSession = {
    strategy_id: null as number | null,
    strategy_name: '',
    mechanic_id: null as number | null,
    mechanic_name: 'Generic Mapping',
    character_name: '',
    league: '',
    map_tier: '',
    notes: ''
  };

  let customLoot = { item_type: 'custom', item_name: '', count: 1, value_in_exalts: 0 };
  let investment = { investment_type: 'Maps', item_name: '', count: 1, value_in_exalts: 0 };
  let newCurrency = { name: '', short_name: '', value_in_exalts: 0 };
  let newChase = { name: '', value_in_exalts: 0, notes: '' };
  let strategyForm = {
    id: 0,
    name: '',
    mechanic_id: null as number | null,
    description: '',
    default_notes: '',
    default_investment_rows: '[]',
    default_chase_items: '[]'
  };

  const nav = [
    ['/', 'Dashboard'],
    ['/sessions/new', 'New Session'],
    ['/sessions/active', 'Active'],
    ['/prices', 'Prices'],
    ['/strategies', 'Strategies'],
    ['/reports', 'Reports'],
    ['/settings', 'Settings']
  ];

  onMount(async () => {
    window.addEventListener('popstate', () => {
      path = window.location.pathname;
      loadRoute();
    });
    setInterval(() => (tick = Date.now()), 1000);
    await boot();
  });

  async function boot() {
    try {
      await api.initializeDatabase();
      await loadShared();
      await loadRoute();
      ready = true;
    } catch (err) {
      error = String(err);
    }
  }

  async function loadShared() {
    [mechanics, strategies] = await Promise.all([api.mechanics(), api.strategies()]);
  }

  async function loadRoute() {
    error = '';
    try {
      if (path === '/') dashboard = await api.dashboard();
      if (path === '/sessions/active') active = await api.activeSession();
      if (path === '/prices') [currencies, chaseItems] = await Promise.all([api.currencies(), api.chaseItems()]);
      if (path === '/strategies') await loadShared();
      if (path === '/reports') reports = await api.reports();
      if (path === '/sessions' || path.startsWith('/sessions/')) sessions = await api.sessions();
      const sessionMatch = path.match(/^\/sessions\/(\d+)$/);
      if (sessionMatch) selectedSession = await api.session(Number(sessionMatch[1]));
    } catch (err) {
      error = String(err);
    }
  }

  function go(next: string) {
    history.pushState({}, '', next);
    path = next;
    loadRoute();
  }

  function fmt(value: number, digits = 1) {
    return Number.isFinite(value) ? value.toLocaleString(undefined, { maximumFractionDigits: digits }) : '0';
  }

  function duration(seconds: number) {
    const s = Math.max(0, Math.floor(seconds));
    const h = Math.floor(s / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`;
  }

  function runningDuration(session: FarmSession | null) {
    if (!session || session.status !== 'running') return session?.duration_seconds ?? 0;
    return Math.floor((tick - new Date(session.started_at).getTime()) / 1000);
  }

  async function createSession() {
    const mechanic = mechanics.find((m) => m.id === newSession.mechanic_id);
    const strategy = strategies.find((s) => s.id === newSession.strategy_id);
    const payload = {
      ...newSession,
      strategy_name: strategy?.name || newSession.strategy_name || 'Manual Strategy',
      mechanic_name: mechanic?.name || newSession.mechanic_name || 'Custom'
    };
    await api.createSession(payload);
    go('/sessions/active');
  }

  async function changeMaps(delta: number) {
    if (!active) return;
    active.session = await api.updateMaps(active.session.id, active.session.maps_run + delta);
    active = await api.activeSession();
  }

  async function setMaps(value: number) {
    if (!active) return;
    active.session = await api.updateMaps(active.session.id, value);
    active = await api.activeSession();
  }

  async function updateLoot(line: SessionLine) {
    if (!active) return;
    active = await api.updateLoot({
      session_id: active.session.id,
      item_type: line.item_type || 'custom',
      item_name: line.item_name,
      count: Number(line.count) || 0,
      value_in_exalts: Number(line.value_in_exalts_snapshot) || 0
    });
  }

  async function addCustomLoot() {
    if (!active || !customLoot.item_name.trim()) return;
    active = await api.updateLoot({
      session_id: active.session.id,
      item_type: customLoot.item_type,
      item_name: customLoot.item_name.trim(),
      count: customLoot.count,
      value_in_exalts: customLoot.value_in_exalts
    });
    customLoot = { item_type: 'custom', item_name: '', count: 1, value_in_exalts: 0 };
  }

  async function addInvestment() {
    if (!active || !investment.item_name.trim()) return;
    active = await api.updateInvestment({
      session_id: active.session.id,
      investment_type: investment.investment_type,
      item_name: investment.item_name.trim(),
      count: investment.count,
      value_in_exalts: investment.value_in_exalts
    });
    investment = { investment_type: 'Maps', item_name: '', count: 1, value_in_exalts: 0 };
  }

  async function updateInvestment(line: SessionLine) {
    if (!active) return;
    active = await api.updateInvestment({
      session_id: active.session.id,
      investment_type: line.investment_type || 'Misc',
      item_name: line.item_name,
      count: Number(line.count) || 0,
      value_in_exalts: Number(line.value_in_exalts_snapshot) || 0
    });
  }

  async function stopActive() {
    if (!active) return;
    const stopped = await api.stopSession(active.session.id);
    go(`/sessions/${stopped.id}`);
  }

  async function cancelActive() {
    if (!active) return;
    await api.cancelSession(active.session.id);
    go('/');
  }

  async function saveCurrency(currency: Currency) {
    await api.updateCurrencyValue(currency.id, Number(currency.value_in_exalts) || 0);
  }

  async function saveChase(item: ChaseItem) {
    await api.updateChaseItemValue(item.id, Number(item.default_value_in_exalts) || 0);
  }

  async function addCurrency() {
    if (!newCurrency.name.trim()) return;
    await api.createCustomCurrency(newCurrency.name, newCurrency.short_name, newCurrency.value_in_exalts);
    newCurrency = { name: '', short_name: '', value_in_exalts: 0 };
    [currencies, chaseItems] = await Promise.all([api.currencies(), api.chaseItems()]);
  }

  async function addChase() {
    if (!newChase.name.trim()) return;
    await api.createChaseItem(newChase.name, newChase.value_in_exalts, newChase.notes);
    newChase = { name: '', value_in_exalts: 0, notes: '' };
    [currencies, chaseItems] = await Promise.all([api.currencies(), api.chaseItems()]);
  }

  function editStrategy(strategy: Strategy) {
    strategyForm = {
      id: strategy.id,
      name: strategy.name,
      mechanic_id: strategy.mechanic_id,
      description: strategy.description,
      default_notes: strategy.default_notes,
      default_investment_rows: strategy.default_investment_rows,
      default_chase_items: strategy.default_chase_items
    };
  }

  async function saveStrategy() {
    if (!strategyForm.name.trim()) return;
    const input = { ...strategyForm };
    if (strategyForm.id) await api.updateStrategy(input);
    else await api.createStrategy(input);
    strategyForm = {
      id: 0,
      name: '',
      mechanic_id: null,
      description: '',
      default_notes: '',
      default_investment_rows: '[]',
      default_chase_items: '[]'
    };
    await loadShared();
  }

  async function deleteStrategy(id: number) {
    await api.deleteStrategy(id);
    await loadShared();
  }
</script>

<main class="app-shell">
  <aside class="sidebar">
    <div class="brand">
      <span class="sigil">E</span>
      <div>
        <strong>Exile Farm Ledger</strong>
        <small>Manual PoE2 farm accounting</small>
      </div>
    </div>
    <nav>
      {#each nav as item}
        <a href={item[0]} class:active={path === item[0]} on:click|preventDefault={() => go(item[0])}>{item[1]}</a>
      {/each}
    </nav>
  </aside>

  <section class="content">
    {#if error}
      <div class="notice danger">{error}</div>
    {/if}

    {#if !ready && !error}
      <div class="notice">Initializing local ledger database...</div>
    {:else if path === '/'}
      <header class="page-head">
        <div>
          <h1>Dashboard</h1>
          <p>Local-first manual farming ledger using Exalted Orb as the base currency.</p>
        </div>
        <button on:click={() => go('/sessions/new')}>New Session</button>
      </header>

      {#if dashboard}
        <div class="metric-grid">
          <article>
            <span>Total profit</span>
            <strong class:profit={dashboard.total_profit_exalts >= 0} class:loss={dashboard.total_profit_exalts < 0}>{fmt(dashboard.total_profit_exalts)} ex</strong>
            <small>{fmt(dashboard.total_profit_divines, 2)} div</small>
          </article>
          <article><span>Sessions</span><strong>{dashboard.total_sessions}</strong><small>completed</small></article>
          <article><span>Maps</span><strong>{dashboard.total_maps}</strong><small>logged runs</small></article>
          <article><span>Time</span><strong>{duration(dashboard.total_time_seconds)}</strong><small>completed</small></article>
        </div>

        <section class="panel">
          <h2>Active session</h2>
          {#if dashboard.active_session}
            <div class="active-strip">
              <div>
                <strong>{dashboard.active_session.strategy_name}</strong>
                <span>{dashboard.active_session.mechanic_name} - {dashboard.active_session.maps_run} maps - {duration(runningDuration(dashboard.active_session))}</span>
              </div>
              <button on:click={() => go('/sessions/active')}>Open</button>
            </div>
          {:else}
            <p class="muted">No running session.</p>
          {/if}
        </section>

        <div class="two-col">
          <section class="panel">
            <h2>Recent sessions</h2>
            <SessionTable rows={dashboard.recent_sessions} {go} />
          </section>
          <section class="panel">
            <h2>Best strategies</h2>
            <ReportTable rows={dashboard.best_strategies} compact />
          </section>
        </div>
      {/if}
    {:else if path === '/sessions/new'}
      <header class="page-head"><h1>New Session</h1></header>
      <section class="panel form-grid">
        <label>Strategy<select bind:value={newSession.strategy_id}><option value={null}>Manual strategy</option>{#each strategies as s}<option value={s.id}>{s.name}</option>{/each}</select></label>
        <label>Strategy name<input bind:value={newSession.strategy_name} placeholder="Manual strategy name" /></label>
        <label>Mechanic<select bind:value={newSession.mechanic_id}><option value={null}>Choose mechanic</option>{#each mechanics as m}<option value={m.id}>{m.name}</option>{/each}</select></label>
        <label>Character / build<input bind:value={newSession.character_name} /></label>
        <label>League<input bind:value={newSession.league} /></label>
        <label>Map tier / area level<input bind:value={newSession.map_tier} /></label>
        <label class="span-2">Notes<textarea bind:value={newSession.notes}></textarea></label>
        <button class="span-2" on:click={createSession}>Start Session</button>
      </section>
    {:else if path === '/sessions/active'}
      <header class="page-head">
        <h1>Active Session</h1>
        {#if active}<div class="timer">{duration(runningDuration(active.session))}</div>{/if}
      </header>
      {#if !active}
        <section class="panel"><p class="muted">No running session.</p><button on:click={() => go('/sessions/new')}>Start one</button></section>
      {:else}
        <ActiveSummary detail={active} runningSeconds={runningDuration(active.session)} {fmt} {duration} />
        <section class="panel run-counter">
          <button on:click={() => changeMaps(-1)}>-1</button>
          <label>Maps / runs<input type="number" min="0" bind:value={active.session.maps_run} on:change={(e) => setMaps(Number(e.currentTarget.value))} /></label>
          <button on:click={() => changeMaps(1)}>+1</button>
        </section>

        <EditableLoot title="Currency loot" rows={active.loot.filter((l) => l.item_type === 'currency')} {updateLoot} {fmt} />
        <EditableLoot title="Chase items" rows={active.loot.filter((l) => l.item_type === 'chase')} {updateLoot} {fmt} />

        <section class="panel">
          <h2>Custom loot</h2>
          <div class="inline-form">
            <select bind:value={customLoot.item_type}><option value="custom">Custom</option><option value="currency">Currency</option><option value="chase">Chase</option></select>
            <input bind:value={customLoot.item_name} placeholder="Item name" />
            <input type="number" min="0" step="0.01" bind:value={customLoot.count} />
            <input type="number" min="0" step="0.01" bind:value={customLoot.value_in_exalts} />
            <button on:click={addCustomLoot}>Add</button>
          </div>
          <EditableLoot title="Logged custom loot" rows={active.loot.filter((l) => l.item_type === 'custom')} {updateLoot} {fmt} embedded />
        </section>

        <section class="panel">
          <h2>Investments</h2>
          <div class="inline-form">
            <input bind:value={investment.investment_type} placeholder="Type" />
            <input bind:value={investment.item_name} placeholder="Item name" />
            <input type="number" min="0" step="0.01" bind:value={investment.count} />
            <input type="number" min="0" step="0.01" bind:value={investment.value_in_exalts} />
            <button on:click={addInvestment}>Add</button>
          </div>
          <EditableInvestment rows={active.investments} {updateInvestment} {fmt} />
        </section>

        <div class="actions">
          <button class="danger-button" on:click={cancelActive}>Cancel Session</button>
          <button on:click={stopActive}>Stop Session</button>
        </div>
      {/if}
    {:else if path.match(/^\/sessions\/\d+$/)}
      <header class="page-head"><h1>Session Detail</h1><button on:click={() => go('/')}>Back</button></header>
      {#if selectedSession}
        <ActiveSummary detail={selectedSession} runningSeconds={selectedSession.session.duration_seconds} {fmt} {duration} />
        <ReadOnlyLines title="Loot" rows={selectedSession.loot} {fmt} />
        <ReadOnlyLines title="Investments" rows={selectedSession.investments} {fmt} />
      {/if}
    {:else if path === '/prices'}
      <header class="page-head"><h1>Price Settings</h1></header>
      <div class="two-col">
        <section class="panel">
          <h2>Currencies</h2>
          <PriceTable rows={currencies} valueKey="value_in_exalts" save={saveCurrency} />
          <div class="inline-form">
            <input bind:value={newCurrency.name} placeholder="Currency name" />
            <input bind:value={newCurrency.short_name} placeholder="Short" />
            <input type="number" min="0" step="0.01" bind:value={newCurrency.value_in_exalts} />
            <button on:click={addCurrency}>Add</button>
          </div>
        </section>
        <section class="panel">
          <h2>Chase Items</h2>
          <PriceTable rows={chaseItems} valueKey="default_value_in_exalts" save={saveChase} />
          <div class="inline-form">
            <input bind:value={newChase.name} placeholder="Item name" />
            <input type="number" min="0" step="0.01" bind:value={newChase.value_in_exalts} />
            <input bind:value={newChase.notes} placeholder="Notes" />
            <button on:click={addChase}>Add</button>
          </div>
        </section>
      </div>
    {:else if path === '/strategies'}
      <header class="page-head"><h1>Strategies</h1></header>
      <div class="two-col">
        <section class="panel">
          <h2>Manage strategy</h2>
          <div class="form-grid single">
            <label>Name<input bind:value={strategyForm.name} /></label>
            <label>Mechanic<select bind:value={strategyForm.mechanic_id}><option value={null}>None</option>{#each mechanics as m}<option value={m.id}>{m.name}</option>{/each}</select></label>
            <label>Description<textarea bind:value={strategyForm.description}></textarea></label>
            <label>Default notes<textarea bind:value={strategyForm.default_notes}></textarea></label>
            <label>Default investments JSON<textarea bind:value={strategyForm.default_investment_rows}></textarea></label>
            <label>Default chase JSON<textarea bind:value={strategyForm.default_chase_items}></textarea></label>
            <button on:click={saveStrategy}>{strategyForm.id ? 'Update' : 'Create'} Strategy</button>
          </div>
        </section>
        <section class="panel">
          <h2>Saved strategies</h2>
          <table>
            <thead><tr><th>Name</th><th>Mechanic</th><th></th></tr></thead>
            <tbody>{#each strategies as s}<tr><td>{s.name}</td><td>{s.mechanic_name || 'None'}</td><td class="row-actions"><button on:click={() => editStrategy(s)}>Edit</button><button class="ghost" on:click={() => deleteStrategy(s.id)}>Delete</button></td></tr>{/each}</tbody>
          </table>
        </section>
      </div>
    {:else if path === '/reports'}
      <header class="page-head"><h1>Reports</h1></header>
      {#if reports}
        <section class="panel"><h2>By mechanic</h2><ReportTable rows={reports.by_mechanic} /></section>
        <section class="panel"><h2>By strategy</h2><ReportTable rows={reports.by_strategy} /></section>
      {/if}
    {:else if path === '/settings'}
      <header class="page-head"><h1>Settings</h1></header>
      <section class="panel prose">
        <h2>MVP boundaries</h2>
        <p>Exile Farm Ledger is a manual local ledger. It does not interact with the game client, automate gameplay, read memory, read the screen, track stash contents, call trade APIs, or require an online account.</p>
        <p>Historical sessions keep the values captured when the session was created or edited. Recalculation with current prices is intentionally left for a later version.</p>
      </section>
    {/if}
  </section>
</main>
