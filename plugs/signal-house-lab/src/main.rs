use gloo::timers::callback::{Interval, Timeout};
use gloo_storage::{LocalStorage, Storage};
use js_sys::{Date, Math};
use std::cell::RefCell;
use std::rc::Rc;
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

const LEVEL_TARGETS: [i32; 3] = [8, 14, 20];
const LEVEL_TIMES: [i32; 3] = [10, 12, 15];

#[derive(Clone, PartialEq)]
enum Mode {
    FreePlay,
    Challenge,
    Story,
    GameOver,
    Victory,
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

fn begin_level(
    level: usize,
    mode: UseStateHandle<Mode>,
    signals: UseStateHandle<i32>,
    challenge_time_left: UseStateHandle<i32>,
    challenge_running: UseStateHandle<bool>,
    combo: UseStateHandle<i32>,
    current_level: UseStateHandle<usize>,
    game_message: UseStateHandle<String>,
    challenge_interval: Rc<RefCell<Option<Interval>>>,
    signal_count_ref: Rc<RefCell<i32>>,
    last_tap_ms_ref: Rc<RefCell<f64>>,
    activate_audio: UseStateHandle<HtmlAudioElement>,
) {
    *challenge_interval.borrow_mut() = None;

    let clamped_level = level.clamp(1, 3);
    let target = LEVEL_TARGETS[clamped_level - 1];
    let time_allowed = LEVEL_TIMES[clamped_level - 1];

    mode.set(Mode::Challenge);
    current_level.set(clamped_level);
    signals.set(0);
    *signal_count_ref.borrow_mut() = 0;
    combo.set(0);
    *last_tap_ms_ref.borrow_mut() = 0.0;
    challenge_time_left.set(time_allowed);
    challenge_running.set(true);
    game_message.set(format!(
        "Level {} started. Reach {} signals in {} seconds.",
        clamped_level, target, time_allowed
    ));

    safe_play(&activate_audio);

    let time_left_handle = challenge_time_left.clone();
    let running_handle = challenge_running.clone();
    let interval_ref = challenge_interval.clone();
    let level_handle = current_level.clone();
    let mode_handle = mode.clone();
    let message_handle = game_message.clone();
    let activate_audio_handle = activate_audio.clone();
    let signal_count_ref_handle = signal_count_ref.clone();

    let mut remaining = time_allowed;
    let target_for_level = target;

    let interval = Interval::new(1000, move || {
        remaining -= 1;

        if remaining > 0 {
            time_left_handle.set(remaining);
        } else {
            time_left_handle.set(0);
            running_handle.set(false);
            *interval_ref.borrow_mut() = None;

            let final_score = *signal_count_ref_handle.borrow();
            let current = *level_handle;

            if final_score >= target_for_level {
                if current < 3 {
                    let next_level = current + 1;
                    level_handle.set(next_level);
                    mode_handle.set(Mode::FreePlay);
                    safe_play(&activate_audio_handle);
                    message_handle.set(format!(
                        "Level {} complete! You reached {} signals. Get ready for Level {}.",
                        current, final_score, next_level
                    ));
                } else {
                    mode_handle.set(Mode::Victory);
                    safe_play(&activate_audio_handle);
                    message_handle.set(format!(
                        "Victory! The Signal House completed all missions with {} signals.",
                        final_score
                    ));
                }
            } else {
                mode_handle.set(Mode::GameOver);
                message_handle.set(format!(
                    "Mission failed. You needed {} signals but reached {}.",
                    target_for_level, final_score
                ));
            }
        }
    });

    *challenge_interval.borrow_mut() = Some(interval);
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
    let super_flash = use_state(|| false);
    let audio_started = use_state(|| false);
    let story_step = use_state(|| 0_usize);

    let current_level = use_state(|| 1_usize);
    let combo = use_state(|| 0_i32);
    let max_combo =
        use_state(|| LocalStorage::get("signal_house_max_combo").unwrap_or(0_i32));
    let game_message =
        use_state(|| "Free play: wake the house and send signals into the sky.".to_string());

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
    let signal_count_ref = use_mut_ref(|| 0_i32);
    let last_tap_ms_ref = use_mut_ref(|| 0.0_f64);

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

    let super_house_flash = {
        let super_flash = super_flash.clone();
        Callback::from(move |_| {
            super_flash.set(true);
            let super_flash = super_flash.clone();
            Timeout::new(450, move || {
                super_flash.set(false);
            })
            .forget();
        })
    };

    let send_signal = {
        let signals = signals.clone();
        let best_score = best_score.clone();
        let combo = combo.clone();
        let max_combo = max_combo.clone();
        let challenge_running = challenge_running.clone();
        let challenge_time_left = challenge_time_left.clone();
        let ping_audio = ping_audio.clone();
        let ensure_audio_started = ensure_audio_started.clone();
        let flash_house = flash_house.clone();
        let super_house_flash = super_house_flash.clone();
        let game_message = game_message.clone();
        let signal_count_ref = signal_count_ref.clone();
        let last_tap_ms_ref = last_tap_ms_ref.clone();

        Callback::from(move |_| {
            ensure_audio_started.emit(());

            if *challenge_running && *challenge_time_left <= 0 {
                return;
            }

            safe_play(&ping_audio);
            flash_house.emit(());

            let now = Date::now();
            let last = *last_tap_ms_ref.borrow();
            let next_combo = if now - last <= 1200.0 {
                *combo + 1
            } else {
                1
            };
            *last_tap_ms_ref.borrow_mut() = now;

            combo.set(next_combo);

            if next_combo > *max_combo {
                max_combo.set(next_combo);
                let _ = LocalStorage::set("signal_house_max_combo", next_combo);
            }

            let bonus = if next_combo >= 3 {
                super_house_flash.emit(());
                2
            } else {
                1
            };

            let next = *signals + bonus;
            *signal_count_ref.borrow_mut() = next;
            signals.set(next);

            if next > *best_score {
                best_score.set(next);
                let _ = LocalStorage::set("signal_house_best_score", next);
            }

            if bonus > 1 {
                game_message.set(format!("Combo x{}! Bonus signal burst!", next_combo));
            } else {
                game_message.set("Signal sent.".to_string());
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
        let combo = combo.clone();
        let game_message = game_message.clone();

        Callback::from(move |_| {
            ensure_audio_started.emit(());
            safe_play(&activate_audio);
            mode.set(Mode::FreePlay);
            challenge_running.set(false);
            challenge_time_left.set(0);
            combo.set(0);
            game_message
                .set("Free play: wake the house and send signals into the sky.".to_string());
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
        let combo = combo.clone();
        let game_message = game_message.clone();

        Callback::from(move |_| {
            ensure_audio_started.emit(());
            safe_play(&activate_audio);
            mode.set(Mode::Story);
            story_step.set(0);
            challenge_running.set(false);
            challenge_time_left.set(0);
            combo.set(0);
            game_message.set(
                "Story mode: follow the creative spark from Colin’s drawings.".to_string(),
            );
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

    let start_level = {
        let mode = mode.clone();
        let signals = signals.clone();
        let challenge_time_left = challenge_time_left.clone();
        let challenge_running = challenge_running.clone();
        let ensure_audio_started = ensure_audio_started.clone();
        let challenge_interval = challenge_interval.clone();
        let current_level = current_level.clone();
        let combo = combo.clone();
        let game_message = game_message.clone();
        let signal_count_ref = signal_count_ref.clone();
        let last_tap_ms_ref = last_tap_ms_ref.clone();
        let activate_audio = activate_audio.clone();

        Callback::from(move |_e: MouseEvent| {
            ensure_audio_started.emit(());

            begin_level(
                *current_level,
                mode.clone(),
                signals.clone(),
                challenge_time_left.clone(),
                challenge_running.clone(),
                combo.clone(),
                current_level.clone(),
                game_message.clone(),
                challenge_interval.clone(),
                signal_count_ref.clone(),
                last_tap_ms_ref.clone(),
                activate_audio.clone(),
            );
        })
    };

    let retry_level = {
        let mode = mode.clone();
        let signals = signals.clone();
        let challenge_time_left = challenge_time_left.clone();
        let challenge_running = challenge_running.clone();
        let ensure_audio_started = ensure_audio_started.clone();
        let challenge_interval = challenge_interval.clone();
        let current_level = current_level.clone();
        let combo = combo.clone();
        let game_message = game_message.clone();
        let signal_count_ref = signal_count_ref.clone();
        let last_tap_ms_ref = last_tap_ms_ref.clone();
        let activate_audio = activate_audio.clone();

        Callback::from(move |_| {
            ensure_audio_started.emit(());

            begin_level(
                *current_level,
                mode.clone(),
                signals.clone(),
                challenge_time_left.clone(),
                challenge_running.clone(),
                combo.clone(),
                current_level.clone(),
                game_message.clone(),
                challenge_interval.clone(),
                signal_count_ref.clone(),
                last_tap_ms_ref.clone(),
                activate_audio.clone(),
            );
        })
    };

    let reset_score = {
        let signals = signals.clone();
        let best_score = best_score.clone();
        let challenge_time_left = challenge_time_left.clone();
        let challenge_running = challenge_running.clone();
        let challenge_interval = challenge_interval.clone();
        let combo = combo.clone();
        let max_combo = max_combo.clone();
        let current_level = current_level.clone();
        let mode = mode.clone();
        let game_message = game_message.clone();
        let signal_count_ref = signal_count_ref.clone();
        let last_tap_ms_ref = last_tap_ms_ref.clone();

        Callback::from(move |_| {
            signals.set(0);
            best_score.set(0);
            challenge_time_left.set(0);
            challenge_running.set(false);
            combo.set(0);
            max_combo.set(0);
            current_level.set(1);
            mode.set(Mode::FreePlay);
            game_message
                .set("Free play: wake the house and send signals into the sky.".to_string());
            *signal_count_ref.borrow_mut() = 0;
            *last_tap_ms_ref.borrow_mut() = 0.0;
            *challenge_interval.borrow_mut() = None;
            let _ = LocalStorage::set("signal_house_best_score", 0_i32);
            let _ = LocalStorage::set("signal_house_max_combo", 0_i32);
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
    let level = (*current_level).clamp(1, 3);
    let target = LEVEL_TARGETS[level - 1];
    let time_allowed = LEVEL_TIMES[level - 1];
    let progress_pct = if time_allowed > 0 {
        ((*challenge_time_left as f64 / time_allowed as f64) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let status_class = match &*mode {
        Mode::Victory => "mission-banner status-win",
        Mode::GameOver => "mission-banner status-lose",
        _ => "mission-banner",
    };

    html! {
        <div class="container">
            <h1>{"🌌 Signal House Lab v5"}</h1>
            <p class="subtitle">
                {"A full game version of The Signal House — with levels, combos, story mode, saved crew names, and cinematic signals powered by Colin’s imagination."}
            </p>

            <div class="skyline">
                <div class="sun-glow"></div>

                <div class="beams">
                    <div class="beam b1"></div>
                    <div class="beam b2"></div>
                    <div class="beam b3"></div>
                </div>

                <div class={classes!(
                    "house",
                    if *active_flash { "active" } else { "" },
                    if *super_flash { "super" } else { "" }
                )}>
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
                    <button onclick={start_level.clone()}>{"Start Level"}</button>
                    <button class="secondary" onclick={start_story}>{"Story Mode"}</button>
                    <button class="ghost" onclick={retry_level}>{"Retry Current Level"}</button>
                    <button class="danger" onclick={reset_score}>{"Reset Progress"}</button>
                </div>

                <div class="controls" style="margin-top: 12px;">
                    <button onclick={send_signal}>{"Send Signal"}</button>
                </div>

                <div class={status_class}>{(*game_message).clone()}</div>

                <div class="stats">
                    <div class="stat">
                        <div class="stat-label">{"Level"}</div>
                        <div class="stat-value">{level}</div>
                    </div>
                    <div class="stat">
                        <div class="stat-label">{"Signals"}</div>
                        <div class="stat-value">{*signals}</div>
                    </div>
                    <div class="stat">
                        <div class="stat-label">{"Best Score"}</div>
                        <div class="stat-value">{*best_score}</div>
                    </div>
                    <div class="stat">
                        <div class="stat-label">{"Combo"}</div>
                        <div class="stat-value">{*combo}</div>
                    </div>
                    <div class="stat">
                        <div class="stat-label">{"Max Combo"}</div>
                        <div class="stat-value">{*max_combo}</div>
                    </div>
                </div>

                <div class="goal-row">
                    <div class="goal-box">
                        <div>{"Current goal:"}</div>
                        <strong>{format!("Reach {} signals", target)}</strong>
                    </div>
                    <div class="goal-box">
                        <div>{"Time allowed:"}</div>
                        <strong>{format!("{} seconds", time_allowed)}</strong>
                    </div>
                </div>

                <div class="stats" style="margin-top: 12px;">
                    <div class="stat">
                        <div class="stat-label">{"Time Left"}</div>
                        <div class="stat-value">{*challenge_time_left}</div>
                    </div>
                </div>

                <div class="progress">
                    <div
                        class="progress-bar"
                        style={format!("width: {}%;", progress_pct)}
                    />
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