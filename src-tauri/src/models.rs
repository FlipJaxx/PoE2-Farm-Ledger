use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Currency {
    pub id: i64,
    pub name: String,
    pub short_name: String,
    pub value_in_exalts: f64,
    pub display_order: i64,
    pub is_default: bool,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct ChaseItem {
    pub id: i64,
    pub name: String,
    pub default_value_in_exalts: f64,
    pub default_value_in_divines: f64,
    pub notes: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct Mechanic {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub is_default: bool,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct Strategy {
    pub id: i64,
    pub name: String,
    pub mechanic_id: Option<i64>,
    pub mechanic_name: Option<String>,
    pub description: String,
    pub default_notes: String,
    pub default_investment_rows: String,
    pub default_chase_items: String,
    pub active: bool,
}

#[derive(Debug, Serialize)]
pub struct FarmSession {
    pub id: i64,
    pub strategy_id: Option<i64>,
    pub strategy_name: String,
    pub mechanic_id: Option<i64>,
    pub mechanic_name: String,
    pub character_name: String,
    pub league: String,
    pub map_tier: String,
    pub notes: String,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_seconds: i64,
    pub maps_run: i64,
    pub total_loot_value_exalts: f64,
    pub total_investment_value_exalts: f64,
    pub net_profit_exalts: f64,
    pub profit_per_hour_exalts: f64,
    pub profit_per_map_exalts: f64,
    pub maps_per_hour: f64,
    pub divine_value_exalts_snapshot: f64,
    pub divine_per_hour: f64,
    pub accumulated_seconds: i64,
    pub segment_started_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SessionLine {
    pub id: i64,
    pub session_id: i64,
    pub item_type: Option<String>,
    pub investment_type: Option<String>,
    pub item_name: String,
    pub count: f64,
    pub value_in_exalts_snapshot: f64,
    pub total_value_exalts: f64,
}

#[derive(Debug, Serialize)]
pub struct SessionDetail {
    pub session: FarmSession,
    pub loot: Vec<SessionLine>,
    pub investments: Vec<SessionLine>,
}

#[derive(Debug, Serialize)]
pub struct DashboardData {
    pub active_session: Option<FarmSession>,
    pub recent_sessions: Vec<FarmSession>,
    pub best_strategies: Vec<ReportRow>,
    pub total_sessions: i64,
    pub total_maps: i64,
    pub total_time_seconds: i64,
    pub total_profit_exalts: f64,
    pub total_profit_divines: f64,
}

#[derive(Debug, Serialize)]
pub struct ReportRow {
    pub group_name: String,
    pub sessions: i64,
    pub average_profit_per_hour: f64,
    pub average_profit_per_map: f64,
    pub total_maps: i64,
    pub total_time_seconds: i64,
    pub total_net_profit: f64,
    pub best_session_profit: f64,
    pub worst_session_profit: f64,
}

#[derive(Debug, Serialize)]
pub struct ReportsData {
    pub by_mechanic: Vec<ReportRow>,
    pub by_strategy: Vec<ReportRow>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionRequest {
    pub strategy_id: Option<i64>,
    pub strategy_name: String,
    pub mechanic_id: Option<i64>,
    pub mechanic_name: String,
    pub character_name: String,
    pub league: String,
    pub map_tier: String,
    pub notes: String,
}

#[derive(Debug, Deserialize)]
pub struct SessionLootRequest {
    pub session_id: i64,
    pub item_type: String,
    pub item_name: String,
    pub count: f64,
    pub value_in_exalts: f64,
}

#[derive(Debug, Deserialize)]
pub struct SessionInvestmentRequest {
    pub session_id: i64,
    pub investment_type: String,
    pub item_name: String,
    pub count: f64,
    pub value_in_exalts: f64,
}

#[derive(Debug, Deserialize)]
pub struct CreateStrategyRequest {
    pub name: String,
    pub mechanic_id: Option<i64>,
    pub description: String,
    pub default_notes: String,
    pub default_investment_rows: String,
    pub default_chase_items: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStrategyRequest {
    pub id: i64,
    pub name: String,
    pub mechanic_id: Option<i64>,
    pub description: String,
    pub default_notes: String,
    pub default_investment_rows: String,
    pub default_chase_items: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateMechanicRequest {
    pub name: String,
    pub description: String,
}
