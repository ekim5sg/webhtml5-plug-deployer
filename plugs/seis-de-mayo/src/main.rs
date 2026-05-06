use yew::prelude::*;

const QUOTES: [&str; 8] = [
    "Your taco energy level is approaching dangerous levels.",
    "WARNING: Guacamole reserves critically low.",
    "The queso has achieved orbit.",
    "Cinco de Mayo was only the tutorial level.",
    "NASA could not calculate this salsa velocity.",
    "You have unlocked Legendary Burrito Status.",
    "Rust ownership rules still cannot protect your tacos.",
    "Seis de Mayo: where regrets meet leftovers.",
];

#[function_component(App)]
fn app() -> Html {
    let taco_count = use_state(|| 0usize);

    let on_add = {
        let taco_count = taco_count.clone();
        Callback::from(move |_| {
            taco_count.set(*taco_count + 1);
        })
    };

    let on_reset = {
        let taco_count = taco_count.clone();
        Callback::from(move |_| {
            taco_count.set(0);
        })
    };

    let quote = QUOTES[*taco_count % QUOTES.len()];

    html! {
        <main class="card">
            <h1 class="title">{ "🌮 Seis de Mayo" }</h1>

            <p class="subtitle">
                { "A highly scientific Rust + Yew WASM application designed to measure post-Cinco taco consumption levels." }
                <br/><br/>
                { "Now successfully deploying with non-hashed Trunk builds. Because nobody wants mysterious leftover files... especially after taco night." }
            </p>

            <div class="counter">
                { format!("{} tacos", *taco_count) }
            </div>

            <div class="buttons">
                <button class="taco" onclick={on_add}>
                    { "🌮 Add Another Taco" }
                </button>

                <button class="reset" onclick={on_reset}>
                    { "🧹 Cleanup Deployment" }
                </button>
            </div>

            <div class="quote">
                { quote }
            </div>

            <div class="footer">
                { "Built with Rust, Yew, Trunk, GitHub Actions, and entirely too much queso." }
            </div>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}