use rusqlite::{params, Connection, Result};

pub fn migrate_and_seed(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS currencies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            short_name TEXT NOT NULL,
            value_in_exalts REAL NOT NULL,
            display_order INTEGER NOT NULL DEFAULT 0,
            is_default INTEGER NOT NULL DEFAULT 0,
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS chase_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            default_value_in_exalts REAL NOT NULL,
            default_value_in_divines REAL NOT NULL DEFAULT 0,
            notes TEXT NOT NULL DEFAULT '',
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS mechanics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL DEFAULT '',
            is_default INTEGER NOT NULL DEFAULT 0,
            active INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS strategies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            mechanic_id INTEGER,
            description TEXT NOT NULL DEFAULT '',
            default_notes TEXT NOT NULL DEFAULT '',
            default_investment_rows TEXT NOT NULL DEFAULT '[]',
            default_chase_items TEXT NOT NULL DEFAULT '[]',
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(mechanic_id) REFERENCES mechanics(id)
        );

        CREATE TABLE IF NOT EXISTS farm_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            strategy_id INTEGER,
            strategy_name TEXT NOT NULL,
            mechanic_id INTEGER,
            mechanic_name TEXT NOT NULL,
            character_name TEXT NOT NULL DEFAULT '',
            league TEXT NOT NULL DEFAULT '',
            map_tier TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL,
            started_at TEXT NOT NULL,
            ended_at TEXT,
            duration_seconds INTEGER NOT NULL DEFAULT 0,
            maps_run INTEGER NOT NULL DEFAULT 0,
            total_loot_value_exalts REAL NOT NULL DEFAULT 0,
            total_investment_value_exalts REAL NOT NULL DEFAULT 0,
            net_profit_exalts REAL NOT NULL DEFAULT 0,
            profit_per_hour_exalts REAL NOT NULL DEFAULT 0,
            profit_per_map_exalts REAL NOT NULL DEFAULT 0,
            maps_per_hour REAL NOT NULL DEFAULT 0,
            divine_value_exalts_snapshot REAL NOT NULL DEFAULT 120,
            divine_per_hour REAL NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(strategy_id) REFERENCES strategies(id),
            FOREIGN KEY(mechanic_id) REFERENCES mechanics(id)
        );

        CREATE TABLE IF NOT EXISTS session_loot (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER NOT NULL,
            item_type TEXT NOT NULL,
            item_name TEXT NOT NULL,
            count REAL NOT NULL DEFAULT 0,
            value_in_exalts_snapshot REAL NOT NULL DEFAULT 0,
            total_value_exalts REAL NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(session_id, item_type, item_name),
            FOREIGN KEY(session_id) REFERENCES farm_sessions(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS session_investments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            session_id INTEGER NOT NULL,
            investment_type TEXT NOT NULL,
            item_name TEXT NOT NULL,
            count REAL NOT NULL DEFAULT 0,
            value_in_exalts_snapshot REAL NOT NULL DEFAULT 0,
            total_value_exalts REAL NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(session_id, investment_type, item_name),
            FOREIGN KEY(session_id) REFERENCES farm_sessions(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS price_snapshots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            item_name TEXT NOT NULL,
            item_type TEXT NOT NULL,
            value_in_exalts REAL NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        ",
    )?;

    ensure_currency_order_column(conn)?;
    seed_currencies(conn)?;
    ensure_chase_divines_column(conn)?;
    seed_chase_items(conn)?;
    seed_mechanics(conn)?;
    ensure_session_pause_columns(conn)?;
    Ok(())
}

fn ensure_session_pause_columns(conn: &Connection) -> Result<()> {
    let (mut has_accumulated, mut has_segment) = (false, false);
    {
        let mut stmt = conn.prepare("PRAGMA table_info(farm_sessions)")?;
        let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
        for column in columns {
            match column?.as_str() {
                "accumulated_seconds" => has_accumulated = true,
                "segment_started_at" => has_segment = true,
                _ => {}
            }
        }
    }
    if !has_accumulated {
        conn.execute(
            "ALTER TABLE farm_sessions ADD COLUMN accumulated_seconds INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    if !has_segment {
        conn.execute("ALTER TABLE farm_sessions ADD COLUMN segment_started_at TEXT", [])?;
        conn.execute(
            "UPDATE farm_sessions SET segment_started_at = started_at
             WHERE status = 'running' AND segment_started_at IS NULL",
            [],
        )?;
    }
    Ok(())
}

fn ensure_currency_order_column(conn: &Connection) -> Result<()> {
    let has_display_order = {
        let mut stmt = conn.prepare("PRAGMA table_info(currencies)")?;
        let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
        let mut found = false;
        for column in columns {
            if column? == "display_order" {
                found = true;
                break;
            }
        }
        found
    };

    if !has_display_order {
        conn.execute(
            "ALTER TABLE currencies ADD COLUMN display_order INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
    }
    Ok(())
}

fn ensure_chase_divines_column(conn: &Connection) -> Result<()> {
    let has_divines = {
        let mut stmt = conn.prepare("PRAGMA table_info(chase_items)")?;
        let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
        let mut found = false;
        for column in columns {
            if column? == "default_value_in_divines" {
                found = true;
                break;
            }
        }
        found
    };

    if !has_divines {
        conn.execute(
            "ALTER TABLE chase_items ADD COLUMN default_value_in_divines REAL NOT NULL DEFAULT 0",
            [],
        )?;
        let divine_rate = current_divine_rate(conn);
        conn.execute(
            "UPDATE chase_items SET default_value_in_divines = default_value_in_exalts / ?1
             WHERE default_value_in_divines = 0 AND default_value_in_exalts > 0",
            params![divine_rate],
        )?;
    }
    Ok(())
}

fn seed_currencies(conn: &Connection) -> Result<()> {
    let currencies = [
        ("Exalted Orb", "ex", 1.0, 10),
        ("Greater Exalted Orb", "gex", 10.0, 20),
        ("Perfect Exalted Orb", "pex", 80.0, 30),
        ("Chaos Orb", "c", 0.5, 40),
        ("Greater Chaos Orb", "gc", 8.0, 50),
        ("Perfect Chaos Orb", "pc", 60.0, 60),
        ("Divine Orb", "div", 120.0, 70),
        ("Orb of Annulment", "annul", 15.0, 80),
        ("Mirror of Kalandra", "mirror", 50000.0, 90),
    ];

    for (name, short_name, value, display_order) in currencies {
        conn.execute(
            "INSERT OR IGNORE INTO currencies (name, short_name, value_in_exalts, display_order, is_default) VALUES (?1, ?2, ?3, ?4, 1)",
            params![name, short_name, value, display_order],
        )?;
        conn.execute(
            "UPDATE currencies SET display_order = ?1 WHERE name = ?2 AND is_default = 1",
            params![display_order, name],
        )?;
    }
    Ok(())
}

fn current_divine_rate(conn: &Connection) -> f64 {
    conn.query_row(
        "SELECT value_in_exalts FROM currencies WHERE name = 'Divine Orb'",
        [],
        |row| row.get(0),
    )
    .unwrap_or(350.0)
}

fn seed_chase_items(conn: &Connection) -> Result<()> {
    let items = [
        ("Aldur's Saga", 3.0),
        ("Aldur's Legacy", 8.0),
        ("Mageblood", 100.0),
        ("Headhunter", 340.0),
        ("Temporalis", 170.0),
        ("Ingenuity", 18.0),
        ("Against the Darkness", 40.0),
    ];

    let divine_rate = current_divine_rate(conn);
    for (name, value_in_divines) in items {
        let value_in_exalts = value_in_divines * divine_rate;
        conn.execute(
            "INSERT OR IGNORE INTO chase_items
             (name, default_value_in_exalts, default_value_in_divines, notes)
             VALUES (?1, ?2, ?3, '')",
            params![name, value_in_exalts, value_in_divines],
        )?;
    }
    Ok(())
}

fn seed_mechanics(conn: &Connection) -> Result<()> {
    let mechanics = [
        "Generic Mapping",
        "Breach",
        "Expedition",
        "Ritual",
        "Delirium",
        "Bossing",
        "Tablets",
        "Custom",
    ];

    for name in mechanics {
        conn.execute(
            "INSERT OR IGNORE INTO mechanics (name, description, is_default) VALUES (?1, '', 1)",
            params![name],
        )?;
    }
    Ok(())
}
