use gloo::timers::callback::Timeout;
use web_sys::{window, HtmlInputElement};
use yew::prelude::*;

const PI: &str = "3.14159265358979323846264338327950288419716939937510\
58209749445923078164062862089986280348253421170679";

const BEST_KEY: &str = "pi_memory_best";
const LEADERBOARD_KEY: &str = "pi_memory_leaderboard";

#[derive(Clone, PartialEq)]
enum Mode {
    Challenge,
    Training,
    Flash,
}

#[derive(Clone, PartialEq)]
enum FlashState {
    Idle,
    Memorize,
    Typing,
}

fn load_best() -> usize {
    window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(BEST_KEY).ok().flatten())
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(0)
}

fn save_best(score: usize) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(BEST_KEY, &score.to_string());
    }
}

fn load_leaderboard() -> Vec<usize> {
    window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(LEADERBOARD_KEY).ok().flatten())
        .map(|raw| {
            raw.split(',')
                .filter_map(|v| v.trim().parse::<usize>().ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn save_leaderboard(scores: &[usize]) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        let joined = scores
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let _ = storage.set_item(LEADERBOARD_KEY, &joined);
    }
}

fn push_leaderboard(score: usize, leaderboard: &mut Vec<usize>) {
    if score == 0 {
        return;
    }
    leaderboard.push(score);
    leaderboard.sort_by(|a, b| b.cmp(a));
    leaderboard.truncate(5);
}

#[function_component(App)]
fn app() -> Html {
    let mode = use_state(|| Mode::Challenge);
    let input = use_state(String::new);
    let score = use_state(|| 0usize);
    let best = use_state(load_best);
    let leaderboard = use_state(load_leaderboard);
    let finished = use_state(|| false);
    let show_pi = use_state(|| false);
    let message = use_state(|| "Type the digits after 3.".to_string());
    let last_good = use_state(|| true);
    let new_record = use_state(|| false);

    let flash_state = use_state(|| FlashState::Idle);
    let flash_seconds = use_state(|| 5u32);
    let flash_visible_count = use_state(|| 8usize);
    let flash_countdown = use_state(|| 0u32);
    let flash_timeout = use_mut_ref(|| Option::<Timeout>::None);

    let digits_only = &PI[2..];

    let reset_run = {
        let input = input.clone();
        let score = score.clone();
        let finished = finished.clone();
        let message = message.clone();
        let last_good = last_good.clone();
        let new_record = new_record.clone();
        let flash_state = flash_state.clone();
        let flash_countdown = flash_countdown.clone();
        let flash_timeout = flash_timeout.clone();

        Callback::from(move |_| {
            input.set(String::new());
            score.set(0);
            finished.set(false);
            message.set("Type the digits after 3.".to_string());
            last_good.set(true);
            new_record.set(false);
            flash_state.set(FlashState::Idle);
            flash_countdown.set(0);
            *flash_timeout.borrow_mut() = None;
        })
    };

    let set_challenge = {
        let mode = mode.clone();
        let input = input.clone();
        let score = score.clone();
        let finished = finished.clone();
        let message = message.clone();
        let last_good = last_good.clone();
        let flash_state = flash_state.clone();
        let flash_countdown = flash_countdown.clone();
        let flash_timeout = flash_timeout.clone();

        Callback::from(move |_| {
            mode.set(Mode::Challenge);
            input.set(String::new());
            score.set(0);
            finished.set(false);
            message.set("Challenge mode: one wrong digit ends the round.".to_string());
            last_good.set(true);
            flash_state.set(FlashState::Idle);
            flash_countdown.set(0);
            *flash_timeout.borrow_mut() = None;
        })
    };

    let set_training = {
        let mode = mode.clone();
        let input = input.clone();
        let score = score.clone();
        let finished = finished.clone();
        let message = message.clone();
        let last_good = last_good.clone();
        let flash_state = flash_state.clone();
        let flash_countdown = flash_countdown.clone();
        let flash_timeout = flash_timeout.clone();

        Callback::from(move |_| {
            mode.set(Mode::Training);
            input.set(String::new());
            score.set(0);
            finished.set(false);
            message.set("Training mode: use the live preview to practice.".to_string());
            last_good.set(true);
            flash_state.set(FlashState::Idle);
            flash_countdown.set(0);
            *flash_timeout.borrow_mut() = None;
        })
    };

    let set_flash = {
        let mode = mode.clone();
        let input = input.clone();
        let score = score.clone();
        let finished = finished.clone();
        let message = message.clone();
        let last_good = last_good.clone();
        let flash_state = flash_state.clone();
        let flash_countdown = flash_countdown.clone();
        let flash_timeout = flash_timeout.clone();

        Callback::from(move |_| {
            mode.set(Mode::Flash);
            input.set(String::new());
            score.set(0);
            finished.set(false);
            message.set("Flash mode: memorize the digits, then type from memory.".to_string());
            last_good.set(true);
            flash_state.set(FlashState::Idle);
            flash_countdown.set(0);
            *flash_timeout.borrow_mut() = None;
        })
    };

    let toggle_pi = {
        let show_pi = show_pi.clone();
        Callback::from(move |_| show_pi.set(!*show_pi))
    };

    let clear_scores = {
        let best = best.clone();
        let leaderboard = leaderboard.clone();
        Callback::from(move |_| {
            best.set(0);
            leaderboard.set(Vec::new());
            save_best(0);
            save_leaderboard(&[]);
        })
    };

    let set_flash_3 = {
        let flash_seconds = flash_seconds.clone();
        Callback::from(move |_| flash_seconds.set(3))
    };

    let set_flash_5 = {
        let flash_seconds = flash_seconds.clone();
        Callback::from(move |_| flash_seconds.set(5))
    };

    let set_flash_8 = {
        let flash_seconds = flash_seconds.clone();
        Callback::from(move |_| flash_seconds.set(8))
    };

    let start_flash_round = {
        let input = input.clone();
        let score = score.clone();
        let finished = finished.clone();
        let message = message.clone();
        let last_good = last_good.clone();
        let flash_state = flash_state.clone();
        let flash_seconds = flash_seconds.clone();
        let flash_visible_count = flash_visible_count.clone();
        let flash_countdown = flash_countdown.clone();
        let flash_timeout = flash_timeout.clone();

        Callback::from(move |_| {
            input.set(String::new());
            score.set(0);
            finished.set(false);
            last_good.set(true);

            let visible_count = *flash_visible_count;
            let seconds = *flash_seconds;
            flash_state.set(FlashState::Memorize);
            flash_countdown.set(seconds);
            message.set(format!(
                "Memorize these {} digits. They will hide in {} second(s).",
                visible_count, seconds
            ));

            *flash_timeout.borrow_mut() = Some(Timeout::new(seconds * 1000, {
                let flash_state = flash_state.clone();
                let flash_countdown = flash_countdown.clone();
                let message = message.clone();
                move || {
                    flash_state.set(FlashState::Typing);
                    flash_countdown.set(0);
                    message.set("Now type the digits from memory.".to_string());
                }
            }));
        })
    };

    let on_input = {
        let mode = mode.clone();
        let input = input.clone();
        let score = score.clone();
        let best = best.clone();
        let leaderboard = leaderboard.clone();
        let finished = finished.clone();
        let message = message.clone();
        let last_good = last_good.clone();
        let new_record = new_record.clone();
        let flash_state = flash_state.clone();
        let flash_visible_count = flash_visible_count.clone();

        Callback::from(move |e: InputEvent| {
            let value = e.target_unchecked_into::<HtmlInputElement>().value();

            match &*mode {
                Mode::Challenge => {
                    if *finished {
                        return;
                    }

                    if digits_only.starts_with(&value) {
                        let len = value.len();
                        input.set(value);
                        score.set(len);
                        last_good.set(true);

                        if len > *best {
                            best.set(len);
                            save_best(len);
                            new_record.set(true);

                            Timeout::new(1400, {
                                let new_record = new_record.clone();
                                move || new_record.set(false)
                            })
                            .forget();
                        }

                        message.set("Still perfect. Keep going.".to_string());
                    } else {
                        last_good.set(false);
                        let final_score = *score;
                        finished.set(true);
                        message.set(format!("Wrong digit. Final score: {}.", final_score));

                        let mut board = (*leaderboard).clone();
                        push_leaderboard(final_score, &mut board);
                        save_leaderboard(&board);
                        leaderboard.set(board);
                    }
                }
                Mode::Training => {
                    if digits_only.starts_with(&value) {
                        let len = value.len();
                        input.set(value);
                        score.set(len);
                        last_good.set(true);

                        if len > *best {
                            best.set(len);
                            save_best(len);
                            new_record.set(true);

                            Timeout::new(1400, {
                                let new_record = new_record.clone();
                                move || new_record.set(false)
                            })
                            .forget();
                        }

                        message.set(format!("Nice. {} correct digit(s) so far.", len));
                    } else {
                        last_good.set(false);
                        message.set("That next digit is off. Check the preview and try again.".to_string());
                    }
                }
                Mode::Flash => {
                    if *flash_state != FlashState::Typing || *finished {
                        return;
                    }

                    let target = digits_only
                        .chars()
                        .take(*flash_visible_count)
                        .collect::<String>();

                    if target.starts_with(&value) {
                        let len = value.len();
                        input.set(value);
                        score.set(len);
                        last_good.set(true);

                        if len > *best {
                            best.set(len);
                            save_best(len);
                            new_record.set(true);

                            Timeout::new(1400, {
                                let new_record = new_record.clone();
                                move || new_record.set(false)
                            })
                            .forget();
                        }

                        if len == target.len() {
                            finished.set(true);
                            message.set(format!(
                                "Perfect flash round! You recalled all {} digit(s).",
                                len
                            ));

                            let mut board = (*leaderboard).clone();
                            push_leaderboard(len, &mut board);
                            save_leaderboard(&board);
                            leaderboard.set(board);
                        } else {
                            message.set(format!(
                                "Flash mode: {} of {} correct.",
                                len,
                                target.len()
                            ));
                        }
                    } else {
                        last_good.set(false);
                        finished.set(true);
                        message.set(format!(
                            "Missed it. Flash score: {} of {}.",
                            *score,
                            target.len()
                        ));

                        let mut board = (*leaderboard).clone();
                        push_leaderboard(*score, &mut board);
                        save_leaderboard(&board);
                        leaderboard.set(board);
                    }
                }
            }
        })
    };

    let preview_correct = digits_only
        .chars()
        .take(input.len())
        .collect::<String>();

    let next_digit = digits_only
        .chars()
        .nth(input.len())
        .map(|c| c.to_string())
        .unwrap_or_default();

    let preview_rest = digits_only
        .chars()
        .skip(input.len() + 1)
        .take(18)
        .collect::<String>();

    let flash_digits = digits_only
        .chars()
        .take(*flash_visible_count)
        .collect::<String>();

    let helper_class = if *last_good { "helper good" } else { "helper bad" };
    let input_class = if *last_good { "challenge-input good" } else { "challenge-input bad" };

    let disable_input = matches!(*mode, Mode::Flash) && *flash_state != FlashState::Typing;

    let confetti_particles: Vec<(f64, f64, f64, f64, usize)> = (0..28)
        .map(|i| {
            let left = 8.0 + ((i * 13) % 80) as f64;
            let delay = (i % 7) as f64 * 0.04;
            let drift = -80.0 + ((i * 29) % 160) as f64;
            let duration = 0.95 + (i % 6) as f64 * 0.08;
            let hue = (i * 37) % 360;
            (left, delay, drift, duration, hue)
        })
        .collect();

    html! {
        <div class="app-shell">
            <section class="hero">
                <div class="kicker">{ "MIKEGYVER STUDIO • PI DAY MEMORY BUILD" }</div>
                <h1>{ "Pi Memory Challenge v3" }</h1>
                <p>
                    { "How many digits of π can you memorize? Colin already reached 8 digits — now there’s a timed Flash Mode to really put memory to the test." }
                </p>
            </section>

            <section class="grid">
                <aside class="card">
                    <h2>{ "Game Controls" }</h2>

                    <h3>{ "Mode" }</h3>
                    <div class="mode-row">
                        <button
                            class={classes!("secondary", matches!(*mode, Mode::Challenge).then_some("active"))}
                            onclick={set_challenge}
                        >
                            { "Challenge" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*mode, Mode::Training).then_some("active"))}
                            onclick={set_training}
                        >
                            { "Training" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*mode, Mode::Flash).then_some("active"))}
                            onclick={set_flash}
                        >
                            { "Flash" }
                        </button>
                    </div>

                    {
                        if matches!(*mode, Mode::Flash) {
                            html! {
                                <>
                                    <h3>{ "Flash Reveal Time" }</h3>
                                    <div class="flash-row">
                                        <button
                                            class={classes!("secondary", (*flash_seconds == 3).then_some("active"))}
                                            onclick={set_flash_3}
                                        >
                                            { "3 sec" }
                                        </button>
                                        <button
                                            class={classes!("secondary", (*flash_seconds == 5).then_some("active"))}
                                            onclick={set_flash_5}
                                        >
                                            { "5 sec" }
                                        </button>
                                        <button
                                            class={classes!("secondary", (*flash_seconds == 8).then_some("active"))}
                                            onclick={set_flash_8}
                                        >
                                            { "8 sec" }
                                        </button>
                                    </div>

                                    <div class="button-row" style="margin-top:10px;">
                                        <button class="primary" onclick={start_flash_round}>{ "Start Flash Round" }</button>
                                    </div>
                                </>
                            }
                        } else {
                            Html::default()
                        }
                    }

                    <h3>{ "Actions" }</h3>
                    <div class="button-row">
                        <button class="primary" onclick={reset_run}>{ "Restart" }</button>
                        <button class="secondary" onclick={toggle_pi}>
                            { if *show_pi { "Hide Pi" } else { "Show Pi" } }
                        </button>
                        <button class="secondary" onclick={clear_scores}>{ "Clear Scores" }</button>
                    </div>

                    <div class="note" style="margin-top: 16px;">
                        {
                            match &*mode {
                                Mode::Challenge => "Challenge mode ends the run on the first wrong digit.",
                                Mode::Training => "Training mode keeps the run open so you can learn with the live preview.",
                                Mode::Flash => "Flash mode briefly reveals a short Pi sequence, then hides it for recall."
                            }
                        }
                    </div>

                    <div class="leaderboard">
                        <h3 style="margin-top:0;">{ "Top 5 Scores" }</h3>
                        {
                            if leaderboard.is_empty() {
                                html! { <div class="note">{ "No scores yet. Start a run." }</div> }
                            } else {
                                html! {
                                    <ol>
                                        { for leaderboard.iter().map(|s| html! { <li>{ format!("{s} digit(s)") }</li> }) }
                                    </ol>
                                }
                            }
                        }
                    </div>
                </aside>

                <main class="card stage">
                    <div class="memory-box">
                        {
                            if *new_record {
                                html! {
                                    <div class="confetti-layer">
                                        {
                                            for confetti_particles.iter().map(|(left, delay, drift, duration, hue)| {
                                                html! {
                                                    <span
                                                        class="confetti-piece"
                                                        style={format!(
                                                            "left:{left:.2}%; animation-delay:{delay:.2}s; --drift:{drift:.2}px; animation-duration:{duration:.2}s; background:hsl({hue} 95% 65%);"
                                                        )}
                                                    />
                                                }
                                            })
                                        }
                                    </div>
                                }
                            } else {
                                Html::default()
                            }
                        }

                        <div class="pi-prefix">{ "3." }</div>

                        <input
                            class={input_class}
                            type="text"
                            inputmode="numeric"
                            autocomplete="off"
                            autocapitalize="off"
                            spellcheck="false"
                            value={(*input).clone()}
                            oninput={on_input}
                            placeholder="type digits..."
                            disabled={disable_input}
                        />

                        <div class={helper_class}>{ (*message).clone() }</div>

                        <div class="score-grid">
                            <div class="score-item">
                                <div class="score-label">{ "Current" }</div>
                                <div class="score-value">{ *score }</div>
                            </div>
                            <div class="score-item">
                                <div class="score-label">{ "Best" }</div>
                                <div class="score-value">{ *best }</div>
                            </div>
                            <div class="score-item">
                                <div class="score-label">{ "Colin Target" }</div>
                                <div class="score-value">{ "8" }</div>
                            </div>
                        </div>

                        {
                            if matches!(*mode, Mode::Flash) {
                                html! {
                                    <div class="flash-box">
                                        <h3 style="margin-top:0;">{ "Flash Round" }</h3>
                                        {
                                            match &*flash_state {
                                                FlashState::Idle => html! {
                                                    <div class="note">
                                                        { format!("Ready to reveal {} digits for {} second(s).", *flash_visible_count, *flash_seconds) }
                                                    </div>
                                                },
                                                FlashState::Memorize => html! {
                                                    <>
                                                        <div class="note">{ format!("Memorize now... {}", *flash_countdown.max(&1)) }</div>
                                                        <div class="flash-digits">{ format!("3.{}", flash_digits) }</div>
                                                    </>
                                                },
                                                FlashState::Typing => html! {
                                                    <>
                                                        <div class="note">{ "Digits hidden. Type what you remember." }</div>
                                                        <div class="flash-hidden">{ "3.••••••••" }</div>
                                                    </>
                                                },
                                            }
                                        }
                                    </div>
                                }
                            } else {
                                Html::default()
                            }
                        }

                        <div class="preview-box">
                            <h3 style="margin-top:0;">{ "Live Preview" }</h3>
                            <div class="preview-line">
                                <span>{ "3." }</span>
                                <span class="preview-correct">{ preview_correct }</span>
                                <span class="preview-next">{ next_digit }</span>
                                <span class="preview-rest">{ preview_rest }</span>
                            </div>
                        </div>

                        {
                            if *show_pi {
                                html! {
                                    <div class="pi-box">
                                        <h3 style="margin-top:0;">{ "Pi Reference" }</h3>
                                        <code>{ PI }</code>
                                    </div>
                                }
                            } else {
                                Html::default()
                            }
                        }
                    </div>
                </main>

                <aside class="card">
                    <h2>{ "Challenge Notes" }</h2>
                    <div class="note">
                        { "Pi starts with 3.14159265… so Colin already has a strong 8-digit target on the board." }
                    </div>

                    <h3>{ "Quick Goals" }</h3>
                    <div class="note">
                        { "4 digits: strong start" }<br/>
                        { "8 digits: match Colin" }<br/>
                        { "10 digits: Pi pro" }<br/>
                        { "20+ digits: memory machine" }
                    </div>

                    <h3>{ "Flash Mode Goal" }</h3>
                    <div class="note">
                        { "Memorize the revealed digits, wait for them to hide, then type them back from memory. Great for family STEM challenges." }
                    </div>
                </aside>
            </section>

            <div class="footer">
                <strong>{ "Pi Day memory mission:" }</strong>
                { " Match Colin’s 8 digits in Challenge Mode, then beat it in Flash Mode." }
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}