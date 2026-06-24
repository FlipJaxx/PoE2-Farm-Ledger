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
            is_default INTEGER NOT NULL DEFAULT 0,
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS chase_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            default_value_in_exalts REAL NOT NULL,
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

    seed_currencies(conn)?;
    seed_chase_items(conn)?;
    seed_mechanics(conn)?;
    Ok(())
}

fn seed_currencies(conn: &Connection) -> Result<()> {
    let currencies = [
        ("Exalted Orb", "ex", 1.0),
        ("Greater Exalted Orb", "gex", 10.0),
        ("Perfect Exalted Orb", "pex", 80.0),
        ("Chaos Orb", "c", 0.5),
        ("Greater Chaos Orb", "gc", 8.0),
        ("Perfect Chaos Orb", "pc", 60.0),
        ("Divine Orb", "div", 120.0),
        ("Orb of Annulment", "annul", 15.0),
        ("Mirror of Kalandra", "mirror", 50000.0),
    ];

    for (name, short_name, value) in currencies {
        conn.execute(
            "INSERT OR IGNORE INTO currencies (name, short_name, value_in_exalts, is_default) VALUES (?1, ?2, ?3, 1)",
            params![name, short_name, value],
        )?;
    }
    Ok(())
}

fn seed_chase_items(conn: &Connection) -> Result<()> {
    let items = [
        ("Aldur's Saga", 350.0),
        ("Aldur's Legacy", 1000.0),
        ("Mageblood", 12000.0),
        ("Headhunter", 8000.0),
        ("Temporalis", 20000.0),
        ("Ingenuity", 2000.0),
        ("Against the Darkness", 5000.0),
    ];

    for (name, value) in items {
        conn.execute(
            "INSERT OR IGNORE INTO chase_items (name, default_value_in_exalts, notes) VALUES (?1, ?2, '')",
            params![name, value],
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
