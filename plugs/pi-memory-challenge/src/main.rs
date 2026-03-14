use web_sys::window;
use yew::prelude::*;

const PI: &str = "3.14159265358979323846264338327950288419716939937510\
58209749445923078164062862089986280348253421170679";

const STORAGE_KEY: &str = "pi_memory_best";

fn load_best() -> usize {
    window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn save_best(score: usize) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(STORAGE_KEY, &score.to_string());
    }
}

#[function_component(App)]
fn app() -> Html {

    let input = use_state(|| "".to_string());
    let score = use_state(|| 0usize);
    let best = use_state(load_best);
    let finished = use_state(|| false);
    let show_pi = use_state(|| false);

    let on_input = {
        let input = input.clone();
        let score = score.clone();
        let best = best.clone();
        let finished = finished.clone();

        Callback::from(move |e: InputEvent| {

            if *finished { return; }

            let value = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();

            let correct = &PI[2..]; // digits after "3."

            if correct.starts_with(&value) {

                score.set(value.len());

                if value.len() > *best {
                    best.set(value.len());
                    save_best(value.len());
                }

                input.set(value);

            } else {
                finished.set(true);
            }
        })
    };

    let reset = {
        let input = input.clone();
        let score = score.clone();
        let finished = finished.clone();

        Callback::from(move |_|{
            input.set("".to_string());
            score.set(0);
            finished.set(false);
        })
    };

    let reveal = {
        let show_pi = show_pi.clone();
        Callback::from(move |_|{
            show_pi.set(!*show_pi);
        })
    };

    html! {

        <div>

            <h1>{ "Pi Memory Challenge" }</h1>

            <div class="subtitle">
                { "How many digits of π can you memorize?" }
            </div>

            <div>
                <strong>{ "3." }</strong>

                <input
                    value={(*input).clone()}
                    oninput={on_input}
                    placeholder="type digits..."
                />
            </div>

            <div class="score">
                { format!("Score: {}", *score) }
            </div>

            <div class="score best">
                { format!("Best: {}", *best) }
            </div>

            {
                if *finished {
                    html!{ <div class="error">{ "Wrong digit! Try again." }</div> }
                } else {
                    html!{}
                }
            }

            <div>

                <button class="primary" onclick={reset}>
                    { "Restart" }
                </button>

                <button class="secondary" onclick={reveal}>
                    { "Show Pi" }
                </button>

            </div>

            {
                if *show_pi {
                    html!{
                        <div class="pi">
                        { PI }
                        </div>
                    }
                } else {
                    html!{}
                }
            }

        </div>

    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}