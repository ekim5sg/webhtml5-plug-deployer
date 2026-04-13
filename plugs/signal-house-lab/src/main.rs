use yew::prelude::*;
use web_sys::HtmlAudioElement;

fn play_sound(src: &str) {
    let audio = HtmlAudioElement::new_with_src(src).unwrap();
    let _ = audio.play();
}

#[function_component(App)]
fn app() -> Html {

    let names = use_state(|| vec!["Colin".to_string(), "Maya".to_string(), "Leo".to_string()]);
    let signals = use_state(|| 0);
    let best = use_state(|| 0);
    let story_mode = use_state(|| false);

    let update_name = {
        let names = names.clone();
        move |index: usize, value: String| {
            let mut new_names = (*names).clone();
            new_names[index] = value;
            names.set(new_names);
        }
    };

    let send_signal = {
        let signals = signals.clone();
        let best = best.clone();
        Callback::from(move |_| {
            let new_val = *signals + 1;
            signals.set(new_val);

            if new_val > *best {
                best.set(new_val);
            }

            play_sound("assets/audio/ping.wav");
        })
    };

    let toggle_story = {
        let story_mode = story_mode.clone();
        Callback::from(move |_| {
            story_mode.set(!*story_mode);
            play_sound("assets/audio/activate.wav");
        })
    };

    html! {
        <div class="container">

            <h1>{"🚀 Signal House Lab v2"}</h1>

            <div class="card">
                <h2>{"👨‍🚀 Crew Names"}</h2>

                { for names.iter().enumerate().map(|(i, name)| {
                    let update_name = update_name.clone();
                    html! {
                        <input
                            value={name.clone()}
                            oninput={Callback::from(move |e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                update_name(i, input.value());
                            })}
                        />
                    }
                })}
            </div>

            <div class="card">
                <h2>{"📡 Send Signal Challenge"}</h2>

                <button onclick={send_signal.clone()}>
                    {"Send Signal"}
                </button>

                <p class="big">{format!("Signals Sent: {}", *signals)}</p>
                <p>{format!("Best Mission: {}", *best)}</p>
            </div>

            <div class="card">
                <h2>{"🎮 Mini Challenge"}</h2>

                <p>{"Tap as fast as you can to power the house!"}</p>

                <button onclick={send_signal}>
                    {"⚡ BOOST SIGNAL"}
                </button>
            </div>

            <div class="card">
                <h2>{"📖 Story Mode"}</h2>

                <button onclick={toggle_story}>
                    {"Toggle Story Mode"}
                </button>

                {
                    if *story_mode {
                        html! {
                            <p>
                            {"This all began with Colin’s drawings...
                              A house. A signal. A connection to space.
                              And now… you're part of the mission."}
                            </p>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>

        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}