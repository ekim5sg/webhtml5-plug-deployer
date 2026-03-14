use js_sys::{Array, Uint8Array};
use wasm_bindgen::JsCast;
use web_sys::{window, Blob, HtmlAudioElement, HtmlInputElement, Url};
use yew::prelude::*;

const PI_DIGITS: &str = "314159265358979323846264338327950288419716939937510\
58209749445923078164062862089986280348253421170679\
82148086513282306647093844609550582231725359408128\
48111745028410270193852110555964462294895493038196";

const SAMPLE_RATE: u32 = 22050;

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

    fn amplitude(&self) -> f32 {
        match self {
            MusicMode::Calm => 0.35,
            MusicMode::Arcade => 0.28,
            MusicMode::Space => 0.32,
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

fn note_duration_secs(digit: u32, bpm: u32) -> f32 {
    let quarter = 60.0 / bpm.max(1) as f32;
    if digit % 2 == 0 {
        quarter / 2.0
    } else {
        quarter
    }
}

fn preview_notes(total_digits: usize) -> String {
    PI_DIGITS
        .chars()
        .cycle()
        .take(total_digits)
        .filter_map(|ch| ch.to_digit(10))
        .map(digit_to_note_name)
        .collect::<Vec<_>>()
        .join(" • ")
}

fn synth_sample(mode: &MusicMode, t: f32, freq: f32) -> f32 {
    let phase = 2.0 * std::f32::consts::PI * freq * t;
    match mode {
        MusicMode::Calm => phase.sin(),
        MusicMode::Arcade => {
            if phase.sin() >= 0.0 { 1.0 } else { -1.0 }
        }
        MusicMode::Space => {
            let tri = (2.0 / std::f32::consts::PI) * phase.sin().asin();
            0.75 * tri + 0.25 * (phase * 0.5).sin()
        }
    }
}

fn generate_wav_bytes(total_digits: usize, bpm: u32, mode: &MusicMode) -> Vec<u8> {
    let digits: Vec<u32> = PI_DIGITS
        .chars()
        .cycle()
        .take(total_digits)
        .filter_map(|ch| ch.to_digit(10))
        .collect();

    let mut samples: Vec<i16> = Vec::new();

    for digit in digits {
        let duration = note_duration_secs(digit, bpm);
        let sample_count = (duration * SAMPLE_RATE as f32) as usize;
        let attack = ((sample_count as f32) * 0.08).max(1.0) as usize;
        let release = ((sample_count as f32) * 0.18).max(1.0) as usize;

        if let Some(freq) = digit_to_freq(digit) {
            for i in 0..sample_count {
                let t = i as f32 / SAMPLE_RATE as f32;
                let mut env = 1.0_f32;

                if i < attack {
                    env = i as f32 / attack as f32;
                } else if i > sample_count.saturating_sub(release) {
                    let remain = sample_count.saturating_sub(i);
                    env = remain as f32 / release as f32;
                }

                let value = synth_sample(mode, t, freq) * env * mode.amplitude();
                let pcm = (value.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                samples.push(pcm);
            }
        } else {
            samples.extend(std::iter::repeat_n(0i16, sample_count));
        }
    }

    let data_len = (samples.len() * 2) as u32;
    let file_len = 36 + data_len;

    let mut wav: Vec<u8> = Vec::with_capacity((44 + data_len) as usize);

    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_len.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // PCM chunk size
    wav.extend_from_slice(&1u16.to_le_bytes());  // PCM format
    wav.extend_from_slice(&1u16.to_le_bytes());  // mono
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    let byte_rate = SAMPLE_RATE * 2;
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes()); // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits/sample

    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());

    for s in samples {
        wav.extend_from_slice(&s.to_le_bytes());
    }

    wav
}

fn make_audio_url(bytes: &[u8]) -> Option<String> {
    let arr = Uint8Array::new_with_length(bytes.len() as u32);
    arr.copy_from(bytes);

    let parts = Array::new();
    parts.push(&arr.buffer());

    let blob = Blob::new_with_u8_array_sequence(&parts).ok()?;
    Url::create_object_url_with_blob(&blob).ok()
}

#[function_component(App)]
fn app() -> Html {
    let digits = use_state(|| 24usize);
    let bpm = use_state(|| 120u32);
    let mode = use_state(|| MusicMode::Calm);
    let is_playing = use_state(|| false);
    let status_text = use_state(|| "Ready".to_string());

    let audio_ref = use_mut_ref(|| None::<HtmlAudioElement>);
    let current_url = use_mut_ref(|| None::<String>);

    let notes_preview = preview_notes(*digits);

    let on_digits_input = {
        let digits = digits.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value().parse::<usize>().unwrap_or(24).clamp(8, 96);
            digits.set(value);
        })
    };

    let on_bpm_input = {
        let bpm = bpm.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value().parse::<u32>().unwrap_or(120).clamp(60, 220);
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
        let status_text = status_text.clone();
        let audio_ref = audio_ref.clone();

        Callback::from(move |_| {
            if let Some(audio) = audio_ref.borrow().as_ref() {
                let _ = audio.pause();
                audio.set_current_time(0.0);
            }
            is_playing.set(false);
            status_text.set("Stopped".to_string());
        })
    };

    let start_playback = {
        let digits = digits.clone();
        let bpm = bpm.clone();
        let mode = mode.clone();
        let is_playing = is_playing.clone();
        let status_text = status_text.clone();
        let audio_ref = audio_ref.clone();
        let current_url = current_url.clone();

        Callback::from(move |_| {
            if let Some(audio) = audio_ref.borrow().as_ref() {
                let _ = audio.pause();
            }

            if let Some(old_url) = current_url.borrow_mut().take() {
                let _ = Url::revoke_object_url(&old_url);
            }

            let bytes = generate_wav_bytes(*digits, *bpm, &mode);
            let Some(url) = make_audio_url(&bytes) else {
                status_text.set("Could not create WAV URL".to_string());
                return;
            };

            let Ok(audio) = HtmlAudioElement::new_with_src(&url) else {
                status_text.set("Could not create audio element".to_string());
                let _ = Url::revoke_object_url(&url);
                return;
            };

            audio.set_preload("auto");

            match audio.play() {
                Ok(_promise) => {
                    *audio_ref.borrow_mut() = Some(audio);
                    *current_url.borrow_mut() = Some(url);
                    is_playing.set(true);
                    status_text.set(format!(
                        "Playing {} notes in {} mode",
                        *digits,
                        mode.label()
                    ));
                }
                Err(_) => {
                    status_text.set("Playback blocked by browser".to_string());
                    let _ = Url::revoke_object_url(&url);
                }
            }
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
                        <button class="primary" onclick={start_playback}>
                            { "Play" }
                        </button>
                        <button class="secondary" onclick={stop_playback}>
                            { "Stop" }
                        </button>
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
                                (*status_text).clone()
                            } else {
                                (*status_text).clone()
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