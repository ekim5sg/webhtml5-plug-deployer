use yew::prelude::*;
use web_sys::{HtmlAudioElement, HtmlCanvasElement, CanvasRenderingContext2d};
use gloo_storage::{LocalStorage, Storage};
use gloo::timers::callback::{Interval, Timeout};
use wasm_bindgen::JsCast;

fn play(audio: &HtmlAudioElement) {
    let _ = audio.set_current_time(0.0);
    let _ = audio.play();
}

fn init_stars() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("stars").unwrap()
        .dyn_into::<HtmlCanvasElement>().unwrap();

    let ctx = canvas.get_context("2d").unwrap().unwrap()
        .dyn_into::<CanvasRenderingContext2d>().unwrap();

    canvas.set_width(window.inner_width().unwrap().as_f64().unwrap() as u32);
    canvas.set_height(window.inner_height().unwrap().as_f64().unwrap() as u32);

    Interval::new(50, move || {
        ctx.set_fill_style(&"#0b1020".into());
        ctx.fill_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

        for _ in 0..50 {
            let x = js_sys::Math::random() * canvas.width() as f64;
            let y = js_sys::Math::random() * canvas.height() as f64;
            ctx.set_fill_style(&"#ffffff".into());
            ctx.fill_rect(x, y, 2.0, 2.0);
        }
    }).forget();
}

#[function_component(App)]
fn app() -> Html {

    let signals = use_state(|| 0);
    let active = use_state(|| false);
    let audio_started = use_state(|| false);
    let story_step = use_state(|| 0);

    let best: UseStateHandle<i32> =
        use_state(|| LocalStorage::get("best").unwrap_or(0));

    let ping = use_state(|| HtmlAudioElement::new_with_src("assets/audio/ping.wav").unwrap());
    let activate = use_state(|| HtmlAudioElement::new_with_src("assets/audio/activate.wav").unwrap());
    let ambient = use_state(|| {
        let a = HtmlAudioElement::new_with_src("assets/audio/ambient.wav").unwrap();
        a.set_loop(true);
        a
    });

    {
        use_effect(|| {
            init_stars();
            || ()
        });
    }

    let send_signal = {
        let signals = signals.clone();
        let active = active.clone();
        let best = best.clone();
        let ping = ping.clone();
        let ambient = ambient.clone();
        let audio_started = audio_started.clone();

        Callback::from(move |_| {

            if !*audio_started {
                let _ = ambient.play();
                audio_started.set(true);
            }

            let val = *signals + 1;
            signals.set(val);
            active.set(true);

            if val > *best {
                best.set(val);
                let _ = LocalStorage::set("best", val);
            }

            play(&ping);

            let active_clone = active.clone();
            Timeout::new(200, move || active_clone.set(false)).forget();
        })
    };

    let start_story = {
        let story_step = story_step.clone();
        let activate = activate.clone();

        Callback::from(move |_| {
            story_step.set(0);
            play(&activate);
        })
    };

    let next_story = {
        let story_step = story_step.clone();
        Callback::from(move |_| story_step.set(*story_step + 1))
    };

    let story = vec![
        "This began with Colin’s drawings...",
        "A house. A signal. A dream.",
        "The Signal House connects Earth to space.",
        "Now you're part of the mission."
    ];

    html! {
        <div class="container">

            <h1>{"🌌 Signal House Lab v4"}</h1>

            <div class={classes!("house", if *active { "active" } else { "" })}></div>

            <div class="card">
                <button onclick={send_signal}>{"Send Signal"}</button>
                <button onclick={start_story}>{"Story Mode"}</button>

                <p class="big">{format!("Signals: {}", *signals)}</p>
                <p>{format!("Best: {}", *best)}</p>
            </div>

            <div class="card">
                <h2>{"Story"}</h2>
                <p>{story.get(*story_step).unwrap_or(&"Mission Complete")}</p>
                <button onclick={next_story}>{"Next"}</button>
            </div>

        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}