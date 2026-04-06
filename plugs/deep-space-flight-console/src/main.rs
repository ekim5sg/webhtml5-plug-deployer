use yew::prelude::*;
use gloo::timers::callback::Interval;
use web_sys::{window, HtmlAudioElement};

const TICK_MS: u32 = 100;
const MOBILE_BREAKPOINT: f64 = 900.0;

const AUDIO_BLACKOUT_WAV: &str = "assets/audio/AUDIO_BLACKOUT_WAV.wav";
const AUDIO_DROGUE_WAV: &str = "assets/audio/AUDIO_DROGUE_WAV.wav";
const AUDIO_MAIN_WAV: &str = "assets/audio/AUDIO_MAIN_WAV.wav";
const AUDIO_SPLASH_WAV: &str = "assets/audio/AUDIO_SPLASH_WAV.wav";

#[derive(Clone, Copy, PartialEq)]
enum MobilePanel {
    Apollo,
    Center,
    Orion,
}

#[derive(Clone, Copy, PartialEq)]
enum MissionPhase {
    Launch,
    Orbit,
    Tli,
    Cruise,
    Flyby,
    Return,
}

impl MissionPhase {
    fn all() -> Vec<Self> {
        vec![
            Self::Launch,
            Self::Orbit,
            Self::Tli,
            Self::Cruise,
            Self::Flyby,
            Self::Return,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Launch => "Launch",
            Self::Orbit => "Orbit",
            Self::Tli => "TLI Burn",
            Self::Cruise => "Cruise",
            Self::Flyby => "Lunar Flyby",
            Self::Return => "Return",
        }
    }

    fn short_label(&self) -> &'static str {
        self.label()
    }
}

fn phase_from_index(idx: usize) -> MissionPhase {
    MissionPhase::all()[idx]
}

fn phase_start_time(p: MissionPhase) -> f64 {
    match p {
        MissionPhase::Launch => 0.0,
        MissionPhase::Orbit => 60.0,
        MissionPhase::Tli => 120.0,
        MissionPhase::Cruise => 180.0,
        MissionPhase::Flyby => 240.0,
        MissionPhase::Return => 300.0,
    }
}

fn mission_duration_total() -> f64 {
    360.0
}

fn window_width() -> f64 {
    window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|v| v.as_f64())
        .unwrap_or(1200.0)
}

#[derive(Default)]
struct AudioPool {
    blackout: Option<HtmlAudioElement>,
    drogue: Option<HtmlAudioElement>,
    main: Option<HtmlAudioElement>,
    splash: Option<HtmlAudioElement>,
    is_primed: bool,
}

fn build_audio(src: &str) -> Option<HtmlAudioElement> {
    let audio = HtmlAudioElement::new_with_src(src).ok()?;
    audio.set_preload("auto");
    Some(audio)
}

fn prime_audio(audio: &HtmlAudioElement) {
    audio.set_muted(true);
    let _ = audio.play();
    audio.pause().ok();
    audio.set_current_time(0.0);
    audio.set_muted(false);
}

fn init_and_prime_pool(pool: &mut AudioPool) -> bool {
    if pool.blackout.is_none() {
        pool.blackout = build_audio(AUDIO_BLACKOUT_WAV);
    }
    if pool.drogue.is_none() {
        pool.drogue = build_audio(AUDIO_DROGUE_WAV);
    }
    if pool.main.is_none() {
        pool.main = build_audio(AUDIO_MAIN_WAV);
    }
    if pool.splash.is_none() {
        pool.splash = build_audio(AUDIO_SPLASH_WAV);
    }

    if let Some(a) = &pool.blackout { prime_audio(a); }
    if let Some(a) = &pool.drogue { prime_audio(a); }
    if let Some(a) = &pool.main { prime_audio(a); }
    if let Some(a) = &pool.splash { prime_audio(a); }

    pool.is_primed =
        pool.blackout.is_some() &&
        pool.drogue.is_some() &&
        pool.main.is_some() &&
        pool.splash.is_some();

    pool.is_primed
}

fn play_from_pool(pool: &AudioPool, key: &str) -> bool {
    let target = match key {
        "blackout" => pool.blackout.as_ref(),
        "drogue" => pool.drogue.as_ref(),
        "main" => pool.main.as_ref(),
        "splash" => pool.splash.as_ref(),
        _ => None,
    };

    if let Some(audio) = target {
        audio.set_current_time(0.0);
        let _ = audio.play();
        true
    } else {
        false
    }
}

fn cue_for_phase(phase: MissionPhase) -> &'static str {
    match phase {
        MissionPhase::Launch => "blackout",
        MissionPhase::Orbit => "drogue",
        MissionPhase::Tli => "main",
        MissionPhase::Cruise => "blackout",
        MissionPhase::Flyby => "drogue",
        MissionPhase::Return => "splash",
    }
}

fn cue_label(key: &str) -> &'static str {
    match key {
        "blackout" => "BLACKOUT",
        "drogue" => "DROGUE",
        "main" => "MAIN",
        "splash" => "SPLASH",
        _ => "UNKNOWN",
    }
}

#[function_component(App)]
fn app() -> Html {
    let time_s = use_state(|| 0.0);
    let playing = use_state(|| true);
    let viewport_width = use_state(window_width);
    let mobile_panel = use_state(|| MobilePanel::Center);

    let audio_enabled = use_state(|| false);
    let audio_status = use_state(|| "Audio locked. Tap Enable Audio first on iPhone.".to_string());

    let audio_pool_ref = use_mut_ref(AudioPool::default);
    let last_phase_ref = use_mut_ref(|| MissionPhase::Launch);

    {
        let time_s = time_s.clone();
        let playing = playing.clone();

        use_effect_with((), move |_| {
            let interval = Interval::new(TICK_MS, move || {
                if *playing {
                    let mut t = *time_s + 1.0;
                    if t > mission_duration_total() {
                        t = mission_duration_total();
                    }
                    time_s.set(t);
                }
            });
            move || drop(interval)
        });
    }

    let state_phase = {
        let t = *time_s;
        if t < 60.0 {
            MissionPhase::Launch
        } else if t < 120.0 {
            MissionPhase::Orbit
        } else if t < 180.0 {
            MissionPhase::Tli
        } else if t < 240.0 {
            MissionPhase::Cruise
        } else if t < 300.0 {
            MissionPhase::Flyby
        } else {
            MissionPhase::Return
        }
    };

    let phase_idx = MissionPhase::all()
        .iter()
        .position(|p| *p == state_phase)
        .unwrap_or(0);

    {
        let audio_enabled = audio_enabled.clone();
        let audio_status = audio_status.clone();
        let audio_pool_ref = audio_pool_ref.clone();
        let last_phase_ref = last_phase_ref.clone();
        let current_phase = state_phase;

        use_effect_with((current_phase, *audio_enabled), move |(phase, enabled)| {
            let mut last_phase = last_phase_ref.borrow_mut();

            if *enabled && *phase != *last_phase {
                let cue = cue_for_phase(*phase);
                let pool = audio_pool_ref.borrow();

                if pool.is_primed && play_from_pool(&pool, cue) {
                    audio_status.set(format!("Cue fired: {} ({})", cue_label(cue), phase.label()));
                } else {
                    audio_status.set(format!("Audio enabled, but cue could not play for {}", phase.label()));
                }
            }

            *last_phase = *phase;
            || ()
        });
    }

    let on_enable_audio = {
        let audio_enabled = audio_enabled.clone();
        let audio_status = audio_status.clone();
        let audio_pool_ref = audio_pool_ref.clone();

        Callback::from(move |_| {
            let ok = {
                let mut pool = audio_pool_ref.borrow_mut();
                init_and_prime_pool(&mut pool)
            };

            audio_enabled.set(ok);
            if ok {
                audio_status.set("Audio enabled and primed for iPhone-safe playback.".to_string());
            } else {
                audio_status.set("Audio enable failed. Check assets/audio paths.".to_string());
            }
        })
    };

    let on_test_audio = {
        let audio_enabled = audio_enabled.clone();
        let audio_status = audio_status.clone();
        let audio_pool_ref = audio_pool_ref.clone();

        Callback::from(move |_| {
            if !*audio_enabled {
                audio_status.set("Tap Enable Audio first.".to_string());
                return;
            }

            let pool = audio_pool_ref.borrow();
            if play_from_pool(&pool, "splash") {
                audio_status.set("Test cue fired: SPLASH".to_string());
            } else {
                audio_status.set("Test cue failed. Check assets/audio files.".to_string());
            }
        })
    };

    let on_jump = {
        let time_s = time_s.clone();
        let playing = playing.clone();

        Callback::from(move |idx: usize| {
            playing.set(false);
            time_s.set(phase_start_time(phase_from_index(idx)));
        })
    };

    let on_toggle_play = {
        let playing = playing.clone();
        Callback::from(move |_| playing.set(!*playing))
    };

    let is_mobile = *viewport_width < MOBILE_BREAKPOINT;

    html! {
        <div style="padding: 16px; font-family: Arial, sans-serif;">
            <h1>{"Apollo vs Orion — iPhone-safe audio patch"}</h1>

            <div style="display:flex; gap:8px; flex-wrap:wrap; margin-bottom:12px;">
                <button type="button" onclick={on_enable_audio.clone()}>{"Enable Audio"}</button>
                <button type="button" onclick={on_test_audio.clone()}>{"Test Audio"}</button>
                <button
                    type="button"
                    onclick={on_toggle_play}
                >
                    { if *playing { "Pause" } else { "Play" } }
                </button>
            </div>

            <div style="margin-bottom:12px; padding:10px; border:1px solid #999;">
                { format!(
                    "Audio: {} | {}",
                    if *audio_enabled { "ENABLED" } else { "LOCKED" },
                    (*audio_status).clone()
                ) }
            </div>

            <div style="margin-bottom:12px;">
                {
                    MissionPhase::all().iter().enumerate().map(|(idx, phase)| {
                        let on_jump = on_jump.clone();
                        html! {
                            <button
                                type="button"
                                style="margin-right:8px; margin-bottom:8px;"
                                onclick={Callback::from(move |_| on_jump.emit(idx))}
                            >
                                { phase.short_label() }
                            </button>
                        }
                    }).collect::<Html>()
                }
            </div>

            <div style="margin-bottom:10px;">
                { format!("Current Phase: {}", state_phase.label()) }
            </div>

            <div style="margin-bottom:10px;">
                { format!("Mission Time: {:.0}s / {:.0}s", *time_s, mission_duration_total()) }
            </div>

            <div style="margin-bottom:10px;">
                { format!("Viewport: {} | Mobile layout: {}", window_width() as i32, if is_mobile { "yes" } else { "no" }) }
            </div>

            <div style="margin-bottom:10px;">
                { format!("Mobile Panel: {}", match *mobile_panel {
                    MobilePanel::Apollo => "Apollo",
                    MobilePanel::Center => "Center",
                    MobilePanel::Orion => "Orion",
                })}
            </div>

            <p>
                {"This patch keeps audio elements alive in a persistent pool and primes them during a direct user gesture. That is much more reliable on iPhone Chrome than creating a fresh HtmlAudioElement later inside a timer/effect."}
            </p>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}