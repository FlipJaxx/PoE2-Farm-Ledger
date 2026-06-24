export type Currency = {
  id: number;
  name: string;
  short_name: string;
  value_in_exalts: number;
  is_default: boolean;
  active: boolean;
};

export type ChaseItem = {
  id: number;
  name: string;
  default_value_in_exalts: number;
  notes: string;
  active: boolean;
};

export type Mechanic = {
  id: number;
  name: string;
  description: string;
  is_default: boolean;
  active: boolean;
};

export type Strategy = {
  id: number;
  name: string;
  mechanic_id: number | null;
  mechanic_name: string | null;
  description: string;
  default_notes: string;
  default_investment_rows: string;
  default_chase_items: string;
  active: boolean;
};

export type FarmSession = {
  id: number;
  strategy_id: number | null;
  strategy_name: string;
  mechanic_id: number | null;
  mechanic_name: string;
  character_name: string;
  league: string;
  map_tier: string;
  notes: string;
  status: string;
  started_at: string;
  ended_at: string | null;
  duration_seconds: number;
  maps_run: number;
  total_loot_value_exalts: number;
  total_investment_value_exalts: number;
  net_profit_exalts: number;
  profit_per_hour_exalts: number;
  profit_per_map_exalts: number;
  maps_per_hour: number;
  divine_value_exalts_snapshot: number;
  divine_per_hour: number;
};

export type SessionLine = {
  id: number;
  session_id: number;
  item_type: string | null;
  investment_type: string | null;
  item_name: string;
  count: number;
  value_in_exalts_snapshot: number;
  total_value_exalts: number;
};

export type SessionDetail = {
  session: FarmSession;
  loot: SessionLine[];
  investments: SessionLine[];
};

export type ReportRow = {
  group_name: string;
  sessions: number;
  average_profit_per_hour: number;
  average_profit_per_map: number;
  total_maps: number;
  total_time_seconds: number;
  total_net_profit: number;
  best_session_profit: number;
  worst_session_profit: number;
};

export type DashboardData = {
  active_session: FarmSession | null;
  recent_sessions: FarmSession[];
  best_strategies: ReportRow[];
  total_sessions: number;
  total_maps: number;
  total_time_seconds: number;
  total_profit_exalts: number;
  total_profit_divines: number;
};

export type ReportsData = {
  by_mechanic: ReportRow[];
  by_strategy: ReportRow[];
};
