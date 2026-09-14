use dioxus::prelude::*;
use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use serde::Deserialize;

static SOLAR_IMAGE: Asset = asset!("/assets/flow/solar.png");
static HOME_IMAGE: Asset = asset!("/assets/flow/home.png");
static GRID_IMAGE: Asset = asset!("/assets/flow/grid.png");
static BATTERY_IMAGE: Asset = asset!("/assets/flow/battery.png");

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct EnergySnapshot {
    timestamp: String,
    solar_kw: f64,
    home_kw: f64,
    battery_soc: f64,
    battery_kw: f64,
    grid_import_kw: f64,
    grid_export_kw: f64,
}

fn main() {
    dioxus::launch(app);
}

fn app() -> Element {
    let mut snapshot = use_signal(|| None::<EnergySnapshot>);
    let mut connection_error = use_signal(|| None::<String>);
    let mut refresh_count = use_signal(|| 0_u64);

    use_future(move || async move {
        loop {
            match Request::get("http://127.0.0.1:3000/api/current")
                .send()
                .await
            {
                Ok(response) => match response.json::<EnergySnapshot>().await {
                    Ok(next_snapshot) => {
                        snapshot.set(Some(next_snapshot));
                        refresh_count += 1;
                        connection_error.set(None);
                    }
                    Err(error) => {
                        connection_error.set(Some(format!("Could not read backend data: {error}")))
                    }
                },
                Err(error) => connection_error.set(Some(format!("Backend unavailable: {error}"))),
            }
            TimeoutFuture::new(10_000).await;
        }
    });

    let page = match snapshot() {
        Some(data) => {
            let grid_value = if data.grid_export_kw > 0.05 {
                format!("{:.2} kW out", data.grid_export_kw)
            } else {
                format!("{:.2} kW in", data.grid_import_kw)
            };
            let battery_state = if data.battery_kw > 0.05 {
                "Charging"
            } else {
                "Ready"
            };
            let grid_state = if data.grid_export_kw > 0.05 {
                "Exporting"
            } else {
                "Importing"
            };

            let advice = if data.grid_export_kw > 0.5 {
                let message = format!(
                    "{:.2} kW of spare generation is going to the grid.",
                    data.grid_export_kw
                );
                rsx! {
                    section { class: "advice advice-use",
                        p { class: "advice-kicker", "POWER ADVICE" }
                        h2 { "☀ USE ELECTRICITY" }
                        p { "{message}" }
                        p { class: "advice-examples", "Good for: dishwasher, laundry, EV charging" }
                    }
                }
            } else if data.grid_import_kw > 0.5 || data.battery_soc < 25.0 {
                let message = if data.grid_import_kw > 0.5 {
                    format!(
                        "The home is drawing {:.2} kW from the grid.",
                        data.grid_import_kw
                    )
                } else {
                    format!("Battery reserve is low at {:.0}%.", data.battery_soc)
                };
                rsx! {
                    section { class: "advice advice-avoid",
                        p { class: "advice-kicker", "POWER ADVICE" }
                        h2 { "⚡ AVOID HIGH-POWER USE" }
                        p { "{message}" }
                        p { class: "advice-examples", "Good for: delay the dryer, EV charging, or electric heating" }
                    }
                }
            } else {
                rsx! {
                    section { class: "advice advice-reduce",
                        p { class: "advice-kicker", "POWER ADVICE" }
                        h2 { "⚖ REDUCE ELECTRICITY USE" }
                        p { "There is no large spare solar supply right now." }
                        p { class: "advice-examples", "Normal use is fine; delay flexible heavy appliances." }
                    }
                }
            };

            rsx! {
                {advice}
                section { class: "flow-panel",
                    div { class: "panel-heading",
                        h2 { "Energy flow" }
                        p { "Live system snapshot" }
                    }
                    div { class: "flow-grid",
                        section { class: "flow-node flow-solar",
                            img { src: SOLAR_IMAGE, alt: "Solar panels" }
                            p { "Total generation" }
                            strong { "{data.solar_kw:.2} kW" }
                            small { "Solar available" }
                        }
                        p { class: "route route-one", "➜" }
                        p { class: "route route-two", "↘" }
                        p { class: "route route-three", "↙" }
                        p { class: "route route-four", "➜" }
                        div { class: "hub", "⚡" }
                        section { class: "flow-node flow-home",
                            img { src: HOME_IMAGE, alt: "House" }
                            p { "Home" }
                            strong { "{data.home_kw:.2} kW" }
                            small { "Using now" }
                        }
                        section { class: if data.grid_export_kw > 0.05 { "flow-node flow-grid-node flow-grid-out" } else { "flow-node flow-grid-node flow-grid-in" },
                            img { src: GRID_IMAGE, alt: "Electricity grid" }
                            p { "Grid" }
                            strong { "{grid_value}" }
                            small { "{grid_state}" }
                        }
                        section { class: "flow-node flow-battery",
                            img { src: BATTERY_IMAGE, alt: "Battery" }
                            p { "Battery" }
                            strong { "{data.battery_soc:.0}%" }
                            small { "{battery_state}" }
                        }
                    }
                    div { class: "battery-reserve",
                        div { class: "battery-label", span { "Battery reserve" } span { "{data.battery_soc:.0}%" } }
                        progress { value: "{data.battery_soc}", max: "100" }
                    }
                }
                section { class: "metrics",
                    article { p { "Generation" } h2 { "{data.solar_kw:.2} kW" } small { "Panels + inverter" } }
                    article { p { "Home use" } h2 { "{data.home_kw:.2} kW" } small { "Using now" } }
                    article { p { "Battery" } h2 { "{data.battery_soc:.0}%" } small { "Stored energy" } }
                    article { p { "Grid" } h2 { "{grid_value}" } small { "Grid flow" } }
                }
                p { class: "footer-status", "FoxESS data: {data.timestamp} • Browser check #{refresh_count}" }
            }
        }
        None => rsx! {
            section { class: "connecting",
                h2 { "Connecting to your energy system…" }
                if let Some(error) = connection_error() { p { "{error}" } }
                else { p { "Waiting for the first cached reading from the local FoxESS backend." } }
            }
        },
    };

    rsx! {
        main { class: "page",
            style { "
                .page {{ max-width: 900px; margin: 0 auto; padding: 2rem; font-family: system-ui, sans-serif; color: #1c2938; }}
                .subtitle, .panel-heading p, .flow-node p, .flow-node small, .metrics p, .metrics small, .footer-status {{ color: #617084; }}
                .advice {{ padding: 1.5rem; border-radius: 20px; margin: 1.5rem 0; border-left: 8px solid; }}
                .advice h2 {{ margin: .35rem 0; }} .advice p {{ margin: 0; font-size: 1.1rem; }}
                .advice-kicker {{ font-weight: 700; letter-spacing: .05em; }} .advice-examples {{ margin-top: .75rem !important; color: #536170; }}
                .advice-use {{ background: #e9f8ef; border-color: #1e8e5a; }} .advice-use .advice-kicker {{ color: #1e8e5a; }}
                .advice-reduce {{ background: #fff7e5; border-color: #b7791f; }} .advice-reduce .advice-kicker {{ color: #b7791f; }}
                .advice-avoid {{ background: #ffefed; border-color: #c4543a; }} .advice-avoid .advice-kicker {{ color: #c4543a; }}
                .flow-panel {{ padding: 1.25rem; border: 1px solid #dce4ef; border-radius: 24px; background: linear-gradient(145deg, #f9fbff, #eef4fb); }}
                .panel-heading, .battery-label {{ display: flex; justify-content: space-between; align-items: baseline; gap: 1rem; flex-wrap: wrap; }} .panel-heading h2 {{ margin: 0; }} .panel-heading p {{ margin: 0; }}
                .flow-grid {{ display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); grid-template-areas: 'solar route1 home' 'route2 hub route3' 'grid route4 battery'; align-items: center; gap: .5rem; margin-top: 1rem; text-align: center; }}
                .flow-node {{ padding: .8rem; min-height: 7.5rem; border-radius: 16px; background: white; border-top: 5px solid; box-shadow: 0 4px 14px rgba(29, 55, 84, .08); }} .flow-node img {{ width: 100%; height: 4.3rem; object-fit: contain; display: block; }} .flow-node p {{ margin: .1rem 0; font-size: .8rem; }} .flow-node strong {{ font-size: 1.05rem; display: block; }} .flow-node small {{ font-size: .8rem; }}
                .flow-solar {{ grid-area: solar; border-color: #d79a00; }} .flow-home {{ grid-area: home; border-color: #3478c6; }} .flow-grid-node {{ grid-area: grid; }} .flow-grid-out {{ border-color: #1e8e5a; }} .flow-grid-in {{ border-color: #c4543a; }} .flow-battery {{ grid-area: battery; border-color: #8067c7; }}
                .route {{ margin: 0; font-size: clamp(2.1rem, 5vw, 4rem); font-weight: 800; line-height: 1; filter: drop-shadow(0 3px 3px rgba(33, 59, 88, .22)); animation: energy-pulse 1.8s ease-in-out infinite; }} .route-one {{ grid-area: route1; color: #e6ae18; }} .route-two {{ grid-area: route2; color: #5f91c4; animation-delay: .3s; }} .route-three {{ grid-area: route3; color: #8a72d1; animation-delay: .6s; }} .route-four {{ grid-area: route4; color: #5f91c4; animation-delay: .9s; }} @keyframes energy-pulse {{ 0%, 100% {{ transform: scale(.92); opacity: .55; }} 50% {{ transform: scale(1.1); opacity: 1; }} }} .hub {{ grid-area: hub; width: 3.5rem; height: 3.5rem; margin: auto; border-radius: 50%; display: grid; place-items: center; background: #dce9f7; color: #375879; font-size: 1.5rem; box-shadow: 0 0 0 6px rgba(124, 161, 199, .18), 0 8px 18px rgba(55, 88, 121, .18); }}
                .battery-reserve {{ margin-top: 1.25rem; }} .battery-label {{ color: #536170; font-size: .9rem; }} progress {{ width: 100%; margin-top: .35rem; accent-color: #8067c7; }}
                .metrics {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(145px, 1fr)); gap: .75rem; margin: 1rem 0; }} .metrics article {{ padding: 1rem; border: 1px solid #dce4ef; border-radius: 16px; background: white; }} .metrics p, .metrics small {{ margin: 0; }} .metrics h2 {{ margin: .35rem 0; }} .footer-status {{ text-align: center; }}
                .connecting {{ margin-top: 2rem; padding: 2rem; border-radius: 20px; background: #fff7e5; }}
                @media (max-width: 560px) {{ .page {{ padding: 1rem; }} .flow-grid {{ gap: .25rem; }} .flow-node {{ padding: .4rem; }} .flow-node img {{ height: 3.2rem; }} .flow-node strong {{ font-size: .85rem; }} }}
            " }
            h1 { style: "margin-bottom: .25rem;", "Home Energy" }
            p { class: "subtitle", "Simple guidance for solar and battery power." }
            {page}
        }
    }
}
