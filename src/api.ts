import { invoke } from '@tauri-apps/api/core';
import type {
  ChaseItem,
  Currency,
  DashboardData,
  FarmSession,
  Mechanic,
  ReportsData,
  SessionDetail,
  Strategy
} from './types';

export const api = {
  initializeDatabase: () => invoke<string>('initialize_database'),
  dashboard: () => invoke<DashboardData>('get_dashboard_data'),
  activeSession: () => invoke<SessionDetail | null>('get_active_session'),
  session: (id: number) => invoke<SessionDetail>('get_session', { id }),
  sessions: () => invoke<FarmSession[]>('list_sessions'),
  mechanics: () => invoke<Mechanic[]>('list_mechanics'),
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
