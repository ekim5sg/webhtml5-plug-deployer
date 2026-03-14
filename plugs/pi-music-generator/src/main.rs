use gloo::timers::callback::Interval;
use web_sys::{window, AudioContext, GainNode, HtmlInputElement, OscillatorNode, OscillatorType};
use yew::prelude::*;

const PI_DIGITS: &str = "314159265358979323846264338327950288419716939937510\
58209749445923078164062862089986280348253421170679\
82148086513282306647093844609550582231725359408128\
48111745028410270193852110555964462294895493038196";

#[derive(Clone, PartialEq)]
enum MusicMode {
    Calm,
    Arcade,
    Space,
}

impl MusicMode {
    fn label(&self) -> &'static str {
        match self {
            MusicMode::Calm => "Calm",
            MusicMode::Arcade => "Arcade",
            MusicMode::Space => "Space",
        }
    }

    fn oscillator(&self) -> OscillatorType {
        match self {
            MusicMode::Calm => OscillatorType::Sine,
            MusicMode::Arcade => OscillatorType::Square,
            MusicMode::Space => OscillatorType::Triangle,
        }
    }

    fn gain_multiplier(&self) -> f32 {
        match self {
            MusicMode::Calm => 0.06,
            MusicMode::Arcade => 0.045,
            MusicMode::Space => 0.055,
        }
    }
}

fn digit_to_freq(digit: u32) -> Option<f32> {
    match digit {
        0 => None,
        1 => Some(261.63), // C4
        2 => Some(293.66), // D4
        3 => Some(329.63), // E4
        4 => Some(349.23), // F4
        5 => Some(392.00), // G4
        6 => Some(440.00), // A4
        7 => Some(493.88), // B4
        8 => Some(523.25), // C5
        9 => Some(587.33), // D5
        _ => None,
    }
}

fn digit_to_note_name(digit: u32) -> &'static str {
    match digit {
        0 => "Rest",
        1 => "C4",
        2 => "D4",
        3 => "E4",
        4 => "F4",
        5 => "G4",
        6 => "A4",
        7 => "B4",
        8 => "C5",
        9 => "D5",
        _ => "?",
    }
}

fn note_duration_ms(digit: u32, bpm: u32) -> u32 {
    let quarter = 60_000 / bpm.max(1);
    if digit % 2 == 0 {
        quarter / 2
    } else {
        quarter
    }
}

fn preview_notes(total_digits: usize) -> String {
    PI_DIGITS
        .chars()
        .take(total_digits)
        .filter_map(|ch| ch.to_digit(10))
        .map(digit_to_note_name)
        .collect::<Vec<_>>()
        .join(" • ")
}

fn play_note(ctx: &AudioContext, freq: Option<f32>, mode: &MusicMode, duration_ms: u32) {
    let Some(freq) = freq else {
        return;
    };

    let Ok(osc) = OscillatorNode::new(ctx) else {
        return;
    };
    let Ok(gain) = GainNode::new(ctx) else {
        return;
    };

    osc.set_type(mode.oscillator());
    osc.frequency().set_value(freq);
    gain.gain().set_value(mode.gain_multiplier());

    let _ = osc.connect_with_audio_node(&gain);
    let _ = gain.connect_with_audio_node(&ctx.destination());

    let now = ctx.current_time();
    let dur_secs = duration_ms as f64 / 1000.0;

    let _ = gain.gain().set_value_at_time(mode.gain_multiplier(), now);
    let _ = gain.gain().linear_ramp_to_value_at_time(0.0001, now + dur_secs * 0.95);

    let _ = osc.start();
    let _ = osc.stop_with_when(now + dur_secs);
}

#[function_component(App)]
fn app() -> Html {
    let digits = use_state(|| 24usize);
    let bpm = use_state(|| 120u32);
    let mode = use_state(|| MusicMode::Calm);
    let is_playing = use_state(|| false);
    let current_index = use_state(|| 0usize);

    let audio_ctx = use_mut_ref(|| None::<AudioContext>);
    let playback_interval = use_mut_ref(|| None::<Interval>);

    let notes_preview = preview_notes(*digits);

    let on_digits_input = {
        let digits = digits.clone();
        let current_index = current_index.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input
                .value()
                .parse::<usize>()
                .unwrap_or(24)
                .clamp(8, 96);
            digits.set(value);
            current_index.set(0);
        })
    };

    let on_bpm_input = {
        let bpm = bpm.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input
                .value()
                .parse::<u32>()
                .unwrap_or(120)
                .clamp(60, 220);
            bpm.set(value);
        })
    };

    let set_calm = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(MusicMode::Calm))
    };

    let set_arcade = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(MusicMode::Arcade))
    };

    let set_space = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(MusicMode::Space))
    };

    let stop_playback = {
        let is_playing = is_playing.clone();
        let current_index = current_index.clone();
        let playback_interval = playback_interval.clone();

        Callback::from(move |_| {
            *playback_interval.borrow_mut() = None;
            is_playing.set(false);
            current_index.set(0);
        })
    };

    let start_playback = {
        let digits = digits.clone();
        let bpm = bpm.clone();
        let mode = mode.clone();
        let is_playing = is_playing.clone();
        let current_index = current_index.clone();
        let audio_ctx = audio_ctx.clone();
        let playback_interval = playback_interval.clone();

        Callback::from(move |_| {
            *playback_interval.borrow_mut() = None;

            let ctx = if let Some(existing) = audio_ctx.borrow().as_ref() {
                existing.clone()
            } else {
                let Ok(created) = AudioContext::new() else {
                    return;
                };
                *audio_ctx.borrow_mut() = Some(created.clone());
                created
            };

            let _ = ctx.resume();

            current_index.set(0);
            is_playing.set(true);

            let digits_value = *digits;
            let bpm_value = *bpm;
            let mode_value = (*mode).clone();

            let current_index_handle = current_index.clone();
            let is_playing_handle = is_playing.clone();
            let ctx_clone = ctx.clone();

            *playback_interval.borrow_mut() = Some(Interval::new(10, move || {
                let index = *current_index_handle;

                if index >= digits_value {
                    is_playing_handle.set(false);
                    return;
                }

                let ch = PI_DIGITS.chars().nth(index).unwrap_or('0');
                let digit = ch.to_digit(10).unwrap_or(0);
                let freq = digit_to_freq(digit);
                let dur = note_duration_ms(digit, bpm_value);

                play_note(&ctx_clone, freq, &mode_value, dur);
                current_index_handle.set(index + 1);
            }));

            // Replace fast polling with timed musical stepping using chained interval recreation.
            let digits_value2 = *digits;
            let bpm_value2 = *bpm;
            let mode_value2 = (*mode).clone();
            let current_index_handle2 = current_index.clone();
            let is_playing_handle2 = is_playing.clone();
            let ctx_clone2 = ctx.clone();
            let playback_interval2 = playback_interval.clone();

            let schedule_next = move || {
                let index = *current_index_handle2;
                if index >= digits_value2 {
                    *playback_interval2.borrow_mut() = None;
                    is_playing_handle2.set(false);
                    current_index_handle2.set(0);
                    return;
                }

                let ch = PI_DIGITS.chars().nth(index).unwrap_or('0');
                let digit = ch.to_digit(10).unwrap_or(0);
                let freq = digit_to_freq(digit);
                let dur = note_duration_ms(digit, bpm_value2);

                play_note(&ctx_clone2, freq, &mode_value2, dur);
                current_index_handle2.set(index + 1);
            };

            *playback_interval.borrow_mut() = None;

            let digits_value3 = *digits;
            let bpm_value3 = *bpm;
            let mode_value3 = (*mode).clone();
            let current_index_handle3 = current_index.clone();
            let is_playing_handle3 = is_playing.clone();
            let ctx_clone3 = ctx.clone();
            let playback_interval3 = playback_interval.clone();

            let tick = move || {
                let index = *current_index_handle3;
                if index >= digits_value3 {
                    *playback_interval3.borrow_mut() = None;
                    is_playing_handle3.set(false);
                    current_index_handle3.set(0);
                    return;
                }

                let ch = PI_DIGITS.chars().nth(index).unwrap_or('0');
                let digit = ch.to_digit(10).unwrap_or(0);
                let freq = digit_to_freq(digit);
                let dur = note_duration_ms(digit, bpm_value3);

                play_note(&ctx_clone3, freq, &mode_value3, dur);
                current_index_handle3.set(index + 1);

                let next_delay = dur;
                *playback_interval3.borrow_mut() = Some(Interval::new(next_delay, {
                    let playback_interval_inner = playback_interval3.clone();
                    let current_index_inner = current_index_handle3.clone();
                    let is_playing_inner = is_playing_handle3.clone();
                    let ctx_inner = ctx_clone3.clone();
                    let mode_inner = mode_value3.clone();

                    move || {
                        let idx = *current_index_inner;
                        if idx >= digits_value3 {
                            *playback_interval_inner.borrow_mut() = None;
                            is_playing_inner.set(false);
                            current_index_inner.set(0);
                            return;
                        }

                        let ch2 = PI_DIGITS.chars().nth(idx).unwrap_or('0');
                        let digit2 = ch2.to_digit(10).unwrap_or(0);
                        let freq2 = digit_to_freq(digit2);
                        let dur2 = note_duration_ms(digit2, bpm_value3);

                        play_note(&ctx_inner, freq2, &mode_inner, dur2);
                        current_index_inner.set(idx + 1);
                    }
                }));
            };

            tick();
        })
    };

    html! {
        <div class="app-shell">
            <section class="hero">
                <div class="kicker">{ "MIKEGYVER STUDIO • PI DAY AUDIO BUILD" }</div>
                <h1>{ "Pi Music Generator" }</h1>
                <p>
                    { "What if the digits of π became music? This mini-app maps π into notes, rests, rhythm, and tone so the number becomes a melody." }
                </p>
            </section>

            <section class="grid">
                <aside class="card">
                    <h2>{ "Music Controls" }</h2>

                    <div class="control-group">
                        <div class="control-label">
                            <span>{ "Digits Used" }</span>
                            <span class="control-value">{ *digits }</span>
                        </div>
                        <input
                            type="range"
                            min="8"
                            max="96"
                            step="1"
                            value={digits.to_string()}
                            oninput={on_digits_input}
                        />
                    </div>

                    <div class="control-group">
                        <div class="control-label">
                            <span>{ "Tempo" }</span>
                            <span class="control-value">{ format!("{} BPM", *bpm) }</span>
                        </div>
                        <input
                            type="range"
                            min="60"
                            max="220"
                            step="1"
                            value={bpm.to_string()}
                            oninput={on_bpm_input}
                        />
                    </div>

                    <h3>{ "Mode" }</h3>
                    <div class="mode-row">
                        <button
                            class={classes!("secondary", matches!(*mode, MusicMode::Calm).then_some("active"))}
                            onclick={set_calm}
                        >
                            { "Calm" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*mode, MusicMode::Arcade).then_some("active"))}
                            onclick={set_arcade}
                        >
                            { "Arcade" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*mode, MusicMode::Space).then_some("active"))}
                            onclick={set_space}
                        >
                            { "Space" }
                        </button>
                    </div>

                    <h3>{ "Playback" }</h3>
                    <div class="button-row">
                        <button class="primary" onclick={start_playback}>{ "Play" }</button>
                        <button class="secondary" onclick={stop_playback}>{ "Stop" }</button>
                    </div>

                    <div class="desc-box">
                        { "Digit mapping: 1=C, 2=D, 3=E, 4=F, 5=G, 6=A, 7=B, 8=high C, 9=high D, 0=rest. Odd digits hold longer, even digits play shorter." }
                    </div>
                </aside>

                <main class="card">
                    <h2>{ "Pi Melody Preview" }</h2>

                    <span class="status-pill">
                        {
                            if *is_playing {
                                format!("Playing • Note {}", (*current_index).min(*digits))
                            } else {
                                "Ready".to_string()
                            }
                        }
                    </span>

                    <div class="info-grid">
                        <div class="info-card">
                            <div class="info-label">{ "Mode" }</div>
                            <div class="info-value">{ mode.label() }</div>
                        </div>
                        <div class="info-card">
                            <div class="info-label">{ "Digits" }</div>
                            <div class="info-value">{ *digits }</div>
                        </div>
                        <div class="info-card">
                            <div class="info-label">{ "Tempo" }</div>
                            <div class="info-value">{ *bpm }</div>
                        </div>
                    </div>

                    <div class="notes-box">
                        <h3 style="margin-top:0;">{ "Generated Note Sequence" }</h3>
                        <div class="note-seq">{ notes_preview }</div>
                    </div>

                    <div class="footer">
                        <strong>{ "Pi Day music mission:" }</strong>
                        { " Let π compose the melody." }
                    </div>
                </main>
            </section>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}