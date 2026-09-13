mod dashboard;

use dashboard::{DashboardApp, EnergySnapshot};

fn main() -> Result<(), eframe::Error> {
    let current = EnergySnapshot {
        timestamp: "Sample data".to_string(),
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
            solar_kw: 3.8,
            home_kw: 1.6,
            ..current.clone()
        },
        EnergySnapshot {
            solar_kw: 5.1,
            home_kw: 1.4,
            ..current.clone()
        },
        EnergySnapshot {
            solar_kw: 4.6,
            home_kw: 1.8,
            ..current.clone()
        },
    ];

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "FoxESS Dashboard",
        options,
        Box::new(|_cc| Ok(Box::new(DashboardApp { current, history }))),
    )
}
