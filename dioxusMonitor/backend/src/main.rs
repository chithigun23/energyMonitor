use axum::{Json, Router, extract::State, routing::get};
use md5::{Digest, Md5};
use reqwest::Client;
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    env,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tower_http::cors::CorsLayer;

#[derive(Clone, Debug, Serialize)]
struct EnergySnapshot {
    timestamp: String,
    solar_kw: f64,
    home_kw: f64,
    battery_soc: f64,
    battery_kw: f64,
    grid_import_kw: f64,
    grid_export_kw: f64,
}

#[derive(Clone)]
struct AppState {
    snapshot: Arc<RwLock<EnergySnapshot>>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let initial_snapshot = EnergySnapshot {
        timestamp: "Waiting for first FoxESS update".to_string(),
        solar_kw: 0.0,
        home_kw: 0.0,
        battery_soc: 0.0,
        battery_kw: 0.0,
        grid_import_kw: 0.0,
        grid_export_kw: 0.0,
    };

    let state = AppState {
        snapshot: Arc::new(RwLock::new(initial_snapshot)),
    };

    let fetch_state = state.clone();

    tokio::spawn(async move {
        loop {
            match fetch_foxess_snapshot().await {
                Ok(snapshot) => {
                    println!("FoxESS updated: {}", snapshot.timestamp);
                    *fetch_state.snapshot.write().expect("Cache lock failed") = snapshot;
                }
                Err(error) => {
                    eprintln!("FoxESS refresh failed: {error}");
                }
            }

            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    });

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/current", get(current_snapshot))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Could not bind to port 3000");

    println!("FoxESS backend listening on port 3000");

    axum::serve(listener, app)
        .await
        .expect("Backend server failed");
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "refresh_interval_seconds": 60,
        "website_refresh_seconds": 10
    }))
}

async fn current_snapshot(State(state): State<AppState>) -> Json<EnergySnapshot> {
    let snapshot = state.snapshot.read().expect("Cache lock failed").clone();

    Json(snapshot)
}

async fn fetch_foxess_snapshot() -> Result<EnergySnapshot, String> {
    let api_key =
        env::var("FOXESS_API_KEY").map_err(|_| "FOXESS_API_KEY is missing from backend/.env")?;

    let inverter_serial = env::var("FOXESS_INVERTER_SERIAL")
        .map_err(|_| "FOXESS_INVERTER_SERIAL is missing from backend/.env")?;

    let path = "/op/v1/device/real/query";

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis()
        .to_string();

    let signature_text = format!("{path}\r\n{api_key}\r\n{timestamp}");

    let mut hasher = Md5::new();
    hasher.update(signature_text.as_bytes());

    let signature = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    let response = Client::new()
        .post(format!("https://www.foxesscloud.com{path}"))
        .header("Token", &api_key)
        .header("Timestamp", &timestamp)
        .header("Signature", signature)
        .header("Lang", "en")
        .header("Content-Type", "application/json")
        .header("Timezone", "Australia/Sydney")
        .json(&json!({
            "sns": [inverter_serial]
        }))
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let payload: Value = response.json().await.map_err(|error| error.to_string())?;

    if payload["errno"].as_i64() != Some(0) {
        return Err(format!("FoxESS rejected the request: {payload}"));
    }

    let device = payload["result"]
        .as_array()
        .and_then(|devices| devices.first())
        .ok_or("FoxESS returned no device data")?;

    let data = device["datas"]
        .as_array()
        .ok_or("FoxESS returned no telemetry values")?;

    Ok(EnergySnapshot {
        timestamp: device["time"]
            .as_str()
            .unwrap_or("Unknown time")
            .to_string(),

        solar_kw: telemetry_value(data, "pvPower") + telemetry_value(data, "generationPower"),
        home_kw: telemetry_value(data, "loadsPower"),
        battery_soc: telemetry_value(data, "SoC"),
        battery_kw: telemetry_value(data, "batPower"),
        grid_import_kw: telemetry_value(data, "gridConsumptionPower"),
        grid_export_kw: telemetry_value(data, "feedinPower"),
    })
}

fn telemetry_value(data: &[Value], variable_name: &str) -> f64 {
    data.iter()
        .find(|item| item["variable"].as_str() == Some(variable_name))
        .and_then(|item| {
            item["value"]
                .as_f64()
                .or_else(|| item["value"].as_str()?.parse::<f64>().ok())
        })
        .unwrap_or(0.0)
}
