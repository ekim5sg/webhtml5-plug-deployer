use js_sys::{Array, Uint8Array};
use web_sys::{Blob, HtmlAudioElement, HtmlInputElement, Url};
use yew::prelude::*;

const PI_DIGITS: &str = "314159265358979323846264338327950288419716939937510\
58209749445923078164062862089986280348253421170679\
82148086513282306647093844609550582231725359408128\
48111745028410270193852110555964462294895493038196\
44288109756659334461284756482337867831652712019091\
45648566923460348610454326648213393607260249141273\
72458700660631558817488152092096282925409171536436";

const SAMPLE_RATE: u32 = 22050;
const PREVIEW_LIMIT: usize = 48;

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
}

#[derive(Clone, PartialEq)]
enum Instrument {
    Piano,
    Oboe,
    ElectricGuitar,
    Violin,
}

impl Instrument {
    fn label(&self) -> &'static str {
        match self {
            Instrument::Piano => "Piano",
            Instrument::Oboe => "Oboe",
            Instrument::ElectricGuitar => "Electric Guitar",
            Instrument::Violin => "Violin",
        }
    }

    fn amplitude(&self) -> f32 {
        match self {
            Instrument::Piano => 0.34,
            Instrument::Oboe => 0.28,
            Instrument::ElectricGuitar => 0.25,
            Instrument::Violin => 0.30,
        }
    }

    fn attack_ratio(&self) -> f32 {
        match self {
            Instrument::Piano => 0.01,
            Instrument::Oboe => 0.06,
            Instrument::ElectricGuitar => 0.015,
            Instrument::Violin => 0.08,
        }
    }

    fn release_ratio(&self) -> f32 {
        match self {
            Instrument::Piano => 0.45,
            Instrument::Oboe => 0.18,
            Instrument::ElectricGuitar => 0.28,
            Instrument::Violin => 0.22,
        }
    }
}

fn digit_to_scale_degree(digit: u32) -> usize {
    match digit {
        0 => 0,
        1 => 0,
        2 => 1,
        3 => 2,
        4 => 3,
        5 => 4,
        6 => 5,
        7 => 6,
        8 => 4,
        9 => 5,
        _ => 0,
    }
}

fn musical_freq_from_digit(index: usize, digit: u32, mode: &MusicMode) -> Option<f32> {
    if digit == 0 {
        return None;
    }

    let major_scale = [261.63_f32, 293.66, 329.63, 349.23, 392.00, 440.00, 493.88];
    let pentatonic = [261.63_f32, 293.66, 329.63, 392.00, 440.00];
    let space_scale = [261.63_f32, 311.13, 392.00, 466.16, 523.25, 622.25];

    let octave_shift = match (digit + (index as u32 % 3)) % 3 {
        0 => 1.0_f32,
        1 => 2.0_f32,
        _ => 0.5_f32,
    };

    match mode {
        MusicMode::Calm => {
            let base = pentatonic[digit_to_scale_degree(digit) % pentatonic.len()];
            Some(base * octave_shift.min(1.0).max(0.5))
        }
        MusicMode::Arcade => {
            let base = major_scale[digit_to_scale_degree(digit) % major_scale.len()];
            Some(base * octave_shift)
        }
        MusicMode::Space => {
            let idx = (digit as usize + index) % space_scale.len();
            Some(space_scale[idx] * octave_shift)
        }
    }
}

fn digit_to_note_name(index: usize, digit: u32, mode: &MusicMode) -> &'static str {
    if digit == 0 {
        return "Rest";
    }

    match mode {
        MusicMode::Calm => match digit_to_scale_degree(digit) % 5 {
            0 => "C",
            1 => "D",
            2 => "E",
            3 => "G",
            _ => "A",
        },
        MusicMode::Arcade => match digit_to_scale_degree(digit) % 7 {
            0 => "C",
            1 => "D",
            2 => "E",
            3 => "F",
            4 => "G",
            5 => "A",
            _ => "B",
        },
        MusicMode::Space => {
            let names = ["C", "Eb", "G", "Bb", "C5", "Eb5"];
            names[(digit as usize + index) % names.len()]
        }
    }
}

fn note_duration_secs(digit: u32, bpm: u32, index: usize) -> f32 {
    let quarter = 60.0 / bpm.max(1) as f32;

    if index % 8 == 7 {
        quarter * 1.4
    } else if digit == 0 {
        quarter * 0.4
    } else if digit % 2 == 0 {
        quarter * 0.55
    } else {
        quarter * 0.9
    }
}

fn preview_notes(total_digits: usize, mode: &MusicMode) -> String {
    let notes = PI_DIGITS
        .chars()
        .cycle()
        .take(total_digits.min(PREVIEW_LIMIT))
        .enumerate()
        .filter_map(|(i, ch)| ch.to_digit(10).map(|d| digit_to_note_name(i, d, mode)))
        .collect::<Vec<_>>();

    if total_digits > PREVIEW_LIMIT {
        format!("{} • ... (showing first {} of {} notes)", notes.join(" • "), PREVIEW_LIMIT, total_digits)
    } else {
        notes.join(" • ")
    }
}

fn instrument_sample(instrument: &Instrument, t: f32, freq: f32) -> f32 {
    let p = 2.0 * std::f32::consts::PI * freq * t;

    match instrument {
        Instrument::Piano => {
            0.90 * p.sin()
                + 0.25 * (2.0 * p).sin()
                + 0.12 * (3.0 * p).sin()
        }
        Instrument::Oboe => {
            0.70 * p.sin()
                + 0.45 * (2.0 * p).sin()
                + 0.30 * (3.0 * p).sin()
                + 0.18 * (4.0 * p).sin()
        }
        Instrument::ElectricGuitar => {
            let raw = 0.85 * p.sin()
                + 0.35 * (2.0 * p).sin()
                + 0.22 * (3.0 * p).sin();
            (raw * 1.8).tanh()
        }
        Instrument::Violin => {
            0.75 * p.sin()
                + 0.20 * (2.0 * p).sin()
                + 0.15 * (3.0 * p).sin()
                + 0.08 * (4.0 * p).sin()
        }
    }
}

fn apply_envelope(i: usize, sample_count: usize, instrument: &Instrument) -> f32 {
    let attack = ((sample_count as f32) * instrument.attack_ratio()).max(1.0) as usize;
    let release = ((sample_count as f32) * instrument.release_ratio()).max(1.0) as usize;

    if i < attack {
        i as f32 / attack as f32
    } else if i > sample_count.saturating_sub(release) {
        let remain = sample_count.saturating_sub(i);
        remain as f32 / release as f32
    } else {
        1.0
    }
}

fn generate_wav_bytes(
    total_digits: usize,
    bpm: u32,
    mode: &MusicMode,
    instrument: &Instrument,
) -> Vec<u8> {
    let digits: Vec<u32> = PI_DIGITS
        .chars()
        .cycle()
        .take(total_digits)
        .filter_map(|ch| ch.to_digit(10))
        .collect();

    let mut samples: Vec<i16> = Vec::new();

    for (index, digit) in digits.into_iter().enumerate() {
        let duration = note_duration_secs(digit, bpm, index);
        let sample_count = (duration * SAMPLE_RATE as f32) as usize;

        if let Some(freq) = musical_freq_from_digit(index, digit, mode) {
            for i in 0..sample_count {
                let t = i as f32 / SAMPLE_RATE as f32;
                let env = apply_envelope(i, sample_count, instrument);
                let value = instrument_sample(instrument, t, freq)
                    * env
                    * instrument.amplitude();
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
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    let byte_rate = SAMPLE_RATE * 2;
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes());
    wav.extend_from_slice(&16u16.to_le_bytes());

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
    let digits = use_state(|| 64usize);
    let bpm = use_state(|| 120u32);
    let mode = use_state(|| MusicMode::Calm);
    let instrument = use_state(|| Instrument::Piano);
    let is_playing = use_state(|| false);
    let status_text = use_state(|| "Ready".to_string());

    let audio_ref = use_mut_ref(|| None::<HtmlAudioElement>);
    let current_url = use_mut_ref(|| None::<String>);

    let notes_preview = preview_notes(*digits, &mode);

    let on_digits_input = {
        let digits = digits.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value().parse::<usize>().unwrap_or(64).clamp(8, 1000);
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

    let set_piano = {
        let instrument = instrument.clone();
        Callback::from(move |_| instrument.set(Instrument::Piano))
    };

    let set_oboe = {
        let instrument = instrument.clone();
        Callback::from(move |_| instrument.set(Instrument::Oboe))
    };

    let set_guitar = {
        let instrument = instrument.clone();
        Callback::from(move |_| instrument.set(Instrument::ElectricGuitar))
    };

    let set_violin = {
        let instrument = instrument.clone();
        Callback::from(move |_| instrument.set(Instrument::Violin))
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
        let instrument = instrument.clone();
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

            status_text.set("Generating WAV...".to_string());

            let bytes = generate_wav_bytes(*digits, *bpm, &mode, &instrument);
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
                        "Playing {} notes • {} • {}",
                        *digits,
                        mode.label(),
                        instrument.label()
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
                <h1>{ "Pi Music Generator v2" }</h1>
                <p>
                    { "Turn the digits of π into a longer, more musical melody with instrument-style synthesis and reliable iPhone-friendly WAV playback." }
                </p>
            </section>

            <section class="grid">
                <aside class="card">
                    <h2>{ "Music Controls" }</h2>

                    <div class="control-group">
                        <div class="control-label">
                            <span>{ "Notes Used" }</span>
                            <span class="control-value">{ *digits }</span>
                        </div>
                        <input
                            type="range"
                            min="8"
                            max="1000"
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

                    <h3>{ "Melody Mode" }</h3>
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

                    <h3>{ "Instrument Flavor" }</h3>
                    <div class="instrument-row">
                        <button
                            class={classes!("secondary", matches!(*instrument, Instrument::Piano).then_some("active"))}
                            onclick={set_piano}
                        >
                            { "Piano" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*instrument, Instrument::Oboe).then_some("active"))}
                            onclick={set_oboe}
                        >
                            { "Oboe" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*instrument, Instrument::ElectricGuitar).then_some("active"))}
                            onclick={set_guitar}
                        >
                            { "Electric Guitar" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*instrument, Instrument::Violin).then_some("active"))}
                            onclick={set_violin}
                        >
                            { "Violin" }
                        </button>
                    </div>

                    <h3>{ "Playback" }</h3>
                    <div class="button-row">
                        <button class="primary" onclick={start_playback}>{ "Play" }</button>
                        <button class="secondary" onclick={stop_playback}>{ "Stop" }</button>
                    </div>

                    <div class="desc-box">
                        { "This version keeps the proven iPhone-safe path: synthesize in Rust, generate a WAV in memory, then play it through HTMLAudioElement." }
                    </div>
                </aside>

                <main class="card">
                    <h2>{ "Pi Melody Preview" }</h2>

                    <span class="status-pill">{ (*status_text).clone() }</span>

                    <div class="info-grid">
                        <div class="info-card">
                            <div class="info-label">{ "Mode" }</div>
                            <div class="info-value">{ mode.label() }</div>
                        </div>
                        <div class="info-card">
                            <div class="info-label">{ "Instrument" }</div>
                            <div class="info-value">{ instrument.label() }</div>
                        </div>
                        <div class="info-card">
                            <div class="info-label">{ "Notes" }</div>
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
                        { " Let π compose a longer melody." }
                    </div>
                </main>
            </section>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}