use gloo_timers::future::TimeoutFuture;
use js_sys::{Array, ArrayBuffer, Uint8Array};
use shine_rs::{encode_pcm_to_mp3, Mp3EncoderConfig, StereoMode};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    AudioBuffer, AudioContext, Blob, BlobPropertyBag, Event, HtmlAnchorElement,
    HtmlAudioElement, HtmlInputElement, InputEvent, Url,
};
use yew::prelude::*;

const FADE_SECONDS: f64 = 12.0;
const OUTPUT_BITRATE: u32 = 192;

#[derive(Clone, PartialEq)]
struct AudioData {
    name: String,
    source_url: String,
    sample_rate: u32,
    channels: Vec<Vec<f32>>,
    duration: f64,
}

#[derive(Clone, PartialEq)]
struct ResultFile {
    url: String,
    name: String,
    bytes: usize,
    duration: f64,
}

fn clock(seconds: f64) -> String {
    let whole = seconds.max(0.0).round() as u64;
    format!("{}:{:02}", whole / 60, whole % 60)
}

fn nearest_supported_rate(rate: u32) -> u32 {
    [32_000_u32, 44_100, 48_000]
        .into_iter()
        .min_by_key(|candidate| candidate.abs_diff(rate))
        .unwrap_or(44_100)
}

fn resample(input: &[f32], from_rate: u32, to_rate: u32, output_frames: usize) -> Vec<f32> {
    if from_rate == to_rate {
        return input.iter().copied().take(output_frames).collect();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    (0..output_frames)
        .map(|index| {
            let source = index as f64 * ratio;
            let lower = source.floor() as usize;
            let upper = (lower + 1).min(input.len().saturating_sub(1));
            let fraction = (source - lower as f64) as f32;
            let a = input.get(lower).copied().unwrap_or(0.0);
            let b = input.get(upper).copied().unwrap_or(a);
            a + (b - a) * fraction
        })
        .collect()
}

fn make_mp3(audio: &AudioData, fade_start: f64) -> Result<Vec<u8>, String> {
    if fade_start < 0.0 || fade_start + FADE_SECONDS > audio.duration + 0.001 {
        return Err("Choose a fade point with at least 12 seconds remaining.".into());
    }

    let output_rate = nearest_supported_rate(audio.sample_rate);
    let end_seconds = fade_start + FADE_SECONDS;
    let output_frames = (end_seconds * output_rate as f64).round() as usize;
    let fade_start_frame = (fade_start * output_rate as f64).round() as usize;
    let fade_frames = output_frames.saturating_sub(fade_start_frame).max(1);
    let channel_count = audio.channels.len().clamp(1, 2);

    let channels: Vec<Vec<f32>> = audio.channels.iter().take(channel_count)
        .map(|channel| resample(channel, audio.sample_rate, output_rate, output_frames))
        .collect();

    let mut interleaved = Vec::with_capacity(output_frames * channel_count);
    for frame in 0..output_frames {
        let gain = if frame < fade_start_frame {
            1.0
        } else {
            1.0 - ((frame - fade_start_frame) as f32 / fade_frames as f32)
        }.clamp(0.0, 1.0);

        for channel in &channels {
            let sample = channel.get(frame).copied().unwrap_or(0.0) * gain;
            interleaved.push((sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16);
        }
    }

    let mode = if channel_count == 1 { StereoMode::Mono } else { StereoMode::Stereo };
    let config = Mp3EncoderConfig::new()
        .sample_rate(output_rate)
        .bitrate(OUTPUT_BITRATE)
        .channels(channel_count as u8)
        .stereo_mode(mode);

    encode_pcm_to_mp3(config, &interleaved)
        .map_err(|error| format!("MP3 encoding failed: {error}"))
}

fn bytes_to_url(bytes: &[u8], mime: &str) -> Result<String, JsValue> {
    let view = Uint8Array::from(bytes);
    let parts = Array::new();
    parts.push(&view.buffer());
    let options = BlobPropertyBag::new();
    options.set_type(mime);
    let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &options)?;
    Url::create_object_url_with_blob(&blob)
}

async fn decode_file(file: web_sys::File) -> Result<AudioData, String> {
    let name = file.name();
    let source_url = Url::create_object_url_with_blob(&file)
        .map_err(|_| "Could not create an audio preview.".to_string())?;

    let buffer_value = JsFuture::from(file.array_buffer()).await
        .map_err(|_| "Could not read the selected file.".to_string())?;
    let array_buffer: ArrayBuffer = buffer_value.dyn_into()
        .map_err(|_| "The uploaded data was not a valid buffer.".to_string())?;

    let context = AudioContext::new()
        .map_err(|_| "Web Audio is unavailable in this browser.".to_string())?;
    let decoded_value = JsFuture::from(
        context.decode_audio_data(&array_buffer)
            .map_err(|_| "The browser rejected this audio format.".to_string())?
    ).await.map_err(|_| "Unable to decode this MP3 or WAV file.".to_string())?;
    let decoded: AudioBuffer = decoded_value.dyn_into()
        .map_err(|_| "The decoded audio buffer was invalid.".to_string())?;

    let sample_rate = decoded.sample_rate() as u32;
    let duration = decoded.duration();
    let channel_count = decoded.number_of_channels().min(2);
    let mut channels = Vec::with_capacity(channel_count as usize);

    for index in 0..channel_count {
        let data: Vec<f32> = decoded.get_channel_data(index)
            .map_err(|_| "Could not read decoded audio samples.".to_string())?;
        channels.push(data);
    }

    let _ = context.close();

    if channels.is_empty() || duration <= FADE_SECONDS {
        return Err("The audio must contain at least one channel and be longer than 12 seconds.".into());
    }

    Ok(AudioData { name, source_url, sample_rate, channels, duration })
}

fn output_name(source: &str, end_seconds: f64) -> String {
    let base = source.rsplit_once('.').map(|(stem, _)| stem).unwrap_or(source);
    let safe: String = base.chars()
        .map(|character| if character.is_alphanumeric() || matches!(character, '-' | '_') { character } else { '-' })
        .collect();
    format!("{}-faded-{}s.mp3", safe.trim_matches('-'), end_seconds.round() as u64)
}

#[function_component(App)]
fn app() -> Html {
    let audio = use_state(|| None::<Rc<AudioData>>);
    let fade_start = use_state(|| 0.0_f64);
    let status = use_state(|| "Select an MP3 or WAV file to begin.".to_string());
    let is_error = use_state(|| false);
    let processing = use_state(|| false);
    let result = use_state(|| None::<ResultFile>);
    let file_input = use_node_ref();
    let preview_audio = use_node_ref();

    let on_file = {
        let audio = audio.clone();
        let fade_start = fade_start.clone();
        let status = status.clone();
        let is_error = is_error.clone();
        let result = result.clone();

        Callback::from(move |event: Event| {
            let input: HtmlInputElement = event.target_unchecked_into();
            let Some(file) = input.files().and_then(|files| files.get(0)) else { return; };
            let audio = audio.clone();
            let fade_start = fade_start.clone();
            let status = status.clone();
            let is_error = is_error.clone();
            let result = result.clone();

            status.set("Decoding audio locally…".into());
            is_error.set(false);
            result.set(None);

            spawn_local(async move {
                match decode_file(file).await {
                    Ok(decoded) => {
                        let suggested = (decoded.duration - FADE_SECONDS).min(45.0).max(0.0);
                        fade_start.set(suggested);
                        status.set(format!("Ready: {:.1} seconds decoded at {} Hz.", decoded.duration, decoded.sample_rate));
                        audio.set(Some(Rc::new(decoded)));
                    }
                    Err(message) => {
                        is_error.set(true);
                        status.set(message);
                        audio.set(None);
                    }
                }
            });
        })
    };

    let open_picker = {
        let file_input = file_input.clone();
        Callback::from(move |_| {
            if let Some(input) = file_input.cast::<HtmlInputElement>() {
                input.click();
            }
        })
    };

    let on_slider = {
        let fade_start = fade_start.clone();
        let result = result.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            if let Ok(value) = input.value().parse::<f64>() {
                fade_start.set(value);
                result.set(None);
            }
        })
    };

    let on_minutes = {
        let fade_start = fade_start.clone();
        let result = result.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            if let Ok(minutes) = input.value().parse::<f64>() {
                fade_start.set((minutes.max(0.0) * 60.0) + (*fade_start % 60.0));
                result.set(None);
            }
        })
    };

    let on_seconds = {
        let fade_start = fade_start.clone();
        let result = result.clone();
        Callback::from(move |event: InputEvent| {
            let input: HtmlInputElement = event.target_unchecked_into();
            if let Ok(seconds) = input.value().parse::<f64>() {
                fade_start.set((*fade_start / 60.0).floor() * 60.0 + seconds.clamp(0.0, 59.9));
                result.set(None);
            }
        })
    };

    let use_position = {
        let fade_start = fade_start.clone();
        let preview_audio = preview_audio.clone();
        let result = result.clone();
        Callback::from(move |_| {
            if let Some(player) = preview_audio.cast::<HtmlAudioElement>() {
                fade_start.set(player.current_time());
                result.set(None);
            }
        })
    };

    let process = {
        let audio = audio.clone();
        let fade_start = fade_start.clone();
        let status = status.clone();
        let is_error = is_error.clone();
        let processing = processing.clone();
        let result = result.clone();

        Callback::from(move |_| {
            let Some(source) = (*audio).clone() else { return; };
            let start = *fade_start;
            let status = status.clone();
            let is_error = is_error.clone();
            let processing = processing.clone();
            let result = result.clone();

            if start + FADE_SECONDS > source.duration + 0.001 {
                is_error.set(true);
                status.set("Move the fade earlier so the complete 12-second fade fits.".into());
                return;
            }

            processing.set(true);
            is_error.set(false);
            status.set("Forging the shortened MP3 on this device…".into());

            spawn_local(async move {
                TimeoutFuture::new(30).await;
                match make_mp3(&source, start) {
                    Ok(bytes) => match bytes_to_url(&bytes, "audio/mpeg") {
                        Ok(url) => {
                            let end = start + FADE_SECONDS;
                            result.set(Some(ResultFile {
                                url,
                                name: output_name(&source.name, end),
                                bytes: bytes.len(),
                                duration: end,
                            }));
                            status.set("Fade forged successfully. Preview or download the result.".into());
                        }
                        Err(_) => {
                            is_error.set(true);
                            status.set("The browser could not create the finished download.".into());
                        }
                    },
                    Err(message) => {
                        is_error.set(true);
                        status.set(message);
                    }
                }
                processing.set(false);
            });
        })
    };

    let download = {
        let result = result.clone();
        Callback::from(move |_| {
            let Some(file) = (*result).clone() else { return; };
            let Some(window) = web_sys::window() else { return; };
            let Some(document) = window.document() else { return; };
            let Ok(element) = document.create_element("a") else { return; };
            let Ok(anchor) = element.dyn_into::<HtmlAnchorElement>() else { return; };
            anchor.set_href(&file.url);
            anchor.set_download(&file.name);
            anchor.click();
        })
    };

    let reset = {
        let audio = audio.clone();
        let result = result.clone();
        let status = status.clone();
        let is_error = is_error.clone();
        Callback::from(move |_| {
            audio.set(None);
            result.set(None);
            is_error.set(false);
            status.set("Select an MP3 or WAV file to begin.".into());
        })
    };

    let max_start = audio.as_ref().map(|item| (item.duration - FADE_SECONDS).max(0.0)).unwrap_or(0.0);
    let valid_start = *fade_start >= 0.0 && *fade_start <= max_start + 0.001;

    html! {
        <main class="app">
            <header class="hero">
                <div>
                    <p class="eyebrow">{"MIKEGYVER STUDIO • PRIVATE AUDIO TOOL"}</p>
                    <h1>{"Fade Forge"}</h1>
                    <p class="subtitle">{"Choose the ending. Forge a precise 12-second fade. Download a real MP3."}</p>
                </div>
                <span class="badge">{"RUST • YEW • WASM"}</span>
            </header>

            <section class="panel">
                <input ref={file_input} class="file-input" type="file" accept=".mp3,.wav,audio/mpeg,audio/wav" onchange={on_file} />
                <div class="dropzone" role="button" tabindex="0" onclick={open_picker}>
                    <div><span class="drop-icon">{"🎚️"}</span><strong>{"Choose an MP3 or WAV file"}</strong><span>{"Nothing is uploaded to a server"}</span></div>
                </div>
                <p class={classes!("status", (*is_error).then_some("error"))}>{&*status}</p>
            </section>

            {if let Some(source) = audio.as_ref() {
                html! {
                    <>
                        <section class="panel">
                            <div class="file-head">
                                <div><h2>{&source.name}</h2><p>{format!("{} channel{} • {} Hz", source.channels.len(), if source.channels.len() == 1 { "" } else { "s" }, source.sample_rate)}</p></div>
                                <span class="duration">{clock(source.duration)}</span>
                            </div>
                            <audio ref={preview_audio} controls=true preload="metadata" src={source.source_url.clone()} />
                        </section>

                        <section class="panel">
                            <div class="timeline-header"><h2>{"Fade starting point"}</h2><span>{clock(*fade_start)}</span></div>
                            <input class="range" type="range" min="0" max={max_start.to_string()} step="0.1" value={fade_start.to_string()} oninput={on_slider} />
                            <div class="ruler"><span>{"0:00"}</span><span>{format!("Latest {}", clock(max_start))}</span></div>

                            <div class="time-controls">
                                <label>{"Minutes"}<input type="number" min="0" value={((*fade_start / 60.0).floor() as u64).to_string()} oninput={on_minutes} /></label>
                                <label>{"Seconds"}<input type="number" min="0" max="59.9" step="0.1" value={format!("{:.1}", *fade_start % 60.0)} oninput={on_seconds} /></label>
                                <button class="position-button" onclick={use_position}>{"Use playback position"}</button>
                            </div>

                            <div class="fade-map">
                                <div class="fade-point"><small>{"FADE BEGINS"}</small><strong>{clock(*fade_start)}</strong></div>
                                <div class="arrow">{"→"}</div>
                                <div class="fade-point"><small>{"SILENCE + CUT"}</small><strong>{clock(*fade_start + FADE_SECONDS)}</strong></div>
                            </div>

                            <button class="process-button" disabled={!valid_start || *processing} onclick={process}>
                                {if *processing { "Forging MP3…" } else { "Apply 12-Second Fade + Shorten" }}
                            </button>
                            <p class="privacy">{"Audio processing happens locally in your browser. Large files may take a moment."}</p>
                        </section>
                    </>
                }
            } else { Html::default() }}

            {if let Some(file) = result.as_ref() {
                html! {
                    <section class="panel result-panel">
                        <h2>{"✓ Finished MP3"}</h2>
                        <div class="result-grid">
                            <div class="metric"><span>{"Duration"}</span><strong>{clock(file.duration)}</strong></div>
                            <div class="metric"><span>{"Fade"}</span><strong>{"12 seconds"}</strong></div>
                            <div class="metric"><span>{"File size"}</span><strong>{format!("{:.1} MB", file.bytes as f64 / 1_048_576.0)}</strong></div>
                        </div>
                        <audio controls=true preload="metadata" src={file.url.clone()} />
                        <div class="actions">
                            <button class="download-button" onclick={download}>{"Download Shortened MP3"}</button>
                            <button class="reset-button" onclick={reset}>{"Start Over"}</button>
                        </div>
                    </section>
                }
            } else { Html::default() }}
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
