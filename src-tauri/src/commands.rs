use crate::calculations::{line_total, session_totals};
use crate::db::AppState;
use crate::models::*;
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use tauri::{AppHandle, State};

#[derive(Deserialize)]
struct DefaultInvestmentRow {
    #[serde(default)]
    investment_type: String,
    #[serde(default)]
    item_name: String,
    #[serde(default)]
    count: f64,
    #[serde(default)]
    value_in_exalts: f64,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum DefaultChaseItem {
    Name(String),
    Row {
        name: Option<String>,
        item_name: Option<String>,
    },
}

fn current_divine_rate(conn: &Connection) -> f64 {
    conn.query_row(
        "SELECT value_in_exalts FROM currencies WHERE name = 'Divine Orb'",
        [],
        |row| row.get(0),
    )
    .unwrap_or(350.0)
}

#[tauri::command]
pub fn initialize_database(app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    state
        .initialize(&app)
        .map(|path| path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn get_dashboard_data(state: State<'_, AppState>) -> Result<DashboardData, String> {
    let conn = state.connection()?;
    let active_session = active_session(&conn)?;
    let recent_sessions = list_sessions_query(&conn, 6)?;
    let best_strategies = report_query(&conn, "strategy_name", 5)?;
    let divine_value: f64 = conn
        .query_row(
            "SELECT value_in_exalts FROM currencies WHERE name = 'Divine Orb'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(120.0);
    let (total_sessions, total_maps, total_time_seconds, total_profit_exalts): (i64, i64, i64, f64) =
        conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(maps_run),0), COALESCE(SUM(duration_seconds),0), COALESCE(SUM(net_profit_exalts),0)
             FROM farm_sessions WHERE status = 'completed'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .map_err(|err| err.to_string())?;

    Ok(DashboardData {
        active_session,
        recent_sessions,
        best_strategies,
        total_sessions,
        total_maps,
        total_time_seconds,
        total_profit_exalts,
        total_profit_divines: if divine_value > 0.0 {
            total_profit_exalts / divine_value
        } else {
            0.0
        },
    })
}

#[tauri::command]
pub fn create_session(
    state: State<'_, AppState>,
    input: CreateSessionRequest,
) -> Result<FarmSession, String> {
    let conn = state.connection()?;
    let running: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM farm_sessions WHERE status = 'running'",
            [],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if running > 0 {
        return Err("Only one farming session can be running at a time".to_string());
    }

    let divine_value: f64 = conn
        .query_row(
            "SELECT value_in_exalts FROM currencies WHERE name = 'Divine Orb'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(120.0);
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO farm_sessions
        (strategy_id, strategy_name, mechanic_id, mechanic_name, character_name, league, map_tier, notes, status, started_at, divine_value_exalts_snapshot)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'running', ?9, ?10)",
        params![
            input.strategy_id,
            input.strategy_name,
            input.mechanic_id,
            input.mechanic_name,
            input.character_name,
            input.league,
            input.map_tier,
            input.notes,
            now,
            divine_value
        ],
    )
    .map_err(|err| err.to_string())?;
    let id = conn.last_insert_rowid();
    seed_session_price_rows(&conn, id, divine_value, input.strategy_id)?;
    if let Some(strategy_id) = input.strategy_id {
        apply_strategy_defaults(&conn, id, strategy_id)?;
        refresh_session_totals(&conn, id)?;
    }
    get_session_row(&conn, id)
}

#[tauri::command]
pub fn get_active_session(state: State<'_, AppState>) -> Result<Option<SessionDetail>, String> {
    let conn = state.connection()?;
    match active_session(&conn)? {
        Some(session) => Ok(Some(session_detail(&conn, session.id)?)),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn get_session(state: State<'_, AppState>, id: i64) -> Result<SessionDetail, String> {
    let conn = state.connection()?;
    session_detail(&conn, id)
}

#[tauri::command]
pub fn update_session_maps(
    state: State<'_, AppState>,
    session_id: i64,
    maps_run: i64,
) -> Result<FarmSession, String> {
    let conn = state.connection()?;
    ensure_session_editable(&conn, session_id)?;
    conn.execute(
        "UPDATE farm_sessions SET maps_run = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![maps_run.max(0), session_id],
    )
    .map_err(|err| err.to_string())?;
    refresh_session_totals(&conn, session_id)?;
    get_session_row(&conn, session_id)
}

#[tauri::command]
pub fn add_or_update_session_loot(
    state: State<'_, AppState>,
    input: SessionLootRequest,
) -> Result<SessionDetail, String> {
    let conn = state.connection()?;
    ensure_session_editable(&conn, input.session_id)?;
    let total = line_total(input.count, input.value_in_exalts);
    conn.execute(
        "INSERT INTO session_loot (session_id, item_type, item_name, count, value_in_exalts_snapshot, total_value_exalts)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(session_id, item_type, item_name) DO UPDATE SET
         count = excluded.count, value_in_exalts_snapshot = excluded.value_in_exalts_snapshot,
         total_value_exalts = excluded.total_value_exalts, updated_at = CURRENT_TIMESTAMP",
        params![input.session_id, input.item_type, input.item_name, input.count.max(0.0), input.value_in_exalts.max(0.0), total],
    )
    .map_err(|err| err.to_string())?;
    refresh_session_totals(&conn, input.session_id)?;
    session_detail(&conn, input.session_id)
}

#[tauri::command]
pub fn add_or_update_session_investment(
    state: State<'_, AppState>,
    input: SessionInvestmentRequest,
) -> Result<SessionDetail, String> {
    let conn = state.connection()?;
    ensure_session_editable(&conn, input.session_id)?;
    let total = line_total(input.count, input.value_in_exalts);
    conn.execute(
        "INSERT INTO session_investments (session_id, investment_type, item_name, count, value_in_exalts_snapshot, total_value_exalts)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(session_id, investment_type, item_name) DO UPDATE SET
         count = excluded.count, value_in_exalts_snapshot = excluded.value_in_exalts_snapshot,
         total_value_exalts = excluded.total_value_exalts, updated_at = CURRENT_TIMESTAMP",
        params![input.session_id, input.investment_type, input.item_name, input.count.max(0.0), input.value_in_exalts.max(0.0), total],
    )
    .map_err(|err| err.to_string())?;
    refresh_session_totals(&conn, input.session_id)?;
    session_detail(&conn, input.session_id)
}

#[tauri::command]
pub fn delete_session_loot_line(
    state: State<'_, AppState>,
    line_id: i64,
) -> Result<SessionDetail, String> {
    let conn = state.connection()?;
    let session_id = delete_session_loot_line_query(&conn, line_id)?;
    session_detail(&conn, session_id)
}

#[tauri::command]
pub fn delete_session_investment_line(
    state: State<'_, AppState>,
    line_id: i64,
) -> Result<SessionDetail, String> {
    let conn = state.connection()?;
    let session_id = delete_session_investment_line_query(&conn, line_id)?;
    session_detail(&conn, session_id)
}

#[tauri::command]
pub fn stop_session(state: State<'_, AppState>, session_id: i64) -> Result<FarmSession, String> {
    let conn = state.connection()?;
    let session = get_session_row(&conn, session_id)?;
    if session.status != "running" {
        return Err("Only a running session can be stopped".to_string());
    }
    let ended_at = Utc::now();
    let started_at = chrono::DateTime::parse_from_rfc3339(&session.started_at)
        .map_err(|err| err.to_string())?
        .with_timezone(&Utc);
    let duration_seconds = (ended_at - started_at).num_seconds().max(0);
    let (loot, investment) = session_line_sums(&conn, session_id)?;
    let totals = session_totals(
        loot,
        investment,
        duration_seconds,
        session.maps_run,
        session.divine_value_exalts_snapshot,
    );
    conn.execute(
        "UPDATE farm_sessions SET status = 'completed', ended_at = ?1, duration_seconds = ?2,
         total_loot_value_exalts = ?3, total_investment_value_exalts = ?4, net_profit_exalts = ?5,
         profit_per_hour_exalts = ?6, profit_per_map_exalts = ?7, maps_per_hour = ?8,
         divine_per_hour = ?9, updated_at = CURRENT_TIMESTAMP WHERE id = ?10",
        params![
            ended_at.to_rfc3339(),
            duration_seconds,
            totals.total_loot_value_exalts,
            totals.total_investment_value_exalts,
            totals.net_profit_exalts,
            totals.profit_per_hour_exalts,
            totals.profit_per_map_exalts,
            totals.maps_per_hour,
            totals.divine_per_hour,
            session_id
        ],
    )
    .map_err(|err| err.to_string())?;
    get_session_row(&conn, session_id)
}

#[tauri::command]
pub fn cancel_session(state: State<'_, AppState>, session_id: i64) -> Result<FarmSession, String> {
    let conn = state.connection()?;
    conn.execute(
        "UPDATE farm_sessions SET status = 'cancelled', ended_at = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2 AND status = 'running'",
        params![Utc::now().to_rfc3339(), session_id],
    )
    .map_err(|err| err.to_string())?;
    get_session_row(&conn, session_id)
}

#[tauri::command]
pub fn list_sessions(state: State<'_, AppState>) -> Result<Vec<FarmSession>, String> {
    list_sessions_query(&state.connection()?, 100)
}

#[tauri::command]
pub fn list_currencies(state: State<'_, AppState>) -> Result<Vec<Currency>, String> {
    let conn = state.connection()?;
    let mut stmt = conn
        .prepare("SELECT id, name, short_name, value_in_exalts, display_order, is_default, active FROM currencies WHERE active = 1 ORDER BY display_order, name")
        .map_err(|err| err.to_string())?;
    let currencies = stmt
        .query_map([], currency_from_row)
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    Ok(currencies)
}

#[tauri::command]
pub fn update_currency_value(
    state: State<'_, AppState>,
    id: i64,
    value_in_exalts: f64,
) -> Result<(), String> {
    let conn = state.connection()?;
    let name: String = conn
        .query_row(
            "SELECT name FROM currencies WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    conn.execute(
        "UPDATE currencies SET value_in_exalts = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![value_in_exalts.max(0.0), id],
    )
    .map_err(|err| err.to_string())?;
    insert_price_snapshot(&conn, &name, "currency", value_in_exalts)?;
    Ok(())
}

#[tauri::command]
pub fn update_currency_order(
    state: State<'_, AppState>,
    currency_ids: Vec<i64>,
) -> Result<(), String> {
    let mut conn = state.connection()?;
    let tx = conn.transaction().map_err(|err| err.to_string())?;
    for (index, id) in currency_ids.iter().enumerate() {
        tx.execute(
            "UPDATE currencies SET display_order = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![(index as i64 + 1) * 10, id],
        )
        .map_err(|err| err.to_string())?;
    }
    tx.commit().map_err(|err| err.to_string())
}

#[tauri::command]
pub fn create_custom_currency(
    state: State<'_, AppState>,
    name: String,
    short_name: String,
    value_in_exalts: f64,
) -> Result<Currency, String> {
    let conn = state.connection()?;
    let name = name.trim();
    let short_name = short_name.trim();
    if name.is_empty() {
        return Err("Currency name is required".to_string());
    }

    let display_order: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(display_order), 0) + 10 FROM currencies",
            [],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    conn.execute(
        "INSERT INTO currencies (name, short_name, value_in_exalts, display_order, is_default)
         VALUES (?1, ?2, ?3, ?4, 0)
         ON CONFLICT(name) DO UPDATE SET
         short_name = excluded.short_name,
         value_in_exalts = excluded.value_in_exalts,
         active = 1,
         updated_at = CURRENT_TIMESTAMP",
        params![name, short_name, value_in_exalts.max(0.0), display_order],
    )
    .map_err(|err| err.to_string())?;
    insert_price_snapshot(&conn, name, "currency", value_in_exalts)?;
    conn.query_row(
        "SELECT id, name, short_name, value_in_exalts, display_order, is_default, active FROM currencies WHERE name = ?1",
        params![name],
        currency_from_row,
    )
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn list_chase_items(state: State<'_, AppState>) -> Result<Vec<ChaseItem>, String> {
    let conn = state.connection()?;
    let mut stmt = conn
        .prepare("SELECT id, name, default_value_in_exalts, default_value_in_divines, notes, active FROM chase_items WHERE active = 1 ORDER BY name")
        .map_err(|err| err.to_string())?;
    let items = stmt
        .query_map([], chase_from_row)
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    Ok(items)
}

#[tauri::command]
pub fn update_chase_item_value(
    state: State<'_, AppState>,
    id: i64,
    value_in_divines: f64,
) -> Result<(), String> {
    let conn = state.connection()?;
    let name: String = conn
        .query_row(
            "SELECT name FROM chase_items WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    let value_in_divines = value_in_divines.max(0.0);
    let value_in_exalts = value_in_divines * current_divine_rate(&conn);
    conn.execute(
        "UPDATE chase_items SET default_value_in_exalts = ?1, default_value_in_divines = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        params![value_in_exalts, value_in_divines, id],
    )
    .map_err(|err| err.to_string())?;
    insert_price_snapshot(&conn, &name, "chase", value_in_exalts)?;
    Ok(())
}

#[tauri::command]
pub fn create_chase_item(
    state: State<'_, AppState>,
    name: String,
    value_in_divines: f64,
    notes: String,
) -> Result<ChaseItem, String> {
    let conn = state.connection()?;
    let name = name.trim();
    if name.is_empty() {
        return Err("Chase item name is required".to_string());
    }

    let value_in_divines = value_in_divines.max(0.0);
    let value_in_exalts = value_in_divines * current_divine_rate(&conn);
    conn.execute(
        "INSERT INTO chase_items (name, default_value_in_exalts, default_value_in_divines, notes)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(name) DO UPDATE SET
         default_value_in_exalts = excluded.default_value_in_exalts,
         default_value_in_divines = excluded.default_value_in_divines,
         notes = excluded.notes,
         active = 1,
         updated_at = CURRENT_TIMESTAMP",
        params![name, value_in_exalts, value_in_divines, notes],
    )
    .map_err(|err| err.to_string())?;
    insert_price_snapshot(&conn, name, "chase", value_in_exalts)?;
    conn.query_row(
        "SELECT id, name, default_value_in_exalts, default_value_in_divines, notes, active FROM chase_items WHERE name = ?1",
        params![name],
        chase_from_row,
    )
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn list_mechanics(state: State<'_, AppState>) -> Result<Vec<Mechanic>, String> {
    let conn = state.connection()?;
    let mut stmt = conn
        .prepare("SELECT id, name, description, is_default, active FROM mechanics WHERE active = 1 ORDER BY id")
        .map_err(|err| err.to_string())?;
    let mechanics = stmt
        .query_map([], mechanic_from_row)
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    Ok(mechanics)
}

#[tauri::command]
pub fn create_mechanic(
    state: State<'_, AppState>,
    input: CreateMechanicRequest,
) -> Result<Mechanic, String> {
    let conn = state.connection()?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err("Mechanic name is required".to_string());
    }

    conn.execute(
        "INSERT INTO mechanics (name, description, is_default) VALUES (?1, ?2, 0)",
        params![name, input.description.trim()],
    )
    .map_err(|err| err.to_string())?;

    let id = conn.last_insert_rowid();
    conn.query_row(
        "SELECT id, name, description, is_default, active FROM mechanics WHERE id = ?1",
        params![id],
        mechanic_from_row,
    )
    .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn create_strategy(
    state: State<'_, AppState>,
    input: CreateStrategyRequest,
) -> Result<Strategy, String> {
    let conn = state.connection()?;
    let name = input.name.trim();
    if name.is_empty() {
        return Err("Strategy name is required".to_string());
    }

    conn.execute(
        "INSERT INTO strategies (name, mechanic_id, description, default_notes, default_investment_rows, default_chase_items)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![name, input.mechanic_id, input.description, input.default_notes, input.default_investment_rows, input.default_chase_items],
    )
    .map_err(|err| err.to_string())?;
    get_strategy_row(&conn, conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_strategy(
    state: State<'_, AppState>,
    input: UpdateStrategyRequest,
) -> Result<Strategy, String> {
    let conn = state.connection()?;
    conn.execute(
        "UPDATE strategies SET name = ?1, mechanic_id = ?2, description = ?3, default_notes = ?4,
         default_investment_rows = ?5, default_chase_items = ?6, updated_at = CURRENT_TIMESTAMP WHERE id = ?7",
        params![input.name, input.mechanic_id, input.description, input.default_notes, input.default_investment_rows, input.default_chase_items, input.id],
    )
    .map_err(|err| err.to_string())?;
    get_strategy_row(&conn, input.id)
}

#[tauri::command]
pub fn delete_strategy(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state
        .connection()?
        .execute(
            "UPDATE strategies SET active = 0, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![id],
        )
        .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_currency(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let conn = state.connection()?;
    let name: String = conn
        .query_row(
            "SELECT name FROM currencies WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    if name == "Exalted Orb" || name == "Divine Orb" {
        return Err(format!("{name} is required and cannot be removed"));
    }
    conn.execute(
        "UPDATE currencies SET active = 0, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![id],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_chase_item(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state
        .connection()?
        .execute(
            "UPDATE chase_items SET active = 0, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![id],
        )
        .map_err(|err| err.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_strategies(state: State<'_, AppState>) -> Result<Vec<Strategy>, String> {
    let conn = state.connection()?;
    let mut stmt = conn
        .prepare(
            "SELECT s.id, s.name, s.mechanic_id, m.name, s.description, s.default_notes,
             s.default_investment_rows, s.default_chase_items, s.active
             FROM strategies s LEFT JOIN mechanics m ON m.id = s.mechanic_id
             WHERE s.active = 1 ORDER BY s.name",
        )
        .map_err(|err| err.to_string())?;
    let strategies = stmt
        .query_map([], strategy_from_row)
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    Ok(strategies)
}

#[tauri::command]
pub fn get_reports_data(state: State<'_, AppState>) -> Result<ReportsData, String> {
    let conn = state.connection()?;
    Ok(ReportsData {
        by_mechanic: report_query(&conn, "mechanic_name", 100)?,
        by_strategy: report_query(&conn, "strategy_name", 100)?,
    })
}

fn seed_session_price_rows(
    conn: &Connection,
    session_id: i64,
    divine_rate: f64,
    strategy_id: Option<i64>,
) -> Result<(), String> {
    {
        let mut stmt = conn
            .prepare("SELECT name, value_in_exalts FROM currencies WHERE active = 1 ORDER BY display_order, name")
            .map_err(|err| err.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })
            .map_err(|err| err.to_string())?;
        for row in rows {
            let (name, value) = row.map_err(|err| err.to_string())?;
            conn.execute(
                "INSERT INTO session_loot (session_id, item_type, item_name, count, value_in_exalts_snapshot, total_value_exalts)
                 VALUES (?1, 'currency', ?2, 0, ?3, 0)",
                params![session_id, name, value],
            )
            .map_err(|err| err.to_string())?;
        }
    }
    let default_chase_names = strategy_id
        .map(|id| strategy_default_chase_names(conn, id))
        .transpose()?
        .unwrap_or_default();
    let (sql, bind_names) = if default_chase_names.is_empty() {
        (
            "SELECT name, default_value_in_divines FROM chase_items WHERE active = 1 ORDER BY name"
                .to_string(),
            false,
        )
    } else {
        let placeholders = (1..=default_chase_names.len())
            .map(|index| format!("?{index}"))
            .collect::<Vec<_>>()
            .join(", ");
        (
            format!(
                "SELECT name, default_value_in_divines FROM chase_items
                 WHERE active = 1 AND name IN ({placeholders}) ORDER BY name"
            ),
            true,
        )
    };
    let mut stmt = conn.prepare(&sql).map_err(|err| err.to_string())?;
    let rows: Vec<(String, f64)> = if bind_names {
        stmt.query_map(rusqlite::params_from_iter(default_chase_names.iter()), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?
    } else {
        stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?
    };
    for (name, value_in_divines) in rows {
        let value_in_exalts = value_in_divines.max(0.0) * divine_rate.max(0.0);
        conn.execute(
            "INSERT INTO session_loot (session_id, item_type, item_name, count, value_in_exalts_snapshot, total_value_exalts)
             VALUES (?1, 'chase', ?2, 0, ?3, 0)",
            params![session_id, name, value_in_exalts],
        )
        .map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn strategy_default_chase_names(conn: &Connection, strategy_id: i64) -> Result<Vec<String>, String> {
    let raw = conn
        .query_row(
            "SELECT default_chase_items FROM strategies WHERE id = ?1",
            params![strategy_id],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(|err| err.to_string())?
        .unwrap_or_else(|| "[]".to_string());

    let Ok(items) = serde_json::from_str::<Vec<DefaultChaseItem>>(&raw) else {
        return Ok(Vec::new());
    };

    Ok(items
        .into_iter()
        .filter_map(|item| match item {
            DefaultChaseItem::Name(name) => Some(name),
            DefaultChaseItem::Row { name, item_name } => name.or(item_name),
        })
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .collect())
}

fn apply_strategy_defaults(
    conn: &Connection,
    session_id: i64,
    strategy_id: i64,
) -> Result<(), String> {
    let row = conn
        .query_row(
            "SELECT default_notes, default_investment_rows FROM strategies WHERE id = ?1",
            params![strategy_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()
        .map_err(|err| err.to_string())?;

    let Some((default_notes, default_investment_rows)) = row else {
        return Ok(());
    };

    if !default_notes.trim().is_empty() {
        conn.execute(
            "UPDATE farm_sessions SET notes = ?1 WHERE id = ?2 AND (notes IS NULL OR notes = '')",
            params![default_notes, session_id],
        )
        .map_err(|err| err.to_string())?;
    }

    if let Ok(rows) =
        serde_json::from_str::<Vec<DefaultInvestmentRow>>(&default_investment_rows)
    {
        for row in rows {
            let item_name = row.item_name.trim();
            if item_name.is_empty() {
                continue;
            }
            let total = line_total(row.count, row.value_in_exalts);
            conn.execute(
                "INSERT INTO session_investments
                 (session_id, investment_type, item_name, count, value_in_exalts_snapshot, total_value_exalts)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(session_id, investment_type, item_name) DO UPDATE SET
                 count = excluded.count, value_in_exalts_snapshot = excluded.value_in_exalts_snapshot,
                 total_value_exalts = excluded.total_value_exalts, updated_at = CURRENT_TIMESTAMP",
                params![
                    session_id,
                    row.investment_type.trim(),
                    item_name,
                    row.count.max(0.0),
                    row.value_in_exalts.max(0.0),
                    total
                ],
            )
            .map_err(|err| err.to_string())?;
        }
    }
    Ok(())
}

fn ensure_session_editable(conn: &Connection, session_id: i64) -> Result<(), String> {
    let session = get_session_row(conn, session_id)?;
    if session.status == "running" || session.status == "completed" {
        Ok(())
    } else {
        Err("Only running or completed sessions can be edited".to_string())
    }
}

fn ensure_running_session(conn: &Connection, session_id: i64) -> Result<(), String> {
    let session = get_session_row(conn, session_id)?;
    if session.status == "running" {
        Ok(())
    } else {
        Err("Only running session lines can be removed".to_string())
    }
}

fn refresh_session_totals(conn: &Connection, session_id: i64) -> Result<(), String> {
    let session = get_session_row(conn, session_id)?;
    if session.status != "running" && session.status != "completed" {
        return Ok(());
    }
    let (loot, investment) = session_line_sums(conn, session_id)?;
    let duration_seconds = if session.status == "running" {
        let started_at = chrono::DateTime::parse_from_rfc3339(&session.started_at)
            .map_err(|err| err.to_string())?
            .with_timezone(&Utc);
        (Utc::now() - started_at).num_seconds().max(0)
    } else {
        session.duration_seconds.max(0)
    };
    let totals = session_totals(
        loot,
        investment,
        duration_seconds,
        session.maps_run,
        session.divine_value_exalts_snapshot,
    );
    conn.execute(
        "UPDATE farm_sessions SET duration_seconds = ?1, total_loot_value_exalts = ?2,
         total_investment_value_exalts = ?3, net_profit_exalts = ?4, profit_per_hour_exalts = ?5,
         profit_per_map_exalts = ?6, maps_per_hour = ?7, divine_per_hour = ?8,
         updated_at = CURRENT_TIMESTAMP WHERE id = ?9",
        params![
            duration_seconds,
            totals.total_loot_value_exalts,
            totals.total_investment_value_exalts,
            totals.net_profit_exalts,
            totals.profit_per_hour_exalts,
            totals.profit_per_map_exalts,
            totals.maps_per_hour,
            totals.divine_per_hour,
            session_id
        ],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn delete_session_loot_line_query(conn: &Connection, line_id: i64) -> Result<i64, String> {
    let (session_id, item_type): (i64, String) = conn
        .query_row(
            "SELECT session_id, item_type FROM session_loot WHERE id = ?1",
            params![line_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|err| err.to_string())?;
    ensure_running_session(conn, session_id)?;
    if item_type == "custom" {
        conn.execute("DELETE FROM session_loot WHERE id = ?1", params![line_id])
            .map_err(|err| err.to_string())?;
    } else {
        conn.execute(
            "UPDATE session_loot SET count = 0, total_value_exalts = 0,
             updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![line_id],
        )
        .map_err(|err| err.to_string())?;
    }
    refresh_session_totals(conn, session_id)?;
    Ok(session_id)
}

fn delete_session_investment_line_query(conn: &Connection, line_id: i64) -> Result<i64, String> {
    let session_id: i64 = conn
        .query_row(
            "SELECT session_id FROM session_investments WHERE id = ?1",
            params![line_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    ensure_running_session(conn, session_id)?;
    conn.execute(
        "DELETE FROM session_investments WHERE id = ?1",
        params![line_id],
    )
    .map_err(|err| err.to_string())?;
    refresh_session_totals(conn, session_id)?;
    Ok(session_id)
}

fn session_line_sums(conn: &Connection, session_id: i64) -> Result<(f64, f64), String> {
    let loot = conn
        .query_row(
            "SELECT COALESCE(SUM(total_value_exalts),0) FROM session_loot WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    let investment = conn
        .query_row(
            "SELECT COALESCE(SUM(total_value_exalts),0) FROM session_investments WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )
        .map_err(|err| err.to_string())?;
    Ok((loot, investment))
}

fn active_session(conn: &Connection) -> Result<Option<FarmSession>, String> {
    conn.query_row(
        session_select_sql("WHERE status = 'running' ORDER BY started_at DESC LIMIT 1").as_str(),
        [],
        farm_session_from_row,
    )
    .optional()
    .map_err(|err| err.to_string())
}

fn session_detail(conn: &Connection, id: i64) -> Result<SessionDetail, String> {
    Ok(SessionDetail {
        session: get_session_row(conn, id)?,
        loot: session_loot_rows(conn, id)?,
        investments: session_investment_rows(conn, id)?,
    })
}

fn get_session_row(conn: &Connection, id: i64) -> Result<FarmSession, String> {
    conn.query_row(
        session_select_sql("WHERE id = ?1").as_str(),
        params![id],
        farm_session_from_row,
    )
    .map_err(|err| err.to_string())
}

fn list_sessions_query(conn: &Connection, limit: i64) -> Result<Vec<FarmSession>, String> {
    let sql = session_select_sql("ORDER BY started_at DESC LIMIT ?1");
    let mut stmt = conn.prepare(&sql).map_err(|err| err.to_string())?;
    let sessions = stmt
        .query_map(params![limit], farm_session_from_row)
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    Ok(sessions)
}

fn session_select_sql(tail: &str) -> String {
    format!(
        "SELECT id, strategy_id, strategy_name, mechanic_id, mechanic_name, character_name, league, map_tier,
         notes, status, started_at, ended_at, duration_seconds, maps_run, total_loot_value_exalts,
         total_investment_value_exalts, net_profit_exalts, profit_per_hour_exalts, profit_per_map_exalts,
         maps_per_hour, divine_value_exalts_snapshot, divine_per_hour FROM farm_sessions {tail}"
    )
}

fn session_loot_rows(conn: &Connection, session_id: i64) -> Result<Vec<SessionLine>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT sl.id, sl.session_id, sl.item_type, sl.item_name, sl.count, sl.value_in_exalts_snapshot, sl.total_value_exalts
             FROM session_loot sl
             LEFT JOIN currencies c ON c.name = sl.item_name
             WHERE sl.session_id = ?1
             ORDER BY
                 CASE sl.item_type WHEN 'currency' THEN 0 WHEN 'chase' THEN 1 ELSE 2 END,
                 COALESCE(c.display_order, 1000000),
                 sl.item_name",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map(params![session_id], |row| {
            Ok(SessionLine {
                id: row.get(0)?,
                session_id: row.get(1)?,
                item_type: Some(row.get(2)?),
                investment_type: None,
                item_name: row.get(3)?,
                count: row.get(4)?,
                value_in_exalts_snapshot: row.get(5)?,
                total_value_exalts: row.get(6)?,
            })
        })
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    Ok(rows)
}

fn session_investment_rows(conn: &Connection, session_id: i64) -> Result<Vec<SessionLine>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, session_id, investment_type, item_name, count, value_in_exalts_snapshot, total_value_exalts
             FROM session_investments WHERE session_id = ?1 ORDER BY investment_type, item_name",
        )
        .map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map(params![session_id], |row| {
            Ok(SessionLine {
                id: row.get(0)?,
                session_id: row.get(1)?,
                item_type: None,
                investment_type: Some(row.get(2)?),
                item_name: row.get(3)?,
                count: row.get(4)?,
                value_in_exalts_snapshot: row.get(5)?,
                total_value_exalts: row.get(6)?,
            })
        })
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    Ok(rows)
}

fn report_query(
    conn: &Connection,
    group_column: &str,
    limit: i64,
) -> Result<Vec<ReportRow>, String> {
    let sql = format!(
        "SELECT {group_column}, COUNT(*),
         CASE WHEN COALESCE(SUM(duration_seconds),0) > 0
              THEN COALESCE(SUM(net_profit_exalts),0) / (SUM(duration_seconds) / 3600.0)
              ELSE 0 END,
         CASE WHEN COALESCE(SUM(maps_run),0) > 0
              THEN COALESCE(SUM(net_profit_exalts),0) / SUM(maps_run)
              ELSE 0 END,
         COALESCE(SUM(maps_run),0), COALESCE(SUM(duration_seconds),0), COALESCE(SUM(net_profit_exalts),0),
         COALESCE(MAX(net_profit_exalts),0), COALESCE(MIN(net_profit_exalts),0)
         FROM farm_sessions WHERE status = 'completed'
         GROUP BY {group_column} ORDER BY 3 DESC LIMIT ?1"
    );
    let mut stmt = conn.prepare(&sql).map_err(|err| err.to_string())?;
    let rows = stmt
        .query_map(params![limit], |row| {
            Ok(ReportRow {
                group_name: row.get(0)?,
                sessions: row.get(1)?,
                average_profit_per_hour: row.get(2)?,
                average_profit_per_map: row.get(3)?,
                total_maps: row.get(4)?,
                total_time_seconds: row.get(5)?,
                total_net_profit: row.get(6)?,
                best_session_profit: row.get(7)?,
                worst_session_profit: row.get(8)?,
            })
        })
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;
    Ok(rows)
}

fn get_strategy_row(conn: &Connection, id: i64) -> Result<Strategy, String> {
    conn.query_row(
        "SELECT s.id, s.name, s.mechanic_id, m.name, s.description, s.default_notes,
         s.default_investment_rows, s.default_chase_items, s.active
         FROM strategies s LEFT JOIN mechanics m ON m.id = s.mechanic_id WHERE s.id = ?1",
        params![id],
        strategy_from_row,
    )
    .map_err(|err| err.to_string())
}

pub(crate) fn insert_price_snapshot(
    conn: &Connection,
    name: &str,
    item_type: &str,
    value: f64,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO price_snapshots (item_name, item_type, value_in_exalts) VALUES (?1, ?2, ?3)",
        params![name, item_type, value.max(0.0)],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn farm_session_from_row(row: &rusqlite::Row) -> rusqlite::Result<FarmSession> {
    Ok(FarmSession {
        id: row.get(0)?,
        strategy_id: row.get(1)?,
        strategy_name: row.get(2)?,
        mechanic_id: row.get(3)?,
        mechanic_name: row.get(4)?,
        character_name: row.get(5)?,
        league: row.get(6)?,
        map_tier: row.get(7)?,
        notes: row.get(8)?,
        status: row.get(9)?,
        started_at: row.get(10)?,
        ended_at: row.get(11)?,
        duration_seconds: row.get(12)?,
        maps_run: row.get(13)?,
        total_loot_value_exalts: row.get(14)?,
        total_investment_value_exalts: row.get(15)?,
        net_profit_exalts: row.get(16)?,
        profit_per_hour_exalts: row.get(17)?,
        profit_per_map_exalts: row.get(18)?,
        maps_per_hour: row.get(19)?,
        divine_value_exalts_snapshot: row.get(20)?,
        divine_per_hour: row.get(21)?,
    })
}

fn currency_from_row(row: &rusqlite::Row) -> rusqlite::Result<Currency> {
    Ok(Currency {
        id: row.get(0)?,
        name: row.get(1)?,
        short_name: row.get(2)?,
        value_in_exalts: row.get(3)?,
        display_order: row.get(4)?,
        is_default: row.get::<_, i64>(5)? == 1,
        active: row.get::<_, i64>(6)? == 1,
    })
}

fn chase_from_row(row: &rusqlite::Row) -> rusqlite::Result<ChaseItem> {
    Ok(ChaseItem {
        id: row.get(0)?,
        name: row.get(1)?,
        default_value_in_exalts: row.get(2)?,
        default_value_in_divines: row.get(3)?,
        notes: row.get(4)?,
        active: row.get::<_, i64>(5)? == 1,
    })
}

fn mechanic_from_row(row: &rusqlite::Row) -> rusqlite::Result<Mechanic> {
    Ok(Mechanic {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        is_default: row.get::<_, i64>(3)? == 1,
        active: row.get::<_, i64>(4)? == 1,
    })
}

fn strategy_from_row(row: &rusqlite::Row) -> rusqlite::Result<Strategy> {
    Ok(Strategy {
        id: row.get(0)?,
        name: row.get(1)?,
        mechanic_id: row.get(2)?,
        mechanic_name: row.get(3)?,
        description: row.get(4)?,
        default_notes: row.get(5)?,
        default_investment_rows: row.get(6)?,
        default_chase_items: row.get(7)?,
        active: row.get::<_, i64>(8)? == 1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::migrate_and_seed;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory database");
        migrate_and_seed(&conn).expect("migrate database");
        conn
    }

    fn insert_session(
        conn: &Connection,
        status: &str,
        strategy_name: &str,
        duration_seconds: i64,
        maps_run: i64,
        net_profit_exalts: f64,
    ) -> i64 {
        conn.execute(
            "INSERT INTO farm_sessions
             (strategy_name, mechanic_name, status, started_at, ended_at, duration_seconds, maps_run,
              total_loot_value_exalts, total_investment_value_exalts, net_profit_exalts,
              profit_per_hour_exalts, profit_per_map_exalts, maps_per_hour, divine_per_hour)
             VALUES (?1, 'Test Mechanic', ?2, '2026-01-01T00:00:00Z', '2026-01-01T01:00:00Z',
              ?3, ?4, ?5, 0, ?5, 0, 0, 0, 0)",
            params![strategy_name, status, duration_seconds, maps_run, net_profit_exalts],
        )
        .expect("insert session");
        conn.last_insert_rowid()
    }

    #[test]
    fn reports_use_weighted_profit_rates() {
        let conn = test_conn();
        insert_session(&conn, "completed", "Weighted", 3600, 10, 100.0);
        insert_session(&conn, "completed", "Weighted", 7200, 30, 500.0);

        let reports = report_query(&conn, "strategy_name", 10).expect("reports");
        let report = reports
            .iter()
            .find(|row| row.group_name == "Weighted")
            .expect("weighted row");

        assert_eq!(report.sessions, 2);
        assert_eq!(report.total_time_seconds, 10800);
        assert_eq!(report.total_maps, 40);
        assert_eq!(report.total_net_profit, 600.0);
        assert_eq!(report.average_profit_per_hour, 200.0);
        assert_eq!(report.average_profit_per_map, 15.0);
    }

    #[test]
    fn completed_session_recalculation_preserves_duration() {
        let conn = test_conn();
        let session_id = insert_session(&conn, "completed", "Completed", 3600, 4, 0.0);
        conn.execute(
            "INSERT INTO session_loot
             (session_id, item_type, item_name, count, value_in_exalts_snapshot, total_value_exalts)
             VALUES (?1, 'custom', 'Rare drop', 2, 50, 100)",
            params![session_id],
        )
        .expect("insert loot");
        conn.execute(
            "INSERT INTO session_investments
             (session_id, investment_type, item_name, count, value_in_exalts_snapshot, total_value_exalts)
             VALUES (?1, 'Maps', 'Waystone', 1, 20, 20)",
            params![session_id],
        )
        .expect("insert investment");

        refresh_session_totals(&conn, session_id).expect("refresh totals");
        let session = get_session_row(&conn, session_id).expect("session");

        assert_eq!(session.duration_seconds, 3600);
        assert_eq!(session.total_loot_value_exalts, 100.0);
        assert_eq!(session.total_investment_value_exalts, 20.0);
        assert_eq!(session.net_profit_exalts, 80.0);
        assert_eq!(session.profit_per_hour_exalts, 80.0);
        assert_eq!(session.profit_per_map_exalts, 20.0);
    }

    #[test]
    fn deleting_running_lines_removes_custom_and_zeros_seeded_loot() {
        let conn = test_conn();
        let session_id = insert_session(&conn, "running", "Running", 0, 1, 0.0);
        conn.execute(
            "INSERT INTO session_loot
             (session_id, item_type, item_name, count, value_in_exalts_snapshot, total_value_exalts)
             VALUES (?1, 'custom', 'Custom item', 1, 10, 10)",
            params![session_id],
        )
        .expect("insert custom loot");
        let custom_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO session_loot
             (session_id, item_type, item_name, count, value_in_exalts_snapshot, total_value_exalts)
             VALUES (?1, 'currency', 'Exalted Orb', 5, 1, 5)",
            params![session_id],
        )
        .expect("insert seeded loot");
        let seeded_id = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO session_investments
             (session_id, investment_type, item_name, count, value_in_exalts_snapshot, total_value_exalts)
             VALUES (?1, 'Maps', 'Waystone', 1, 3, 3)",
            params![session_id],
        )
        .expect("insert investment");
        let investment_id = conn.last_insert_rowid();

        delete_session_loot_line_query(&conn, custom_id).expect("delete custom");
        delete_session_loot_line_query(&conn, seeded_id).expect("zero seeded");
        delete_session_investment_line_query(&conn, investment_id).expect("delete investment");

        let custom_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM session_loot WHERE id = ?1",
                params![custom_id],
                |row| row.get(0),
            )
            .expect("custom count");
        let (seeded_count, seeded_total): (f64, f64) = conn
            .query_row(
                "SELECT count, total_value_exalts FROM session_loot WHERE id = ?1",
                params![seeded_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .expect("seeded row");
        let investment_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM session_investments WHERE id = ?1",
                params![investment_id],
                |row| row.get(0),
            )
            .expect("investment count");

        assert_eq!(custom_count, 0);
        assert_eq!(seeded_count, 0.0);
        assert_eq!(seeded_total, 0.0);
        assert_eq!(investment_count, 0);
    }

    #[test]
    fn deleting_completed_lines_is_rejected() {
        let conn = test_conn();
        let session_id = insert_session(&conn, "completed", "Completed", 3600, 1, 0.0);
        conn.execute(
            "INSERT INTO session_loot
             (session_id, item_type, item_name, count, value_in_exalts_snapshot, total_value_exalts)
             VALUES (?1, 'custom', 'Custom item', 1, 10, 10)",
            params![session_id],
        )
        .expect("insert loot");
        let line_id = conn.last_insert_rowid();

        let err = delete_session_loot_line_query(&conn, line_id).expect_err("delete rejected");
        assert_eq!(err, "Only running session lines can be removed");
    }
}
