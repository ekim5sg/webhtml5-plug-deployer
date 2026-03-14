use gloo::timers::callback::Interval;
use js_sys::Math;
use web_sys::HtmlInputElement;
use yew::prelude::*;

const PI_DIGITS_100: &str =
    "3.14159265358979323846264338327950288419716939937510\
     58209749445923078164062862089986280348253421170679";

fn format_num(value: f64) -> String {
    format!("{value:.2}")
}

#[function_component(App)]
fn app() -> Html {
    let radius = use_state(|| 120.0_f64);
    let speed = use_state(|| 2.2_f64);
    let angle = use_state(|| 0.0_f64);
    let is_running = use_state(|| true);
    let show_labels = use_state(|| true);
    let show_astronaut = use_state(|| false);
    let digits_count = use_state(|| 50_usize);

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
        "A perfect circular orbit makes Pi Day a fun bridge between math and space.",
        "The first digits of pi are 3.14159, which is why Pi Day is celebrated on March 14.",
    ];

    let fact_index = (((*angle / 120.0).floor() as usize) + (*digits_count / 25) - 1) % facts.len();
    let digits_to_show = (*digits_count).min(PI_DIGITS_100.len());
    let digits_display = &PI_DIGITS_100[..digits_to_show];

    html! {
        <div class="app-shell">
            <section class="hero">
                <div class="kicker">{ "MIKEGYVER STUDIO • PI DAY LAUNCH" }</div>
                <h1>{ "Orbit the Pie: Pi Day Space Simulator" }</h1>
                <p>
                    { "Adjust the orbit radius, change the speed, and watch how π shapes motion in space. A playful Rust + Yew STEM mini-app built for Pi Day." }
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
                            min="70"
                            max="160"
                            step="1"
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
                            { "Reset" }
                        </button>
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