use md5::{Digest, Md5};
use reqwest::Client;
use serde_json::json;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let api_key = env::var("FOXESS_API_KEY")?;
    let inverter_serial = env::var("FOXESS_INVERTER_SERIAL")?;

    let path = "/op/v1/device/real/query";
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_millis()
        .to_string();

    let signature_input = format!("{path}\r\n{api_key}\r\n{timestamp}");
    let mut hasher = Md5::new();
    hasher.update(signature_input.as_bytes());
    let digest = hasher.finalize();

    let signature = digest
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();

    let body = json!({
    "sns": [inverter_serial]
    });

    let response = Client::new()
        .post(format!("https://www.foxesscloud.com{path}"))
        .header("token", &api_key)
        .header("timestamp", &timestamp)
        .header("signature", signature)
        .header("lang", "en")
        .header("Content-Type", "application/json")
        .header("User-Agent", "foxess-dashboard/0.1")
        .header("Timezone", "Australia/Sydney")
        .json(&body)
        .send()
        .await?;

    //println!("HTTP status: {}", response.status());

    let text = response.text().await?;

    let payload: serde_json::Value = serde_json::from_str(&text)?;

    if let Some(result) = payload["result"].as_array() {
        for device in result {
            println!("\nTimestamp: {}", device["time"]);

            if let Some(datas) = device["datas"].as_array() {
                println!("{:<30} {:>12}  {}", "Variable", "Value", "Unit");
                println!("{}", "-".repeat(58));

                for item in datas {
                    let variable = item["variable"].as_str().unwrap_or("unknown");
                    let value = item["value"]
                        .to_string()
                        .trim_matches('"')
                        .to_string();
                    let unit = item["unit"].as_str().unwrap_or("");

                    println!("{:<30} {:>12}  {}", variable, value, unit);
                }
            }
        }
    } else {
        println!("No telemetry result found.");
        println!("{text}");
    }

    Ok(())
}