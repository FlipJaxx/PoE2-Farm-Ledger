import { invoke } from '@tauri-apps/api/core';
import type {
  ChaseItem,
  Currency,
  DashboardData,
  FarmSession,
  Mechanic,
  ReportRow,
  ReportsData,
  SessionDetail,
  SessionLine,
  Strategy
} from './types';

const tauriApi = {
  initializeDatabase: () => invoke<string>('initialize_database'),
  dashboard: () => invoke<DashboardData>('get_dashboard_data'),
  activeSession: () => invoke<SessionDetail | null>('get_active_session'),
  session: (id: number) => invoke<SessionDetail>('get_session', { id }),
  sessions: () => invoke<FarmSession[]>('list_sessions'),
  mechanics: () => invoke<Mechanic[]>('list_mechanics'),
  createMechanic: (input: Record<string, unknown>) => invoke<Mechanic>('create_mechanic', { input }),
  strategies: () => invoke<Strategy[]>('list_strategies'),
  currencies: () => invoke<Currency[]>('list_currencies'),
  chaseItems: () => invoke<ChaseItem[]>('list_chase_items'),
  reports: () => invoke<ReportsData>('get_reports_data'),
  createSession: (input: Record<string, unknown>) => invoke<FarmSession>('create_session', { input }),
  updateMaps: (sessionId: number, mapsRun: number) =>
    invoke<FarmSession>('update_session_maps', { sessionId, mapsRun }),
  updateLoot: (input: Record<string, unknown>) => invoke<SessionDetail>('add_or_update_session_loot', { input }),
  updateInvestment: (input: Record<string, unknown>) =>
    invoke<SessionDetail>('add_or_update_session_investment', { input }),
  stopSession: (sessionId: number) => invoke<FarmSession>('stop_session', { sessionId }),
  cancelSession: (sessionId: number) => invoke<FarmSession>('cancel_session', { sessionId }),
  updateCurrencyValue: (id: number, valueInExalts: number) =>
    invoke<void>('update_currency_value', { id, valueInExalts }),
  updateCurrencyOrder: (currencyIds: number[]) => invoke<void>('update_currency_order', { currencyIds }),
  createCustomCurrency: (name: string, shortName: string, valueInExalts: number) =>
    invoke<Currency>('create_custom_currency', { name, shortName, valueInExalts }),
  updateChaseItemValue: (id: number, valueInExalts: number) =>
    invoke<void>('update_chase_item_value', { id, valueInExalts }),
  createChaseItem: (name: string, valueInExalts: number, notes: string) =>
    invoke<ChaseItem>('create_chase_item', { name, valueInExalts, notes }),
  createStrategy: (input: Record<string, unknown>) => invoke<Strategy>('create_strategy', { input }),
  updateStrategy: (input: Record<string, unknown>) => invoke<Strategy>('update_strategy', { input }),
  deleteStrategy: (id: number) => invoke<void>('delete_strategy', { id })
};

type Store = {
  currencies: Currency[];
  currencyOrderVersion?: number;
  chaseItems: ChaseItem[];
  mechanics: Mechanic[];
  strategies: Strategy[];
  sessions: FarmSession[];
  details: Record<number, { loot: SessionLine[]; investments: SessionLine[] }>;
  nextId: number;
};

const storageKey = 'exile-farm-ledger-dev-store';

function initialStore(): Store {
  return {
    currencies: [
      { id: 1, name: 'Exalted Orb', short_name: 'ex', value_in_exalts: 1, display_order: 10, is_default: true, active: true },
      {
        id: 2,
        name: 'Greater Exalted Orb',
        short_name: 'gex',
        value_in_exalts: 10,
        display_order: 20,
        is_default: true,
        active: true
      },
      {
        id: 3,
        name: 'Perfect Exalted Orb',
        short_name: 'pex',
        value_in_exalts: 80,
        display_order: 30,
        is_default: true,
        active: true
      },
      { id: 4, name: 'Chaos Orb', short_name: 'c', value_in_exalts: 0.5, display_order: 40, is_default: true, active: true },
      {
        id: 5,
        name: 'Greater Chaos Orb',
        short_name: 'gc',
        value_in_exalts: 8,
        display_order: 50,
        is_default: true,
        active: true
      },
      {
        id: 6,
        name: 'Perfect Chaos Orb',
        short_name: 'pc',
        value_in_exalts: 60,
        display_order: 60,
        is_default: true,
        active: true
      },
      { id: 7, name: 'Divine Orb', short_name: 'div', value_in_exalts: 120, display_order: 70, is_default: true, active: true }
    ],
    currencyOrderVersion: 1,
    chaseItems: [
      { id: 10, name: 'Perfect Jeweller Orb', default_value_in_exalts: 18, notes: '', active: true },
      { id: 11, name: 'Audience with the King', default_value_in_exalts: 75, notes: '', active: true }
    ],
    mechanics: [
      { id: 20, name: 'Generic Mapping', description: 'Manual mapping session', is_default: true, active: true },
      { id: 21, name: 'Breach', description: 'Breach-focused farm', is_default: true, active: true },
      { id: 22, name: 'Expedition', description: 'Expedition-focused farm', is_default: true, active: true }
    ],
    strategies: [
      {
        id: 30,
        name: 'T15 Breach Loop',
        mechanic_id: 21,
        mechanic_name: 'Breach',
        description: 'Sample browser-dev strategy.',
        default_notes: '',
        default_investment_rows: '[]',
        default_chase_items: '[]',
        active: true
      }
    ],
    sessions: [],
    details: {},
    nextId: 100
  };
}

function loadStore(): Store {
  const stored = localStorage.getItem(storageKey);
  if (!stored) return initialStore();
  const store = normalizeStore(JSON.parse(stored) as Store);
  saveStore(store);
  return store;
}

function saveStore(store: Store) {
  localStorage.setItem(storageKey, JSON.stringify(store));
}

function normalizeStore(store: Store): Store {
  const defaultOrder = new Map(initialStore().currencies.map((currency) => [currency.name, currency.display_order]));
  const hasCurrentCurrencyOrder = store.currencyOrderVersion === 1;
  const usedIds = new Set<number>();
  let maxId = Math.max(store.nextId ?? 0, ...store.currencies.map((currency) => currency.id));
  store.currencies = store.currencies.map((currency, index) => ({
    ...currency,
    display_order: hasCurrentCurrencyOrder
      ? (currency.display_order ?? defaultOrder.get(currency.name) ?? (index + 1) * 10)
      : (defaultOrder.get(currency.name) ?? currency.display_order ?? (index + 1) * 10)
  }));
  store.currencies = store.currencies.map((currency) => {
    if (!usedIds.has(currency.id)) {
      usedIds.add(currency.id);
      return currency;
    }
    maxId += 1;
    usedIds.add(maxId);
    return { ...currency, id: maxId };
  });
  for (const currency of initialStore().currencies) {
    if (!store.currencies.some((row) => row.name === currency.name)) {
      const id = usedIds.has(currency.id) ? maxId + 1 : currency.id;
      maxId = Math.max(maxId, id);
      usedIds.add(id);
      store.currencies.push({ ...currency, id });
    }
  }
  store.currencyOrderVersion = 1;
  store.nextId = Math.max(store.nextId ?? 0, maxId);
  store.currencies.sort((a, b) => a.display_order - b.display_order || a.name.localeCompare(b.name));
  return store;
}

function nextId(store: Store) {
  store.nextId += 1;
  return store.nextId;
}

function lineTotal(count: number, value: number) {
  return Math.max(0, Number(count) || 0) * Math.max(0, Number(value) || 0);
}

function recalculate(store: Store, sessionId: number) {
  const session = store.sessions.find((row) => row.id === sessionId);
  const detail = store.details[sessionId];
  if (!session || !detail) return;

  const loot = detail.loot.reduce((sum, line) => sum + line.total_value_exalts, 0);
  const investment = detail.investments.reduce((sum, line) => sum + line.total_value_exalts, 0);
  const duration =
    session.status === 'running'
      ? Math.max(0, Math.floor((Date.now() - new Date(session.started_at).getTime()) / 1000))
      : session.duration_seconds;
  const hours = duration / 3600;

  session.duration_seconds = duration;
  session.total_loot_value_exalts = loot;
  session.total_investment_value_exalts = investment;
  session.net_profit_exalts = loot - investment;
  session.profit_per_hour_exalts = hours > 0 ? session.net_profit_exalts / hours : 0;
  session.profit_per_map_exalts = session.maps_run > 0 ? session.net_profit_exalts / session.maps_run : 0;
  session.maps_per_hour = hours > 0 ? session.maps_run / hours : 0;
  session.divine_per_hour =
    session.divine_value_exalts_snapshot > 0 ? session.profit_per_hour_exalts / session.divine_value_exalts_snapshot : 0;
}

function sessionDetail(store: Store, id: number): SessionDetail {
  recalculate(store, id);
  const session = store.sessions.find((row) => row.id === id);
  if (!session) throw new Error('Session not found');
  const detail = store.details[id] ?? { loot: [], investments: [] };
  return { session, loot: detail.loot, investments: detail.investments };
}

function emptyReport(groupName: string): ReportRow {
  return {
    group_name: groupName,
    sessions: 0,
    average_profit_per_hour: 0,
    average_profit_per_map: 0,
    total_maps: 0,
    total_time_seconds: 0,
    total_net_profit: 0,
    best_session_profit: 0,
    worst_session_profit: 0
  };
}

function reportsFor(store: Store, key: 'mechanic_name' | 'strategy_name', limit = 100): ReportRow[] {
  const groups = new Map<string, FarmSession[]>();
  for (const session of store.sessions.filter((row) => row.status === 'completed')) {
    const name = session[key] || 'Unassigned';
    groups.set(name, [...(groups.get(name) ?? []), session]);
  }

  return [...groups.entries()]
    .map(([name, rows]) => {
      const report = emptyReport(name);
      report.sessions = rows.length;
      report.total_maps = rows.reduce((sum, row) => sum + row.maps_run, 0);
      report.total_time_seconds = rows.reduce((sum, row) => sum + row.duration_seconds, 0);
      report.total_net_profit = rows.reduce((sum, row) => sum + row.net_profit_exalts, 0);
      report.average_profit_per_hour =
        rows.reduce((sum, row) => sum + row.profit_per_hour_exalts, 0) / Math.max(1, rows.length);
      report.average_profit_per_map =
        rows.reduce((sum, row) => sum + row.profit_per_map_exalts, 0) / Math.max(1, rows.length);
      report.best_session_profit = Math.max(...rows.map((row) => row.net_profit_exalts));
      report.worst_session_profit = Math.min(...rows.map((row) => row.net_profit_exalts));
      return report;
    })
    .sort((a, b) => b.average_profit_per_hour - a.average_profit_per_hour)
    .slice(0, limit);
}

function createBrowserApi() {
  return {
    initializeDatabase: async () => {
      saveStore(loadStore());
      return 'browser-dev-localStorage';
    },
    dashboard: async (): Promise<DashboardData> => {
      const store = loadStore();
      for (const session of store.sessions) recalculate(store, session.id);
      saveStore(store);
      const completed = store.sessions.filter((row) => row.status === 'completed');
      const totalProfit = completed.reduce((sum, row) => sum + row.net_profit_exalts, 0);
      const divine = store.currencies.find((row) => row.name === 'Divine Orb')?.value_in_exalts ?? 120;
      return {
        active_session: store.sessions.find((row) => row.status === 'running') ?? null,
        recent_sessions: store.sessions.slice(0, 6),
        best_strategies: reportsFor(store, 'strategy_name', 5),
        total_sessions: completed.length,
        total_maps: completed.reduce((sum, row) => sum + row.maps_run, 0),
        total_time_seconds: completed.reduce((sum, row) => sum + row.duration_seconds, 0),
        total_profit_exalts: totalProfit,
        total_profit_divines: divine > 0 ? totalProfit / divine : 0
      };
    },
    activeSession: async () => {
      const store = loadStore();
      const active = store.sessions.find((row) => row.status === 'running');
      return active ? sessionDetail(store, active.id) : null;
    },
    session: async (id: number) => sessionDetail(loadStore(), id),
    sessions: async () => loadStore().sessions,
    mechanics: async () => loadStore().mechanics.filter((row) => row.active),
    createMechanic: async (input: Record<string, unknown>) => {
      const store = loadStore();
      const name = String(input.name || '').trim();
      if (!name) throw new Error('Mechanic name is required');
      if (store.mechanics.some((row) => row.name.toLowerCase() === name.toLowerCase() && row.active)) {
        throw new Error('Mechanic already exists');
      }
      const mechanic = {
        id: nextId(store),
        name,
        description: String(input.description || '').trim(),
        is_default: false,
        active: true
      };
      store.mechanics.push(mechanic);
      saveStore(store);
      return mechanic;
    },
    strategies: async () => loadStore().strategies.filter((row) => row.active),
    currencies: async () =>
      loadStore()
        .currencies.filter((row) => row.active)
        .sort((a, b) => a.display_order - b.display_order || a.name.localeCompare(b.name)),
    chaseItems: async () => loadStore().chaseItems.filter((row) => row.active),
    reports: async () => {
      const store = loadStore();
      return { by_mechanic: reportsFor(store, 'mechanic_name'), by_strategy: reportsFor(store, 'strategy_name') };
    },
    createSession: async (input: Record<string, unknown>) => {
      const store = loadStore();
      if (store.sessions.some((row) => row.status === 'running')) throw new Error('Only one farming session can be running at a time');
      const id = nextId(store);
      const session: FarmSession = {
        id,
        strategy_id: Number(input.strategy_id) || null,
        strategy_name: String(input.strategy_name || 'Manual Strategy'),
        mechanic_id: Number(input.mechanic_id) || null,
        mechanic_name: String(input.mechanic_name || 'Custom'),
        character_name: String(input.character_name || ''),
        league: String(input.league || ''),
        map_tier: String(input.map_tier || ''),
        notes: String(input.notes || ''),
        status: 'running',
        started_at: new Date().toISOString(),
        ended_at: null,
        duration_seconds: 0,
        maps_run: 0,
        total_loot_value_exalts: 0,
        total_investment_value_exalts: 0,
        net_profit_exalts: 0,
        profit_per_hour_exalts: 0,
        profit_per_map_exalts: 0,
        maps_per_hour: 0,
        divine_value_exalts_snapshot: store.currencies.find((row) => row.name === 'Divine Orb')?.value_in_exalts ?? 120,
        divine_per_hour: 0
      };
      store.sessions.unshift(session);
      store.details[id] = {
        loot: [
          ...store.currencies.map((row) => ({
            id: nextId(store),
            session_id: id,
            item_type: 'currency',
            investment_type: null,
            item_name: row.name,
            count: 0,
            value_in_exalts_snapshot: row.value_in_exalts,
            total_value_exalts: 0
          })),
          ...store.chaseItems.map((row) => ({
            id: nextId(store),
            session_id: id,
            item_type: 'chase',
            investment_type: null,
            item_name: row.name,
            count: 0,
            value_in_exalts_snapshot: row.default_value_in_exalts,
            total_value_exalts: 0
          }))
        ],
        investments: []
      };
      saveStore(store);
      return session;
    },
    updateMaps: async (sessionId: number, mapsRun: number) => {
      const store = loadStore();
      const session = store.sessions.find((row) => row.id === sessionId);
      if (!session) throw new Error('Session not found');
      session.maps_run = Math.max(0, Number(mapsRun) || 0);
      recalculate(store, sessionId);
      saveStore(store);
      return session;
    },
    updateLoot: async (input: Record<string, unknown>) => {
      const store = loadStore();
      const sessionId = Number(input.session_id);
      const lines = store.details[sessionId].loot;
      const itemName = String(input.item_name || '');
      const itemType = String(input.item_type || 'custom');
      const line =
        lines.find((row) => row.item_type === itemType && row.item_name === itemName) ??
        lines[lines.push({
          id: nextId(store),
          session_id: sessionId,
          item_type: itemType,
          investment_type: null,
          item_name: itemName,
          count: 0,
          value_in_exalts_snapshot: 0,
          total_value_exalts: 0
        }) - 1];
      line.count = Math.max(0, Number(input.count) || 0);
      line.value_in_exalts_snapshot = Math.max(0, Number(input.value_in_exalts) || 0);
      line.total_value_exalts = lineTotal(line.count, line.value_in_exalts_snapshot);
      recalculate(store, sessionId);
      saveStore(store);
      return sessionDetail(store, sessionId);
    },
    updateInvestment: async (input: Record<string, unknown>) => {
      const store = loadStore();
      const sessionId = Number(input.session_id);
      const lines = store.details[sessionId].investments;
      const itemName = String(input.item_name || '');
      const investmentType = String(input.investment_type || 'Misc');
      const line =
        lines.find((row) => row.investment_type === investmentType && row.item_name === itemName) ??
        lines[lines.push({
          id: nextId(store),
          session_id: sessionId,
          item_type: null,
          investment_type: investmentType,
          item_name: itemName,
          count: 0,
          value_in_exalts_snapshot: 0,
          total_value_exalts: 0
        }) - 1];
      line.count = Math.max(0, Number(input.count) || 0);
      line.value_in_exalts_snapshot = Math.max(0, Number(input.value_in_exalts) || 0);
      line.total_value_exalts = lineTotal(line.count, line.value_in_exalts_snapshot);
      recalculate(store, sessionId);
      saveStore(store);
      return sessionDetail(store, sessionId);
    },
    stopSession: async (sessionId: number) => {
      const store = loadStore();
      const session = store.sessions.find((row) => row.id === sessionId);
      if (!session) throw new Error('Session not found');
      session.status = 'completed';
      session.ended_at = new Date().toISOString();
      recalculate(store, sessionId);
      saveStore(store);
      return session;
    },
    cancelSession: async (sessionId: number) => {
      const store = loadStore();
      const session = store.sessions.find((row) => row.id === sessionId);
      if (!session) throw new Error('Session not found');
      session.status = 'cancelled';
      session.ended_at = new Date().toISOString();
      saveStore(store);
      return session;
    },
    updateCurrencyValue: async (id: number, valueInExalts: number) => {
      const store = loadStore();
      const currency = store.currencies.find((row) => row.id === id);
      if (currency) currency.value_in_exalts = Math.max(0, Number(valueInExalts) || 0);
      saveStore(store);
    },
    updateCurrencyOrder: async (currencyIds: number[]) => {
      const store = loadStore();
      for (const [index, id] of currencyIds.entries()) {
        const currency = store.currencies.find((row) => row.id === id);
        if (currency) currency.display_order = (index + 1) * 10;
      }
      store.currencyOrderVersion = 1;
      saveStore(store);
    },
    createCustomCurrency: async (name: string, shortName: string, valueInExalts: number) => {
      const store = loadStore();
      const displayOrder = Math.max(0, ...store.currencies.map((row) => row.display_order)) + 10;
      const currency = {
        id: nextId(store),
        name: name.trim(),
        short_name: shortName.trim(),
        value_in_exalts: Math.max(0, Number(valueInExalts) || 0),
        display_order: displayOrder,
        is_default: false,
        active: true
      };
      store.currencies.push(currency);
      saveStore(store);
      return currency;
    },
    updateChaseItemValue: async (id: number, valueInExalts: number) => {
      const store = loadStore();
      const item = store.chaseItems.find((row) => row.id === id);
      if (item) item.default_value_in_exalts = Math.max(0, Number(valueInExalts) || 0);
      saveStore(store);
    },
    createChaseItem: async (name: string, valueInExalts: number, notes: string) => {
      const store = loadStore();
      const item = {
        id: nextId(store),
        name: name.trim(),
        default_value_in_exalts: Math.max(0, Number(valueInExalts) || 0),
        notes,
        active: true
      };
      store.chaseItems.push(item);
      saveStore(store);
      return item;
    },
    createStrategy: async (input: Record<string, unknown>) => {
      const store = loadStore();
      const mechanicId = Number(input.mechanic_id) || null;
      const strategy = {
        id: nextId(store),
        name: String(input.name || '').trim(),
        mechanic_id: mechanicId,
        mechanic_name: store.mechanics.find((row) => row.id === mechanicId)?.name ?? null,
        description: String(input.description || ''),
        default_notes: String(input.default_notes || ''),
        default_investment_rows: String(input.default_investment_rows || '[]'),
        default_chase_items: String(input.default_chase_items || '[]'),
        active: true
      };
      store.strategies.push(strategy);
      saveStore(store);
      return strategy;
    },
    updateStrategy: async (input: Record<string, unknown>) => {
      const store = loadStore();
      const strategy = store.strategies.find((row) => row.id === Number(input.id));
      if (!strategy) throw new Error('Strategy not found');
      const mechanicId = Number(input.mechanic_id) || null;
      Object.assign(strategy, {
        name: String(input.name || '').trim(),
        mechanic_id: mechanicId,
        mechanic_name: store.mechanics.find((row) => row.id === mechanicId)?.name ?? null,
        description: String(input.description || ''),
        default_notes: String(input.default_notes || ''),
        default_investment_rows: String(input.default_investment_rows || '[]'),
        default_chase_items: String(input.default_chase_items || '[]')
      });
      saveStore(store);
      return strategy;
    },
    deleteStrategy: async (id: number) => {
      const store = loadStore();
      const strategy = store.strategies.find((row) => row.id === id);
      if (strategy) strategy.active = false;
      saveStore(store);
    }
  };
}

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export const api = isTauri ? tauriApi : createBrowserApi();
