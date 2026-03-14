use gloo::timers::callback::{Interval, Timeout};
use js_sys::Math;
use web_sys::{window, HtmlInputElement};
use yew::prelude::*;

const PI_DIGITS_100: &str =
    "3.14159265358979323846264338327950288419716939937510\
     58209749445923078164062862089986280348253421170679";

const BEST_SCORE_KEY: &str = "orbit_the_pie_best_score";

#[derive(Clone, PartialEq)]
struct Mission {
    title: &'static str,
    prompt: &'static str,
    target_type: MissionTarget,
    target_value: f64,
    tolerance: f64,
}

#[derive(Clone, PartialEq)]
enum MissionTarget {
    Radius,
    Circumference,
    Area,
}

fn format_num(value: f64) -> String {
    format!("{value:.2}")
}

fn mission_label(target: &MissionTarget) -> &'static str {
    match target {
        MissionTarget::Radius => "Radius",
        MissionTarget::Circumference => "Circumference",
        MissionTarget::Area => "Area",
    }
}

fn mission_actual_value(target: &MissionTarget, radius: f64) -> f64 {
    match target {
        MissionTarget::Radius => radius,
        MissionTarget::Circumference => 2.0 * std::f64::consts::PI * radius,
        MissionTarget::Area => std::f64::consts::PI * radius * radius,
    }
}

fn missions() -> Vec<Mission> {
    vec![
        Mission {
            title: "Mission 1",
            prompt: "Set the radius so the circumference is about 314.16.",
            target_type: MissionTarget::Circumference,
            target_value: 314.16,
            tolerance: 2.0,
        },
        Mission {
            title: "Mission 2",
            prompt: "Find a radius that makes the area close to 10,000.",
            target_type: MissionTarget::Area,
            target_value: 10_000.0,
            tolerance: 120.0,
        },
        Mission {
            title: "Mission 3",
            prompt: "Set the radius to classic Pi Day mode: 31.4.",
            target_type: MissionTarget::Radius,
            target_value: 31.4,
            tolerance: 0.25,
        },
    ]
}

fn load_best_score() -> u32 {
    window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(BEST_SCORE_KEY).ok().flatten())
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(0)
}

fn save_best_score(score: u32) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(BEST_SCORE_KEY, &score.to_string());
    }
}

#[function_component(App)]
fn app() -> Html {
    let radius = use_state(|| 94.0_f64);
    let speed = use_state(|| 2.2_f64);
    let angle = use_state(|| 0.0_f64);
    let is_running = use_state(|| true);
    let show_labels = use_state(|| true);
    let show_astronaut = use_state(|| false);
    let digits_count = use_state(|| 50_usize);

    let mission_list = use_memo((), |_| missions());
    let mission_index = use_state(|| 0_usize);
    let mission_message = use_state(|| {
        "Mission ready. Adjust the orbit and use the current radius to test your answer.".to_string()
    });
    let mission_score = use_state(|| 0_u32);
    let best_score = use_state(load_best_score);
    let mission_completed = use_state(|| false);
    let confetti_active = use_state(|| false);
    let confetti_burst = use_state(|| 0_u32);

    {
        let angle = angle.clone();
        let is_running = is_running.clone();
        let speed = speed.clone();

        use_effect_with((is_running.clone(), speed.clone()), move |_| {
            let interval = if *is_running {
                let angle = angle.clone();
                let speed_value = *speed;

                Some(Interval::new(30, move || {
                    let next = (*angle + speed_value) % 360.0;
                    angle.set(next);
                }))
            } else {
                None
            };

            move || drop(interval)
        });
    }

    let on_radius_input = {
        let radius = radius.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<f64>() {
                radius.set(value);
            }
        })
    };

    let on_speed_input = {
        let speed = speed.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(value) = input.value().parse::<f64>() {
                speed.set(value);
            }
        })
    };

    let set_digits_25 = {
        let digits_count = digits_count.clone();
        Callback::from(move |_| digits_count.set(25))
    };

    let set_digits_50 = {
        let digits_count = digits_count.clone();
        Callback::from(move |_| digits_count.set(50))
    };

    let set_digits_100 = {
        let digits_count = digits_count.clone();
        Callback::from(move |_| digits_count.set(100))
    };

    let toggle_running = {
        let is_running = is_running.clone();
        Callback::from(move |_| is_running.set(!*is_running))
    };

    let reset_orbit = {
        let angle = angle.clone();
        Callback::from(move |_| angle.set(0.0))
    };

    let set_satellite = {
        let show_astronaut = show_astronaut.clone();
        Callback::from(move |_| show_astronaut.set(false))
    };

    let set_astronaut = {
        let show_astronaut = show_astronaut.clone();
        Callback::from(move |_| show_astronaut.set(true))
    };

    let toggle_labels = {
        let show_labels = show_labels.clone();
        Callback::from(move |_| show_labels.set(!*show_labels))
    };

    let set_radius_50 = {
        let radius = radius.clone();
        Callback::from(move |_| radius.set(50.0))
    };

    let set_radius_75 = {
        let radius = radius.clone();
        Callback::from(move |_| radius.set(75.0))
    };

    let set_radius_100 = {
        let radius = radius.clone();
        Callback::from(move |_| radius.set(100.0))
    };

    let pi_day_preset = {
        let radius = radius.clone();
        let speed = speed.clone();
        let mission_message = mission_message.clone();
        Callback::from(move |_| {
            radius.set(31.4);
            speed.set(3.14);
            mission_message.set("Pi Day preset activated: radius = 31.4 and speed = 3.14.".to_string());
        })
    };

    let reset_scores = {
        let mission_score = mission_score.clone();
        let best_score = best_score.clone();
        let mission_completed = mission_completed.clone();
        let mission_message = mission_message.clone();
        Callback::from(move |_| {
            mission_score.set(0);
            best_score.set(0);
            mission_completed.set(false);
            save_best_score(0);
            mission_message.set("Scores reset. Ready for a fresh Pi mission run.".to_string());
        })
    };

    let current_mission = mission_list[*mission_index].clone();

    let check_mission = {
        let radius = radius.clone();
        let mission_message = mission_message.clone();
        let mission_score = mission_score.clone();
        let best_score = best_score.clone();
        let mission_completed = mission_completed.clone();
        let current_mission = current_mission.clone();
        let confetti_active = confetti_active.clone();
        let confetti_burst = confetti_burst.clone();

        Callback::from(move |_| {
            let actual = mission_actual_value(&current_mission.target_type, *radius);
            let diff = (actual - current_mission.target_value).abs();

            if diff <= current_mission.tolerance {
                if !*mission_completed {
                    let new_score = *mission_score + 1;
                    mission_score.set(new_score);

                    if new_score > *best_score {
                        best_score.set(new_score);
                        save_best_score(new_score);
                    }

                    confetti_active.set(true);
                    confetti_burst.set(*confetti_burst + 1);

                    {
                        let confetti_active = confetti_active.clone();
                        Timeout::new(1400, move || {
                            confetti_active.set(false);
                        })
                        .forget();
                    }
                }

                mission_completed.set(true);
                mission_message.set(format!(
                    "Success! {} target {:.2} reached with {:.2}.",
                    mission_label(&current_mission.target_type),
                    current_mission.target_value,
                    actual
                ));
            } else {
                mission_completed.set(false);
                mission_message.set(format!(
                    "Not quite. Current {} is {:.2}. Target is {:.2} (within ±{:.2}).",
                    mission_label(&current_mission.target_type).to_lowercase(),
                    actual,
                    current_mission.target_value,
                    current_mission.tolerance
                ));
            }
        })
    };

    let next_mission = {
        let mission_index = mission_index.clone();
        let mission_message = mission_message.clone();
        let mission_completed = mission_completed.clone();
        let mission_list = mission_list.clone();

        Callback::from(move |_| {
            let next = (*mission_index + 1) % mission_list.len();
            mission_index.set(next);
            mission_completed.set(false);
            mission_message.set(format!(
                "{} ready. {}",
                mission_list[next].title, mission_list[next].prompt
            ));
        })
    };

    let orbit_size = *radius * 2.0;
    let stage_size = 100.0_f64;
    let orbiter_size = 12.0_f64;

    let angle_rad = (*angle).to_radians();
    let x = 50.0 + (*radius / 2.1) * angle_rad.cos();
    let y = 50.0 + (*radius / 2.1) * angle_rad.sin();

    let orbiter_style = format!(
        "left: calc({x:.3}% - {orbiter_size:.3}% / 2); \
         top: calc({y:.3}% - {orbiter_size:.3}% / 2);"
    );

    let orbit_ring_percent = (orbit_size / 2.1).min(stage_size - 8.0);
    let orbit_ring_style = format!(
        "width: {:.3}%; height: {:.3}%;",
        orbit_ring_percent, orbit_ring_percent
    );

    let circumference = 2.0 * std::f64::consts::PI * *radius;
    let diameter = *radius * 2.0;
    let area = std::f64::consts::PI * *radius * *radius;

    let emoji = if *show_astronaut { "👨🏽‍🚀" } else { "🛰️" };

    let star_positions = vec![
        (8.0, 14.0),
        (14.0, 72.0),
        (20.0, 26.0),
        (27.0, 84.0),
        (33.0, 12.0),
        (41.0, 65.0),
        (48.0, 18.0),
        (57.0, 82.0),
        (63.0, 10.0),
        (72.0, 24.0),
        (79.0, 68.0),
        (85.0, 36.0),
        (90.0, 18.0),
        (92.0, 78.0),
    ];

    let facts = [
        "Pi is the ratio of a circle’s circumference to its diameter.",
        "Circular motion makes Pi Day a natural bridge between math and space.",
        "March 14 is celebrated as Pi Day because the first digits are 3.14.",
    ];

    let fact_index = (((*angle / 120.0).floor() as usize) + (*digits_count / 25) - 1) % facts.len();
    let digits_to_show = (*digits_count).min(PI_DIGITS_100.len());
    let digits_display = &PI_DIGITS_100[..digits_to_show];

    let confetti_particles: Vec<(f64, f64, f64, f64)> = (0..28)
        .map(|i| {
            let left = 8.0 + Math::random() * 84.0;
            let delay = Math::random() * 0.45;
            let drift = -70.0 + Math::random() * 140.0;
            let duration = 0.9 + Math::random() * 0.8 + (i as f64 * 0.005);
            (left, delay, drift, duration)
        })
        .collect();

    html! {
        <div class="app-shell">
            <section class="hero">
                <div class="kicker">{ "MIKEGYVER STUDIO • PI DAY LAUNCH" }</div>
                <h1>{ "Orbit the Pie: Pi Day Space Simulator v3" }</h1>
                <p>
                    { "Adjust the orbit radius, explore live circle math, complete Pi missions, save your best score, and celebrate success with confetti." }
                </p>
            </section>

            <section class="grid">
                <aside class="card panel">
                    <h2>{ "Mission Controls" }</h2>

                    <div class="control-group">
                        <div class="control-label">
                            <span>{ "Orbit Radius" }</span>
                            <span class="control-value">{ format!("{} px", format_num(*radius)) }</span>
                        </div>
                        <input
                            type="range"
                            min="20"
                            max="160"
                            step="0.1"
                            value={radius.to_string()}
                            oninput={on_radius_input}
                        />
                    </div>

                    <div class="control-group">
                        <div class="control-label">
                            <span>{ "Orbit Speed" }</span>
                            <span class="control-value">{ format_num(*speed) }</span>
                        </div>
                        <input
                            type="range"
                            min="0.4"
                            max="6.0"
                            step="0.1"
                            value={speed.to_string()}
                            oninput={on_speed_input}
                        />
                    </div>

                    <h3>{ "Quick Radius Presets" }</h3>
                    <div class="toggle-row">
                        <button class="secondary" onclick={set_radius_50}>{ "50" }</button>
                        <button class="secondary" onclick={set_radius_75}>{ "75" }</button>
                        <button class="secondary" onclick={set_radius_100}>{ "100" }</button>
                        <button class="secondary" onclick={pi_day_preset}>{ "Pi Day" }</button>
                    </div>

                    <h3>{ "Orbiter Type" }</h3>
                    <div class="toggle-row">
                        <button
                            class={classes!("secondary", (!*show_astronaut).then_some("active"))}
                            onclick={set_satellite}
                        >
                            { "🛰️ Satellite" }
                        </button>
                        <button
                            class={classes!("secondary", (*show_astronaut).then_some("active"))}
                            onclick={set_astronaut}
                        >
                            { "👨🏽‍🚀 Astronaut" }
                        </button>
                    </div>

                    <h3>{ "View Options" }</h3>
                    <div class="toggle-row">
                        <button
                            class={classes!("secondary", (*show_labels).then_some("active"))}
                            onclick={toggle_labels}
                        >
                            { if *show_labels { "Hide Labels" } else { "Show Labels" } }
                        </button>
                    </div>

                    <h3>{ "Flight Actions" }</h3>
                    <div class="button-row">
                        <button class="primary" onclick={toggle_running}>
                            { if *is_running { "Pause Orbit" } else { "Launch Orbit" } }
                        </button>
                        <button class="secondary" onclick={reset_orbit}>
                            { "Reset Orbit" }
                        </button>
                    </div>

                    <h3>{ "Mission Mode" }</h3>
                    <div class="mission-card">
                        <div class="mission-head">
                            <strong>{ current_mission.title }</strong>
                            <span>{ format!("Score: {}", *mission_score) }</span>
                        </div>
                        <div class="score-row">
                            <span>{ format!("Best: {}", *best_score) }</span>
                            <button class="secondary small-btn" onclick={reset_scores}>{ "Reset Scores" }</button>
                        </div>
                        <p class="mission-prompt">{ current_mission.prompt }</p>
                        <div class="button-row">
                            <button class="primary" onclick={check_mission}>{ "Use Current Radius" }</button>
                            <button class="secondary" onclick={next_mission}>{ "Next Mission" }</button>
                        </div>
                        <div class={classes!("mission-status", (*mission_completed).then_some("success"))}>
                            { (*mission_message).clone() }
                        </div>
                    </div>
                </aside>

                <main class="card stage-card">
                    <div class="stage-title">
                        <div>
                            <h2>{ "Orbit Visualization" }</h2>
                            <div class="stage-sub">
                                { "Mission: keep a clean circular path around the Pie Planet." }
                            </div>
                        </div>
                        <div class="stage-sub">
                            { format!("Angle: {}°", format_num(*angle)) }
                        </div>
                    </div>

                    <div class="stage">
                        <div class="pi-badge">{ "π = 3.14159…" }</div>

                        {
                            if *confetti_active {
                                html! {
                                    <div class="confetti-layer" key={format!("burst-{}", *confetti_burst)}>
                                        {
                                            for confetti_particles.iter().enumerate().map(|(i, (left, delay, drift, duration))| {
                                                let hue = (i * 37) % 360;
                                                html! {
                                                    <span
                                                        class="confetti-piece"
                                                        style={format!(
                                                            "left:{left:.2}%; animation-delay:{delay:.2}s; --drift:{drift:.2}px; animation-duration:{duration:.2}s; background:hsl({hue} 95% 65%);"
                                                        )}
                                                    />
                                                }
                                            })
                                        }
                                    </div>
                                }
                            } else {
                                Html::default()
                            }
                        }

                        { for star_positions.iter().map(|(left, top)| {
                            let twinkle = 0.55 + Math::random() * 0.45;
                            html! {
                                <div
                                    class="star"
                                    style={format!("left:{left}%; top:{top}%; opacity:{twinkle:.2};")}
                                />
                            }
                        })}

                        <div class="orbit-ring" style={orbit_ring_style}></div>

                        <div class="pie-planet">
                            <div class="pie-slice" style="transform: translate(-50%, -100%) rotate(0deg);"></div>
                            <div class="pie-slice" style="transform: translate(-50%, -100%) rotate(72deg);"></div>
                            <div class="pie-slice" style="transform: translate(-50%, -100%) rotate(144deg);"></div>
                            <div class="pie-slice" style="transform: translate(-50%, -100%) rotate(216deg);"></div>
                            <div class="pie-slice" style="transform: translate(-50%, -100%) rotate(288deg);"></div>
                        </div>

                        <div class="orbiter" style={orbiter_style}>
                            { emoji }
                        </div>

                        {
                            if *show_labels {
                                html! {
                                    <div class="orbit-label">
                                        { format!("Radius: {} • Circumference: {}", format_num(*radius), format_num(circumference)) }
                                    </div>
                                }
                            } else {
                                Html::default()
                            }
                        }
                    </div>

                    <div class="footer-note">
                        <strong>{ "Pi Day STEM note:" }</strong>
                        { " In a circle, circumference and area both depend on π — and circular motion makes that math feel alive." }
                    </div>
                </main>

                <aside class="card panel">
                    <h2>{ "Live Pi Math" }</h2>

                    <div class="math-grid">
                        <div class="math-item">
                            <span class="math-label">{ "Radius (r)" }</span>
                            <div class="math-value">{ format_num(*radius) }</div>
                        </div>

                        <div class="math-item">
                            <span class="math-label">{ "Diameter (2r)" }</span>
                            <div class="math-value">{ format_num(diameter) }</div>
                        </div>

                        <div class="math-item">
                            <span class="math-label">{ "Circumference (2πr)" }</span>
                            <div class="math-value">{ format_num(circumference) }</div>
                        </div>

                        <div class="math-item">
                            <span class="math-label">{ "Area (πr²)" }</span>
                            <div class="math-value">{ format_num(area) }</div>
                        </div>
                    </div>

                    <h3>{ "Pi Digits" }</h3>
                    <div class="toggle-row">
                        <button
                            class={classes!("secondary", (*digits_count == 25).then_some("active"))}
                            onclick={set_digits_25}
                        >
                            { "25" }
                        </button>
                        <button
                            class={classes!("secondary", (*digits_count == 50).then_some("active"))}
                            onclick={set_digits_50}
                        >
                            { "50" }
                        </button>
                        <button
                            class={classes!("secondary", (*digits_count == 100).then_some("active"))}
                            onclick={set_digits_100}
                        >
                            { "100" }
                        </button>
                    </div>

                    <div class="digits-box">
                        <code>{ digits_display }</code>
                    </div>

                    <h3>{ "Did You Know?" }</h3>
                    <div class="fact-box">
                        <p>{ facts[fact_index] }</p>
                    </div>
                </aside>
            </section>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}