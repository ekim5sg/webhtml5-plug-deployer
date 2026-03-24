use gloo::storage::{LocalStorage, Storage};
use gloo::timers::callback::Interval;
use js_sys::{Array, Date, Uint8Array};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use wasm_bindgen::JsValue;
use web_sys::{
    Blob, BlobPropertyBag, HtmlAudioElement, HtmlInputElement, Notification,
    NotificationPermission, Url,
};
use yew::prelude::*;

const STORAGE_KEY: &str = "steadysip_state_v4";
const DEFAULT_GOAL_OZ: u32 = 96;
const DEFAULT_INTERVAL_MIN: u32 = 45;

thread_local! {
    static ACTIVE_AUDIO: RefCell<Option<HtmlAudioElement>> = RefCell::new(None);
    static ACTIVE_AUDIO_URL: RefCell<Option<String>> = RefCell::new(None);
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct IntakeEntry {
    timestamp_ms: f64,
    ounces: u32,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Symptoms {
    dizziness: bool,
    dry_mouth: bool,
    dark_urine: bool,
    racing_pulse: bool,
    near_faint: bool,
}

impl Default for Symptoms {
    fn default() -> Self {
        Self {
            dizziness: false,
            dry_mouth: false,
            dark_urine: false,
            racing_pulse: false,
            near_faint: false,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct AppState {
    day_key: String,
    daily_goal_oz: u32,
    reminder_interval_min: u32,
    total_oz: u32,
    entries: Vec<IntakeEntry>,
    symptoms: Symptoms,
    notifications_enabled: bool,
    sound_enabled: bool,
    audio_unlocked: bool,
    wake_start_hour: u32,
    wake_end_hour: u32,
    last_reminder_ms: f64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            day_key: today_key(),
            daily_goal_oz: DEFAULT_GOAL_OZ,
            reminder_interval_min: DEFAULT_INTERVAL_MIN,
            total_oz: 0,
            entries: vec![],
            symptoms: Symptoms::default(),
            notifications_enabled: false,
            sound_enabled: false,
            audio_unlocked: false,
            wake_start_hour: 7,
            wake_end_hour: 22,
            last_reminder_ms: 0.0,
        }
    }
}

fn today_key() -> String {
    let d = Date::new_0();
    format!(
        "{:04}-{:02}-{:02}",
        d.get_full_year(),
        d.get_month() + 1,
        d.get_date()
    )
}

fn now_ms() -> f64 {
    Date::new_0().get_time()
}

fn current_hour_local() -> u32 {
    Date::new_0().get_hours() as u32
}

fn load_state() -> AppState {
    match LocalStorage::get::<AppState>(STORAGE_KEY) {
        Ok(mut state) => {
            if state.day_key != today_key() {
                state.day_key = today_key();
                state.total_oz = 0;
                state.entries.clear();
                state.symptoms = Symptoms::default();
                state.last_reminder_ms = 0.0;
                save_state(&state);
            }
            state
        }
        Err(_) => AppState::default(),
    }
}

fn save_state(state: &AppState) {
    let _ = LocalStorage::set(STORAGE_KEY, state);
}

fn pace_target_by_now(goal_oz: u32, start_hour: u32, end_hour: u32) -> u32 {
    if end_hour <= start_hour {
        return 0;
    }

    let hour = current_hour_local();

    if hour <= start_hour {
        return 0;
    }

    if hour >= end_hour {
        return goal_oz;
    }

    let elapsed = hour - start_hour;
    let span = end_hour - start_hour;

    ((goal_oz as f64) * (elapsed as f64 / span as f64)).round() as u32
}

fn percent(total: u32, goal: u32) -> u32 {
    if goal == 0 {
        0
    } else {
        (((total as f64 / goal as f64) * 100.0).round() as u32).min(999)
    }
}

fn in_wake_window(start_hour: u32, end_hour: u32) -> bool {
    let h = current_hour_local();
    h >= start_hour && h < end_hour
}

fn can_notify() -> bool {
    Notification::permission() != NotificationPermission::Denied
}

fn notifications_granted() -> bool {
    Notification::permission() == NotificationPermission::Granted
}

fn maybe_send_notification(title: &str) {
    if notifications_granted() {
        let _ = Notification::new(title);
    }
}

fn le_u16(v: u16) -> [u8; 2] {
    [(v & 0xFF) as u8, ((v >> 8) & 0xFF) as u8]
}

fn le_u32(v: u32) -> [u8; 4] {
    [
        (v & 0xFF) as u8,
        ((v >> 8) & 0xFF) as u8,
        ((v >> 16) & 0xFF) as u8,
        ((v >> 24) & 0xFF) as u8,
    ]
}

fn make_sine_wav(sample_rate: u32, freq_hz: f32, duration_ms: u32, volume: f32) -> Vec<u8> {
    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let bytes_per_sample = (bits_per_sample / 8) as u32;
    let num_samples = (sample_rate as u64 * duration_ms as u64 / 1000) as u32;
    let data_size = num_samples * channels as u32 * bytes_per_sample;
    let byte_rate = sample_rate * channels as u32 * bytes_per_sample;
    let block_align = channels * (bits_per_sample / 8);

    let mut wav = Vec::with_capacity((44 + data_size) as usize);

    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&le_u32(36 + data_size));
    wav.extend_from_slice(b"WAVE");

    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&le_u32(16));
    wav.extend_from_slice(&le_u16(1));
    wav.extend_from_slice(&le_u16(channels));
    wav.extend_from_slice(&le_u32(sample_rate));
    wav.extend_from_slice(&le_u32(byte_rate));
    wav.extend_from_slice(&le_u16(block_align));
    wav.extend_from_slice(&le_u16(bits_per_sample));

    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&le_u32(data_size));

    let two_pi = std::f32::consts::PI * 2.0;
    let attack_samples = (sample_rate / 100) as u32;
    let release_samples = (sample_rate / 6) as u32;

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        let env = if i < attack_samples {
            i as f32 / attack_samples.max(1) as f32
        } else if i > num_samples.saturating_sub(release_samples) {
            let remain = num_samples.saturating_sub(i);
            remain as f32 / release_samples.max(1) as f32
        } else {
            1.0
        };

        let sample = (two_pi * freq_hz * t).sin() * volume * env;
        let s = (sample * i16::MAX as f32) as i16;
        wav.extend_from_slice(&s.to_le_bytes());
    }

    wav
}

fn make_double_chime_wav() -> Vec<u8> {
    let a = make_sine_wav(22_050, 880.0, 160, 0.23);
    let b = make_sine_wav(22_050, 1174.66, 180, 0.20);

    let silence_bytes = vec![0u8; (22_050 / 12 * 2) as usize];

    let mut pcm = a[44..].to_vec();
    pcm.extend_from_slice(&silence_bytes);
    pcm.extend_from_slice(&b[44..]);

    let channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let bytes_per_sample = (bits_per_sample / 8) as u32;
    let sample_rate = 22_050u32;
    let data_size = pcm.len() as u32;
    let byte_rate = sample_rate * channels as u32 * bytes_per_sample;
    let block_align = channels * (bits_per_sample / 8);

    let mut wav = Vec::with_capacity(44 + pcm.len());

    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&le_u32(36 + data_size));
    wav.extend_from_slice(b"WAVE");

    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&le_u32(16));
    wav.extend_from_slice(&le_u16(1));
    wav.extend_from_slice(&le_u16(channels));
    wav.extend_from_slice(&le_u32(sample_rate));
    wav.extend_from_slice(&le_u32(byte_rate));
    wav.extend_from_slice(&le_u16(block_align));
    wav.extend_from_slice(&le_u16(bits_per_sample));

    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&le_u32(data_size));
    wav.extend_from_slice(&pcm);

    wav
}

fn play_wav_bytes(bytes: &[u8]) {
    let uint8 = Uint8Array::new_with_length(bytes.len() as u32);
    uint8.copy_from(bytes);

    let parts = Array::new();
    parts.push(&uint8.into());

    let bag = BlobPropertyBag::new();
    bag.set_type("audio/wav");

    let blob = match Blob::new_with_u8_array_sequence_and_options(&parts, &bag) {
        Ok(b) => b,
        Err(_) => return,
    };

    let url = match Url::create_object_url_with_blob(&blob) {
        Ok(u) => u,
        Err(_) => return,
    };

    let audio = match HtmlAudioElement::new_with_src(&url) {
        Ok(a) => a,
        Err(_) => {
            let _ = Url::revoke_object_url(&url);
            return;
        }
    };

    audio.set_preload("auto");
    audio.set_autoplay(false);
    audio.set_loop(false);

    ACTIVE_AUDIO.with(|slot| {
        if let Some(old) = slot.borrow_mut().take() {
            let _ = old.pause();
        }
    });

    ACTIVE_AUDIO_URL.with(|slot| {
        if let Some(old_url) = slot.borrow_mut().take() {
            let _ = Url::revoke_object_url(&old_url);
        }
    });

    ACTIVE_AUDIO.with(|slot| {
        *slot.borrow_mut() = Some(audio.clone());
    });

    ACTIVE_AUDIO_URL.with(|slot| {
        *slot.borrow_mut() = Some(url.clone());
    });

    audio.load();
    let _ = audio.play();
}

fn play_reminder_chime() {
    let wav = make_double_chime_wav();
    play_wav_bytes(&wav);
}

#[function_component(App)]
fn app() -> Html {
    let state = use_state(load_state);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            let interval = Interval::new(60_000, move || {
                let mut next = (*state).clone();

                if next.day_key != today_key() {
                    next = AppState::default();
                    save_state(&next);
                    state.set(next);
                    return;
                }

                let interval_ms = (next.reminder_interval_min as f64) * 60_000.0;
                let due = now_ms() - next.last_reminder_ms >= interval_ms;

                let behind = next.total_oz + 8
                    < pace_target_by_now(
                        next.daily_goal_oz,
                        next.wake_start_hour,
                        next.wake_end_hour,
                    );

                if in_wake_window(next.wake_start_hour, next.wake_end_hour) && due {
                    if notifications_granted() || next.notifications_enabled {
                        if behind {
                            maybe_send_notification("Steady Sip — behind pace. Sip 8–16 oz.");
                        } else {
                            maybe_send_notification("Steady Sip — time for a quick water check.");
                        }
                    }

                    if next.sound_enabled && next.audio_unlocked {
                        play_reminder_chime();
                    }

                    next.last_reminder_ms = now_ms();
                    save_state(&next);
                    state.set(next);
                }
            });

            move || drop(interval)
        });
    }

    let total_pct = percent(state.total_oz, state.daily_goal_oz);
    let pace_now = pace_target_by_now(
        state.daily_goal_oz,
        state.wake_start_hour,
        state.wake_end_hour,
    );
    let remaining = state.daily_goal_oz.saturating_sub(state.total_oz);

    let status_banner = if state.symptoms.near_faint || state.symptoms.racing_pulse {
        html! {
            <div class="banner danger">
                {"Today is not a “push through it” day. Sit or lie down if you feel faint, hydrate, and follow your clinician’s guidance. Seek medical care for worsening symptoms."}
            </div>
        }
    } else if state.total_oz < pace_now {
        html! {
            <div class="banner warn">
                {format!(
                    "You’re behind your pace target by {} oz. A 12–16 oz catch-up sip would help.",
                    pace_now.saturating_sub(state.total_oz)
                )}
            </div>
        }
    } else {
        html! {
            <div class="banner good">
                {"You’re on pace. Keep stacking small wins instead of waiting until you feel thirsty."}
            </div>
        }
    };

    let add_oz = {
        let state = state.clone();
        Callback::from(move |oz: u32| {
            let mut next = (*state).clone();
            next.total_oz = next.total_oz.saturating_add(oz);
            next.entries.insert(
                0,
                IntakeEntry {
                    timestamp_ms: now_ms(),
                    ounces: oz,
                },
            );
            save_state(&next);
            state.set(next);
        })
    };

    let undo_last = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut next = (*state).clone();
            if let Some(last) = next.entries.first().cloned() {
                next.total_oz = next.total_oz.saturating_sub(last.ounces);
                next.entries.remove(0);
                save_state(&next);
                state.set(next);
            }
        })
    };

    let reset_day = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut next = (*state).clone();
            next.day_key = today_key();
            next.total_oz = 0;
            next.entries.clear();
            next.symptoms = Symptoms::default();
            next.last_reminder_ms = 0.0;
            save_state(&next);
            state.set(next);
        })
    };

    let request_notifications = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut next = (*state).clone();

            if can_notify() {
                let _ = Notification::request_permission();
                next.notifications_enabled = true;
            }

            save_state(&next);
            state.set(next);
        })
    };

    let enable_sound = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut next = (*state).clone();
            next.sound_enabled = true;
            next.audio_unlocked = true;
            save_state(&next);
            state.set(next);

            play_reminder_chime();
        })
    };

    let test_sound = Callback::from(move |_| {
        play_reminder_chime();
    });

    let set_goal = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(v) = input.value().parse::<u32>() {
                let mut next = (*state).clone();
                next.daily_goal_oz = v.clamp(32, 200);
                save_state(&next);
                state.set(next);
            }
        })
    };

    let set_interval = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(v) = input.value().parse::<u32>() {
                let mut next = (*state).clone();
                next.reminder_interval_min = v.clamp(15, 180);
                save_state(&next);
                state.set(next);
            }
        })
    };

    let set_wake_start = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(v) = input.value().parse::<u32>() {
                let mut next = (*state).clone();
                next.wake_start_hour = v.clamp(0, 23);
                save_state(&next);
                state.set(next);
            }
        })
    };

    let set_wake_end = {
        let state = state.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            if let Ok(v) = input.value().parse::<u32>() {
                let mut next = (*state).clone();
                next.wake_end_hour = v.clamp(1, 23);
                save_state(&next);
                state.set(next);
            }
        })
    };

    let toggle_symptom = {
        let state = state.clone();
        Callback::from(move |name: String| {
            let mut next = (*state).clone();

            match name.as_str() {
                "dizziness" => next.symptoms.dizziness = !next.symptoms.dizziness,
                "dry_mouth" => next.symptoms.dry_mouth = !next.symptoms.dry_mouth,
                "dark_urine" => next.symptoms.dark_urine = !next.symptoms.dark_urine,
                "racing_pulse" => next.symptoms.racing_pulse = !next.symptoms.racing_pulse,
                "near_faint" => next.symptoms.near_faint = !next.symptoms.near_faint,
                _ => {}
            }

            save_state(&next);
            state.set(next);
        })
    };

    html! {
        <main class="app-shell">
            <header class="topbar">
                <div class="brand">{"MikeGyver Studio"}</div>
                <div class="title-row">
                    <div>
                        <h1 class="title">{"Steady Sip"}</h1>
                        <div class="subtitle">
                            {"A calm hydration companion built for consistency, not panic. Default daily target: 96 oz."}
                        </div>
                    </div>
                </div>
            </header>

            <div class="grid">
                <section class="card">
                    <h2>{"Today"}</h2>

                    <div class="hero-amount">
                        <div class="big">{state.total_oz}</div>
                        <div class="unit">{format!("oz of {} oz", state.daily_goal_oz)}</div>
                    </div>

                    <div class="progress-wrap">
                        <div class="progress-meta">
                            <span>{format!("{}% complete", total_pct.min(100))}</span>
                            <span>{format!("{} oz left", remaining)}</span>
                        </div>
                        <div class="progress-bar">
                            <div
                                class="progress-fill"
                                style={format!("width: {}%;", total_pct.min(100))}
                            ></div>
                        </div>
                    </div>

                    <div class="quick-actions">
                        <button class="btn primary" onclick={{
                            let add_oz = add_oz.clone();
                            Callback::from(move |_| add_oz.emit(8))
                        }}>{"Add 8 oz"}</button>

                        <button class="btn primary" onclick={{
                            let add_oz = add_oz.clone();
                            Callback::from(move |_| add_oz.emit(12))
                        }}>{"Add 12 oz"}</button>

                        <button class="btn primary" onclick={{
                            let add_oz = add_oz.clone();
                            Callback::from(move |_| add_oz.emit(16))
                        }}>{"Add 16 oz"}</button>

                        <button class="btn good" onclick={{
                            let add_oz = add_oz.clone();
                            Callback::from(move |_| add_oz.emit(20))
                        }}>{"Add 20 oz"}</button>
                    </div>

                    <div class="row" style="margin-top: 12px;">
                        <button class="btn warn" onclick={undo_last}>{"Undo Last"}</button>
                        <button class="btn danger" onclick={reset_day}>{"Reset Day"}</button>
                    </div>

                    <hr class="sep" />
                    {status_banner}

                    <div class="footer-note">
                        {"This app is a habit tool. It is not a substitute for urgent care, ER follow-up, or physician guidance after dehydration, low blood pressure, rapid pulse, or fainting."}
                    </div>
                </section>

                <aside class="stack">
                    <section class="card">
                        <h3>{"Settings"}</h3>

                        <div class="stack">
                            <div>
                                <div class="label">{"Daily Goal (oz)"}</div>
                                <input
                                    class="input"
                                    type="number"
                                    value={state.daily_goal_oz.to_string()}
                                    oninput={set_goal}
                                />
                            </div>

                            <div>
                                <div class="label">{"Reminder Interval (minutes)"}</div>
                                <input
                                    class="input"
                                    type="number"
                                    value={state.reminder_interval_min.to_string()}
                                    oninput={set_interval}
                                />
                            </div>

                            <div class="row">
                                <div style="flex:1; min-width: 120px;">
                                    <div class="label">{"Wake Start Hour"}</div>
                                    <input
                                        class="input"
                                        type="number"
                                        value={state.wake_start_hour.to_string()}
                                        oninput={set_wake_start}
                                    />
                                </div>
                                <div style="flex:1; min-width: 120px;">
                                    <div class="label">{"Wake End Hour"}</div>
                                    <input
                                        class="input"
                                        type="number"
                                        value={state.wake_end_hour.to_string()}
                                        oninput={set_wake_end}
                                    />
                                </div>
                            </div>

                            <button class="btn primary" onclick={request_notifications}>
                                {"Enable Reminders"}
                            </button>

                            <button class="btn good" onclick={enable_sound}>
                                {
                                    if state.audio_unlocked {
                                        "Chime Enabled"
                                    } else {
                                        "Enable Audible Chime"
                                    }
                                }
                            </button>

                            <button class="btn warn" onclick={test_sound}>
                                {"Test Chime"}
                            </button>

                            <div class="small muted">
                                {"On iPhone, tap “Enable Audible Chime” once to unlock audio playback for this session."}
                            </div>
                        </div>
                    </section>

                    <section class="card">
                        <h3>{"Pace"}</h3>
                        <div class="kpi">
                            <div class="kpi-box">
                                <div class="kpi-label">{"Pace target now"}</div>
                                <div class="kpi-value">{format!("{} oz", pace_now)}</div>
                            </div>
                            <div class="kpi-box">
                                <div class="kpi-label">{"Behind / ahead"}</div>
                                <div class="kpi-value">
                                    {
                                        if state.total_oz >= pace_now {
                                            format!("+{} oz", state.total_oz - pace_now)
                                        } else {
                                            format!("-{} oz", pace_now - state.total_oz)
                                        }
                                    }
                                </div>
                            </div>
                            <div class="kpi-box">
                                <div class="kpi-label">{"Goal left"}</div>
                                <div class="kpi-value">{format!("{} oz", remaining)}</div>
                            </div>
                        </div>
                    </section>
                </aside>
            </div>

            <div class="grid" style="margin-top:16px;">
                <section class="card">
                    <h3>{"Symptom Check"}</h3>

                    <div class="check-grid">
                        <label class="check-card">
                            <input
                                type="checkbox"
                                checked={state.symptoms.dizziness}
                                onchange={{
                                    let toggle_symptom = toggle_symptom.clone();
                                    Callback::from(move |_| toggle_symptom.emit("dizziness".to_string()))
                                }}
                            />
                            <span>{"Dizziness / lightheaded"}</span>
                        </label>

                        <label class="check-card">
                            <input
                                type="checkbox"
                                checked={state.symptoms.dry_mouth}
                                onchange={{
                                    let toggle_symptom = toggle_symptom.clone();
                                    Callback::from(move |_| toggle_symptom.emit("dry_mouth".to_string()))
                                }}
                            />
                            <span>{"Dry mouth"}</span>
                        </label>

                        <label class="check-card">
                            <input
                                type="checkbox"
                                checked={state.symptoms.dark_urine}
                                onchange={{
                                    let toggle_symptom = toggle_symptom.clone();
                                    Callback::from(move |_| toggle_symptom.emit("dark_urine".to_string()))
                                }}
                            />
                            <span>{"Dark urine"}</span>
                        </label>

                        <label class="check-card">
                            <input
                                type="checkbox"
                                checked={state.symptoms.racing_pulse}
                                onchange={{
                                    let toggle_symptom = toggle_symptom.clone();
                                    Callback::from(move |_| toggle_symptom.emit("racing_pulse".to_string()))
                                }}
                            />
                            <span>{"Racing pulse"}</span>
                        </label>

                        <label class="check-card">
                            <input
                                type="checkbox"
                                checked={state.symptoms.near_faint}
                                onchange={{
                                    let toggle_symptom = toggle_symptom.clone();
                                    Callback::from(move |_| toggle_symptom.emit("near_faint".to_string()))
                                }}
                            />
                            <span>{"Near-faint / faint feeling"}</span>
                        </label>
                    </div>

                    <div class="footer-note">
                        {"Adult dehydration symptoms commonly include thirst, dark urine, dizziness/lightheadedness, fatigue, and dry mouth. Fainting, rapid pulse, or worsening symptoms deserve medical attention."}
                    </div>
                </section>

                <section class="card">
                    <h3>{"Today’s Log"}</h3>
                    <div class="history-list">
                        {
                            if state.entries.is_empty() {
                                html! {
                                    <div class="muted">{"No entries yet today."}</div>
                                }
                            } else {
                                html! {
                                    <>
                                        {for state.entries.iter().map(|entry| {
                                            let t = Date::new(&JsValue::from_f64(entry.timestamp_ms));
                                            let hh = t.get_hours();
                                            let mm = t.get_minutes();

                                            html! {
                                                <div class="history-item">
                                                    <span>{format!("+{} oz", entry.ounces)}</span>
                                                    <span class="muted">{format!("{:02}:{:02}", hh, mm)}</span>
                                                </div>
                                            }
                                        })}
                                    </>
                                }
                            }
                        }
                    </div>
                </section>
            </div>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}