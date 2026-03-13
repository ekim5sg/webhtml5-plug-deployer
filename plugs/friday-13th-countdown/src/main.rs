use gloo::timers::callback::Interval;
use js_sys::Date;
use web_sys::HtmlAudioElement;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct CountdownParts {
    days: u64,
    hours: u64,
    minutes: u64,
    seconds: u64,
}

#[derive(Clone, PartialEq)]
struct AppState {
    is_friday_13th: bool,
    next_label: String,
    countdown: CountdownParts,
    now_label: String,
}

fn pad2(value: i32) -> String {
    format!("{value:02}")
}

fn month_name(month_index: i32) -> &'static str {
    match month_index {
        0 => "January",
        1 => "February",
        2 => "March",
        3 => "April",
        4 => "May",
        5 => "June",
        6 => "July",
        7 => "August",
        8 => "September",
        9 => "October",
        10 => "November",
        11 => "December",
        _ => "Unknown",
    }
}

fn weekday_name(day_index: i32) -> &'static str {
    match day_index {
        0 => "Sunday",
        1 => "Monday",
        2 => "Tuesday",
        3 => "Wednesday",
        4 => "Thursday",
        5 => "Friday",
        6 => "Saturday",
        _ => "Unknown",
    }
}

fn is_local_friday_13th(now: &Date) -> bool {
    now.get_day() == 5 && now.get_date() == 13
}

fn local_midnight_date(year: i32, month_index: i32, day: i32) -> Date {
    Date::new_with_year_month_day_hr_min_sec_milli(
        year as u32,
        month_index,
        day,
        0,
        0,
        0,
        0,
    )
}

fn next_friday_13th_after(now: &Date) -> Date {
    let current_year = now.get_full_year() as i32;
    let current_month = now.get_month() as i32;
    let current_day = now.get_date() as i32;

    for offset in 0..240 {
        let total_month = current_month + offset;
        let year = current_year + total_month.div_euclid(12);
        let month = total_month.rem_euclid(12);

        let candidate = local_midnight_date(year, month, 13);

        let candidate_is_future = year > current_year
            || (year == current_year && month > current_month)
            || (year == current_year && month == current_month && 13 > current_day);

        if candidate.get_day() == 5 && candidate_is_future {
            return candidate;
        }
    }

    local_midnight_date(current_year + 20, 0, 13)
}

fn format_event_label(date: &Date) -> String {
    let weekday = weekday_name(date.get_day() as i32);
    let month = month_name(date.get_month() as i32);
    let day = date.get_date();
    let year = date.get_full_year();

    format!("{weekday}, {month} {day}, {year}")
}

fn format_now_label(now: &Date) -> String {
    let weekday = weekday_name(now.get_day() as i32);
    let month = month_name(now.get_month() as i32);
    let day = now.get_date();
    let year = now.get_full_year();
    let hour = now.get_hours();
    let minute = now.get_minutes();
    let second = now.get_seconds();

    format!(
        "{weekday}, {month} {day}, {year} • {}:{}:{}",
        pad2(hour as i32),
        pad2(minute as i32),
        pad2(second as i32)
    )
}

fn breakdown_ms(ms_remaining: f64) -> CountdownParts {
    let total_seconds = (ms_remaining.max(0.0) / 1000.0).floor() as u64;
    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;

    CountdownParts {
        days,
        hours,
        minutes,
        seconds,
    }
}

fn build_state() -> AppState {
    let now = Date::new_0();
    let friday_13th_today = is_local_friday_13th(&now);
    let next_target = next_friday_13th_after(&now);
    let diff_ms = next_target.get_time() - now.get_time();

    AppState {
        is_friday_13th: friday_13th_today,
        next_label: format_event_label(&next_target),
        countdown: breakdown_ms(diff_ms),
        now_label: format_now_label(&now),
    }
}

#[function_component(App)]
fn app() -> Html {
    let state = use_state(build_state);
    let sound_on = use_state(|| false);
    let audio_ref = use_node_ref();
    let audio_error = use_state(|| None::<String>);

    {
        let state = state.clone();
        use_effect_with((), move |_| {
            let interval = Interval::new(1000, move || {
                state.set(build_state());
            });

            move || drop(interval)
        });
    }

    {
        let audio_ref = audio_ref.clone();
        let is_friday = state.is_friday_13th;
        let sound_enabled = *sound_on;
        let audio_error = audio_error.clone();

        use_effect_with((is_friday, sound_enabled), move |_| {
            if let Some(audio) = audio_ref.cast::<HtmlAudioElement>() {
                audio.set_loop(true);

                if is_friday && sound_enabled {
                    match audio.play() {
                        Ok(_promise) => {
                            audio_error.set(None);
                        }
                        Err(err) => {
                            web_sys::console::log_1(&err);
                            audio_error.set(Some(
                                "Audio could not start automatically. Tap Sound On again."
                                    .to_string(),
                            ));
                        }
                    }
                } else {
                    let _ = audio.pause();
                    audio.set_current_time(0.0);
                    audio_error.set(None);
                }
            }

            || ()
        });
    }

    let on_toggle_sound = {
        let sound_on = sound_on.clone();
        let audio_ref = audio_ref.clone();
        let state = state.clone();
        let audio_error = audio_error.clone();

        Callback::from(move |_| {
            let enable = !*sound_on;
            sound_on.set(enable);

            if let Some(audio) = audio_ref.cast::<HtmlAudioElement>() {
                audio.set_loop(true);

                if enable && (*state).is_friday_13th {
                    audio.set_muted(false);

                    match audio.play() {
                        Ok(_promise) => {
                            web_sys::console::log_1(&"Play attempt OK".into());
                            audio_error.set(None);
                        }
                        Err(err) => {
                            web_sys::console::log_1(&err);
                            audio_error.set(Some(
                                "Playback was blocked or the MP3 was not found.".to_string(),
                            ));
                        }
                    }
                } else {
                    let _ = audio.pause();
                    audio.set_current_time(0.0);
                    audio_error.set(None);
                }
            } else {
                audio_error.set(Some("Audio element was not found.".to_string()));
            }
        })
    };

    let media_class = if state.is_friday_13th {
        "media-pane"
    } else {
        "media-pane duck"
    };

    let date_chip_class = if state.is_friday_13th {
        "date-chip"
    } else {
        "date-chip safe"
    };

    html! {
        <div class="page">
            <audio
                ref={audio_ref}
                src="assets/audio/friday13.mp3"
                preload="auto"
                playsinline=true
            />

            <main class="shell">
                <section class="hero">
                    <div class="hero-inner">
                        <div class={media_class}>
                            <div class="media-overlay"></div>
                        </div>

                        <div class="content-pane">
                            <div class="kicker">
                                {
                                    if state.is_friday_13th {
                                        "🔪 Friday the 13th is live today"
                                    } else {
                                        "🦆 The 13th has passed for now"
                                    }
                                }
                            </div>

                            <h1 class="title">
                                { "Friday the " }<span class="accent">{ "13th" }</span>{ " Countdown" }
                            </h1>

                            <p class="subtitle">
                                {
                                    if state.is_friday_13th {
                                        "Jason stays on screen all day on Friday the 13th. At midnight, the app automatically switches to the duck and stops the theme."
                                    } else {
                                        "The app has automatically switched to the duck because it is no longer Friday the 13th. The countdown keeps rolling toward the next one."
                                    }
                                }
                            </p>

                            <div class={date_chip_class}>
                                {
                                    if state.is_friday_13th {
                                        format!("Today is Friday the 13th • Next one: {}", state.next_label)
                                    } else {
                                        format!("Next Friday the 13th: {}", state.next_label)
                                    }
                                }
                            </div>

                            <div class="count-grid">
                                <div class="count-card">
                                    <div class="count-value">{ state.countdown.days }</div>
                                    <div class="count-label">{ "Days" }</div>
                                </div>
                                <div class="count-card">
                                    <div class="count-value">{ state.countdown.hours }</div>
                                    <div class="count-label">{ "Hours" }</div>
                                </div>
                                <div class="count-card">
                                    <div class="count-value">{ state.countdown.minutes }</div>
                                    <div class="count-label">{ "Minutes" }</div>
                                </div>
                                <div class="count-card">
                                    <div class="count-value">{ state.countdown.seconds }</div>
                                    <div class="count-label">{ "Seconds" }</div>
                                </div>
                            </div>

                            {
                                if state.is_friday_13th {
                                    html! {
                                        <div class="controls">
                                            <button
                                                class={classes!("btn", if *sound_on { "btn-primary" } else { "btn-secondary" })}
                                                onclick={on_toggle_sound}
                                                type="button"
                                            >
                                                {
                                                    if *sound_on {
                                                        "🔊 Sound On"
                                                    } else {
                                                        "🔇 Sound Off"
                                                    }
                                                }
                                            </button>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }

                            <p class="note">
                                { format!("Local time: {}", state.now_label) }
                                <br />
                                { "On iPhone/Safari, audio playback usually requires a user tap." }
                                {
                                    if let Some(err) = &*audio_error {
                                        html! {
                                            <>
                                                <br />
                                                { err }
                                            </>
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
                            </p>
                        </div>
                    </div>
                </section>
            </main>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}