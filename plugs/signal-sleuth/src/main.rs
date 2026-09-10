use serde::{Deserialize, Serialize};
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{
    HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, Position,
};
use yew::prelude::*;

const STORAGE_KEY: &str = "signal-sleuth-reports-v1";
const STUDIO: (f64, f64) = (30.0972, -95.6161);
const KRLY: (f64, f64) = (30.1017, -95.6218);

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Report {
    frequency: String,
    station: String,
    latitude: f64,
    longitude: f64,
    strength: u8,
    quality: String,
    features: String,
    notes: String,
    timestamp: String,
}

fn distance_miles(a: (f64, f64), b: (f64, f64)) -> f64 {
    let earth_radius = 3958.8;

    let latitude_1 = a.0.to_radians();
    let latitude_2 = b.0.to_radians();
    let latitude_difference = (b.0 - a.0).to_radians();
    let longitude_difference = (b.1 - a.1).to_radians();

    let haversine = (latitude_difference / 2.0).sin().powi(2)
        + latitude_1.cos()
            * latitude_2.cos()
            * (longitude_difference / 2.0).sin().powi(2);

    earth_radius
        * 2.0
        * haversine
            .sqrt()
            .atan2((1.0 - haversine).sqrt())
}

fn bearing(a: (f64, f64), b: (f64, f64)) -> f64 {
    let latitude_1 = a.0.to_radians();
    let latitude_2 = b.0.to_radians();
    let longitude_difference = (b.1 - a.1).to_radians();

    let y = longitude_difference.sin() * latitude_2.cos();

    let x = latitude_1.cos() * latitude_2.sin()
        - latitude_1.sin()
            * latitude_2.cos()
            * longitude_difference.cos();

    (y.atan2(x).to_degrees() + 360.0) % 360.0
}

fn compass(degrees: f64) -> &'static str {
    const POINTS: [&str; 8] = [
        "N", "NE", "E", "SE", "S", "SW", "W", "NW",
    ];

    POINTS[((degrees / 45.0).round() as usize) % 8]
}

fn load_reports() -> Vec<Report> {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(STORAGE_KEY).ok().flatten())
        .and_then(|value| serde_json::from_str(&value).ok())
        .unwrap_or_default()
}

fn save_reports(reports: &[Report]) {
    let storage = web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten());

    let json = serde_json::to_string(reports);

    if let (Some(storage), Ok(json)) = (storage, json) {
        let _ = storage.set_item(STORAGE_KEY, &json);
    }
}

#[function_component(App)]
fn app() -> Html {
    let frequency = use_state(|| "99.9 FM".to_string());
    let station = use_state(|| "Mystery Signal".to_string());
    let latitude = use_state(|| STUDIO.0.to_string());
    let longitude = use_state(|| STUDIO.1.to_string());
    let strength = use_state(|| 65_u8);
    let quality = use_state(|| "Clear".to_string());
    let features = use_state(String::new);
    let notes = use_state(String::new);
    let status = use_state(|| {
        "Ready for a field observation".to_string()
    });
    let reports = use_state(load_reports);

    let current_latitude = latitude
        .parse::<f64>()
        .unwrap_or(STUDIO.0);

    let current_longitude = longitude
        .parse::<f64>()
        .unwrap_or(STUDIO.1);

    let current_location = (
        current_latitude,
        current_longitude,
    );

    let studio_distance =
        distance_miles(STUDIO, current_location);

    let krly_distance =
        distance_miles(KRLY, current_location);

    let current_bearing =
        bearing(STUDIO, current_location);

    let locate = {
        let latitude = latitude.clone();
        let longitude = longitude.clone();
        let status = status.clone();

        Callback::from(move |_| {
            status.set(
                "Requesting iPhone location…".to_string()
            );

            if let Some(navigator) =
                web_sys::window().map(|window| window.navigator())
            {
                if let Ok(geolocation) = navigator.geolocation() {
                    let latitude = latitude.clone();
                    let longitude = longitude.clone();
                    let status = status.clone();

                    let success =
                        Closure::<dyn FnMut(Position)>::new(
                            move |position: Position| {
                                latitude.set(format!(
                                    "{:.6}",
                                    position.coords().latitude()
                                ));

                                longitude.set(format!(
                                    "{:.6}",
                                    position.coords().longitude()
                                ));

                                status.set(
                                    "Location locked — ready to record"
                                        .to_string(),
                                );
                            },
                        );

                    let _ = geolocation.get_current_position(
                        success.as_ref().unchecked_ref(),
                    );

                    success.forget();
                } else {
                    status.set(
                        "Location unavailable — enter coordinates manually"
                            .to_string(),
                    );
                }
            }
        })
    };

    let record = {
        let frequency = frequency.clone();
        let station = station.clone();
        let strength = strength.clone();
        let quality = quality.clone();
        let features = features.clone();
        let notes = notes.clone();
        let reports = reports.clone();
        let status = status.clone();

        Callback::from(move |_| {
            let mut updated_reports = (*reports).clone();

            let timestamp = js_sys::Date::new_0()
                .to_locale_string(
                    "en-US",
                    &wasm_bindgen::JsValue::UNDEFINED,
                )
                .as_string()
                .unwrap_or_default();

            updated_reports.insert(
                0,
                Report {
                    frequency: (*frequency).clone(),
                    station: (*station).clone(),
                    latitude: current_latitude,
                    longitude: current_longitude,
                    strength: *strength,
                    quality: (*quality).clone(),
                    features: (*features).clone(),
                    notes: (*notes).clone(),
                    timestamp,
                },
            );

            save_reports(&updated_reports);
            reports.set(updated_reports);

            status.set(
                "Observation secured in this iPhone"
                    .to_string(),
            );

            notes.set(String::new());
        })
    };

    let clear = {
        let reports = reports.clone();
        let status = status.clone();

        Callback::from(move |_| {
            save_reports(&[]);
            reports.set(Vec::new());
            status.set("Field log cleared".to_string());
        })
    };

    let report_cards: Html = reports
        .iter()
        .take(8)
        .map(|report| {
            html! {
                <div class="report">
                    <div class="report-head">
                        <strong>
                            {
                                format!(
                                    "{} • {}",
                                    report.frequency,
                                    report.station
                                )
                            }
                        </strong>

                        <small>
                            {format!("{}%", report.strength)}
                        </small>
                    </div>

                    <p>
                        {
                            format!(
                                "{} • {:.5}, {:.5}",
                                report.quality,
                                report.latitude,
                                report.longitude
                            )
                        }
                    </p>

                    {
                        if report.notes.is_empty() {
                            Html::default()
                        } else {
                            html! {
                                <p>{&report.notes}</p>
                            }
                        }
                    }

                    <small>{&report.timestamp}</small>
                </div>
            }
        })
        .collect();

    html! {
        <main class="app">
            <header class="topbar">
                <div>
                    <p class="eyebrow">
                        {"MIKEGYVER STUDIO FIELD SYSTEM"}
                    </p>

                    <h1>{"Signal Sleuth"}</h1>

                    <p class="subtitle">
                        {"Tomball-area radio intelligence console"}
                    </p>
                </div>

                <span class="wasm">
                    {"RUST • WASM"}
                </span>
            </header>

            <div class="status">
                <i class="pulse"></i>
                {&*status}
            </div>

            <section class="grid">
                <article class="panel form">
                    <h2>{"New observation"}</h2>

                    <p class="hint">
                        {"Log what your radio and your eyes detect."}
                    </p>

                    <div class="form-grid">
                        <label>
                            {"Frequency"}

                            <input
                                value={(*frequency).clone()}
                                oninput={{
                                    let state = frequency.clone();

                                    Callback::from(
                                        move |event: InputEvent| {
                                            let input =
                                                event.target_unchecked_into::<
                                                    HtmlInputElement
                                                >();

                                            state.set(input.value());
                                        },
                                    )
                                }}
                            />
                        </label>

                        <label>
                            {"Station / label"}

                            <input
                                value={(*station).clone()}
                                oninput={{
                                    let state = station.clone();

                                    Callback::from(
                                        move |event: InputEvent| {
                                            let input =
                                                event.target_unchecked_into::<
                                                    HtmlInputElement
                                                >();

                                            state.set(input.value());
                                        },
                                    )
                                }}
                            />
                        </label>

                        <label>
                            {"Latitude"}

                            <input
                                inputmode="decimal"
                                value={(*latitude).clone()}
                                oninput={{
                                    let state = latitude.clone();

                                    Callback::from(
                                        move |event: InputEvent| {
                                            let input =
                                                event.target_unchecked_into::<
                                                    HtmlInputElement
                                                >();

                                            state.set(input.value());
                                        },
                                    )
                                }}
                            />
                        </label>

                        <label>
                            {"Longitude"}

                            <input
                                inputmode="decimal"
                                value={(*longitude).clone()}
                                oninput={{
                                    let state = longitude.clone();

                                    Callback::from(
                                        move |event: InputEvent| {
                                            let input =
                                                event.target_unchecked_into::<
                                                    HtmlInputElement
                                                >();

                                            state.set(input.value());
                                        },
                                    )
                                }}
                            />
                        </label>

                        <label class="span-2">
                            {"Signal strength"}

                            <div class="range-row">
                                <input
                                    type="range"
                                    min="0"
                                    max="100"
                                    value={strength.to_string()}
                                    oninput={{
                                        let state = strength.clone();

                                        Callback::from(
                                            move |event: InputEvent| {
                                                let input =
                                                    event.target_unchecked_into::<
                                                        HtmlInputElement
                                                    >();

                                                state.set(
                                                    input
                                                        .value()
                                                        .parse()
                                                        .unwrap_or(0),
                                                );
                                            },
                                        )
                                    }}
                                />

                                <span class="meter-value">
                                    {format!("{}%", *strength)}
                                </span>
                            </div>
                        </label>

                        <label>
                            {"Audio quality"}

                            <select
                                onchange={{
                                    let state = quality.clone();

                                    Callback::from(
                                        move |event: Event| {
                                            let input =
                                                event.target_unchecked_into::<
                                                    HtmlSelectElement
                                                >();

                                            state.set(input.value());
                                        },
                                    )
                                }}
                            >
                                <option>{"Clear"}</option>
                                <option>{"Noisy"}</option>
                                <option>{"Fading"}</option>
                                <option>{"Interference"}</option>
                            </select>
                        </label>

                        <label>
                            {"Tower features"}

                            <input
                                placeholder="Dishes, panels, lights…"
                                value={(*features).clone()}
                                oninput={{
                                    let state = features.clone();

                                    Callback::from(
                                        move |event: InputEvent| {
                                            let input =
                                                event.target_unchecked_into::<
                                                    HtmlInputElement
                                                >();

                                            state.set(input.value());
                                        },
                                    )
                                }}
                            />
                        </label>

                        <label class="span-2">
                            {"Field notes"}

                            <textarea
                                placeholder="What did you hear or observe?"
                                value={(*notes).clone()}
                                oninput={{
                                    let state = notes.clone();

                                    Callback::from(
                                        move |event: InputEvent| {
                                            let input =
                                                event.target_unchecked_into::<
                                                    HtmlTextAreaElement
                                                >();

                                            state.set(input.value());
                                        },
                                    )
                                }}
                            />
                        </label>
                    </div>

                    <div class="actions">
                        <button onclick={locate}>
                            {"⌖ Use My Location"}
                        </button>

                        <button onclick={clear}>
                            {"Clear Log"}
                        </button>

                        <button
                            class="primary"
                            onclick={record}
                        >
                            {"Record Observation"}
                        </button>
                    </div>
                </article>

                <article class="panel">
                    <h2>{"Live analysis"}</h2>

                    <p class="hint">
                        {"Calculated locally by Rust."}
                    </p>

                    <div class="analysis">
                        <div class="metric">
                            <span>{"From studio"}</span>
                            <strong>
                                {
                                    format!(
                                        "{:.2} mi",
                                        studio_distance
                                    )
                                }
                            </strong>
                        </div>

                        <div class="metric">
                            <span>{"From KRLY"}</span>
                            <strong>
                                {
                                    format!(
                                        "{:.2} mi",
                                        krly_distance
                                    )
                                }
                            </strong>
                        </div>

                        <div class="metric">
                            <span>{"Bearing"}</span>
                            <strong>
                                {
                                    format!(
                                        "{:.0}° {}",
                                        current_bearing,
                                        compass(current_bearing)
                                    )
                                }
                            </strong>
                        </div>

                        <div class="metric">
                            <span>{"Signal"}</span>
                            <strong>
                                {format!("{} / 100", *strength)}
                            </strong>
                        </div>

                        <div class="metric wide">
                            <span>{"Assessment"}</span>

                            <strong>
                                {
                                    if *strength > 79 {
                                        "Strong local candidate"
                                    } else if *strength > 49 {
                                        "Usable investigation lead"
                                    } else {
                                        "Weak or distant signal"
                                    }
                                }
                            </strong>
                        </div>
                    </div>
                </article>

                <article class="panel skynet">
                    <h2>{"SKYNET Analysis Mode"}</h2>

                    <div class="terminal">
                        {
                            format!(
                                "> OBSERVATIONS: {:02}\n\
                                 > NODE BEARING: {:.0}° {}\n\
                                 > SIGNAL CONFIDENCE: {}%\n\
                                 > HUMAN OBSERVER: DETECTED",
                                reports.len(),
                                current_bearing,
                                compass(current_bearing),
                                *strength
                            )
                        }
                    </div>
                </article>

                <article class="panel">
                    <h2>{"Field log"}</h2>

                    <p class="hint">
                        {
                            format!(
                                "{} observations stored on this device",
                                reports.len()
                            )
                        }
                    </p>

                    <div class="reports">
                        {
                            if reports.is_empty() {
                                html! {
                                    <div class="empty">
                                        {"No signals captured yet."}
                                    </div>
                                }
                            } else {
                                report_cards
                            }
                        }
                    </div>
                </article>
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}