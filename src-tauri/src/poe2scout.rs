use crate::commands::insert_price_snapshot;
use crate::db::AppState;
use chrono::Utc;
use reqwest::Url;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tauri::State;

const BASE_URL: &str = "https://api.poe2scout.com";
const REALM: &str = "poe2";
const PER_PAGE: &str = "200";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct League {
    value: String,
    short_name: Option<String>,
    is_current: bool,
    divine_price: Option<f64>,
    chaos_divine_price: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CurrencyPage {
    current_page: i64,
    pages: i64,
    total: i64,
    items: Vec<ScoutCurrency>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct ScoutCurrency {
    text: String,
    api_id: String,
    current_price: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResult {
    league: String,
    updated: Vec<UpdatedCurrency>,
    skipped: Vec<String>,
    fetched_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedCurrency {
    name: String,
    value_in_exalts: f64,
}

#[tauri::command]
pub async fn refresh_currency_prices(state: State<'_, AppState>) -> Result<RefreshResult, String> {
    let client = reqwest::Client::builder()
        .user_agent(format!(
            "ExileFarmLedger/{} (+CONTACT_INFO_HERE)",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|err| format!("Could not create poe2scout HTTP client: {err}"))?;

    let league = fetch_current_league(&client).await?;
    let base_prices = fetch_currency_prices(&client, &league).await?;
    let exalted = base_prices
        .get("Exalted Orb")
        .copied()
        .ok_or_else(|| "poe2scout response did not include a price for Exalted Orb".to_string())?;
    if exalted <= 0.0 {
        return Err("poe2scout returned an invalid Exalted Orb price".to_string());
    }

    let prices_in_exalts = base_prices
        .into_iter()
        .map(|(name, price)| (name, price / exalted))
        .collect::<HashMap<_, _>>();
    let fetched_at = Utc::now().to_rfc3339();

    let mut conn = state.connection()?;
    let tx = conn.transaction().map_err(|err| err.to_string())?;
    let active_currencies = {
        let mut stmt = tx
            .prepare("SELECT id, name FROM currencies WHERE active = 1")
            .map_err(|err| err.to_string())?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| err.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())?;
        rows
    };

    let mut updated = Vec::new();
    let mut skipped = Vec::new();
    for (id, name) in active_currencies {
        if let Some(value) = prices_in_exalts.get(&name).copied() {
            tx.execute(
                "UPDATE currencies SET value_in_exalts = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![value, id],
            )
            .map_err(|err| err.to_string())?;
            insert_price_snapshot(&tx, &name, "currency", value)?;
            updated.push(UpdatedCurrency {
                name,
                value_in_exalts: value,
            });
        } else {
            skipped.push(name);
        }
    }
    tx.commit().map_err(|err| err.to_string())?;

    Ok(RefreshResult {
        league,
        updated,
        skipped,
        fetched_at,
    })
}

async fn fetch_current_league(client: &reqwest::Client) -> Result<String, String> {
    let mut url = Url::parse(BASE_URL).map_err(|err| err.to_string())?;
    url.path_segments_mut()
        .map_err(|_| "Could not build poe2scout leagues URL".to_string())?
        .extend([REALM, "Leagues"]);

    let leagues = client
        .get(url)
        .send()
        .await
        .map_err(|err| format!("Could not fetch poe2scout leagues: {err}"))?
        .error_for_status()
        .map_err(|err| format!("poe2scout leagues request failed: {err}"))?
        .json::<Vec<League>>()
        .await
        .map_err(|err| format!("Could not parse poe2scout leagues response: {err}"))?;

    leagues
        .into_iter()
        .find(|league| league.is_current)
        .map(|league| league.value)
        .ok_or_else(|| "poe2scout did not report a current league".to_string())
}

async fn fetch_currency_prices(
    client: &reqwest::Client,
    league: &str,
) -> Result<HashMap<String, f64>, String> {
    let mut page = 1;
    let mut prices = HashMap::new();

    loop {
        let response = fetch_currency_page(client, league, page).await?;
        for item in response.items {
            if let Some(price) = item.current_price {
                prices.insert(item.text, price);
            }
        }

        if response.current_page >= response.pages || page >= response.pages {
            break;
        }
        page += 1;
    }

    Ok(prices)
}

async fn fetch_currency_page(
    client: &reqwest::Client,
    league: &str,
    page: i64,
) -> Result<CurrencyPage, String> {
    let mut url = Url::parse(BASE_URL).map_err(|err| err.to_string())?;
    url.path_segments_mut()
        .map_err(|_| "Could not build poe2scout currencies URL".to_string())?
        .extend([REALM, "Leagues", league, "Currencies", "ByCategory"]);
    url.query_pairs_mut()
        .append_pair("Category", "currency")
        .append_pair("Page", &page.to_string())
        .append_pair("PerPage", PER_PAGE);

    client
        .get(url)
        .send()
        .await
        .map_err(|err| format!("Could not fetch poe2scout currency prices: {err}"))?
        .error_for_status()
        .map_err(|err| format!("poe2scout currency request failed: {err}"))?
        .json::<CurrencyPage>()
        .await
        .map_err(|err| format!("Could not parse poe2scout currency response: {err}"))
}
