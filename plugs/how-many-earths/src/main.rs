use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlanetKey {
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
}

#[derive(Clone, Copy, PartialEq)]
struct Planet {
    key: PlanetKey,
    name: &'static str,
    earths_fit: u32,
    color: &'static str,
    accent: &'static str,
    blurb: &'static str,
    fact: &'static str,
}

const PLANETS: [Planet; 4] = [
    Planet {
        key: PlanetKey::Jupiter,
        name: "Jupiter",
        earths_fit: 1321,
        color: "#c99062",
        accent: "#f5c49b",
        blurb: "The largest planet in the solar system.",
        fact: "Jupiter is so large that about 1,321 Earths could fit inside its volume.",
    },
    Planet {
        key: PlanetKey::Saturn,
        name: "Saturn",
        earths_fit: 764,
        color: "#d9b36f",
        accent: "#f7dfb1",
        blurb: "Famous for its spectacular ring system.",
        fact: "Saturn could contain roughly 764 Earths by volume.",
    },
    Planet {
        key: PlanetKey::Uranus,
        name: "Uranus",
        earths_fit: 63,
        color: "#6fd0d8",
        accent: "#c8fbff",
        blurb: "An ice giant that rotates on its side.",
        fact: "Uranus is still massive, with room for about 63 Earths.",
    },
    Planet {
        key: PlanetKey::Neptune,
        name: "Neptune",
        earths_fit: 58,
        color: "#4a7dff",
        accent: "#a9beff",
        blurb: "A deep blue ice giant with supersonic winds.",
        fact: "Neptune could fit about 58 Earths inside its total volume.",
    },
];

fn planet_by_key(key: PlanetKey) -> Planet {
    PLANETS
        .iter()
        .copied()
        .find(|p| p.key == key)
        .unwrap_or(PLANETS[0])
}

#[function_component(App)]
fn app() -> Html {
    let selected = use_state(|| PlanetKey::Jupiter);
    let fill_percent = use_state(|| 100u32);

    let planet = planet_by_key(*selected);
    let current_count =
        ((planet.earths_fit as f64) * (*fill_percent as f64 / 100.0)).round() as u32;
    let current_count = current_count.min(planet.earths_fit);

    let selected_for_jupiter = selected.clone();
    let selected_for_saturn = selected.clone();
    let selected_for_uranus = selected.clone();
    let selected_for_neptune = selected.clone();
    let fill_for_slider = fill_percent.clone();

    let earths_text = format_number(current_count);
    let total_text = format_number(planet.earths_fit);

    html! {
        <div class="app-shell">
            <section class="hero">
                <div class="eyebrow">{"MikeGyver Studio • STEM Build"}</div>
                <h1>{"How Many Earths?"}</h1>
                <p class="subtitle">
                    {"Earth may feel enormous to us, but compared with the giant planets, it is surprisingly small. "}
                    {"Use the slider to fill each giant planet with Earths and explore the dramatic volume differences across our solar system."}
                </p>
            </section>

            <section class="grid">
                <div class="card">
                    <div class="card-inner">
                        <h2 class="section-title">{"Choose a giant planet"}</h2>

                        <div class="planet-tabs">
                            <button
                                class={classes!("planet-btn", if *selected == PlanetKey::Jupiter { "active" } else { "" })}
                                onclick={Callback::from(move |_| selected_for_jupiter.set(PlanetKey::Jupiter))}
                            >
                                {"🟠 Jupiter"}
                            </button>

                            <button
                                class={classes!("planet-btn", if *selected == PlanetKey::Saturn { "active" } else { "" })}
                                onclick={Callback::from(move |_| selected_for_saturn.set(PlanetKey::Saturn))}
                            >
                                {"🪐 Saturn"}
                            </button>

                            <button
                                class={classes!("planet-btn", if *selected == PlanetKey::Uranus { "active" } else { "" })}
                                onclick={Callback::from(move |_| selected_for_uranus.set(PlanetKey::Uranus))}
                            >
                                {"🩵 Uranus"}
                            </button>

                            <button
                                class={classes!("planet-btn", if *selected == PlanetKey::Neptune { "active" } else { "" })}
                                onclick={Callback::from(move |_| selected_for_neptune.set(PlanetKey::Neptune))}
                            >
                                {"🔵 Neptune"}
                            </button>
                        </div>

                        <div class="viz-wrap">
                            { render_planet_svg(planet, current_count, *fill_percent) }

                            <div class="controls">
                                <div class="slider-row">
                                    <span>{"Fill level"}</span>
                                    <strong>{format!("{}%", *fill_percent)}</strong>
                                </div>

                                <input
                                    type="range"
                                    min="0"
                                    max="100"
                                    step="1"
                                    value={(*fill_percent).to_string()}
                                    oninput={Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        if let Ok(value) = input.value().parse::<u32>() {
                                            fill_for_slider.set(value.min(100));
                                        }
                                    })}
                                />
                            </div>
                        </div>
                    </div>
                </div>

                <div class="card">
                    <div class="card-inner">
                        <h2 class="section-title">{"Volume comparison"}</h2>

                        <div class="label">{format!("Earths currently shown inside {}", planet.name)}</div>
                        <div class="big-number">{earths_text}</div>
                        <div class="label">{format!("Maximum estimated fit: {} Earths", total_text)}</div>

                        <div class="mini-grid">
                            <div class="mini-stat">
                                <strong>{planet.name}</strong>
                                <div class="label">{planet.blurb}</div>
                            </div>

                            <div class="mini-stat">
                                <strong>{"Earth"}</strong>
                                <div class="label">{"Small compared with the gas and ice giants, but uniquely suited for life."}</div>
                            </div>
                        </div>

                        <div class="fact">
                            {planet.fact}
                        </div>

                        <div class="footer-note">
                            {"This app uses a volume-comparison style explanation: how many Earth-sized worlds could fit inside the selected planet. "}
                            {"It is a powerful way to visualize scale, even though these giant planets are not solid containers."}
                        </div>
                    </div>
                </div>
            </section>

            <div class="source">
                {"Educational visualization based on commonly cited NASA Solar System Exploration volume comparisons."}
            </div>
        </div>
    }
}

fn render_planet_svg(planet: Planet, current_count: u32, fill_percent: u32) -> Html {
    let cx = 300.0;
    let cy = 300.0;
    let big_r = 220.0;

    let display_earths = earth_bubbles_for_percent(fill_percent);
    let positions = bubble_positions();
    let count_text = format!("{} / {}", format_number(current_count), format_number(planet.earths_fit));

    html! {
        <svg class="svg-box" viewBox="0 0 600 600" role="img" aria-label="Planet volume comparison visualization">
            <defs>
                <radialGradient id="planetGrad" cx="35%" cy="30%" r="70%">
                    <stop offset="0%" stop-color={planet.accent}/>
                    <stop offset="100%" stop-color={planet.color}/>
                </radialGradient>

                <radialGradient id="earthGrad" cx="35%" cy="30%" r="70%">
                    <stop offset="0%" stop-color="#8fe0ff"/>
                    <stop offset="100%" stop-color="#2878c8"/>
                </radialGradient>
            </defs>

            <rect x="0" y="0" width="600" height="600" rx="24" fill="transparent" />

            {
                if planet.key == PlanetKey::Saturn {
                    html! {
                        <>
                            <ellipse
                                cx={cx.to_string()}
                                cy={cy.to_string()}
                                rx="285"
                                ry="75"
                                fill="none"
                                stroke="rgba(247,223,177,0.55)"
                                stroke-width="18"
                            />
                            <ellipse
                                cx={cx.to_string()}
                                cy={cy.to_string()}
                                rx="285"
                                ry="75"
                                fill="none"
                                stroke="rgba(255,255,255,0.12)"
                                stroke-width="4"
                            />
                        </>
                    }
                } else {
                    html! {}
                }
            }

            <circle
                cx={cx.to_string()}
                cy={cy.to_string()}
                r={big_r.to_string()}
                fill="url(#planetGrad)"
                opacity="0.95"
                stroke="rgba(255,255,255,0.18)"
                stroke-width="3"
            />

            <circle
                cx="125"
                cy="125"
                r="36"
                fill="url(#earthGrad)"
                stroke="rgba(255,255,255,0.35)"
                stroke-width="2"
            />
            <text x="125" y="176" text-anchor="middle" fill="#dbeeff" font-size="18" font-weight="700">
                {"Earth"}
            </text>

            {
                for positions
                    .into_iter()
                    .enumerate()
                    .filter(|(i, _)| *i < display_earths)
                    .map(|(_, (x, y, r))| {
                        html! {
                            <circle
                                cx={x.to_string()}
                                cy={y.to_string()}
                                r={r.to_string()}
                                fill="url(#earthGrad)"
                                opacity="0.88"
                                stroke="rgba(255,255,255,0.18)"
                                stroke-width="1.5"
                            />
                        }
                    })
            }

            <rect
                x="150"
                y="485"
                width="300"
                height="70"
                rx="18"
                fill="rgba(6,10,20,0.68)"
                stroke="rgba(255,255,255,0.10)"
            />
            <text x="300" y="513" text-anchor="middle" fill="#ffffff" font-size="24" font-weight="800">
                {planet.name}
            </text>
            <text x="300" y="540" text-anchor="middle" fill="#c8d7f2" font-size="18">
                {count_text}
            </text>
        </svg>
    }
}

fn earth_bubbles_for_percent(fill_percent: u32) -> usize {
    let max_bubbles = 24usize;
    ((fill_percent as f64 / 100.0) * max_bubbles as f64).round() as usize
}

fn bubble_positions() -> Vec<(f64, f64, f64)> {
    vec![
        (250.0, 180.0, 24.0),
        (300.0, 170.0, 24.0),
        (350.0, 180.0, 24.0),
        (225.0, 225.0, 23.0),
        (275.0, 225.0, 23.0),
        (325.0, 225.0, 23.0),
        (375.0, 225.0, 23.0),
        (205.0, 275.0, 22.0),
        (255.0, 275.0, 22.0),
        (305.0, 275.0, 22.0),
        (355.0, 275.0, 22.0),
        (405.0, 275.0, 22.0),
        (220.0, 325.0, 22.0),
        (270.0, 325.0, 22.0),
        (320.0, 325.0, 22.0),
        (370.0, 325.0, 22.0),
        (240.0, 375.0, 21.0),
        (290.0, 375.0, 21.0),
        (340.0, 375.0, 21.0),
        (390.0, 375.0, 21.0),
        (255.0, 420.0, 20.0),
        (305.0, 420.0, 20.0),
        (355.0, 420.0, 20.0),
        (205.0, 375.0, 20.0),
    ]
}

fn format_number(n: u32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    for (i, ch) in chars.iter().enumerate() {
        result.push(*ch);
        let remaining = len - i - 1;
        if remaining > 0 && remaining % 3 == 0 {
            result.push(',');
        }
    }

    result
}

fn main() {
    yew::Renderer::<App>::new().render();
}