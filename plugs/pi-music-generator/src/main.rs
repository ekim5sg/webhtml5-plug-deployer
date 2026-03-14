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

const SAMPLE_RATE: u32 = 22_050;
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
enum Preset {
    SoftKeys,
    BrightPluck,
    BowedAir,
    CosmicLead,
}

impl Preset {
    fn label(&self) -> &'static str {
        match self {
            Preset::SoftKeys => "Soft Keys",
            Preset::BrightPluck => "Bright Pluck",
            Preset::BowedAir => "Bowed Air",
            Preset::CosmicLead => "Cosmic Lead",
        }
    }

    fn amplitude(&self) -> f32 {
        match self {
            Preset::SoftKeys => 0.34,
            Preset::BrightPluck => 0.28,
            Preset::BowedAir => 0.27,
            Preset::CosmicLead => 0.30,
        }
    }

    fn attack_ratio(&self) -> f32 {
        match self {
            Preset::SoftKeys => 0.02,
            Preset::BrightPluck => 0.004,
            Preset::BowedAir => 0.10,
            Preset::CosmicLead => 0.025,
        }
    }

    fn release_ratio(&self) -> f32 {
        match self {
            Preset::SoftKeys => 0.34,
            Preset::BrightPluck => 0.48,
            Preset::BowedAir => 0.20,
            Preset::CosmicLead => 0.28,
        }
    }

    fn vibrato_depth(&self) -> f32 {
        match self {
            Preset::SoftKeys => 0.0015,
            Preset::BrightPluck => 0.0,
            Preset::BowedAir => 0.010,
            Preset::CosmicLead => 0.014,
        }
    }

    fn vibrato_rate(&self) -> f32 {
        match self {
            Preset::SoftKeys => 4.0,
            Preset::BrightPluck => 0.0,
            Preset::BowedAir => 5.4,
            Preset::CosmicLead => 6.8,
        }
    }

    fn echo_mix(&self) -> f32 {
        match self {
            Preset::SoftKeys => 0.08,
            Preset::BrightPluck => 0.06,
            Preset::BowedAir => 0.12,
            Preset::CosmicLead => 0.18,
        }
    }

    fn echo_delay_ms(&self) -> u32 {
        match self {
            Preset::SoftKeys => 90,
            Preset::BrightPluck => 70,
            Preset::BowedAir => 120,
            Preset::CosmicLead => 150,
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

fn note_duration_secs(digit: u32, bpm: u32, index: usize, mode: &MusicMode) -> f32 {
    let quarter = 60.0 / bpm.max(1) as f32;

    let base = if index % 8 == 7 {
        quarter * 1.35
    } else if digit == 0 {
        quarter * 0.38
    } else if digit % 2 == 0 {
        quarter * 0.52
    } else {
        quarter * 0.86
    };

    match mode {
        MusicMode::Calm => base * 1.08,
        MusicMode::Arcade => base * 0.88,
        MusicMode::Space => base * 1.00,
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
        format!(
            "{} • ... (showing first {} of {} notes)",
            notes.join(" • "),
            PREVIEW_LIMIT,
            total_digits
        )
    } else {
        notes.join(" • ")
    }
}

fn soft_clip(x: f32) -> f32 {
    (x * 1.4).tanh()
}

fn synth_soft_keys(t: f32, freq: f32) -> f32 {
    let p = 2.0 * std::f32::consts::PI * freq * t;
    let body = 0.88 * p.sin()
        + 0.20 * (2.0 * p).sin()
        + 0.09 * (3.0 * p).sin()
        + 0.04 * (4.0 * p).sin();
    soft_clip(body)
}

fn synth_bright_pluck(t: f32, freq: f32) -> f32 {
    let p = 2.0 * std::f32::consts::PI * freq * t;
    let sparkle = 0.72 * p.sin()
        + 0.34 * (2.0 * p).sin()
        + 0.20 * (3.0 * p).sin()
        + 0.10 * (5.0 * p).sin();
    soft_clip(sparkle * 1.05)
}

fn synth_bowed_air(t: f32, freq: f32, vib: f32) -> f32 {
    let f = freq * (1.0 + vib);
    let p = 2.0 * std::f32::consts::PI * f * t;
    let smooth = 0.78 * p.sin()
        + 0.26 * (2.0 * p).sin()
        + 0.14 * (3.0 * p).sin()
        + 0.06 * (4.0 * p).sin();
    soft_clip(smooth * 0.95)
}

fn synth_cosmic_lead(t: f32, freq: f32, vib: f32) -> f32 {
    let f = freq * (1.0 + vib);
    let p = 2.0 * std::f32::consts::PI * f * t;
    let lead = 0.70 * p.sin()
        + 0.22 * (2.0 * p).sin()
        + 0.16 * (3.0 * p).sin()
        + 0.08 * (0.5 * p).sin();
    soft_clip(lead * 1.15)
}

fn preset_sample(preset: &Preset, t: f32, freq: f32) -> f32 {
    let vib = if preset.vibrato_depth() > 0.0 {
        (2.0 * std::f32::consts::PI * preset.vibrato_rate() * t).sin() * preset.vibrato_depth()
    } else {
        0.0
    };

    match preset {
        Preset::SoftKeys => synth_soft_keys(t, freq),
        Preset::BrightPluck => synth_bright_pluck(t, freq),
        Preset::BowedAir => synth_bowed_air(t, freq, vib),
        Preset::CosmicLead => synth_cosmic_lead(t, freq, vib),
    }
}

fn apply_envelope(i: usize, sample_count: usize, preset: &Preset) -> f32 {
    let attack = ((sample_count as f32) * preset.attack_ratio()).max(1.0) as usize;
    let release = ((sample_count as f32) * preset.release_ratio()).max(1.0) as usize;

    if i < attack {
        i as f32 / attack as f32
    } else if i > sample_count.saturating_sub(release) {
        let remain = sample_count.saturating_sub(i);
        remain as f32 / release as f32
    } else {
        1.0
    }
}

fn add_echo(samples: &mut [f32], preset: &Preset) {
    let delay_samples = ((preset.echo_delay_ms() as f32 / 1000.0) * SAMPLE_RATE as f32) as usize;
    if delay_samples == 0 || delay_samples >= samples.len() {
        return;
    }

    let mix = preset.echo_mix();
    for i in delay_samples..samples.len() {
        let delayed = samples[i - delay_samples] * mix;
        samples[i] = (samples[i] + delayed).clamp(-1.0, 1.0);
    }
}

fn generate_wav_bytes(
    total_digits: usize,
    bpm: u32,
    mode: &MusicMode,
    preset: &Preset,
) -> Vec<u8> {
    let digits: Vec<u32> = PI_DIGITS
        .chars()
        .cycle()
        .take(total_digits)
        .filter_map(|ch| ch.to_digit(10))
        .collect();

    let mut float_samples: Vec<f32> = Vec::new();

    for (index, digit) in digits.into_iter().enumerate() {
        let duration = note_duration_secs(digit, bpm, index, mode);
        let sample_count = (duration * SAMPLE_RATE as f32) as usize;

        if let Some(freq) = musical_freq_from_digit(index, digit, mode) {
            for i in 0..sample_count {
                let t = i as f32 / SAMPLE_RATE as f32;
                let env = apply_envelope(i, sample_count, preset);
                let value = preset_sample(preset, t, freq) * env * preset.amplitude();
                float_samples.push(value.clamp(-1.0, 1.0));
            }
        } else {
            float_samples.extend(std::iter::repeat_n(0.0_f32, sample_count));
        }
    }

    add_echo(&mut float_samples, preset);

    let samples: Vec<i16> = float_samples
        .into_iter()
        .map(|v| (v.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
        .collect();

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
    let preset = use_state(|| Preset::SoftKeys);
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

    let set_soft_keys = {
        let preset = preset.clone();
        Callback::from(move |_| preset.set(Preset::SoftKeys))
    };

    let set_bright_pluck = {
        let preset = preset.clone();
        Callback::from(move |_| preset.set(Preset::BrightPluck))
    };

    let set_bowed_air = {
        let preset = preset.clone();
        Callback::from(move |_| preset.set(Preset::BowedAir))
    };

    let set_cosmic_lead = {
        let preset = preset.clone();
        Callback::from(move |_| preset.set(Preset::CosmicLead))
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
        let preset = preset.clone();
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

            let bytes = generate_wav_bytes(*digits, *bpm, &mode, &preset);
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
                        preset.label()
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
                <h1>{ "Pi Music Generator v3" }</h1>
                <p>
                    { "Turn the digits of π into a longer, more musical melody with improved synth presets and the proven iPhone-safe WAV playback path." }
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

                    <h3>{ "Preset" }</h3>
                    <div class="instrument-row">
                        <button
                            class={classes!("secondary", matches!(*preset, Preset::SoftKeys).then_some("active"))}
                            onclick={set_soft_keys}
                        >
                            { "Soft Keys" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*preset, Preset::BrightPluck).then_some("active"))}
                            onclick={set_bright_pluck}
                        >
                            { "Bright Pluck" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*preset, Preset::BowedAir).then_some("active"))}
                            onclick={set_bowed_air}
                        >
                            { "Bowed Air" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*preset, Preset::CosmicLead).then_some("active"))}
                            onclick={set_cosmic_lead}
                        >
                            { "Cosmic Lead" }
                        </button>
                    </div>

                    <h3>{ "Playback" }</h3>
                    <div class="button-row">
                        <button class="primary" onclick={start_playback}>{ "Play" }</button>
                        <button class="secondary" onclick={stop_playback}>{ "Stop" }</button>
                    </div>

                    <div class="desc-box">
                        { "Path A+ keeps the reliable approach: generate the sound in Rust, bake it into a WAV, then play it through HTMLAudioElement for iPhone-safe playback." }
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
                            <div class="info-label">{ "Preset" }</div>
                            <div class="info-value">{ preset.label() }</div>
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
                        { " Let π compose a longer, better-sounding melody." }
                    </div>
                </main>
            </section>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}