use gloo::storage::{LocalStorage, Storage};
use gloo::timers::callback::Interval;
use js_sys::Date;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::{window, HtmlAudioElement};
use yew::prelude::*;

const APP_TITLE: &str = "Artemis II: Ride Along 🚀";
const BRAND: &str = "MikeGyver Studio";
const SHARE_URL: &str = "https://www.webhtml5.info/artemis-ii-ride-along/";
const BADGE_KEY: &str = "artemis_ii_badges_v1";
const LAUNCH_ISO: &str = "2026-04-01T23:24:00Z";

#[derive(Clone, PartialEq)]
struct Phase {
    name: &'static str,
    iso: &'static str,
    narrative: &'static str,
    kid_text: &'static str,
    location: &'static str,
    note: &'static str,
}

fn mission_phases() -> Vec<Phase> {
    vec![
        Phase {
            name: "Launch",
            iso: "2026-04-01T23:24:00Z",
            narrative: "The Space Launch System roared off the pad and Orion began humanity’s next crewed journey beyond low Earth orbit. Artemis II did not just leave Earth — it reopened a path that had been quiet since Apollo.",
            kid_text: "Launch is the giant push off the starting line. The rocket gives Orion enough power to climb away from Earth.",
            location: "Leaving Earth",
            note: "SLS and Orion lift off from Kennedy Space Center.",
        },
        Phase {
            name: "Orbit Checkout",
            iso: "2026-04-02T02:24:00Z",
            narrative: "Now in orbit, the crew and mission teams verify the spacecraft’s health, systems, power, communications, and readiness for the much bigger step ahead.",
            kid_text: "Think of this like checking your backpack before the big field trip bus leaves town.",
            location: "Earth Orbit",
            note: "Crew and ground teams verify the spacecraft before heading outward.",
        },
        Phase {
            name: "Trans-Lunar Injection",
            iso: "2026-04-02T05:24:00Z",
            narrative: "This is the commitment burn. Orion fires at just the right time to stop circling Earth and begin the long arc toward the Moon.",
            kid_text: "TLI is like taking the perfect slingshot shot so you leave the playground and head for the next world.",
            location: "Departing Earth",
            note: "The TLI burn sends Orion out of Earth orbit and onto its lunar path.",
        },
        Phase {
            name: "Deep Space Coast",
            iso: "2026-04-02T08:24:00Z",
            narrative: "With Earth shrinking behind them, the crew enters the quiet, precise work of translunar flight. Navigation, communication, life support, and trajectory all matter here. This is where exploration becomes discipline.",
            kid_text: "They are not just flying fast — they are staying on the right invisible road in space.",
            location: "Deep Space",
            note: "Orion coasts between Earth and Moon while teams watch trajectory and systems.",
        },
        Phase {
            name: "Lunar Flyby",
            iso: "2026-04-04T23:24:00Z",
            narrative: "At lunar flyby, Orion uses the Moon’s gravity and its own precise path to sweep around the far side and prove the systems, navigation, and human readiness needed for the missions that follow.",
            kid_text: "The spacecraft swings around the Moon instead of landing — like rounding a cone on a giant racetrack.",
            location: "Near Moon",
            note: "Artemis II loops around the Moon and sets up for the journey home.",
        },
        Phase {
            name: "Return Burn",
            iso: "2026-04-07T23:24:00Z",
            narrative: "Home is the target now. Reentry planning, spacecraft health, and trajectory control all lead toward the fiery return through Earth’s atmosphere.",
            kid_text: "Coming home from the Moon still takes teamwork, math, and careful flying.",
            location: "Heading Home",
            note: "The mission transitions from lunar return to Earth approach.",
        },
    ]
}

#[derive(Clone, PartialEq, Default, Serialize, Deserialize)]
struct BadgeState {
    launch_witness: bool,
    deep_space_rider: bool,
    moon_flyby_crew: bool,
}

impl BadgeState {
    fn load() -> Self {
        LocalStorage::get(BADGE_KEY).unwrap_or_default()
    }

    fn save(&self) {
        let _ = LocalStorage::set(BADGE_KEY, self);
    }
}

fn parse_ms(iso: &str) -> f64 {
    Date::new(&JsValue::from_str(iso)).get_time()
}

fn now_ms() -> f64 {
    Date::new_0().get_time()
}

fn format_duration(ms: f64) -> String {
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
        if now >= parse_ms(phase.iso) {
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

    {
        let now = now.clone();
        use_effect_with((), move |_| {
            let interval = Interval::new(1000, move || {
                now.set(now_ms());
            });

            move || drop(interval)
        });
    }

    let phases = mission_phases();
    let launch_ms = parse_ms(LAUNCH_ISO);
    let met_ms = (*now - launch_ms).max(0.0);
    let idx = phase_index(&phases, *now);
    let active = phases[idx].clone();

    {
        let badge_state = badge_state.clone();
        let now_value = *now;

        use_effect_with(now_value, move |_| {
            let mut next = (*badge_state).clone();
            let phases = mission_phases();

            if now_value >= parse_ms(phases[0].iso) {
                next.launch_witness = true;
            }
            if now_value >= parse_ms(phases[3].iso) {
                next.deep_space_rider = true;
            }
            if now_value >= parse_ms(phases[4].iso) {
                next.moon_flyby_crew = true;
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
        .map(|p| format_countdown((parse_ms(p.iso) - *now).max(0.0)));

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
        let met = format_duration(met_ms);

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
                    { "A cinematic mission companion following Artemis II from launch to lunar flyby and home again — with real mission elapsed time, story mode, kid-friendly STEM, and milestone tracking." }
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
                    <div class="met">{ format_duration(met_ms) }</div>
                    <div class="met-sub">
                        { format!("Launch reference: {}", LAUNCH_ISO) }
                    </div>
                    <div class="status-pill">
                        <span>{ "Current location:" }</span>
                        <strong>{ active.location }</strong>
                    </div>
                </div>

                <div class="card">
                    <div class="label">{ "Current Phase" }</div>
                    <div class="phase-name">{ active.name }</div>
                    <div class="phase-description">{ active.note }</div>

                    {
                        if let Some(next) = next_phase {
                            html! {
                                <>
                                    <div class="label" style="margin-top:16px;">{ "Next Event" }</div>
                                    <div class="small-text">
                                        {
                                            format!(
                                                "{} in {}",
                                                next.name,
                                                next_phase_countdown.unwrap_or_else(|| "00:00:00".to_string())
                                            )
                                        }
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
                    <div class="story-text">{ active.narrative }</div>
                </div>

                <div class="card">
                    <div class="label">
                        { if *kid_mode { "Kid Mode: Mission Explain" } else { "Mission Explain" } }
                    </div>
                    <div class="kid-text">
                        {
                            if *kid_mode {
                                active.kid_text
                            } else {
                                "Toggle Kid Mode to switch the mission explainer into a younger-reader STEM voice."
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
                            let phase_ms = parse_ms(phase.iso);

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
                                        <div class="timeline-title">{ phase.name }</div>
                                        <div class="timeline-time">{ phase.iso }</div>
                                    </div>
                                    <div class="timeline-note">{ phase.note }</div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
            </section>

            <div class="footer-brand">{ "MikeGyver Studio" }</div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}