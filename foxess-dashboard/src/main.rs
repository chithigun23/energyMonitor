mod dashboard;

use dashboard::{DashboardApp, EnergySnapshot};

fn main() -> Result<(), eframe::Error> {
    let current = EnergySnapshot {
        timestamp: "Sample data".to_string(),
        timestamp_seconds: 0.0,
        solar_kw: 4.2,
        home_kw: 1.3,
        battery_soc: 78.0,
        battery_kw: -1.4,
        grid_import_kw: 0.0,
        grid_export_kw: 2.9,
    };

    let history = vec![
        current.clone(),
        EnergySnapshot {
            timestamp: "Sample 2".to_string(),
            timestamp_seconds: 300.0,
            solar_kw: 3.8,
            home_kw: 1.6,
            ..current.clone()
        },
        EnergySnapshot {
            timestamp: "Sample 3".to_string(),
            timestamp_seconds: 600.0,
            solar_kw: 5.1,
            home_kw: 1.4,
            ..current.clone()
        },
        EnergySnapshot {
            timestamp: "Sample 4".to_string(),
            timestamp_seconds: 900.0,
            solar_kw: 4.6,
            home_kw: 1.8,
            ..current.clone()
        },
    ];

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "FoxESS Dashboard",
        options,
        Box::new(|_creation_context| Ok(Box::new(DashboardApp { current, history }))),
    )
}
