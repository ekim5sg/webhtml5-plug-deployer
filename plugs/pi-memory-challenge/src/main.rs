use gloo::timers::callback::Timeout;
use web_sys::{window, HtmlInputElement};
use yew::prelude::*;

const PI: &str = "3.14159265358979323846264338327950288419716939937510\
58209749445923078164062862089986280348253421170679";

const BEST_KEY: &str = "pi_memory_best";
const LEADERBOARD_KEY: &str = "pi_memory_leaderboard";
const NAMED_LEADERBOARD_KEY: &str = "pi_memory_named_leaderboard";

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

#[derive(Clone, PartialEq)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
    Genius,
}

impl Difficulty {
    fn label(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
            Difficulty::Genius => "Genius",
        }
    }

    fn digits(&self) -> usize {
        match self {
            Difficulty::Easy => 5,
            Difficulty::Medium => 8,
            Difficulty::Hard => 12,
            Difficulty::Genius => 20,
        }
    }
}

#[derive(Clone, PartialEq)]
struct NamedScore {
    name: String,
    score: usize,
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

fn load_named_leaderboard() -> Vec<NamedScore> {
    window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(NAMED_LEADERBOARD_KEY).ok().flatten())
        .map(|raw| {
            raw.split('\n')
                .filter_map(|line| {
                    let mut parts = line.splitn(2, '|');
                    let name = parts.next()?.trim().to_string();
                    let score = parts.next()?.trim().parse::<usize>().ok()?;
                    if name.is_empty() {
                        None
                    } else {
                        Some(NamedScore { name, score })
                    }
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn save_named_leaderboard(scores: &[NamedScore]) {
    if let Some(storage) = window().and_then(|w| w.local_storage().ok().flatten()) {
        let joined = scores
            .iter()
            .map(|s| format!("{}|{}", s.name, s.score))
            .collect::<Vec<_>>()
            .join("\n");
        let _ = storage.set_item(NAMED_LEADERBOARD_KEY, &joined);
    }
}

fn push_named_score(name: &str, score: usize, board: &mut Vec<NamedScore>) {
    if score == 0 {
        return;
    }

    let clean_name = if name.trim().is_empty() {
        "Player".to_string()
    } else {
        name.trim().to_string()
    };

    board.push(NamedScore {
        name: clean_name,
        score,
    });

    board.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.name.cmp(&b.name)));
    board.truncate(8);
}

fn finish_round(
    final_score: usize,
    player_name: &str,
    leaderboard: &UseStateHandle<Vec<usize>>,
    named_leaderboard: &UseStateHandle<Vec<NamedScore>>,
) {
    let mut board = (*leaderboard).clone();
    push_leaderboard(final_score, &mut board);
    save_leaderboard(&board);
    leaderboard.set(board);

    let mut named = (*named_leaderboard).clone();
    push_named_score(player_name, final_score, &mut named);
    save_named_leaderboard(&named);
    named_leaderboard.set(named);
}

#[function_component(App)]
fn app() -> Html {
    let mode = use_state(|| Mode::Challenge);
    let difficulty = use_state(|| Difficulty::Medium);

    let input = use_state(String::new);
    let score = use_state(|| 0usize);
    let best = use_state(load_best);
    let leaderboard = use_state(load_leaderboard);
    let named_leaderboard = use_state(load_named_leaderboard);
    let finished = use_state(|| false);
    let show_pi = use_state(|| false);
    let message = use_state(|| "Type the digits after 3.".to_string());
    let last_good = use_state(|| true);
    let new_record = use_state(|| false);
    let player_name = use_state(|| "Colin".to_string());

    let flash_state = use_state(|| FlashState::Idle);
    let flash_seconds = use_state(|| 5u32);
    let flash_timeout = use_mut_ref(|| Option::<Timeout>::None);

    let digits_only = &PI[2..];
    let target_digits = difficulty.digits();

    let reset_run = {
        let input = input.clone();
        let score = score.clone();
        let finished = finished.clone();
        let message = message.clone();
        let last_good = last_good.clone();
        let new_record = new_record.clone();
        let flash_state = flash_state.clone();
        let flash_timeout = flash_timeout.clone();

        Callback::from(move |_| {
            input.set(String::new());
            score.set(0);
            finished.set(false);
            message.set("Type the digits after 3.".to_string());
            last_good.set(true);
            new_record.set(false);
            flash_state.set(FlashState::Idle);
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
        let flash_timeout = flash_timeout.clone();

        Callback::from(move |_| {
            mode.set(Mode::Challenge);
            input.set(String::new());
            score.set(0);
            finished.set(false);
            message.set("Challenge mode: one wrong digit ends the round.".to_string());
            last_good.set(true);
            flash_state.set(FlashState::Idle);
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
        let flash_timeout = flash_timeout.clone();

        Callback::from(move |_| {
            mode.set(Mode::Training);
            input.set(String::new());
            score.set(0);
            finished.set(false);
            message.set("Training mode: use the live preview to practice.".to_string());
            last_good.set(true);
            flash_state.set(FlashState::Idle);
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
        let flash_timeout = flash_timeout.clone();

        Callback::from(move |_| {
            mode.set(Mode::Flash);
            input.set(String::new());
            score.set(0);
            finished.set(false);
            message.set("Flash mode: memorize the digits, then type from memory.".to_string());
            last_good.set(true);
            flash_state.set(FlashState::Idle);
            *flash_timeout.borrow_mut() = None;
        })
    };

    let set_easy = {
        let difficulty = difficulty.clone();
        Callback::from(move |_| difficulty.set(Difficulty::Easy))
    };

    let set_medium = {
        let difficulty = difficulty.clone();
        Callback::from(move |_| difficulty.set(Difficulty::Medium))
    };

    let set_hard = {
        let difficulty = difficulty.clone();
        Callback::from(move |_| difficulty.set(Difficulty::Hard))
    };

    let set_genius = {
        let difficulty = difficulty.clone();
        Callback::from(move |_| difficulty.set(Difficulty::Genius))
    };

    let on_name_input = {
        let player_name = player_name.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            player_name.set(input.value());
        })
    };

    let toggle_pi = {
        let show_pi = show_pi.clone();
        Callback::from(move |_| show_pi.set(!*show_pi))
    };

    let clear_scores = {
        let best = best.clone();
        let leaderboard = leaderboard.clone();
        let named_leaderboard = named_leaderboard.clone();
        Callback::from(move |_| {
            best.set(0);
            leaderboard.set(Vec::new());
            named_leaderboard.set(Vec::new());
            save_best(0);
            save_leaderboard(&[]);
            save_named_leaderboard(&[]);
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
        let flash_timeout = flash_timeout.clone();
        let difficulty = difficulty.clone();

        Callback::from(move |_| {
            input.set(String::new());
            score.set(0);
            finished.set(false);
            last_good.set(true);

            let visible_count = difficulty.digits();
            let seconds = *flash_seconds;

            flash_state.set(FlashState::Memorize);
            message.set(format!(
                "Memorize these {} digits. They will hide in {} second(s).",
                visible_count, seconds
            ));

            *flash_timeout.borrow_mut() = Some(Timeout::new(seconds * 1000, {
                let flash_state = flash_state.clone();
                let message = message.clone();
                move || {
                    flash_state.set(FlashState::Typing);
                    message.set("Now type the digits from memory.".to_string());
                }
            }));
        })
    };

    let on_digit_input = {
        let mode = mode.clone();
        let input = input.clone();
        let score = score.clone();
        let best = best.clone();
        let leaderboard = leaderboard.clone();
        let named_leaderboard = named_leaderboard.clone();
        let finished = finished.clone();
        let message = message.clone();
        let last_good = last_good.clone();
        let new_record = new_record.clone();
        let flash_state = flash_state.clone();
        let difficulty = difficulty.clone();
        let player_name = player_name.clone();

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

                        if len >= difficulty.digits() {
                            message.set(format!(
                                "Nice — {} reached the {} tier target of {} digits.",
                                if player_name.trim().is_empty() { "Player" } else { &*player_name },
                                difficulty.label(),
                                difficulty.digits()
                            ));
                        } else {
                            message.set("Still perfect. Keep going.".to_string());
                        }
                    } else {
                        last_good.set(false);
                        let final_score = *score;
                        finished.set(true);
                        message.set(format!(
                            "Wrong digit. {} finished with {}.",
                            if player_name.trim().is_empty() { "Player" } else { &*player_name },
                            final_score
                        ));

                        finish_round(final_score, &player_name, &leaderboard, &named_leaderboard);
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

                        if len >= difficulty.digits() {
                            message.set(format!(
                                "Great work. {} matched the {} goal of {} digits.",
                                if player_name.trim().is_empty() { "Player" } else { &*player_name },
                                difficulty.label(),
                                difficulty.digits()
                            ));
                        } else {
                            message.set(format!("Nice. {} correct digit(s) so far.", len));
                        }
                    } else {
                        last_good.set(false);
                        message.set("That next digit is off. Check the preview and try again.".to_string());
                    }
                }
                Mode::Flash => {
                    if *flash_state != FlashState::Typing || *finished {
                        return;
                    }

                    let target_len = difficulty.digits();
                    let target = digits_only.chars().take(target_len).collect::<String>();

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
                                "Perfect {} flash round! {} recalled all {} digit(s).",
                                difficulty.label(),
                                if player_name.trim().is_empty() { "Player" } else { &*player_name },
                                len
                            ));

                            finish_round(len, &player_name, &leaderboard, &named_leaderboard);
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
                            "Missed it. {} scored {} of {} in Flash Mode.",
                            if player_name.trim().is_empty() { "Player" } else { &*player_name },
                            *score,
                            target.len()
                        ));

                        finish_round(*score, &player_name, &leaderboard, &named_leaderboard);
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
        .take(target_digits)
        .collect::<String>();

    let helper_class = if *last_good {
        "helper good"
    } else {
        "helper bad"
    };

    let input_class = if *last_good {
        "challenge-input good"
    } else {
        "challenge-input bad"
    };

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

    let display_name = if player_name.trim().is_empty() {
        "Player".to_string()
    } else {
        player_name.trim().to_string()
    };

    html! {
        <div class="app-shell">
            <section class="hero">
                <div class="kicker">{ "MIKEGYVER STUDIO • PI DAY MEMORY BUILD" }</div>
                <h1>{ "Pi Memory Challenge v5" }</h1>
                <p>
                    { "Challenge, train, and flash-memorize Pi digits with named classroom-style score tracking. Colin already reached 8 digits — now the whole family or class can jump in." }
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

                    <h3>{ "Difficulty" }</h3>
                    <div class="difficulty-row">
                        <button
                            class={classes!("secondary", matches!(*difficulty, Difficulty::Easy).then_some("active"))}
                            onclick={set_easy}
                        >
                            { "Easy" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*difficulty, Difficulty::Medium).then_some("active"))}
                            onclick={set_medium}
                        >
                            { "Medium" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*difficulty, Difficulty::Hard).then_some("active"))}
                            onclick={set_hard}
                        >
                            { "Hard" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*difficulty, Difficulty::Genius).then_some("active"))}
                            onclick={set_genius}
                        >
                            { "Genius" }
                        </button>
                    </div>

                    <div class="goal-box">
                        <div class="note">
                            { "Selected tier: " }
                            <span class="goal-strong">
                                { format!("{} ({} digits)", difficulty.label(), difficulty.digits()) }
                            </span>
                        </div>
                    </div>

                    <div class="classroom-box">
                        <div class="classroom-head">
                            <h3 style="margin:0;">{ "Classroom Mode" }</h3>
                            <span class="current-player">{ format!("Current: {}", display_name) }</span>
                        </div>
                        <div class="name-row">
                            <input
                                class="name-input"
                                type="text"
                                value={(*player_name).clone()}
                                oninput={on_name_input}
                                placeholder="Enter player name..."
                            />
                            <div class="note">
                                { "Each finished round saves a named score to the local leaderboard." }
                            </div>
                        </div>
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
                            oninput={on_digit_input}
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
                            <div class="score-item">
                                <div class="score-label">{ "Tier Goal" }</div>
                                <div class="score-value">{ target_digits }</div>
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
                                                        { format!(
                                                            "Ready to reveal {} digits for {} second(s).",
                                                            target_digits,
                                                            *flash_seconds
                                                        ) }
                                                    </div>
                                                },
                                                FlashState::Memorize => html! {
                                                    <>
                                                        <div class="note">{ "Memorize now..." }</div>
                                                        <div class="flash-digits">{ format!("3.{}", flash_digits) }</div>
                                                    </>
                                                },
                                                FlashState::Typing => html! {
                                                    <>
                                                        <div class="note">{ "Digits hidden. Type what you remember." }</div>
                                                        <div class="flash-hidden">
                                                            { format!("3.{}", "•".repeat(target_digits.min(20))) }
                                                        </div>
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
                    <h2>{ "Classroom Board" }</h2>

                    <div class="note">
                        { "Named local results make this version great for Colin, family STEM play, or a quick classroom challenge." }
                    </div>

                    <div class="leaderboard named-board">
                        <h3 style="margin-top:0;">{ "Named Leaderboard" }</h3>
                        {
                            if named_leaderboard.is_empty() {
                                html! { <div class="note">{ "No named scores yet. Finish a round to save one." }</div> }
                            } else {
                                html! {
                                    <ol>
                                        {
                                            for named_leaderboard.iter().map(|entry| html! {
                                                <li>
                                                    <span>{ entry.name.clone() }</span>
                                                    <strong>{ format!("{} digits", entry.score) }</strong>
                                                </li>
                                            })
                                        }
                                    </ol>
                                }
                            }
                        }
                    </div>

                    <h3>{ "Difficulty Tiers" }</h3>
                    <div class="note">
                        { "Easy: 5 digits" }<br/>
                        { "Medium: 8 digits" }<br/>
                        { "Hard: 12 digits" }<br/>
                        { "Genius: 20 digits" }
                    </div>

                    <h3>{ "MVP Golden" }</h3>
                    <div class="note">
                        { "This version is a strong stopping point: polished UI, multiple play modes, difficulty tiers, persistent scores, named leaderboard, and classroom-friendly flow." }
                    </div>
                </aside>
            </section>

            <div class="footer">
                <strong>{ "Pi Day memory mission:" }</strong>
                { format!(" {} is on deck — match Colin’s 8 digits, then push into {} mode at {} digits.", display_name, difficulty.label(), difficulty.digits()) }
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}