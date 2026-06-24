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
