use gloo_timers::callback::Interval;
use web_sys::HtmlAudioElement;
use yew::prelude::*;

const DAY_MS: f64 = 86_400_000.0;

#[derive(Clone, Copy, PartialEq)]
enum EventKind {
    Halloween,
    Friday13,
}

#[derive(Clone, Copy)]
struct Event {
    kind: EventKind,
    year: i32,
    month: i32,
    day: i32,
    target_ms: f64,
}

#[derive(Clone, Copy)]
struct Remaining {
    days: u64,
    hours: u64,
    minutes: u64,
    seconds: u64,
}

fn days_from_civil(year: i32, month: i32, day: i32) -> i64 {
    let adjusted_year = year - if month <= 2 { 1 } else { 0 };
    let era = if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    } / 400;

    let year_of_era = adjusted_year - era * 400;
    let shifted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era =
        year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    (era * 146_097 + day_of_era - 719_468) as i64
}

fn weekday(year: i32, month: i32, day: i32) -> i32 {
    (days_from_civil(year, month, day) + 4).rem_euclid(7) as i32
}

fn first_sunday(year: i32, month: i32) -> i32 {
    1 + (7 - weekday(year, month, 1)).rem_euclid(7)
}

fn central_utc_offset_at_midnight(year: i32, month: i32, day: i32) -> i32 {
    match month {
        1 | 2 | 12 => 6,
        4..=10 => 5,
        3 => {
            let second_sunday = first_sunday(year, 3) + 7;

            if day <= second_sunday {
                6
            } else {
                5
            }
        }
        11 => {
            let dst_end = first_sunday(year, 11);

            if day <= dst_end {
                5
            } else {
                6
            }
        }
        _ => 6,
    }
}

fn central_midnight_ms(year: i32, month: i32, day: i32) -> f64 {
    let days = days_from_civil(year, month, day);
    let offset = central_utc_offset_at_midnight(year, month, day);

    ((days * 86_400) + (offset as i64 * 3_600)) as f64 * 1_000.0
}

fn current_year() -> i32 {
    js_sys::Date::new_0().get_utc_full_year() as i32
}

fn next_halloween(now: f64) -> Event {
    let start_year = current_year() - 1;

    for year in start_year..=(start_year + 4) {
        let target_ms = central_midnight_ms(year, 10, 31);

        if target_ms > now {
            return Event {
                kind: EventKind::Halloween,
                year,
                month: 10,
                day: 31,
                target_ms,
            };
        }
    }

    unreachable!()
}

fn next_friday_13(now: f64) -> Event {
    let start_year = current_year() - 1;

    for year in start_year..=(start_year + 8) {
        for month in 1..=12 {
            if weekday(year, month, 13) == 5 {
                let target_ms = central_midnight_ms(year, month, 13);

                if target_ms > now {
                    return Event {
                        kind: EventKind::Friday13,
                        year,
                        month,
                        day: 13,
                        target_ms,
                    };
                }
            }
        }
    }

    unreachable!()
}

fn active_celebration(now: f64) -> Option<EventKind> {
    let year = current_year();

    for candidate_year in (year - 1)..=(year + 1) {
        let halloween = central_midnight_ms(candidate_year, 10, 31);

        if now >= halloween && now < halloween + DAY_MS {
            return Some(EventKind::Halloween);
        }

        for month in 1..=12 {
            if weekday(candidate_year, month, 13) == 5 {
                let friday = central_midnight_ms(candidate_year, month, 13);

                if now >= friday && now < friday + DAY_MS {
                    return Some(EventKind::Friday13);
                }
            }
        }
    }

    None
}

fn remaining(target_ms: f64, now: f64) -> Remaining {
    let total_seconds = ((target_ms - now).max(0.0) / 1_000.0).floor() as u64;

    Remaining {
        days: total_seconds / 86_400,
        hours: (total_seconds % 86_400) / 3_600,
        minutes: (total_seconds % 3_600) / 60,
        seconds: total_seconds % 60,
    }
}

fn month_name(month: i32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

#[derive(Properties, PartialEq)]
struct CountdownProps {
    kind: EventKind,
    now: f64,
    simulated_target_ms: Option<f64>,
}

#[function_component(Countdown)]
fn countdown(props: &CountdownProps) -> Html {
    let mut event = match props.kind {
        EventKind::Halloween => next_halloween(props.now),
        EventKind::Friday13 => next_friday_13(props.now),
    };

    let simulated = props.simulated_target_ms.is_some();
    if let Some(target_ms) = props.simulated_target_ms {
        event.target_ms = target_ms;
    }

    let time = remaining(event.target_ms, props.now);

    let (class_name, icon, title) = match event.kind {
        EventKind::Halloween => ("countdown-card halloween", "🎃", "Halloween"),
        EventKind::Friday13 => ("countdown-card friday", "🏒", "Friday the 13th"),
    };

    html! {
        <article class={class_name}>
            <div class="card-icon">{icon}</div>

            <h2>{title}</h2>

            <p class="event-date">
                {format!(
                    "{} {}, {}",
                    month_name(event.month),
                    event.day,
                    event.year
                )}
            </p>

            <div class="timer">
                <div class="time-box">
                    <strong>{time.days}</strong>
                    <span>{"Days"}</span>
                </div>

                <div class="time-box">
                    <strong>{format!("{:02}", time.hours)}</strong>
                    <span>{"Hours"}</span>
                </div>

                <div class="time-box">
                    <strong>{format!("{:02}", time.minutes)}</strong>
                    <span>{"Minutes"}</span>
                </div>

                <div class="time-box">
                    <strong>{format!("{:02}", time.seconds)}</strong>
                    <span>{"Seconds"}</span>
                </div>
            </div>

            <p class="central-note">
                {if simulated { "ADMIN SIMULATION • Celebration begins at zero" } else { "Countdown ends at midnight Central Time" }}
            </p>
        </article>
    }
}

#[derive(Properties, PartialEq)]
struct RainProps {
    kind: EventKind,
}

#[function_component(CelebrationRain)]
fn celebration_rain(props: &RainProps) -> Html {
    let symbols: &[&str] = match props.kind {
        EventKind::Halloween => &[
            "🦇", "👻", "🎃", "🧙", "🐺", "🧟", "👹", "💀",
        ],
        EventKind::Friday13 => &[
            "🏒", "🔪", "🩸", "⛺", "🌲", "🌕", "⚠️",
        ],
    };

    let title = match props.kind {
        EventKind::Halloween => "Happy Halloween!",
        EventKind::Friday13 => "Happy Friday the 13th!",
    };

    html! {
        <>
            <div class="celebration-banner">{title}</div>

            <div class="rain" aria-hidden="true">
                {
                    for (0..44).map(|index| {
                        let symbol = symbols[index % symbols.len()];
                        let left = (index * 37 + 11) % 100;
                        let duration = 5.0 + ((index * 17) % 50) as f64 / 10.0;
                        let delay = -(((index * 29) % 80) as f64) / 10.0;
                        let size = 1.4 + ((index * 13) % 25) as f64 / 10.0;

                        let style = format!(
                            "left:{left}%;animation-duration:{duration}s;\
                             animation-delay:{delay}s;font-size:{size}rem"
                        );

                        html! {
                            <span class="creature" style={style}>{symbol}</span>
                        }
                    })
                }
            </div>
        </>
    }
}

#[function_component(App)]
fn app() -> Html {
    let now = use_state(js_sys::Date::now);
    let selected_music = use_state(|| None::<&'static str>);
    let audio_status = use_state(|| "Music is off.".to_string());
    let audio_ref = use_node_ref();
    let simulation = use_state(|| None::<(EventKind, f64)>);

    let admin_mode = web_sys::window()
        .and_then(|window| window.location().search().ok())
        .map(|query| query.contains("admin=1") || query.contains("admin=true"))
        .unwrap_or(false);

    {
        let now = now.clone();

        use_effect_with((), move |_| {
            let interval = Interval::new(1_000, move || {
                now.set(js_sys::Date::now());
            });

            move || drop(interval)
        });
    }

    let select_music = {
        let selected_music = selected_music.clone();
        let audio_status = audio_status.clone();
        let audio_ref = audio_ref.clone();

        Callback::from(move |track: &'static str| {
            if let Some(audio) = audio_ref.cast::<HtmlAudioElement>() {
                audio.pause();
                audio.set_src(track);
                audio.set_loop(true);
                audio.set_volume(0.55);

                selected_music.set(Some(track));

                let label = if track.contains("halloscream") {
                    "Playing Halloscream."
                } else {
                    "Playing Friday the 13th."
                };

                match audio.play() {
                    Ok(_) => audio_status.set(label.to_string()),
                    Err(_) => audio_status.set(
                        "The browser could not start this track.".to_string()
                    ),
                }
            }
        })
    };

    let stop_music = {
        let selected_music = selected_music.clone();
        let audio_status = audio_status.clone();
        let audio_ref = audio_ref.clone();

        Callback::from(move |_| {
            if let Some(audio) = audio_ref.cast::<HtmlAudioElement>() {
                audio.pause();
                audio.set_current_time(0.0);
            }

            selected_music.set(None);
            audio_status.set("Music is off.".to_string());
        })
    };

    let audio_error = {
        let selected_music = selected_music.clone();
        let audio_status = audio_status.clone();

        Callback::from(move |_| {
            selected_music.set(None);
            audio_status.set(
                "Track unavailable. Upload the MP3 to assets/audio and try again."
                    .to_string()
            );
        })
    };

    let simulated_celebration = (*simulation).and_then(|(kind, target_ms)| {
        if *now >= target_ms && *now < target_ms + 12_000.0 {
            Some(kind)
        } else {
            None
        }
    });

    let celebration = simulated_celebration.or_else(|| active_celebration(*now));

    let halloween_simulation = (*simulation).and_then(|(kind, target_ms)| {
        (kind == EventKind::Halloween && *now < target_ms).then_some(target_ms)
    });

    let friday_simulation = (*simulation).and_then(|(kind, target_ms)| {
        (kind == EventKind::Friday13 && *now < target_ms).then_some(target_ms)
    });

    let simulate_halloween = {
        let simulation = simulation.clone();
        Callback::from(move |_| {
            simulation.set(Some((EventKind::Halloween, js_sys::Date::now() + 10_000.0)));
        })
    };

    let simulate_friday = {
        let simulation = simulation.clone();
        Callback::from(move |_| {
            simulation.set(Some((EventKind::Friday13, js_sys::Date::now() + 10_000.0)));
        })
    };

    let cancel_simulation = {
        let simulation = simulation.clone();
        Callback::from(move |_| simulation.set(None))
    };

    html! {
        <main class="app">
            {
                if let Some(kind) = celebration {
                    html! { <CelebrationRain kind={kind} /> }
                } else {
                    Html::default()
                }
            }

            <header class="hero">
                <div>
                    <p class="eyebrow">
                        {"MIKEGYVER STUDIO • CENTRAL TIME EVENT SYSTEM"}
                    </p>

                    <h1>{"Midnight Monsters"}</h1>

                    <p class="subtitle">
                        {"Two spooky dates. Two ticking clocks. One haunted browser."}
                    </p>
                </div>

                <span class="rust-badge">{"RUST • YEW • WASM"}</span>
            </header>

            <section class="countdowns">
                <Countdown kind={EventKind::Halloween} now={*now} simulated_target_ms={halloween_simulation} />
                <Countdown kind={EventKind::Friday13} now={*now} simulated_target_ms={friday_simulation} />
            </section>

            {if admin_mode {
                html! {
                    <section class="admin-panel">
                        <div>
                            <span class="admin-label">{"ADMIN EFFECTS LAB"}</span>
                            <h3>{"Ten-second celebration simulator"}</h3>
                            <p>{"Only one simulation runs at a time. Normal visitors never see this panel."}</p>
                        </div>
                        <div class="admin-buttons">
                            <button onclick={simulate_halloween}>{"🎃 Simulate Halloween"}</button>
                            <button onclick={simulate_friday}>{"🏒 Simulate Friday the 13th"}</button>
                            <button class="cancel" onclick={cancel_simulation}>{"Cancel"}</button>
                        </div>
                    </section>
                }
            } else { Html::default() }}

            <section class="music-panel">
                <h3>{"Haunted soundtrack"}</h3>

                <p>{"Choose a background track or enjoy the silence."}</p>

                <div class="music-buttons">
                    <button
                        class={classes!(
                            "music-button",
                            (*selected_music == Some("assets/audio/halloscream.mp3"))
                                .then_some("active")
                        )}
                        onclick={{
                            let select_music = select_music.clone();

                            Callback::from(move |_| {
                                select_music.emit("assets/audio/halloscream.mp3")
                            })
                        }}
                    >
                        {"🎃 Play Halloscream"}
                    </button>

                    <button
                        class={classes!(
                            "music-button",
                            (*selected_music == Some("assets/audio/friday13.mp3"))
                                .then_some("active")
                        )}
                        onclick={{
                            let select_music = select_music.clone();

                            Callback::from(move |_| {
                                select_music.emit("assets/audio/friday13.mp3")
                            })
                        }}
                    >
                        {"🏒 Play Friday the 13th"}
                    </button>

                    <button
                        class="music-button stop"
                        onclick={stop_music}
                    >
                        {"■ Music Off"}
                    </button>
                </div>

                <p class="audio-status">{&*audio_status}</p>

                <audio
                    ref={audio_ref}
                    preload="none"
                    onerror={audio_error}
                />
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
