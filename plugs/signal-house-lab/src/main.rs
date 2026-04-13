use yew::prelude::*;
use gloo::timers::callback::Interval;
use gloo_storage::{LocalStorage, Storage};
use web_sys::{HtmlAudioElement, HtmlInputElement};

fn play(src: &str) {
    if let Ok(a) = HtmlAudioElement::new_with_src(src) {
        let _ = a.play();
    }
}

#[function_component(App)]
fn app() -> Html {

    let signals = use_state(|| 0);
    let active = use_state(|| false);
    let best: UseStateHandle<i32> =
        use_state(|| LocalStorage::get("best").unwrap_or(0));

    let play_signal = {
        let signals = signals.clone();
        let active = active.clone();
        let best = best.clone();

        Callback::from(move |_| {
            let val = *signals + 1;
            signals.set(val);
            active.set(true);

            if val > *best {
                best.set(val);
                let _ = LocalStorage::set("best", val);
            }

            play("assets/audio/ping.wav");

            let active_clone = active.clone();
            gloo::timers::callback::Timeout::new(200, move || {
                active_clone.set(false);
            }).forget();
        })
    };

    html! {
        <div class="container">

            <h1>{"🌌 Signal House Lab v3"}</h1>

            <div class={classes!("house", if *active { "active" } else { "" })}></div>

            <div class="beam"></div>

            <div class="card">
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