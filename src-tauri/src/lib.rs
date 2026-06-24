mod calculations;
mod commands;
mod db;
mod models;
mod schema;

use db::AppState;

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::initialize_database,
            commands::get_dashboard_data,
            commands::create_session,
            commands::get_active_session,
            commands::get_session,
            commands::update_session_maps,
            commands::add_or_update_session_loot,
            commands::add_or_update_session_investment,
            commands::stop_session,
            commands::cancel_session,
            commands::list_sessions,
            commands::list_currencies,
            commands::update_currency_value,
            commands::update_currency_order,
            commands::create_custom_currency,
            commands::list_chase_items,
            commands::update_chase_item_value,
            commands::create_chase_item,
            commands::list_mechanics,
            commands::create_mechanic,
            commands::create_strategy,
            commands::update_strategy,
            commands::delete_strategy,
            commands::list_strategies,
            commands::get_reports_data
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Exile Farm Ledger");
}
