use gloo::timers::callback::{Interval, Timeout};
use gloo_storage::{LocalStorage, Storage};
use js_sys::Math;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlAudioElement, HtmlCanvasElement, HtmlInputElement};
use yew::prelude::*;

const STORY_STEPS: [&str; 5] = [
    "This all began with Colin’s drawings. Two hand-drawn ideas sparked the MikeGyver Studio creative engine that became The Signal House.",
    "First came the structure: a house full of shapes, symmetry, and pathways that felt like energy moving through the walls.",
    "Then came the sky: sunrise, motion, and a feeling that something beyond Earth was connected to the house below.",
    "So the question became: what if this house could wake up, send signals upward, and invite friends into the mission?",
    "Now it’s your turn. Wake the Signal House, send the signal, and keep the imagination going.",
];

#[derive(Clone, PartialEq)]
enum Mode {
    FreePlay,
    Challenge,
    Story,
}

fn safe_play(audio: &HtmlAudioElement) {
    let _ = audio.set_current_time(0.0);
    let _ = audio.play();
}

fn safe_loop_start(audio: &HtmlAudioElement) {
    if audio.paused() {
        let _ = audio.play();
    }
}

fn draw_starfield(canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) {
    let width = canvas.width() as f64;
    let height = canvas.height() as f64;

    ctx.set_fill_style_str("#0b1020");
    ctx.fill_rect(0.0, 0.0, width, height);

    for _ in 0..70 {
        let x = Math::random() * width;
        let y = Math::random() * height;
        let size = 1.0 + (Math::random() * 2.2);
        let alpha = 0.35 + (Math::random() * 0.65);
        let color = format!("rgba(255,255,255,{alpha})");
        ctx.set_fill_style_str(&color);
        ctx.fill_rect(x, y, size, size);
    }
}

fn init_stars() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    let Some(canvas_el) = document.get_element_by_id("stars") else {
        return;
    };
    let Ok(canvas) = canvas_el.dyn_into::<HtmlCanvasElement>() else {
        return;
    };
    let Ok(Some(ctx_any)) = canvas.get_context("2d") else {
        return;
    };
    let Ok(ctx) = ctx_any.dyn_into::<CanvasRenderingContext2d>() else {
        return;
    };

    let resize_and_draw = {
        let canvas = canvas.clone();
        let ctx = ctx.clone();
        move || {
            let Some(window) = web_sys::window() else {
                return;
            };
            let width = window
                .inner_width()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(1280.0) as u32;
            let height = window
                .inner_height()
                .ok()
                .and_then(|v| v.as_f64())
                .unwrap_or(720.0) as u32;

            canvas.set_width(width);
            canvas.set_height(height);
            draw_starfield(&canvas, &ctx);
        }
    };

    resize_and_draw();

    let canvas_clone = canvas.clone();
    let ctx_clone = ctx.clone();
    Interval::new(240, move || {
        draw_starfield(&canvas_clone, &ctx_clone);
    })
    .forget();
}

#[function_component(App)]
fn app() -> Html {
    let mode = use_state(|| Mode::FreePlay);
    let signals = use_state(|| 0_i32);
    let best_score =
        use_state(|| LocalStorage::get("signal_house_best_score").unwrap_or(0_i32));
    let challenge_time_left = use_state(|| 0_i32);
    let challenge_running = use_state(|| false);
    let active_flash = use_state(|| false);
    let audio_started = use_state(|| false);
    let story_step = use_state(|| 0_usize);

    let crew_names = use_state(|| {
        LocalStorage::get("signal_house_crew_names").unwrap_or_else(|_| {
            vec![
                "Colin".to_string(),
                "Maya".to_string(),
                "Leo".to_string(),
            ]
        })
    });

    let ping_audio =
        use_state(|| HtmlAudioElement::new_with_src("assets/audio/ping.wav").unwrap());
    let activate_audio =
        use_state(|| HtmlAudioElement::new_with_src("assets/audio/activate.wav").unwrap());
    let ambient_audio = use_state(|| {
        let a = HtmlAudioElement::new_with_src("assets/audio/ambient.wav").unwrap();
        a.set_loop(true);
        a.set_volume(0.45);
        a
    });

    let challenge_interval = use_mut_ref(|| Option::<Interval>::None);

    {
        use_effect(|| {
            init_stars();
            || ()
        });
    }

    let ensure_audio_started = {
        let audio_started = audio_started.clone();
        let ambient_audio = ambient_audio.clone();

        Callback::from(move |_| {
            if !*audio_started {
                safe_loop_start(&ambient_audio);
                audio_started.set(true);
            }
        })
    };

    let flash_house = {
        let active_flash = active_flash.clone();
        Callback::from(move |_| {
            active_flash.set(true);
            let active_flash = active_flash.clone();
            Timeout::new(220, move || {
                active_flash.set(false);
            })
            .forget();
        })
    };

    let send_signal = {
        let signals = signals.clone();
        let best_score = best_score.clone();
        let challenge_running = challenge_running.clone();
        let challenge_time_left = challenge_time_left.clone();
        let ping_audio = ping_audio.clone();
        let ensure_audio_started = ensure_audio_started.clone();
        let flash_house = flash_house.clone();

        Callback::from(move |_| {
            ensure_audio_started.emit(());
            safe_play(&ping_audio);
            flash_house.emit(());

            if *challenge_running && *challenge_time_left <= 0 {
                return;
            }

            let next = *signals + 1;
            signals.set(next);

            if next > *best_score {
                best_score.set(next);
                let _ = LocalStorage::set("signal_house_best_score", next);
            }
        })
    };

    let start_free_play = {
        let mode = mode.clone();
        let challenge_running = challenge_running.clone();
        let challenge_time_left = challenge_time_left.clone();
        let activate_audio = activate_audio.clone();
        let ensure_audio_started = ensure_audio_started.clone();
        let challenge_interval = challenge_interval.clone();

        Callback::from(move |_| {
            ensure_audio_started.emit(());
            safe_play(&activate_audio);
            mode.set(Mode::FreePlay);
            challenge_running.set(false);
            challenge_time_left.set(0);
            *challenge_interval.borrow_mut() = None;
        })
    };

    let start_story = {
        let mode = mode.clone();
        let story_step = story_step.clone();
        let challenge_running = challenge_running.clone();
        let challenge_time_left = challenge_time_left.clone();
        let activate_audio = activate_audio.clone();
        let ensure_audio_started = ensure_audio_started.clone();
        let challenge_interval = challenge_interval.clone();

        Callback::from(move |_| {
            ensure_audio_started.emit(());
            safe_play(&activate_audio);
            mode.set(Mode::Story);
            story_step.set(0);
            challenge_running.set(false);
            challenge_time_left.set(0);
            *challenge_interval.borrow_mut() = None;
        })
    };

    let next_story = {
        let story_step = story_step.clone();
        Callback::from(move |_| {
            let next = (*story_step + 1).min(STORY_STEPS.len() - 1);
            story_step.set(next);
        })
    };

    let prev_story = {
        let story_step = story_step.clone();
        Callback::from(move |_| {
            let next = story_step.saturating_sub(1);
            story_step.set(next);
        })
    };

    let start_challenge = {
        let mode = mode.clone();
        let signals = signals.clone();
        let challenge_time_left = challenge_time_left.clone();
        let challenge_running = challenge_running.clone();
        let activate_audio = activate_audio.clone();
        let ensure_audio_started = ensure_audio_started.clone();
        let challenge_interval = challenge_interval.clone();

        Callback::from(move |_| {
            ensure_audio_started.emit(());
            safe_play(&activate_audio);

            *challenge_interval.borrow_mut() = None;

            mode.set(Mode::Challenge);
            signals.set(0);
            challenge_time_left.set(10);
            challenge_running.set(true);

            let time_left_handle = challenge_time_left.clone();
            let running_handle = challenge_running.clone();
            let interval_ref = challenge_interval.clone();

            let mut remaining = 10_i32;

            let interval = Interval::new(1000, move || {
                remaining -= 1;

                if remaining > 0 {
                    time_left_handle.set(remaining);
                } else {
                    time_left_handle.set(0);
                    running_handle.set(false);
                    *interval_ref.borrow_mut() = None;
                }
            });

            *challenge_interval.borrow_mut() = Some(interval);
        })
    };

    let reset_score = {
        let signals = signals.clone();
        let best_score = best_score.clone();
        let challenge_time_left = challenge_time_left.clone();
        let challenge_running = challenge_running.clone();
        let challenge_interval = challenge_interval.clone();

        Callback::from(move |_| {
            signals.set(0);
            best_score.set(0);
            challenge_time_left.set(0);
            challenge_running.set(false);
            *challenge_interval.borrow_mut() = None;
            let _ = LocalStorage::set("signal_house_best_score", 0_i32);
        })
    };

    let update_name = {
        let crew_names = crew_names.clone();

        Callback::from(move |(index, value): (usize, String)| {
            let mut next = (*crew_names).clone();
            if index < next.len() {
                next[index] = value;
                let _ = LocalStorage::set("signal_house_crew_names", next.clone());
                crew_names.set(next);
            }
        })
    };

    let current_story_text = STORY_STEPS[*story_step];
    let mission_banner = match (&*mode, *challenge_running, *challenge_time_left) {
        (Mode::Challenge, true, t) => {
            format!("Challenge live: send as many signals as you can in {t} seconds.")
        }
        (Mode::Challenge, false, 0) => "Challenge complete. Great job, crew.".to_string(),
        (Mode::Story, _, _) => {
            "Story mode: follow the creative spark from Colin’s drawings.".to_string()
        }
        _ => "Free play: wake the house and send signals into the sky.".to_string(),
    };

    html! {
        <div class="container">
            <h1>{"🌌 Signal House Lab v4"}</h1>
            <p class="subtitle">
                {"A cinematic Rust + Yew experience inspired by Colin’s drawings — part story, part mission, part imagination lab."}
            </p>

            <div class="skyline">
                <div class="sun-glow"></div>

                <div class="beams">
                    <div class="beam b1"></div>
                    <div class="beam b2"></div>
                    <div class="beam b3"></div>
                </div>

                <div class={classes!("house", if *active_flash { "active" } else { "" })}>
                    <div class="house-windows">
                        <div class="window w1"></div>
                        <div class="window w2"></div>
                        <div class="window w3"></div>
                        <div class="window w4"></div>
                    </div>
                </div>

                <div class="particles">
                    <div class="particle p1"></div>
                    <div class="particle p2"></div>
                    <div class="particle p3"></div>
                    <div class="particle p4"></div>
                    <div class="particle p5"></div>
                </div>
            </div>

            <div class="card">
                <h2>{"Mission Controls"}</h2>
                <div class="controls">
                    <button onclick={start_free_play}>{"Free Play"}</button>
                    <button onclick={start_challenge.clone()}>{"Start 10s Challenge"}</button>
                    <button class="secondary" onclick={start_story}>{"Story Mode"}</button>
                    <button class="ghost" onclick={reset_score}>{"Reset Score"}</button>
                </div>

                <div class="controls" style="margin-top: 12px;">
                    <button onclick={send_signal}>{"Send Signal"}</button>
                </div>

                <div class="mission-banner">{mission_banner}</div>

                <div class="stats">
                    <div class="stat">
                        <div class="stat-label">{"Signals"}</div>
                        <div class="stat-value">{*signals}</div>
                    </div>
                    <div class="stat">
                        <div class="stat-label">{"Best Score"}</div>
                        <div class="stat-value">{*best_score}</div>
                    </div>
                    <div class="stat">
                        <div class="stat-label">{"Time Left"}</div>
                        <div class="stat-value">{*challenge_time_left}</div>
                    </div>
                </div>
            </div>

            <div class="card">
                <h2>{"Crew Setup"}</h2>
                <div class="names">
                    {
                        for crew_names.iter().enumerate().map(|(idx, name)| {
                            let update_name = update_name.clone();
                            html! {
                                <div class="name-field">
                                    <label for={format!("crew-{idx}")}>{format!("Friend {}", idx + 1)}</label>
                                    <input
                                        id={format!("crew-{idx}")}
                                        type="text"
                                        value={name.clone()}
                                        oninput={Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            update_name.emit((idx, input.value()));
                                        })}
                                    />
                                </div>
                            }
                        })
                    }
                </div>
            </div>

            <div class="card">
                <h2>{"Story Mode"}</h2>
                <p class="story-text">{current_story_text}</p>
                <p class="story-progress">
                    {format!("Story step {} of {}", *story_step + 1, STORY_STEPS.len())}
                </p>

                <div class="controls">
                    <button class="ghost" onclick={prev_story}>{"Previous"}</button>
                    <button class="secondary" onclick={next_story}>{"Next"}</button>
                </div>
            </div>

            <p class="footer-note">
                {"Audio files expected in /assets/audio/: ping.wav, activate.wav, ambient.wav"}
            </p>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}