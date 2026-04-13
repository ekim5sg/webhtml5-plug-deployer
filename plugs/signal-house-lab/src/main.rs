use yew::prelude::*;
use web_sys::HtmlAudioElement;
use gloo_storage::{LocalStorage, Storage};
use gloo::timers::callback::Timeout;

#[function_component(App)]
fn app() -> Html {

    // GAME STATE
    let signals = use_state(|| 0);
    let active = use_state(|| false);
    let audio_started = use_state(|| false);

    let best: UseStateHandle<i32> =
        use_state(|| LocalStorage::get("best_score").unwrap_or(0));

    // AUDIO OBJECTS (REUSED — IMPORTANT)
    let ping = use_state(|| HtmlAudioElement::new_with_src("assets/audio/ping.wav").unwrap());
    let activate = use_state(|| HtmlAudioElement::new_with_src("assets/audio/activate.wav").unwrap());
    let ambient = use_state(|| {
        let a = HtmlAudioElement::new_with_src("assets/audio/ambient.wav").unwrap();
        a.set_loop(true);
        a
    });

    // PLAY FUNCTION (NO OVERLAP BUGS)
    let play = |audio: &HtmlAudioElement| {
        let _ = audio.set_current_time(0.0);
        let _ = audio.play();
    };

    // SIGNAL BUTTON
    let play_signal = {
        let signals = signals.clone();
        let active = active.clone();
        let best = best.clone();
        let ping = ping.clone();
        let ambient = ambient.clone();
        let audio_started = audio_started.clone();

        Callback::from(move |_| {

            // START AMBIENT ON FIRST TAP (iPhone SAFE)
            if !*audio_started {
                let _ = ambient.play();
                audio_started.set(true);
            }

            let new_val = *signals + 1;
            signals.set(new_val);
            active.set(true);

            if new_val > *best {
                best.set(new_val);
                let _ = LocalStorage::set("best_score", new_val);
            }

            play(&ping);

            let active_clone = active.clone();
            Timeout::new(200, move || {
                active_clone.set(false);
            }).forget();
        })
    };

    // ACTIVATE BUTTON
    let activate_mode = {
        let activate = activate.clone();

        Callback::from(move |_| {
            play(&activate);
        })
    };

    html! {
        <div class="container">

            <h1>{"🌌 Signal House Lab v3"}</h1>

            <div class={classes!("house", if *active { "active" } else { "" })}></div>
            <div class="beam"></div>

            <div class="card">
                <button onclick={activate_mode.clone()}>
                    {"Start Mission"}
                </button>

                <button onclick={play_signal}>
                    {"Send Signal"}
                </button>

                <p class="big">{format!("Signals: {}", *signals)}</p>
                <p>{format!("Best: {}", *best)}</p>
            </div>

        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}