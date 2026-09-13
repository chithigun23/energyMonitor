use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};

#[derive(Clone, Debug)]
pub struct EnergySnapshot {
    pub timestamp: String,
    pub solar_kw: f64,
    pub home_kw: f64,
    pub battery_soc: f64,
    pub battery_kw: f64,
    pub grid_import_kw: f64,
    pub grid_export_kw: f64,
}

pub struct DashboardApp {
    pub current: EnergySnapshot,
    pub history: Vec<EnergySnapshot>,
}

impl eframe::App for DashboardApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("FoxESS KH10 Energy Dashboard");
        ui.label(format!("Updated: {}", self.current.timestamp));

        ui.separator();

        ui.horizontal_wrapped(|ui| {
            metric(ui, "Solar", self.current.solar_kw, "kW");
            metric(ui, "Home", self.current.home_kw, "kW");
            metric(ui, "Battery SoC", self.current.battery_soc, "%");
            metric(ui, "Battery power", self.current.battery_kw, "kW");
            metric(ui, "Grid import", self.current.grid_import_kw, "kW");
            metric(ui, "Grid export", self.current.grid_export_kw, "kW");
        });

        ui.separator();

        let solar_points = PlotPoints::from_iter(
            self.history
                .iter()
                .enumerate()
                .map(|(i, snapshot)| [i as f64, snapshot.solar_kw]),
        );

        let home_points = PlotPoints::from_iter(
            self.history
                .iter()
                .enumerate()
                .map(|(i, snapshot)| [i as f64, snapshot.home_kw]),
        );

        let battery_points = PlotPoints::from_iter(
            self.history
                .iter()
                .enumerate()
                .map(|(i, snapshot)| [i as f64, snapshot.battery_kw]),
        );

        let grid_import_points = PlotPoints::from_iter(
            self.history
                .iter()
                .enumerate()
                .map(|(i, snapshot)| [i as f64, snapshot.grid_import_kw]),
        );

        Plot::new("power-history")
            .legend(egui_plot::Legend::default())
            .x_axis_label("Reading")
            .y_axis_label("Power (kW)")
            .show(ui, |plot_ui| {
                plot_ui.line(Line::new("Solar", solar_points));
                plot_ui.line(Line::new("Home", home_points));
                plot_ui.line(Line::new("Battery", battery_points));
                plot_ui.line(Line::new("Grid import", grid_import_points));
            });
    }
}

fn metric(ui: &mut egui::Ui, label: &str, value: f64, unit: &str) {
    ui.vertical(|ui| {
        ui.label(label);
        ui.heading(format!("{value:.2} {unit}"));
    });
}
