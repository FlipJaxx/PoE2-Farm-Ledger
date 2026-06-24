use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SessionTotals {
    pub total_loot_value_exalts: f64,
    pub total_investment_value_exalts: f64,
    pub net_profit_exalts: f64,
    pub profit_per_hour_exalts: f64,
    pub profit_per_map_exalts: f64,
    pub maps_per_hour: f64,
    pub divine_per_hour: f64,
}

pub fn line_total(count: f64, value_in_exalts: f64) -> f64 {
    count.max(0.0) * value_in_exalts.max(0.0)
}

pub fn session_totals(
    loot: f64,
    investment: f64,
    duration_seconds: i64,
    maps_run: i64,
    divine_value_exalts: f64,
) -> SessionTotals {
    let duration_hours = duration_seconds.max(0) as f64 / 3600.0;
    let maps = maps_run.max(0) as f64;
    let net = loot - investment;
    let profit_per_hour = if duration_hours > 0.0 {
        net / duration_hours
    } else {
        0.0
    };
    let profit_per_map = if maps > 0.0 { net / maps } else { 0.0 };
    let maps_per_hour = if duration_hours > 0.0 {
        maps / duration_hours
    } else {
        0.0
    };
    let divine_per_hour = if divine_value_exalts > 0.0 {
        profit_per_hour / divine_value_exalts
    } else {
        0.0
    };

    SessionTotals {
        total_loot_value_exalts: loot,
        total_investment_value_exalts: investment,
        net_profit_exalts: net,
        profit_per_hour_exalts: profit_per_hour,
        profit_per_map_exalts: profit_per_map,
        maps_per_hour,
        divine_per_hour,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_session_metrics() {
        let totals = session_totals(900.0, 300.0, 7200, 20, 120.0);

        assert_eq!(totals.net_profit_exalts, 600.0);
        assert_eq!(totals.profit_per_hour_exalts, 300.0);
        assert_eq!(totals.profit_per_map_exalts, 30.0);
        assert_eq!(totals.maps_per_hour, 10.0);
        assert_eq!(totals.divine_per_hour, 2.5);
    }

    #[test]
    fn division_by_zero_is_safe() {
        let totals = session_totals(10.0, 20.0, 0, 0, 0.0);

        assert_eq!(totals.net_profit_exalts, -10.0);
        assert_eq!(totals.profit_per_hour_exalts, 0.0);
        assert_eq!(totals.profit_per_map_exalts, 0.0);
        assert_eq!(totals.maps_per_hour, 0.0);
        assert_eq!(totals.divine_per_hour, 0.0);
    }
}
