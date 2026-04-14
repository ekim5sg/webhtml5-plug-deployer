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
    let score = use_state(|| 0_i32);

    let increase = {
        let score = score.clone();
        Callback::from(move |_| {
            let next = (*score + 1).min(10);
            score.set(next);
        })
    };

    let decrease = {
        let score = score.clone();
        Callback::from(move |_| {
            let next = (*score - 1).max(0);
            score.set(next);
        })
    };

    let reset = {
        let score = score.clone();
        Callback::from(move |_| score.set(0))
    };

    let meter_percent = (*score as f64 / 10.0) * 100.0;

    html! {
        <main>
            <section class="card">
                <div class="topline">{ "Rust • Yew • WebAssembly" }</div>

                <h1 class="title">
                    { "Your First " }
                    <span class="accent">{ "Real Logic" }</span>
                </h1>

                <p class="subtitle">
                    { "Click the buttons and watch Rust decide what status message to show. This tiny app teaches mutable state, functions, and if / else logic in the browser." }
                </p>

                <section class="status-card">
                    <div class="status-header">
                        <div class="status-title">{ "Launch Readiness Meter" }</div>
                    </div>

                    <div class="score-wrap">
                        <div class="score">{ *score }</div>
                        <div class="message">{ status_message(*score) }</div>

                        <div class="meter" aria-label="readiness meter">
                            <div class="meter-fill" style={format!("width: {:.0}%;", meter_percent)}></div>
                        </div>

                        <div class="controls">
                            <button onclick={increase}>{ "Boost Readiness" }</button>
                            <button class="secondary" onclick={decrease}>{ "Lower" }</button>
                            <button class="secondary" onclick={reset}>{ "Reset" }</button>
                        </div>
                    </div>
                </section>

                <p class="footer-note">
                    { "Tip: run trunk serve for local learning, then trunk build --release when you want a production-ready dist folder." }
                </p>
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}