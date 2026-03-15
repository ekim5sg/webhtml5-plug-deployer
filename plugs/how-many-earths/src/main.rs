use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PlanetKey {
    Jupiter,
    Saturn,
    Uranus,
    Neptune,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CompareMode {
    Diameter,
    Volume,
}

#[derive(Clone, Copy, PartialEq)]
struct Planet {
    key: PlanetKey,
    name: &'static str,
    earths_fit: u32,
    diameter_earths: f64,
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
        diameter_earths: 11.2,
        color: "#c99062",
        accent: "#f5c49b",
        blurb: "The largest planet in the solar system.",
        fact: "Jupiter is about 11 Earths wide, but about 1,321 Earths fit inside by volume.",
    },
    Planet {
        key: PlanetKey::Saturn,
        name: "Saturn",
        earths_fit: 764,
        diameter_earths: 9.45,
        color: "#d9b36f",
        accent: "#f7dfb1",
        blurb: "Famous for its spectacular ring system.",
        fact: "Saturn is about 9.5 Earths wide, but its volume is large enough for about 764 Earths.",
    },
    Planet {
        key: PlanetKey::Uranus,
        name: "Uranus",
        earths_fit: 63,
        diameter_earths: 4.0,
        color: "#6fd0d8",
        accent: "#c8fbff",
        blurb: "An ice giant that rotates on its side.",
        fact: "Uranus is about 4 Earths wide, yet it can hold about 63 Earths by volume.",
    },
    Planet {
        key: PlanetKey::Neptune,
        name: "Neptune",
        earths_fit: 58,
        diameter_earths: 3.9,
        color: "#4a7dff",
        accent: "#a9beff",
        blurb: "A deep blue ice giant with supersonic winds.",
        fact: "Neptune is only about 3.9 Earths wide, but its total volume is still about 58 Earths.",
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
    let compare_mode = use_state(|| CompareMode::Volume);
    let show_answer = use_state(|| false);

    let planet = planet_by_key(*selected);

    let current_volume_count =
        ((planet.earths_fit as f64) * (*fill_percent as f64 / 100.0)).round() as u32;
    let current_volume_count = current_volume_count.min(planet.earths_fit);

    let current_diameter_value =
        planet.diameter_earths * (*fill_percent as f64 / 100.0);
    let current_diameter_value = current_diameter_value.min(planet.diameter_earths);

    let selected_for_jupiter = selected.clone();
    let fill_for_jupiter = fill_percent.clone();
    let show_answer_jupiter = show_answer.clone();

    let selected_for_saturn = selected.clone();
    let fill_for_saturn = fill_percent.clone();
    let show_answer_saturn = show_answer.clone();

    let selected_for_uranus = selected.clone();
    let fill_for_uranus = fill_percent.clone();
    let show_answer_uranus = show_answer.clone();

    let selected_for_neptune = selected.clone();
    let fill_for_neptune = fill_percent.clone();
    let show_answer_neptune = show_answer.clone();

    let fill_for_slider = fill_percent.clone();

    let mode_for_diameter = compare_mode.clone();
    let mode_for_volume = compare_mode.clone();

    let reset_fill = fill_percent.clone();
    let reset_answer = show_answer.clone();

    let reveal_answer = show_answer.clone();
    let hide_answer = show_answer.clone();

    let metric_label = match *compare_mode {
        CompareMode::Diameter => format!("{:.1}", current_diameter_value),
        CompareMode::Volume => format_number(current_volume_count),
    };

    let max_label = match *compare_mode {
        CompareMode::Diameter => format!("{:.1} Earths across", planet.diameter_earths),
        CompareMode::Volume => format!("{} Earths by volume", format_number(planet.earths_fit)),
    };

    let headline_label = match *compare_mode {
        CompareMode::Diameter => format!("Earths across {}", planet.name),
        CompareMode::Volume => format!("Earths inside {}", planet.name),
    };

    let explainer_text = match *compare_mode {
        CompareMode::Diameter => format!(
            "{} is about {:.1} Earths wide. Diameter compares how wide something is from one side to the other.",
            planet.name, planet.diameter_earths
        ),
        CompareMode::Volume => format!(
            "{} could fit about {} Earths by volume. Volume compares the total 3D space inside something.",
            planet.name,
            format_number(planet.earths_fit)
        ),
    };

    let current_mode_text = match *compare_mode {
        CompareMode::Diameter => "Diameter",
        CompareMode::Volume => "Volume",
    };

    html! {
        <div class="app-shell">
            <section class="hero">
                <div class="eyebrow">{"MikeGyver Studio • STEM Build"}</div>
                <h1>{"How Many Earths?"}</h1>
                <p class="subtitle">
                    {"Switch between diameter and volume to see why a planet can be only a few Earths wide, yet still hold dozens, hundreds, or even more than a thousand Earths by total space."}
                </p>
            </section>

            <section class="grid">
                <div class="card">
                    <div class="card-inner">
                        <h2 class="section-title">{"Choose a giant planet"}</h2>

                        <div class="planet-tabs">
                            <button
                                class={classes!("planet-btn", if *selected == PlanetKey::Jupiter { "active" } else { "" })}
                                onclick={Callback::from(move |_| {
                                    selected_for_jupiter.set(PlanetKey::Jupiter);
                                    fill_for_jupiter.set(100);
                                    show_answer_jupiter.set(false);
                                })}
                            >
                                {"🟠 Jupiter"}
                            </button>

                            <button
                                class={classes!("planet-btn", if *selected == PlanetKey::Saturn { "active" } else { "" })}
                                onclick={Callback::from(move |_| {
                                    selected_for_saturn.set(PlanetKey::Saturn);
                                    fill_for_saturn.set(100);
                                    show_answer_saturn.set(false);
                                })}
                            >
                                {"🪐 Saturn"}
                            </button>

                            <button
                                class={classes!("planet-btn", if *selected == PlanetKey::Uranus { "active" } else { "" })}
                                onclick={Callback::from(move |_| {
                                    selected_for_uranus.set(PlanetKey::Uranus);
                                    fill_for_uranus.set(100);
                                    show_answer_uranus.set(false);
                                })}
                            >
                                {"🩵 Uranus"}
                            </button>

                            <button
                                class={classes!("planet-btn", if *selected == PlanetKey::Neptune { "active" } else { "" })}
                                onclick={Callback::from(move |_| {
                                    selected_for_neptune.set(PlanetKey::Neptune);
                                    fill_for_neptune.set(100);
                                    show_answer_neptune.set(false);
                                })}
                            >
                                {"🔵 Neptune"}
                            </button>
                        </div>

                        <h2 class="section-title">{"Compare by"}</h2>
                        <div class="mode-toggle">
                            <button
                                class={classes!("mode-btn", if *compare_mode == CompareMode::Diameter { "active" } else { "" })}
                                onclick={Callback::from(move |_| mode_for_diameter.set(CompareMode::Diameter))}
                            >
                                {"↔️ Diameter"}
                            </button>

                            <button
                                class={classes!("mode-btn", if *compare_mode == CompareMode::Volume { "active" } else { "" })}
                                onclick={Callback::from(move |_| mode_for_volume.set(CompareMode::Volume))}
                            >
                                {"🧊 Volume"}
                            </button>
                        </div>

                        <div class="toolbar">
                            <button
                                class="toolbar-btn"
                                onclick={Callback::from(move |_| {
                                    reset_fill.set(100);
                                    reset_answer.set(false);
                                })}
                            >
                                {"↺ Reset Reveal"}
                            </button>

                            <button
                                class="toolbar-btn secondary"
                                onclick={Callback::from(move |_| {
                                    hide_answer.set(false);
                                })}
                            >
                                {"🙈 Hide Answer"}
                            </button>
                        </div>

                        <div class="stat-strip">
                            <div class="stat-pill">
                                <span class="k">{"Planet"}</span>
                                <span class="v">{planet.name}</span>
                            </div>
                            <div class="stat-pill">
                                <span class="k">{"Mode"}</span>
                                <span class="v">{current_mode_text}</span>
                            </div>
                            <div class="stat-pill">
                                <span class="k">{"Fill"}</span>
                                <span class="v">{format!("{}%", *fill_percent)}</span>
                            </div>
                        </div>

                        <div class="viz-wrap">
                            {
                                match *compare_mode {
                                    CompareMode::Diameter => {
                                        render_diameter_svg(planet, current_diameter_value, *fill_percent)
                                    }
                                    CompareMode::Volume => {
                                        render_volume_svg(planet, current_volume_count, *fill_percent)
                                    }
                                }
                            }

                            <div class="controls">
                                <div class="slider-row">
                                    <span>{"Reveal level"}</span>
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
                        <h2 class="section-title">{"Comparison result"}</h2>

                        <div class="label">{headline_label}</div>
                        <div class="big-number">{metric_label}</div>
                        <div class="label">{max_label}</div>

                        <div class="explainer-box">
                            <strong>{"Why these numbers are so different"}</strong>
                            <div class="label">
                                {"Diameter grows in one dimension. Volume grows in three dimensions. "}
                                {"That means when a planet gets wider, the total space inside it increases much faster."}
                            </div>
                        </div>

                        <div class="compare-grid">
                            <div class="compare-stat">
                                <strong>{"Diameter"}</strong>
                                <div class="label">{format!("{:.1} Earths across", planet.diameter_earths)}</div>
                            </div>

                            <div class="compare-stat">
                                <strong>{"Volume"}</strong>
                                <div class="label">{format!("{} Earths inside", format_number(planet.earths_fit))}</div>
                            </div>
                        </div>

                        {render_formula_card(planet)}
                        {render_cube_ladder(planet)}
                        {render_challenge_card(planet, *show_answer, reveal_answer)}

                        <div class="fact">
                            {explainer_text}
                        </div>

                        <div class="mini-grid">
                            <div class="mini-stat">
                                <strong>{planet.name}</strong>
                                <div class="label">{planet.blurb}</div>
                            </div>

                            <div class="mini-stat">
                                <strong>{"Earth"}</strong>
                                <div class="label">{"Small compared with the giant planets, but uniquely suited for life."}</div>
                            </div>
                        </div>

                        <div class="footer-note">
                            {planet.fact}
                        </div>
                    </div>
                </div>
            </section>

            <div class="source">
                {"Educational visualization based on commonly cited NASA Solar System Exploration comparisons. "}
                {"This app emphasizes learning intuition: width is 1D, but volume grows like width × width × width."}
            </div>
        </div>
    }
}

fn render_formula_card(planet: Planet) -> Html {
    let rounded = planet.diameter_earths.round() as u32;
    let cube_estimate = rounded * rounded * rounded;
    let actual = planet.earths_fit;
    let diff = cube_estimate.abs_diff(actual);

    let note = format!(
        "A quick kid-friendly estimate is to round the diameter to {} and cube it: {} × {} × {} = {}. The actual comparison is about {}, so the estimate is close.",
        rounded,
        rounded,
        rounded,
        rounded,
        format_number(cube_estimate),
        format_number(actual)
    );

    let detail = if diff == 0 {
        "This one matches exactly.".to_string()
    } else {
        format!(
            "That estimate is off by about {} Earths, which is why we call it an approximation.",
            format_number(diff)
        )
    };

    html! {
        <div class="formula-card">
            <strong class="formula-title">{"Quick formula intuition"}</strong>
            <div class="formula-math">
                {format!("{} × {} × {} ≈ {}", rounded, rounded, rounded, format_number(cube_estimate))}
            </div>
            <div class="formula-note">
                {note}
                {" "}
                {detail}
            </div>
        </div>
    }
}

fn render_cube_ladder(planet: Planet) -> Html {
    let rounded = planet.diameter_earths.round() as u32;
    let squared = rounded * rounded;
    let cubed = rounded * rounded * rounded;

    html! {
        <div class="cube-ladder">
            <div class="cube-step">
                <span class="title">{"Step 1: width"}</span>
                <div class="desc">
                    {format!("{} is about {} Earths across.", planet.name, rounded)}
                </div>
            </div>

            <div class="cube-step">
                <span class="title">{"Step 2: width × width"}</span>
                <div class="desc">
                    {format!("{} × {} = {} gives a 2D area-style intuition.", rounded, rounded, squared)}
                </div>
            </div>

            <div class="cube-step">
                <span class="title">{"Step 3: width × width × width"}</span>
                <div class="desc">
                    {format!("{} × {} × {} = {} gives the 3D volume-style intuition.", rounded, rounded, rounded, format_number(cubed))}
                </div>
            </div>
        </div>
    }
}

fn render_challenge_card(
    planet: Planet,
    show_answer: bool,
    reveal_answer: UseStateHandle<bool>,
) -> Html {
    let question = format!(
        "Kid Challenge: {} is about {:.1} Earths across. Why is the volume comparison so much bigger than {:.1}?",
        planet.name, planet.diameter_earths, planet.diameter_earths
    );

    html! {
        <div class="challenge-card">
            <strong class="challenge-title">{"🚀 Try this question"}</strong>
            <div class="challenge-question">{question}</div>

            <div class="challenge-actions">
                <button
                    class="challenge-btn"
                    onclick={Callback::from(move |_| reveal_answer.set(true))}
                >
                    {"Show answer"}
                </button>
            </div>

            {
                if show_answer {
                    html! {
                        <div class="challenge-answer">
                            {"Because diameter is just one line across, but volume measures 3D space. "}
                            {"When a planet gets wider, the inside grows in three directions, so the number of Earths it could hold rises much faster."}
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

fn render_volume_svg(planet: Planet, current_count: u32, fill_percent: u32) -> Html {
    let cx = 300.0;
    let cy = 300.0;
    let big_r = 220.0;

    let display_earths = earth_bubbles_for_percent(fill_percent);
    let positions = bubble_positions();
    let count_text = format!("{} / {}", format_number(current_count), format_number(planet.earths_fit));

    html! {
        <svg class="svg-box" viewBox="0 0 600 600" role="img" aria-label="Planet volume comparison visualization">
            <defs>
                <radialGradient id="planetGradVol" cx="35%" cy="30%" r="70%">
                    <stop offset="0%" stop-color={planet.accent}/>
                    <stop offset="100%" stop-color={planet.color}/>
                </radialGradient>

                <radialGradient id="earthGradVol" cx="35%" cy="30%" r="70%">
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
                fill="url(#planetGradVol)"
                opacity="0.95"
                stroke="rgba(255,255,255,0.18)"
                stroke-width="3"
            />

            <circle
                cx="125"
                cy="125"
                r="36"
                fill="url(#earthGradVol)"
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
                                fill="url(#earthGradVol)"
                                opacity="0.88"
                                stroke="rgba(255,255,255,0.18)"
                                stroke-width="1.5"
                            />
                        }
                    })
            }

            <rect
                x="135"
                y="485"
                width="330"
                height="74"
                rx="18"
                fill="rgba(6,10,20,0.70)"
                stroke="rgba(255,255,255,0.10)"
            />
            <text x="300" y="513" text-anchor="middle" fill="#ffffff" font-size="24" font-weight="800">
                {format!("{} • Volume", planet.name)}
            </text>
            <text x="300" y="542" text-anchor="middle" fill="#c8d7f2" font-size="18">
                {count_text}
            </text>
        </svg>
    }
}

fn render_diameter_svg(planet: Planet, current_diameter_value: f64, fill_percent: u32) -> Html {
    let max_display = 12.0;
    let giant_width = (planet.diameter_earths / max_display) * 420.0;
    let giant_width = giant_width.max(120.0);
    let current_width = giant_width * (fill_percent as f64 / 100.0);

    html! {
        <svg class="svg-box" viewBox="0 0 600 600" role="img" aria-label="Planet diameter comparison visualization">
            <defs>
                <radialGradient id="planetGradDia" cx="35%" cy="30%" r="70%">
                    <stop offset="0%" stop-color={planet.accent}/>
                    <stop offset="100%" stop-color={planet.color}/>
                </radialGradient>

                <radialGradient id="earthGradDia" cx="35%" cy="30%" r="70%">
                    <stop offset="0%" stop-color="#8fe0ff"/>
                    <stop offset="100%" stop-color="#2878c8"/>
                </radialGradient>
            </defs>

            <rect x="0" y="0" width="600" height="600" rx="24" fill="transparent" />

            <text x="300" y="58" text-anchor="middle" fill="#eaf2ff" font-size="26" font-weight="800">
                {"Diameter Comparison"}
            </text>
            <text x="300" y="84" text-anchor="middle" fill="#9db0d3" font-size="16">
                {"How many Earths fit across the width?"}
            </text>

            <line
                x1="90"
                y1="320"
                x2="510"
                y2="320"
                stroke="rgba(255,255,255,0.18)"
                stroke-width="4"
                stroke-linecap="round"
            />

            <circle
                cx="120"
                cy="320"
                r="28"
                fill="url(#earthGradDia)"
                stroke="rgba(255,255,255,0.35)"
                stroke-width="2"
            />
            <text x="120" y="370" text-anchor="middle" fill="#dbeeff" font-size="18" font-weight="700">
                {"Earth"}
            </text>

            <rect
                x="150"
                y="270"
                width={current_width.to_string()}
                height="100"
                rx="50"
                fill="url(#planetGradDia)"
                opacity="0.95"
                stroke="rgba(255,255,255,0.18)"
                stroke-width="3"
            />

            {
                if planet.key == PlanetKey::Saturn {
                    html! {
                        <ellipse
                            cx={(150.0 + current_width / 2.0).to_string()}
                            cy="320"
                            rx={(current_width / 2.0 + 36.0).to_string()}
                            ry="38"
                            fill="none"
                            stroke="rgba(247,223,177,0.5)"
                            stroke-width="10"
                        />
                    }
                } else {
                    html! {}
                }
            }

            {
                for (0..planet.diameter_earths.ceil() as usize).map(|i| {
                    let x = 150.0 + 20.0 + (i as f64 * 36.0);
                    html! {
                        <circle
                            cx={x.to_string()}
                            cy="320"
                            r="12"
                            fill="rgba(143,224,255,0.85)"
                            opacity="0.75"
                        />
                    }
                })
            }

            <rect
                x="120"
                y="450"
                width="360"
                height="88"
                rx="18"
                fill="rgba(6,10,20,0.70)"
                stroke="rgba(255,255,255,0.10)"
            />
            <text x="300" y="485" text-anchor="middle" fill="#ffffff" font-size="24" font-weight="800">
                {format!("{} • Diameter", planet.name)}
            </text>
            <text x="300" y="515" text-anchor="middle" fill="#c8d7f2" font-size="18">
                {format!("{:.1} / {:.1} Earths across", current_diameter_value, planet.diameter_earths)}
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