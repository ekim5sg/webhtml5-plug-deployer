use web_sys::HtmlAudioElement;
use yew::prelude::*;

fn status_message(score: i32) -> &'static str {
    if score < 3 {
        "Systems waking up"
    } else if score < 6 {
        "Fueling in progress"
    } else if score < 9 {
        "Mission almost ready"
    } else {
        "Launch ready"
    }
}

#[function_component(App)]
fn app() -> Html {
    let score = use_state(|| 0);
    let victory_played = use_state(|| false);

    {
        let score = score.clone();
        let victory_played = victory_played.clone();

        use_effect_with(((*score), (*victory_played)), move |(score, already_played)| {
            if *score >= 9 && !*already_played {
                if let Ok(audio) = HtmlAudioElement::new_with_src("assets/audio/lesson-victory.wav") {
                    let _ = audio.play();
                    victory_played.set(true);
                }
            }

            || ()
        });
    }

    let on_increase = {
        let score = score.clone();
        Callback::from(move |_| score.set(*score + 1))
    };

    let on_reset = {
        let score = score.clone();
        let victory_played = victory_played.clone();

        Callback::from(move |_| {
            score.set(0);
            victory_played.set(false);
        })
    };

    html! {
        <main class="page">
            <section class="card">
                <div class="topline">{ "Rust • Yew • WebAssembly" }</div>

                <h1 class="title">{ "Launch Readiness Meter" }</h1>

                <p class="subtitle">
                    { "Click the button to raise the score. Rust uses simple if/else logic to decide which status message to show." }
                </p>

                <section class="meter">
                    <div class="score-label">{ "Current score" }</div>
                    <div class="score">{ *score }</div>
                    <div class="status">{ status_message(*score) }</div>

                    <p class="helper">
                        { "When the score reaches Launch Ready, the lesson plays a victory WAV once." }
                    </p>

                    <div class="button-row">
                        <button onclick={on_increase}>{ "Boost Readiness" }</button>
                        <button onclick={on_reset}>{ "Reset" }</button>
                    </div>

                    <div class="badges">
                        <span class="badge">{ "Simple state" }</span>
                        <span class="badge">{ "if / else logic" }</span>
                        <span class="badge">{ "Victory WAV" }</span>
                    </div>
                </section>
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}