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

    let on_increase = {
        let score = score.clone();
        Callback::from(move |_| score.set(*score + 1))
    };

    let on_reset = {
        let score = score.clone();
        Callback::from(move |_| score.set(0))
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
                        { "This is the heart of the lesson: state changes, Rust logic runs, and the browser updates immediately." }
                    </p>

                    <div class="button-row">
                        <button onclick={on_increase}>{ "Boost Readiness" }</button>
                        <button onclick={on_reset}>{ "Reset" }</button>
                    </div>

                    <div class="badges">
                        <span class="badge">{ "Simple state" }</span>
                        <span class="badge">{ "if / else logic" }</span>
                        <span class="badge">{ "Browser feedback" }</span>
                    </div>
                </section>
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}