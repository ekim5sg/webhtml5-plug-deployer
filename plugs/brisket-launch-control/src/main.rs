use gloo::timers::callback::Interval;
use js_sys::Date;
use wasm_bindgen::JsCast;
use web_sys::{HtmlAudioElement, HtmlInputElement};
use yew::prelude::*;

#[derive(Clone, PartialEq)]
enum FoodMode {
    Brisket,
    Tofu,
}

fn play_theme() {
    let Some(window) = web_sys::window() else {
        return;
    };

    let Some(document) = window.document() else {
        return;
    };

    let audio = if let Some(existing) = document.get_element_by_id("bbq-theme-audio") {
        match existing.dyn_into::<HtmlAudioElement>() {
            Ok(audio) => audio,
            Err(_) => return,
        }
    } else {
        let Ok(element) = document.create_element("audio") else {
            return;
        };

        let Ok(audio) = element.dyn_into::<HtmlAudioElement>() else {
            return;
        };

        audio.set_id("bbq-theme-audio");
        audio.set_src("/brisket-launch-control/assets/brisket-launch-control/audio/memorial-bbq-theme.mp3");
        audio.set_loop(true);
        audio.set_autoplay(false);
        audio.set_preload("auto");
        audio.set_volume(0.55);

        if let Some(body) = document.body() {
            let _ = body.append_child(&audio);
        }

        audio
    };

    audio.set_current_time(0.0);
    let _ = audio.play();
}

#[function_component(App)]
fn app() -> Html {
    let now = use_state(|| Date::new_0().get_time());
    let adults = use_state(|| 10u32);
    let kids = use_state(|| 4u32);
    let mode = use_state(|| FoodMode::Brisket);
    let music_started = use_state(|| false);

    {
        let now = now.clone();

        use_effect_with((), move |_| {
            let timer = Interval::new(1000, move || {
                now.set(Date::new_0().get_time());
            });

            move || drop(timer)
        });
    }

    // Friday before Memorial Day 2026: May 22, 2026, 5 PM Texas time.
    // May is daylight time in Texas, so this is Central time UTC-05:00.
    let launch = Date::new(&"2026-05-22T17:00:00-05:00".into()).get_time();
    let remaining = (launch - *now).max(0.0) as u64 / 1000;

    let days = remaining / 86_400;
    let hours = (remaining % 86_400) / 3_600;
    let minutes = (remaining % 3_600) / 60;
    let seconds = remaining % 60;

    let adult_count = *adults as f64;
    let kid_count = *kids as f64;

    let raw_needed = match *mode {
        FoodMode::Brisket => {
            // Brisket math: 1 lb raw per adult, 0.5 lb per child, then +20%.
            (adult_count * 1.0 + kid_count * 0.5) * 1.20
        }
        FoodMode::Tofu => {
            // Vegetarian payload: 0.5 lb tofu adult, 0.25 child, then +20%.
            (adult_count * 0.5 + kid_count * 0.25) * 1.20
        }
    };

    let cooked_estimate = match *mode {
        FoodMode::Brisket => raw_needed * 0.50,
        FoodMode::Tofu => raw_needed * 0.85,
    };

    let food_label = match *mode {
        FoodMode::Brisket => "raw packer brisket",
        FoodMode::Tofu => "tofu payload",
    };

    let joke = match *mode {
        FoodMode::Brisket => "Mission note: If Houston says we have a problem, buy another packer.",
        FoodMode::Tofu => "Mission note: Vegetarian friends are cleared for docking at the side-dish station.",
    };

    let start_music = {
        let music_started = music_started.clone();

        Callback::from(move |_| {
            play_theme();
            music_started.set(true);
        })
    };

    let on_adults = {
        let adults = adults.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();

            if let Ok(v) = input.value().parse::<u32>() {
                adults.set(v.min(500));
            }
        })
    };

    let on_kids = {
        let kids = kids.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();

            if let Ok(v) = input.value().parse::<u32>() {
                kids.set(v.min(500));
            }
        })
    };

    let set_brisket = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(FoodMode::Brisket))
    };

    let set_tofu = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(FoodMode::Tofu))
    };

    html! {
        <main class="app">
            <section class="hero">
                <span class="badge">{"TEXAS MEMORIAL WEEKEND MISSION CONTROL"}</span>

                <h1>{"Brisket Launch Control"}</h1>

                <p class="subtitle">
                    {"Countdown to Friday 5 PM — when the smoker clears the tower."}
                </p>

                <button class="music-btn" onclick={start_music}>
                    {
                        if *music_started {
                            "Theme Music Looping"
                        } else {
                            "Start BBQ Theme Music"
                        }
                    }
                </button>

                <div class="countdown">
                    <div class="timebox">
                        <div class="num">{days}</div>
                        <div class="label">{"Days"}</div>
                    </div>

                    <div class="timebox">
                        <div class="num">{hours}</div>
                        <div class="label">{"Hours"}</div>
                    </div>

                    <div class="timebox">
                        <div class="num">{minutes}</div>
                        <div class="label">{"Minutes"}</div>
                    </div>

                    <div class="timebox">
                        <div class="num">{seconds}</div>
                        <div class="label">{"Seconds"}</div>
                    </div>
                </div>
            </section>

            <section class="grid">
                <div class="card">
                    <h2>{"Payload Calculator"}</h2>

                    <div class="controls">
                        <label>
                            {"Adults reporting to the backyard:"}
                            <input
                                type="number"
                                value={adults.to_string()}
                                oninput={on_adults}
                                min="0"
                            />
                        </label>

                        <label>
                            {"Kids, cousins, and snack-powered astronauts:"}
                            <input
                                type="number"
                                value={kids.to_string()}
                                oninput={on_kids}
                                min="0"
                            />
                        </label>

                        <div class="toggle">
                            <button
                                onclick={set_brisket}
                                class={if *mode == FoodMode::Brisket { "" } else { "off" }}
                            >
                                {"Brisket Mode"}
                            </button>

                            <button
                                onclick={set_tofu}
                                class={if *mode == FoodMode::Tofu { "" } else { "off" }}
                            >
                                {"Tofu Mode"}
                            </button>
                        </div>
                    </div>
                </div>

                <div class="card">
                    <h2>{"Launch Recommendation"}</h2>

                    <div class="result">
                        {format!("{:.1} lb", raw_needed)}
                    </div>

                    <p class="small">
                        {format!("Buy about {:.1} lb of {}.", raw_needed, food_label)}
                    </p>

                    <p class="small">
                        {format!("Estimated served yield after mission turbulence: {:.1} lb.", cooked_estimate)}
                    </p>

                    <div class="mission">{joke}</div>
                </div>
            </section>

            <section class="card" style="margin-top:18px;">
                <h2>{"Flight Rules"}</h2>

                <p class="small">
                    {"Adult brisket math: 1 lb raw per adult. Child brisket math: 0.5 lb raw per child. Safety margin: 20%, because somebody always brings a friend named Bubba."}
                </p>

                <p class="small">
                    {"Official Texas status: boots optional, sauce debated, leftovers mandatory."}
                </p>
            </section>

            <div class="footer">
                {"MikeGyver Studio • Lone Star Launch Sequence Armed"}
            </div>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}