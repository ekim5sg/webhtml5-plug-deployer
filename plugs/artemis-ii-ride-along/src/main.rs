use gloo::storage::{LocalStorage, Storage};
use gloo::timers::callback::Interval;
use gloo_net::http::Request;
use js_sys::Date;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, HtmlAudioElement};
use yew::prelude::*;

const APP_TITLE: &str = "Artemis II: Ride Along 🚀";
const BRAND: &str = "MikeGyver Studio";
const SHARE_URL: &str = "https://www.webhtml5.info/artemis-ii-ride-along/";
const BADGE_KEY: &str = "artemis_ii_badges_v1";
const MILESTONES_URL: &str = "assets/data/milestones.json";

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
struct Phase {
    name: String,
    iso: String,
    narrative: String,
    kid_text: String,
    location: String,
    note: String,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, Debug)]
struct MissionConfig {
    mission_name: String,
    launch_iso: String,
    source_note: Option<String>,
    phases: Vec<Phase>,
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
struct BadgeState {
    launch_witness: bool,
    deep_space_rider: bool,
    moon_flyby_crew: bool,
    splashdown_seen: bool,
}

impl BadgeState {
    fn load() -> Self {
        LocalStorage::get(BADGE_KEY).unwrap_or_default()
    }

    fn save(&self) {
        let _ = LocalStorage::set(BADGE_KEY, self);
    }
}

fn fallback_config() -> MissionConfig {
    MissionConfig {
        mission_name: "Artemis II".to_string(),
        launch_iso: "2026-04-01T22:35:00Z".to_string(),
        source_note: Some("Fallback mission config".to_string()),
        phases: vec![
            Phase {
                name: "Launch".to_string(),
                iso: "2026-04-01T22:35:00Z".to_string(),
                narrative: "The Space Launch System thundered off the pad and Orion began NASA’s first crewed journey beyond low Earth orbit since Apollo.".to_string(),
                kid_text: "Launch is the giant push off the starting line.".to_string(),
                location: "Leaving Earth".to_string(),
                note: "SLS and Orion lift off from Kennedy Space Center.".to_string(),
            },
            Phase {
                name: "Orbit Checkout".to_string(),
                iso: "2026-04-02T01:35:00Z".to_string(),
                narrative: "Now in orbit, the crew and mission teams verify spacecraft health and readiness.".to_string(),
                kid_text: "This is like checking your backpack before the big trip.".to_string(),
                location: "Earth Orbit".to_string(),
                note: "Crew and flight controllers verify Orion systems in Earth orbit.".to_string(),
            },
            Phase {
                name: "Trans-Lunar Injection".to_string(),
                iso: "2026-04-02T04:35:00Z".to_string(),
                narrative: "This is the commitment burn that sends Orion toward the Moon.".to_string(),
                kid_text: "This is the slingshot that sends the spacecraft toward the Moon.".to_string(),
                location: "Departing Earth".to_string(),
                note: "The TLI burn sends Orion onto its lunar path.".to_string(),
            },
            Phase {
                name: "Deep Space Coast".to_string(),
                iso: "2026-04-02T07:35:00Z".to_string(),
                narrative: "Orion coasts through deep space as teams monitor the journey.".to_string(),
                kid_text: "They stay on the right invisible road through space.".to_string(),
                location: "Deep Space".to_string(),
                note: "Orion coasts between Earth and Moon.".to_string(),
            },
            Phase {
                name: "Lunar Flyby".to_string(),
                iso: "2026-04-04T22:35:00Z".to_string(),
                narrative: "Orion loops around the Moon and begins setting up for the return.".to_string(),
                kid_text: "The spacecraft loops around the Moon instead of landing.".to_string(),
                location: "Near Moon".to_string(),
                note: "Artemis II loops around the Moon and starts the journey home.".to_string(),
            },
            Phase {
                name: "Return Burn".to_string(),
                iso: "2026-04-08T22:35:00Z".to_string(),
                narrative: "The mission transitions to Earth return operations.".to_string(),
                kid_text: "Going home still takes careful teamwork.".to_string(),
                location: "Heading Home".to_string(),
                note: "The mission transitions from lunar return to Earth approach.".to_string(),
            },
            Phase {
                name: "Splashdown".to_string(),
                iso: "2026-04-11T22:35:00Z".to_string(),
                narrative: "Orion returns to Earth and splashes down to complete the mission.".to_string(),
                kid_text: "The mission ends with a splash in the ocean.".to_string(),
                location: "Earth Return".to_string(),
                note: "Orion splashes down to complete the mission.".to_string(),
            },
        ],
    }
}

fn parse_ms(iso: &str) -> f64 {
    Date::new(&JsValue::from_str(iso)).get_time()
}

fn now_ms() -> f64 {
    Date::new_0().get_time()
}

fn format_met(ms: f64) -> String {
    if ms <= 0.0 {
        return "T-00:00:00".to_string();
    }

    let total_seconds = (ms / 1000.0).floor() as i64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("T+{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn format_countdown(ms: f64) -> String {
    let total_seconds = (ms / 1000.0).max(0.0).floor() as i64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn phase_index(phases: &[Phase], now: f64) -> usize {
    let mut current = 0usize;

    for (idx, phase) in phases.iter().enumerate() {
        if now >= parse_ms(&phase.iso) {
            current = idx;
        }
    }

    current
}

fn copy_share_text(text: &str) {
    if let Some(win) = window() {
        let clipboard = win.navigator().clipboard();
        let _ = clipboard.write_text(text);
    }
}

fn try_play_audio() {
    if let Ok(audio) = HtmlAudioElement::new_with_src("assets/audio/space-chime.wav") {
        audio.set_volume(0.9);
        let _ = audio.play();
    }
}

#[function_component(App)]
fn app() -> Html {
    let now = use_state(now_ms);
    let kid_mode = use_state(|| false);
    let share_copied = use_state(|| false);
    let audio_enabled = use_state(|| false);
    let badge_state = use_state(BadgeState::load);
    let config = use_state(|| None::<MissionConfig>);
    let load_error = use_state(|| None::<String>);

    {
        let now = now.clone();
        use_effect_with((), move |_| {
            let interval = Interval::new(1000, move || {
                now.set(now_ms());
            });

            move || drop(interval)
        });
    }

    {
        let config = config.clone();
        let load_error = load_error.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                match Request::get(MILESTONES_URL).send().await {
                    Ok(response) => match response.json::<MissionConfig>().await {
                        Ok(parsed) => config.set(Some(parsed)),
                        Err(err) => {
                            load_error.set(Some(format!("JSON parse error: {err}")));
                            config.set(Some(fallback_config()));
                        }
                    },
                    Err(err) => {
                        load_error.set(Some(format!("Fetch error: {err}")));
                        config.set(Some(fallback_config()));
                    }
                }
            });

            || {}
        });
    }

    let current_config = (*config).clone().unwrap_or_else(fallback_config);

    let phases = current_config.phases.clone();
    let launch_ms = parse_ms(&current_config.launch_iso);
    let met_ms = (*now - launch_ms).max(0.0);
    let idx = phase_index(&phases, *now);
    let active = phases[idx].clone();

    {
        let badge_state = badge_state.clone();
        let phases_for_badges = phases.clone();
        let now_value = *now;

        use_effect_with((now_value, phases_for_badges.clone()), move |_| {
            let mut next = (*badge_state).clone();

            if let Some(phase) = phases_for_badges.get(0) {
                if now_value >= parse_ms(&phase.iso) {
                    next.launch_witness = true;
                }
            }

            if let Some(phase) = phases_for_badges.iter().find(|p| p.name == "Deep Space Coast") {
                if now_value >= parse_ms(&phase.iso) {
                    next.deep_space_rider = true;
                }
            }

            if let Some(phase) = phases_for_badges.iter().find(|p| p.name == "Lunar Flyby") {
                if now_value >= parse_ms(&phase.iso) {
                    next.moon_flyby_crew = true;
                }
            }

            if let Some(phase) = phases_for_badges.iter().find(|p| p.name == "Splashdown") {
                if now_value >= parse_ms(&phase.iso) {
                    next.splashdown_seen = true;
                }
            }

            if next != *badge_state {
                next.save();
                badge_state.set(next);
            }

            || {}
        });
    }

    let next_phase = phases.get(idx + 1).cloned();
    let next_phase_countdown = next_phase
        .as_ref()
        .map(|p| format_countdown((parse_ms(&p.iso) - *now).max(0.0)));

    let toggle_kid = {
        let kid_mode = kid_mode.clone();
        Callback::from(move |_| {
            kid_mode.set(!*kid_mode);
        })
    };

    let on_enable_audio = {
        let audio_enabled = audio_enabled.clone();
        Callback::from(move |_| {
            try_play_audio();
            audio_enabled.set(true);
        })
    };

    let on_share = {
        let share_copied = share_copied.clone();
        let met = format_met(met_ms);

        Callback::from(move |_| {
            let text = format!(
                "I’m riding Artemis II 🚀 {} into the mission. Join the journey: {}",
                met, SHARE_URL
            );
            copy_share_text(&text);
            share_copied.set(true);
        })
    };

    html! {
        <div class="app-shell">
            <section class="hero">
                <div class="brand">{ BRAND }</div>
                <h1>{ APP_TITLE }</h1>
                <p>
                    { "A cinematic mission companion following Artemis II from launch to lunar flyby and home again — with live mission elapsed time, story mode, kid-friendly STEM, and milestone tracking from JSON." }
                </p>

                <div class="controls">
                    <button class="primary" onclick={on_share}>
                        { if *share_copied { "Share text copied" } else { "Copy share text" } }
                    </button>

                    <button class="secondary" onclick={toggle_kid}>
                        { if *kid_mode { "Kid Mode: ON" } else { "Kid Mode: OFF" } }
                    </button>

                    <button class="ghost" onclick={on_enable_audio}>
                        { if *audio_enabled { "Audio ready" } else { "Enable audio" } }
                    </button>
                </div>
            </section>

            <section class="top-grid">
                <div class="card">
                    <div class="label">{ "Mission Elapsed Time" }</div>
                    <div class="met">{ format_met(met_ms) }</div>
                    <div class="met-sub">
                        { format!("Launch reference: {}", current_config.launch_iso) }
                    </div>
                    <div class="status-pill">
                        <span>{ "Current location:" }</span>
                        <strong>{ active.location.clone() }</strong>
                    </div>
                </div>

                <div class="card">
                    <div class="label">{ "Current Phase" }</div>
                    <div class="phase-name">{ active.name.clone() }</div>
                    <div class="phase-description">{ active.note.clone() }</div>

                    {
                        if let Some(next) = next_phase {
                            html! {
                                <>
                                    <div class="label" style="margin-top:16px;">{ "Next Event" }</div>
                                    <div class="small-text">
                                        { format!("{} in {}", next.name, next_phase_countdown.unwrap_or_else(|| "00:00:00".to_string())) }
                                    </div>
                                </>
                            }
                        } else {
                            html! {
                                <div class="small-text" style="margin-top:16px;">
                                    { "Final major mission phase reached." }
                                </div>
                            }
                        }
                    }
                </div>
            </section>

            <section class="bottom-grid">
                <div class="card">
                    <div class="label">{ "Story Mode" }</div>
                    <div class="story-text">{ active.narrative.clone() }</div>
                </div>

                <div class="card">
                    <div class="label">
                        { if *kid_mode { "Kid Mode: Mission Explain" } else { "Mission Explain" } }
                    </div>
                    <div class="kid-text">
                        {
                            if *kid_mode {
                                active.kid_text.clone()
                            } else {
                                "Toggle Kid Mode to switch the mission explainer into a younger-reader STEM voice.".to_string()
                            }
                        }
                    </div>
                </div>

                <div class="card">
                    <div class="label">{ "Mission Badges" }</div>
                    <div class="badge-row">
                        <div class={classes!("badge", badge_state.launch_witness.then_some("unlocked"))}>
                            { "Launch Witness" }
                        </div>
                        <div class={classes!("badge", badge_state.deep_space_rider.then_some("unlocked"))}>
                            { "Deep Space Rider" }
                        </div>
                        <div class={classes!("badge", badge_state.moon_flyby_crew.then_some("unlocked"))}>
                            { "Moon Flyby Crew" }
                        </div>
                        <div class={classes!("badge", badge_state.splashdown_seen.then_some("unlocked"))}>
                            { "Splashdown Seen" }
                        </div>
                    </div>
                    <div class="share-text" style="margin-top:14px;">
                        { "Badges are saved locally so returning visitors can keep their mission progress." }
                    </div>
                </div>
            </section>

            <section class="card" style="margin-top:18px;">
                <div class="label">{ "Mission Timeline" }</div>
                <div class="timeline">
                    {
                        phases.iter().enumerate().map(|(phase_idx, phase)| {
                            let phase_ms = parse_ms(&phase.iso);

                            let class_name = if *now >= phase_ms {
                                if phase_idx == idx {
                                    classes!("timeline-item", "active")
                                } else {
                                    classes!("timeline-item", "done")
                                }
                            } else {
                                classes!("timeline-item")
                            };

                            html! {
                                <div class={class_name}>
                                    <div class="timeline-top">
                                        <div class="timeline-title">{ phase.name.clone() }</div>
                                        <div class="timeline-time">{ phase.iso.clone() }</div>
                                    </div>
                                    <div class="timeline-note">{ phase.note.clone() }</div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
            </section>

            <section class="card" style="margin-top:18px;">
                <div class="label">{ "Config Source" }</div>
                <div class="small-text">{ format!("Loaded from: {}", MILESTONES_URL) }</div>
                {
                    if let Some(note) = current_config.source_note.clone() {
                        html! { <div class="small-text" style="margin-top:8px;">{ note }</div> }
                    } else {
                        html! {}
                    }
                }
                {
                    if let Some(err) = (*load_error).clone() {
                        html! { <div class="small-text" style="margin-top:8px;">{ format!("Using fallback config because: {}", err) }</div> }
                    } else {
                        html! {}
                    }
                }
            </section>

            <div class="footer-brand">{ "MikeGyver Studio" }</div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}