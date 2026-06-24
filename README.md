# Exile Farm Ledger

Manual investment and loot tracking tool for PoE2 farm strategies, built using Codex.

Exile Farm Ledger is a local-first Path of Exile 2 farming ledger built with Tauri, Svelte, TypeScript, Rust, and SQLite.

This app is intentionally not an automation tool. It does not interact with the game client, send input, read memory, read the screen, track stash contents, call trade APIs, or require an online account.

## Run

Install Node.js, Rust, and the Tauri prerequisites for Windows, then run:

```powershell
npm install
npm run tauri dev
```

## Test

```powershell
cd src-tauri
cargo test
```
