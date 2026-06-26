# Security & Risk Analysis — Exile Farm Ledger (PoE2 Farm Ledger)

**Subject:** `PoE2-Farm-Ledger-main` (v0.1.2), source code as distributed on GitHub
**Method:** Full static source review of the Tauri/Rust backend, Svelte/TypeScript frontend, build/release pipeline, and configuration files. No binary/installer was analyzed (source only).
**Audience:** Players deciding whether to install this third-party tool.

## TL;DR

This is a small, **local-only** desktop app for manually logging Path of Exile 2 farming sessions (loot, investments, profit). It does **not** touch the game client, read memory, automate input, or require a game account. The only network calls are (a) a public read-only price API and (b) GitHub's update-check endpoint. Source code is plain, readable, and shows no signs of obfuscation, telemetry, or data exfiltration. The main real-world risk is the **unsigned Windows installer**, which is a trust/provenance issue, not a code-behavior issue found in this review.

**Overall risk rating: Low**, conditional on installing only from the official GitHub releases page and verifying you got the genuine `FlipJaxx/PoE2-Farm-Ledger` repo.

---

## 1. What the app actually does

- Built with **Tauri 2** (Rust backend + a Svelte/TypeScript webview frontend), packaged as a Windows NSIS installer (per `tauri.conf.json`, only the `nsis` bundle target is configured).
- All farming data (sessions, loot, investments, currency prices, strategies) is stored in a **local SQLite database** under the OS's app-data directory (`db.rs`). There is no cloud sync, no account system, no remote database.
- The README explicitly states, and the code confirms, that it does not interact with the PoE2 game client, does not read memory/screen, does not send game input, and does not call trade APIs.

## 2. Network surface (everything the app can talk to)

There are exactly two network destinations in the entire codebase:

| Destination | Purpose | Triggered by | Data sent |
|---|---|---|---|
| `api.poe2scout.com` | Public currency price lookup (`src-tauri/src/poe2scout.rs`) | Only when the user clicks "refresh prices" | None — it's a GET request with a generic `User-Agent` string; no user data, telemetry, or identifiers are sent |
| `github.com/FlipJaxx/PoE2-Farm-Ledger/releases/...` | Tauri auto-updater checking for new versions (`tauri.conf.json`) | Automatically on app startup | Standard HTTP GET, no personal data |

No analytics SDKs, crash reporters, ad networks, or any other third-party services are present anywhere in `package.json`, `Cargo.toml`, or the source. There is no code that reads files outside the app's own data directory, no clipboard access, no screen capture, no keylogging/input-hooking of any kind.

## 3. Tauri permission/capability model (sandboxing)

Tauri 2 requires apps to explicitly declare which OS-level capabilities the **webview/JS frontend** is allowed to invoke. This app's capability file (`src-tauri/capabilities/default.json`) grants only:

- `core:default` — baseline Tauri runtime APIs
- `updater:default` — permission to check/install updates
- `process:default` — permission to relaunch/exit the app (used after an update installs)

Notably **absent**: filesystem access, shell/command execution, HTTP fetch from the webview, clipboard, dialog/file-picker, or any "all" wildcard permission. This means even if something were wrong with the frontend code, it has no OS-level capability to read your files, run commands, or reach the network directly — all of that is mediated through a small, fixed set of Rust commands (`commands.rs`) that only do SQLite reads/writes and the one price-fetch call described above.

The Content-Security-Policy in `tauri.conf.json` is also restrictive:
```
default-src 'self'; img-src 'self' asset: data:; style-src 'self' 'unsafe-inline'; script-src 'self'; connect-src 'self' ipc: http://ipc.localhost
```
This blocks loading of any external scripts or making arbitrary `fetch()`/`XHR` calls from the webview — consistent with the app being self-contained.

## 4. Code-level review findings

- **SQL injection:** All user-supplied values go through parameterized `rusqlite` queries (`params![...]`). The few places that build SQL strings dynamically (`format!(...)`) only interpolate **hardcoded, internal** column names (e.g. `"strategy_name"`/`"mechanic_name"` literals) or a count of `?N` placeholders — never raw user input. No injection vector found.
- **Dependencies:** A small, mainstream dependency set (`rusqlite`, `reqwest` with `rustls`, `serde`, `chrono`, `tauri` + official `updater`/`process` plugins). No unusual, abandoned, or red-flag crates/npm packages.
- **Update mechanism:** Uses Tauri's official updater plugin with a **minisign public key embedded in config** to verify update artifact signatures before installing — i.e., the app won't auto-install an update unless it's signed with the matching private key, mitigating tampering of update payloads in transit (assuming the developer's signing key itself isn't compromised).
- **No obfuscation:** Source is clean, idiomatic Rust/TypeScript with tests (`commands.rs` includes unit tests for session math). This is not minified/obfuscated malware-style code.
- **Local data only:** SQLite file lives in the standard per-user Tauri app-data folder; nothing is written elsewhere, no registry changes beyond what NSIS itself does for install/uninstall.

## 5. Actual risks and caveats

1. **Unsigned installer (the main real risk).** The GitHub Actions release workflow (`.github/workflows/release.yml`) builds the Windows installer and explicitly notes: *"Windows may show SmartScreen warnings because the installer is not code-signed."* This is **not a malware indicator** — it's normal for a small open-source/indie project without a paid EV code-signing certificate. But it does mean:
   - Windows SmartScreen/Defender may flag the installer as "unrecognized publisher."
   - There is no certificate-based proof of the binary's publisher identity beyond GitHub's hosting of the release and repo.
   - Practical mitigation: only download from the official `https://github.com/FlipJaxx/PoE2-Farm-Ledger/releases` page, verify the download isn't from a third-party mirror, and let your antivirus scan it.

2. **Source-vs-binary trust gap.** This analysis covers the **source code** of this exact zip snapshot. It does not verify that any specific compiled `.exe` release was built from this exact source (i.e., supply-chain trust still depends on GitHub Actions CI being the actual build path, which it appears to be — the release workflow runs `cargo test`, `svelte-check`, and `tauri build` directly from the repo).

3. **No code signing = no revocation path.** If a future malicious version were ever published, there's no certificate-authority-backed mechanism to revoke trust; users rely on the GitHub repo's reputation/history.

4. **Auto-update trust boundary.** The updater fetches `latest.json` from GitHub Releases on every launch and can install updates with only a minisign signature check. If the maintainer's GitHub account or signing key were compromised, a malicious update could theoretically be pushed. This is a standard auto-updater risk shared by most small desktop apps, not something specific to bad practice here — the signature check is a real (if modest) mitigation.

5. **Game ToS risk, not security risk.** Since the app is purely a manual spreadsheet-like ledger with no game-client interaction, it carries effectively no risk of being flagged as third-party automation/botting by anti-cheat — worth noting for players worried about ban risk rather than malware risk.

## 6. Recommendation for cautious users

- Download only from the official repository's Releases page; check the URL is `github.com/FlipJaxx/PoE2-Farm-Ledger`.
- Expect a SmartScreen "unknown publisher" prompt — this is expected for an unsigned indie app, not proof of malice.
- If you want extra assurance, the source is fully readable (it's a small codebase, a few thousand lines total) — anyone with basic Rust/TS knowledge can audit it in under an hour, which is what this report is based on.
- No special permissions, accounts, or game folder access are required to use it, and none are requested.
