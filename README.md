# Exile Farm Ledger

Exile Farm Ledger is a local-first farming ledger for Path of Exile 2. It is built for manually tracking farming sessions, investments, loot, price assumptions, strategy performance, and reports without connecting to the game client or any online account.

The app is built with Tauri, Svelte, TypeScript, Rust, and SQLite.

## Purpose

The goal is to make farm testing easier to compare over time:

- Track what you invested before or during a farming session.
- Track loot manually as currency, chase items, or custom rows.
- Keep historical session values based on the prices used at the time.
- Compare strategies and mechanics by profit, maps, and time.
- Stay local, private, and intentionally manual.

## What It Does

### Dashboard

- Shows total completed-session profit, maps, time, and session count.
- Shows the active session if one is running.
- Shows recent sessions and best strategy summaries.

### Sessions

- Create a new farming session from a saved strategy or as a manual strategy.
- Track one active session at a time.
- Update map/run count.
- Add or edit currency loot, chase item loot, custom loot, and investments.
- Stop a session to preserve final duration and calculated metrics.
- Cancel a running session.
- Open historical session details.

### Strategies and Mechanics

- Create and edit strategies.
- Create mechanics used to group strategies and reports.
- Store default notes for a strategy.
- Store default investment rows for a strategy.
- Apply strategy default notes and investment rows automatically when a session starts.

`default_investment_rows` is a JSON array:

```json
[
  {
    "investment_type": "Maps",
    "item_name": "T15 Waystone",
    "count": 6,
    "value_in_exalts": 0.5
  }
]
```

`default_chase_items` is currently stored for future use, but chase items are already seeded into new sessions from the chase item price list.

### Prices

- Edit currency values in Exalted Orb units.
- Fetch current currency prices from poe2scout in the desktop app.
- Store fetched currency values as whole Exalted Orb values to keep the price overview compact.
- Create custom currencies.
- Reorder currencies with Up/Down controls.
- Edit chase item values.
- Create custom chase items.
- Friendly validation is shown for missing names and duplicate names.

poe2scout refresh only updates currencies that already exist in the local price list. Fetched values are converted to Exalted Orb units, rounded to the nearest whole exalt, and written together with matching price snapshots. Manual currency and chase item edits still support decimal values.

### Updates

- Checks GitHub releases for app updates on startup.
- Shows an update banner when a newer release is available.
- Installs updates through Tauri's signed updater artifacts.

### Reports

- Compare completed sessions grouped by mechanic.
- Compare completed sessions grouped by strategy.
- Review averages and totals for maps, time, profit per hour, profit per map, best session, and worst session.

## Boundaries

Exile Farm Ledger is intentionally not an automation tool.

It does not:

- interact with the game client
- send input to the game
- read memory
- read the screen
- track stash contents
- call trade APIs
- require an online account

All entries are manual.

## Data and Privacy

The app stores data locally in SQLite through Tauri's app data directory. No backend server or cloud account is required.

SQLite connections enable:

- foreign key enforcement
- WAL journal mode
- a 5 second busy timeout for write contention

## Tech Stack

- Tauri 2
- Svelte 5
- TypeScript
- Rust
- SQLite via `rusqlite`
- Vite

## Requirements

Install:

- Node.js
- Rust and Cargo
- Tauri prerequisites for Windows

Tauri's Windows prerequisites are documented here:

https://v2.tauri.app/start/prerequisites/

## Run in Development

Install dependencies:

```powershell
npm install
```

Run the Tauri desktop app:

```powershell
npm run tauri dev
```

For frontend-only browser testing:

```powershell
npm run dev
```

The browser mode uses a local development fallback instead of the Rust/Tauri backend. It is useful for UI work, but the full desktop app should be tested with Tauri.

## Build

Build the frontend:

```powershell
npm run build
```

Build the Tauri app:

```powershell
npm run tauri build
```

## Test and Checks

Frontend and Svelte diagnostics:

```powershell
npx svelte-check --tsconfig ./tsconfig.json
```

Rust tests:

```powershell
cd src-tauri
cargo test
```

## Project Structure

```text
src/
  App.svelte                  Main UI and routes
  api.ts                      Tauri API wrapper and browser-dev fallback
  components/                 Tables and session UI components
  styles.css                  Design tokens and global styling

src-tauri/
  src/commands.rs             Tauri command handlers
  src/db.rs                   SQLite connection setup
  src/schema.rs               Migration and seed logic
  src/models.rs               Shared Rust data models
  migrations/                 Initial schema reference
  icons/                      Generated Tauri app icons
  tauri.conf.json             Tauri app configuration

assets/
  icon-source-1024.png        Source image used to generate Tauri icons
```

## Current Notes

- Historical sessions keep the values captured when the session was created or edited.
- Recalculation of historical sessions with current prices is intentionally left for a later version.
- Currency prices can be refreshed from poe2scout or maintained manually.
- Chase item prices are manually maintained.
- Strategy defaults currently apply notes and investment rows when a session starts.
