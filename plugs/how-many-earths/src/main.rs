use web_sys::{HtmlAudioElement, HtmlInputElement};
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

#[derive(Clone, Copy, PartialEq)]
struct QuizQuestion {
    prompt: &'static str,
    options: [&'static str; 4],
    correct_index: usize,
    explanation: &'static str,
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

const QUIZ_QUESTIONS: [QuizQuestion; 3] = [
    QuizQuestion {
        prompt: "Which planet in this app can hold the most Earths by volume?",
        options: ["Saturn", "Neptune", "Jupiter", "Uranus"],
        correct_index: 2,
        explanation: "Jupiter is the largest planet here and can hold about 1,321 Earths by volume.",
    },
    QuizQuestion {
        prompt: "Why is a planet's volume comparison so much bigger than its diameter comparison?",
        options: [
            "Because planets stretch at night",
            "Because volume measures 3D space, not just width",
            "Because only rings count",
            "Because Earth gets smaller in space",
        ],
        correct_index: 1,
        explanation: "Diameter is one line across. Volume measures space in three dimensions, so it grows much faster.",
    },
    QuizQuestion {
        prompt: "Jupiter is about 11 Earths wide. Which quick estimate best matches its volume idea?",
        options: ["11 + 11 + 11", "11 × 2", "11 × 11 × 11", "11 - 1"],
        correct_index: 2,
        explanation: "A quick cube-style intuition is 11 × 11 × 11, which gets close to Jupiter's volume comparison.",
    },
];

fn planet_by_key(key: PlanetKey) -> Planet {
    PLANETS
        .iter()
        .copied()
        .find(|p| p.key == key)
        .unwrap_or(PLANETS[0])
}

fn play_theme(audio_ref: &NodeRef) {
    if let Some(audio) = audio_ref.cast::<HtmlAudioElement>() {
        let _ = audio.pause();
        audio.set_current_time(0.0);
        let _ = audio.play();
    }
}

fn stop_theme(audio_ref: &NodeRef) {
    if let Some(audio) = audio_ref.cast::<HtmlAudioElement>() {
        let _ = audio.pause();
        audio.set_current_time(0.0);
    }
}

#[function_component(App)]
fn app() -> Html {
    let selected = use_state(|| PlanetKey::Jupiter);
    let fill_percent = use_state(|| 100u32);
    let compare_mode = use_state(|| CompareMode::Volume);
    let show_answer = use_state(|| false);

    let quiz_index = use_state(|| 0usize);
    let quiz_score = use_state(|| 0usize);
    let quiz_selected = use_state(|| Option::<usize>::None);
    let quiz_checked = use_state(|| false);
    let quiz_finished = use_state(|| false);
    let audio_unlocked = use_state(|| false);

    let audio_ref = use_node_ref();

    let planet = planet_by_key(*selected);

    let current_volume_count =
        ((planet.earths_fit as f64) * (*fill_percent as f64 / 100.0)).round() as u32;
    let current_volume_count = current_volume_count.min(planet.earths_fit);

    let current_diameter_value =
        planet.diameter_earths * (*fill_percent as f64 / 100.0);
    let current_diameter_value = current_diameter_value.min(planet.diameter_earths);

    let current_question = QUIZ_QUESTIONS[*quiz_index];

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

    let play_audio_handle = audio_ref.clone();
    let stop_audio_handle = audio_ref.clone();

    let restart_quiz_index = quiz_index.clone();
    let restart_quiz_score = quiz_score.clone();
    let restart_quiz_selected = quiz_selected.clone();
    let restart_quiz_checked = quiz_checked.clone();
    let restart_quiz_finished = quiz_finished.clone();
    let restart_audio_unlocked = audio_unlocked.clone();
    let restart_audio_ref = audio_ref.clone();

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

    let is_current_correct = quiz_selected
        .as_ref()
        .is_some_and(|selected_index| *selected_index == current_question.correct_index);

    html! {
        <div class="app-shell">
            <audio
                ref={audio_ref.clone()}
                preload="auto"
                loop=true
                src="assets/audio/how-many-earths-theme.mp3"
            />

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

                            <button
                                class="toolbar-btn music"
                                onclick={Callback::from(move |_| {
                                    play_theme(&play_audio_handle);
                                })}
                            >
                                {"▶️ Play Theme"}
                            </button>

                            <button
                                class="toolbar-btn music"
                                onclick={Callback::from(move |_| {
                                    stop_theme(&stop_audio_handle);
                                })}
                            >
                                {"⏹ Stop Theme"}
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
                        {render_quiz_card(
                            current_question,
                            *quiz_index,
                            *quiz_score,
                            *quiz_selected,
                            *quiz_checked,
                            *quiz_finished,
                            *audio_unlocked,
                            is_current_correct,
                            quiz_selected.clone(),
                            quiz_checked.clone(),
                            quiz_index.clone(),
                            quiz_score.clone(),
                            quiz_finished.clone(),
                            audio_unlocked.clone(),
                            audio_ref.clone(),
                            restart_quiz_index,
                            restart_quiz_score,
                            restart_quiz_selected,
                            restart_quiz_checked,
                            restart_quiz_finished,
                            restart_audio_unlocked,
                            restart_audio_ref,
                        )}

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
                {"Theme audio expects a file at assets/audio/how-many-earths-theme.mp3 and is played through HTMLAudioElement for iPhone-browser-friendly playback."}
            </div>
        </div>
    }
}

#[allow(clippy::too_many_arguments)]
fn render_quiz_card(
    current_question: QuizQuestion,
    quiz_index: usize,
    quiz_score: usize,
    quiz_selected: Option<usize>,
    quiz_checked: bool,
    quiz_finished: bool,
    audio_unlocked: bool,
    is_current_correct: bool,
    quiz_selected_handle: UseStateHandle<Option<usize>>,
    quiz_checked_handle: UseStateHandle<bool>,
    quiz_index_handle: UseStateHandle<usize>,
    quiz_score_handle: UseStateHandle<usize>,
    quiz_finished_handle: UseStateHandle<bool>,
    audio_unlocked_handle: UseStateHandle<bool>,
    audio_ref: NodeRef,
    restart_quiz_index: UseStateHandle<usize>,
    restart_quiz_score: UseStateHandle<usize>,
    restart_quiz_selected: UseStateHandle<Option<usize>>,
    restart_quiz_checked: UseStateHandle<bool>,
    restart_quiz_finished: UseStateHandle<bool>,
    restart_audio_unlocked: UseStateHandle<bool>,
    restart_audio_ref: NodeRef,
) -> Html {
    let total_questions = QUIZ_QUESTIONS.len();

    html! {
        <div class="quiz-card">
            <div class="quiz-head">
                <div class="quiz-title">{"🧠 Quiz Mode"}</div>
                <div class="quiz-progress">
                    {format!("Score: {} / {}", quiz_score, total_questions)}
                </div>
            </div>

            {
                if quiz_finished {
                    html! {
                        <>
                            <div class="quiz-finished">
                                <strong>{"Quiz complete"}</strong>
                                <div>
                                    {format!("You scored {} out of {}.", quiz_score, total_questions)}
                                </div>

                                {
                                    if quiz_score == total_questions {
                                        html! {
                                            <div class="unlock-note">
                                                {"Perfect score unlocked the theme song. Tap Play Theme anytime, or it should have started automatically on the final correct answer."}
                                            </div>
                                        }
                                    } else {
                                        html! {
                                            <div class="unlock-note">
                                                {"Almost there. Get a perfect score to unlock the celebratory theme-song moment automatically."}
                                            </div>
                                        }
                                    }
                                }
                            </div>

                            <div class="quiz-actions">
                                <button
                                    class="quiz-btn"
                                    onclick={Callback::from(move |_| {
                                        restart_quiz_index.set(0);
                                        restart_quiz_score.set(0);
                                        restart_quiz_selected.set(None);
                                        restart_quiz_checked.set(false);
                                        restart_quiz_finished.set(false);
                                        restart_audio_unlocked.set(false);
                                        stop_theme(&restart_audio_ref);
                                    })}
                                >
                                    {"Restart Quiz"}
                                </button>
                            </div>

                            <div class="audio-status">
                                {
                                    if audio_unlocked {
                                        "Theme unlocked."
                                    } else {
                                        "Theme not unlocked yet."
                                    }
                                }
                            </div>
                        </>
                    }
                } else {
                    html! {
                        <>
                            <div class="quiz-progress">
                                {format!("Question {} of {}", quiz_index + 1, total_questions)}
                            </div>

                            <div class="quiz-question">{current_question.prompt}</div>

                            <div class="quiz-options">
                                {
                                    for current_question.options.iter().enumerate().map(|(idx, option)| {
                                        let quiz_selected_for_option = quiz_selected_handle.clone();
                                        let selected = quiz_selected == Some(idx);
                                        let class_name = if quiz_checked {
                                            if idx == current_question.correct_index {
                                                classes!("quiz-option", "correct")
                                            } else if selected {
                                                classes!("quiz-option", "wrong")
                                            } else {
                                                classes!("quiz-option")
                                            }
                                        } else if selected {
                                            classes!("quiz-option", "selected")
                                        } else {
                                            classes!("quiz-option")
                                        };

                                        html! {
                                            <button
                                                class={class_name}
                                                onclick={Callback::from(move |_| {
                                                    if !quiz_checked {
                                                        quiz_selected_for_option.set(Some(idx));
                                                    }
                                                })}
                                            >
                                                {(*option).to_string()}
                                            </button>
                                        }
                                    })
                                }
                            </div>

                            {
                                if quiz_checked {
                                    html! {
                                        <div class={classes!("quiz-feedback", if is_current_correct { "good" } else { "bad" })}>
                                            {
                                                if is_current_correct {
                                                    "Correct! "
                                                } else {
                                                    "Not quite. "
                                                }
                                            }
                                            {current_question.explanation}
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }

                            <div class="quiz-actions">
                                {
                                    if !quiz_checked {
                                        let quiz_checked_for_check = quiz_checked_handle.clone();
                                        let quiz_score_for_check = quiz_score_handle.clone();
                                        let audio_unlocked_for_check = audio_unlocked_handle.clone();
                                        let audio_ref_for_check = audio_ref.clone();
                                        let quiz_selected_for_check = quiz_selected_handle.clone();

                                        html! {
                                            <button
                                                class="quiz-btn"
                                                onclick={Callback::from(move |_| {
                                                    if let Some(selected_idx) = *quiz_selected_for_check {
                                                        let correct = selected_idx == current_question.correct_index;
                                                        if correct {
                                                            quiz_score_for_check.set(*quiz_score_for_check + 1);
                                                            let final_correct_perfect = quiz_index + 1 == total_questions
                                                                && *quiz_score_for_check + 1 == total_questions;
                                                            if final_correct_perfect {
                                                                audio_unlocked_for_check.set(true);
                                                                play_theme(&audio_ref_for_check);
                                                            }
                                                        }
                                                        quiz_checked_for_check.set(true);
                                                    }
                                                })}
                                            >
                                                {"Check Answer"}
                                            </button>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }

                                {
                                    if quiz_checked && quiz_index + 1 < total_questions {
                                        let quiz_index_for_next = quiz_index_handle.clone();
                                        let quiz_selected_for_next = quiz_selected_handle.clone();
                                        let quiz_checked_for_next = quiz_checked_handle.clone();
                                        html! {
                                            <button
                                                class="quiz-btn"
                                                onclick={Callback::from(move |_| {
                                                    quiz_index_for_next.set(*quiz_index_for_next + 1);
                                                    quiz_selected_for_next.set(None);
                                                    quiz_checked_for_next.set(false);
                                                })}
                                            >
                                                {"Next Question"}
                                            </button>
                                        }
                                    } else if quiz_checked && quiz_index + 1 == total_questions {
                                        let quiz_finished_for_done = quiz_finished_handle.clone();
                                        let quiz_selected_for_done = quiz_selected_handle.clone();
                                        let quiz_checked_for_done = quiz_checked_handle.clone();
                                        html! {
                                            <button
                                                class="quiz-btn"
                                                onclick={Callback::from(move |_| {
                                                    quiz_finished_for_done.set(true);
                                                    quiz_selected_for_done.set(None);
                                                    quiz_checked_for_done.set(false);
                                                })}
                                            >
                                                {"Finish Quiz"}
                                            </button>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </div>
                        </>
                    }
                }
            }
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