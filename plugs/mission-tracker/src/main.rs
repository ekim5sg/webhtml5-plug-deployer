use web_sys::{HtmlInputElement, SubmitEvent};
use yew::prelude::*;

fn distance_km(speed: f64, day: u32) -> f64 {
    speed * f64::from(day) * 24.0
}

fn parse_speed(raw: &str) -> Result<f64, &'static str> {
    let speed: f64 = raw.trim().parse().map_err(|_| "Invalid speed. Enter a number.")?;
    if !speed.is_finite() || !(0.0..=1_000_000.0).contains(&speed) {
        return Err("Speed must be finite, between 0 and 1,000,000 km/h.");
    }
    Ok(speed)
}

fn grouped(value: f64) -> String {
    let digits = format!("{:.0}", value.abs());
    let mut output = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            output.push(',');
        }
        output.push(ch);
    }
    output
}

#[function_component(App)]
fn app() -> Html {
    let raw = use_state(|| "24000".to_owned());
    let report = use_state(|| Some(24_000.0_f64));
    let error = use_state(|| None::<String>);
    let dirty = use_state(|| false);

    let oninput = {
        let raw = raw.clone();
        let error = error.clone();
        let dirty = dirty.clone();
        Callback::from(move |event: InputEvent| {
            if let Some(input) = event.target_dyn_into::<HtmlInputElement>() {
                raw.set(input.value());
                error.set(None);
                dirty.set(true);
            }
        })
    };
    let onsubmit = {
        let raw = raw.clone();
        let report = report.clone();
        let error = error.clone();
        let dirty = dirty.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            match parse_speed(raw.as_str()) {
                Ok(speed) => { report.set(Some(speed)); error.set(None); }
                Err(message) => { report.set(None); error.set(Some(message.to_owned())); }
            }
            dirty.set(false);
        })
    };
    let nominal = {
        let raw = raw.clone(); let report = report.clone();
        let error = error.clone(); let dirty = dirty.clone();
        Callback::from(move |_: MouseEvent| {
            raw.set("24000".to_owned()); report.set(Some(24_000.0));
            error.set(None); dirty.set(false);
        })
    };
    let invalid = {
        let raw = raw.clone(); let report = report.clone();
        let error = error.clone(); let dirty = dirty.clone();
        Callback::from(move |_: MouseEvent| {
            raw.set("lost-signal".to_owned()); report.set(None);
            error.set(parse_speed("lost-signal").err().map(str::to_owned));
            dirty.set(false);
        })
    };

    html! {
        <main>
            <header>
                <span class="brand">{"MG / FLIGHT SYSTEMS"}</span>
                <span class="badge">{"RUST + YEW / WASM"}</span>
            </header>
            <section class="hero">
                <p class="eyebrow">{"MISSION TRACKER / EXPERIMENT 001"}</p>
                <h1>{"Five days."}<br/>{"One flight."}<span>{" All Rust."}</span></h1>
                <p class="intro">{"Set your spacecraft’s speed. Let Rust calculate the journey, one day at a time."}</p>
            </section>
            <div class="layout">
                <section class="panel controls" aria-labelledby="flight-title">
                    <p class="eyebrow">{"01 / FLIGHT PARAMETERS"}</p>
                    <h2 id="flight-title">{"Set your course."}</h2>
                    <form {onsubmit}>
                        <label for="speed">{"CONSTANT SPEED"}<span>{"km/h"}</span></label>
                        <input id="speed" type="text" inputmode="decimal"
                            value={(*raw).clone()} {oninput} autocomplete="off"
                            aria-describedby="speed-help" aria-invalid={error.is_some().to_string()}/>
                        <p id="speed-help" class="muted">{"Allowed range: 0–1,000,000 km/h. Scientific notation is supported."}</p>
                        <button class="primary" type="submit">{"Run five-day mission →"}</button>
                    </form>
                    <div class="presets">
                        <button type="button" onclick={nominal}>{"Reset: 24,000"}</button>
                        <button type="button" onclick={invalid}>{"Test invalid input"}</button>
                    </div>
                    <div class="feedback" aria-live="polite">
                        if let Some(message) = &*error {
                            <p class="error" role="alert">{message}<br/>{"Mission rejected safely. Correct the input and run again."}</p>
                        } else if *dirty {
                            <p class="muted">{"Input changed. Run the mission to update the report."}</p>
                        } else {
                            <p class="success">{"Telemetry validated. Mission report ready."}</p>
                        }
                    </div>
                    <div class="formula"><span>{"THE EQUATION"}</span><code>{"distance = speed × days × 24"}</code></div>
                </section>
                <section class="panel output" aria-labelledby="report-title">
                    <p class="eyebrow">{"02 / MISSION DISTANCE REPORT"}</p>
                    <h2 id="report-title">{"Flight telemetry."}</h2>
                    if let Some(speed) = *report {
                        <p class="muted">{format!("Report speed: {} km/h{}", grouped(speed), if *dirty { " · previous run" } else { "" })}</p>
                        <div class="distance"><span>{"DAY 05 / DISTANCE TRAVELED"}</span><strong>{grouped(distance_km(speed, 5))}<small>{" km"}</small></strong></div>
                        <div class="table-wrap"><table>
                            <thead><tr><th scope="col">{"Day"}</th><th scope="col">{"Hours"}</th><th scope="col">{"Distance (km)"}</th></tr></thead>
                            <tbody>{for (1_u32..=5).map(|day| html! {
                                <tr><th scope="row">{format!("{:02}", day)}</th><td>{day * 24}</td><td>{grouped(distance_km(speed, day))}</td></tr>
                            })}</tbody>
                        </table></div>
                        <p class="muted">{"Distances rounded to the nearest kilometer for display."}</p>
                    } else {
                        <div class="empty"><span>{"!"}</span><h3>{"Awaiting valid telemetry"}</h3><p>{"No report was generated. Enter a valid speed to resume the mission."}</p></div>
                    }
                </section>
            </div>
            <section class="explanation"><p class="eyebrow">{"MODEL NOTES"}</p><p>{"A constant-speed distance model: no acceleration, gravity, orbital mechanics, or return leg. The speed validation and all five distance calculations execute in Rust compiled to WebAssembly."}</p></section>
            <footer>{"Michael Givens / MikeGyver Studio"}<span>{"MISSION TRACKER · 1.0"}</span></footer>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}