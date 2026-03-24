use js_sys::{Array, Date};
use serde::Serialize;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{
    Blob, BlobPropertyBag, HtmlAnchorElement, HtmlInputElement, HtmlSelectElement,
    HtmlTextAreaElement, Url,
};
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct ClipResult {
    title_hook: String,
    short_description: String,
    selected_preview: String,
    promo_caption: String,
    words: usize,
    estimated_seconds: usize,
    score: usize,
}

#[derive(Serialize)]
struct ExportPayload {
    episode_title: String,
    target_seconds: usize,
    estimated_words_per_minute: usize,
    generated_at: String,
    hook_title: String,
    short_description: String,
    preview_script: String,
    promo_caption: String,
    preview_word_count: usize,
    estimated_preview_seconds: usize,
    preview_score: usize,
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    let episode_title = use_state(|| String::new());
    let transcript = use_state(|| String::new());
    let target_seconds = use_state(|| "45".to_string());
    let words_per_minute = use_state(|| "150".to_string());
    let preview_style = use_state(|| "Hooky".to_string());
    let result = use_state(|| None as Option<ClipResult>);
    let status = use_state(|| {
        "Paste your transcript, choose a target length, and generate a preview clip.".to_string()
    });

    let on_episode_title = {
        let episode_title = episode_title.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            episode_title.set(input.value());
        })
    };

    let on_transcript = {
        let transcript = transcript.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            transcript.set(input.value());
        })
    };

    let on_target_seconds = {
        let target_seconds = target_seconds.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            target_seconds.set(input.value());
        })
    };

    let on_wpm = {
        let words_per_minute = words_per_minute.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            words_per_minute.set(input.value());
        })
    };

    let on_style = {
        let preview_style = preview_style.clone();
        Callback::from(move |e: Event| {
            let input: HtmlSelectElement = e.target_unchecked_into();
            preview_style.set(input.value());
        })
    };

    let on_generate = {
        let episode_title = episode_title.clone();
        let transcript = transcript.clone();
        let target_seconds = target_seconds.clone();
        let words_per_minute = words_per_minute.clone();
        let preview_style = preview_style.clone();
        let result = result.clone();
        let status = status.clone();

        Callback::from(move |_| {
            let title = episode_title.trim().to_string();
            let source = transcript.trim().to_string();

            if source.is_empty() {
                status.set("Paste your episode script or transcript first.".to_string());
                result.set(None);
                return;
            }

            let secs = target_seconds
                .parse::<usize>()
                .ok()
                .filter(|v| *v > 0)
                .unwrap_or(45);

            let wpm = words_per_minute
                .parse::<usize>()
                .ok()
                .filter(|v| *v > 0)
                .unwrap_or(150);

            let generated = generate_preview(&title, &source, secs, wpm, preview_style.as_str());
            status.set("Preview clip generated.".to_string());
            result.set(Some(generated));
        })
    };

    let on_clear = {
        let episode_title = episode_title.clone();
        let transcript = transcript.clone();
        let target_seconds = target_seconds.clone();
        let words_per_minute = words_per_minute.clone();
        let preview_style = preview_style.clone();
        let result = result.clone();
        let status = status.clone();

        Callback::from(move |_| {
            episode_title.set(String::new());
            transcript.set(String::new());
            target_seconds.set("45".to_string());
            words_per_minute.set("150".to_string());
            preview_style.set("Hooky".to_string());
            result.set(None);
            status.set("Cleared. Ready for your next episode.".to_string());
        })
    };

    let copy_preview = {
        let result = result.clone();
        let status = status.clone();

        Callback::from(move |_| {
            if let Some(r) = &*result {
                copy_to_clipboard(&r.selected_preview);
                status.set("Preview script copied.".to_string());
            }
        })
    };

    let copy_package = {
        let result = result.clone();
        let status = status.clone();

        Callback::from(move |_| {
            if let Some(r) = &*result {
                let package = format!(
                    "Hook Title:\n{}\n\nShort Description:\n{}\n\nPromo Caption:\n{}\n\nPreview Script:\n{}",
                    r.title_hook, r.short_description, r.promo_caption, r.selected_preview
                );
                copy_to_clipboard(&package);
                status.set("Full package copied.".to_string());
            }
        })
    };

    let export_json = {
        let result = result.clone();
        let episode_title = episode_title.clone();
        let target_seconds = target_seconds.clone();
        let words_per_minute = words_per_minute.clone();
        let status = status.clone();

        Callback::from(move |_| {
            if let Some(r) = &*result {
                let payload = ExportPayload {
                    episode_title: (*episode_title).clone(),
                    target_seconds: target_seconds.parse::<usize>().unwrap_or(45),
                    estimated_words_per_minute: words_per_minute.parse::<usize>().unwrap_or(150),
                    generated_at: Date::new_0().to_iso_string().into(),
                    hook_title: r.title_hook.clone(),
                    short_description: r.short_description.clone(),
                    preview_script: r.selected_preview.clone(),
                    promo_caption: r.promo_caption.clone(),
                    preview_word_count: r.words,
                    estimated_preview_seconds: r.estimated_seconds,
                    preview_score: r.score,
                };

                match serde_json::to_string_pretty(&payload) {
                    Ok(json) => {
                        download_text_file(
                            "spotify_preview_clip_package.json",
                            &json,
                            "application/json",
                        );
                        status.set("JSON export downloaded.".to_string());
                    }
                    Err(_) => {
                        status.set("Could not create JSON export.".to_string());
                    }
                }
            }
        })
    };

    let transcript_word_count = count_words_str(transcript.as_str());

    let est_minutes = match words_per_minute.parse::<usize>() {
        Ok(wpm) if wpm > 0 => transcript_word_count as f64 / wpm as f64,
        _ => 0.0,
    };

    html! {
        <div class="app">
            <div class="topbar">
                <div class="brand">{ "MikeGyver Studio" }</div>
                <div class="badge">{ "Spotify Preview Clip Generator" }</div>
            </div>

            <section class="hero">
                <div class="card hero-card">
                    <h1>{ "Turn long episodes into tight preview clips." }</h1>
                    <p>
                        { "Paste your transcript, choose a target length, and let this app find a compelling preview segment with a hook title, short description, and promo caption." }
                    </p>
                </div>

                <div class="card quick-stats">
                    <div class="stat">
                        <div class="stat-label">{ "Transcript words" }</div>
                        <div class="stat-value">{ transcript_word_count }</div>
                    </div>
                    <div class="stat">
                        <div class="stat-label">{ "Estimated episode length" }</div>
                        <div class="stat-value">{ format!("{:.1} min", est_minutes) }</div>
                    </div>
                    <div class="stat">
                        <div class="stat-label">{ "Status" }</div>
                        <div class="stat-value" style="font-size:0.98rem;">{ (*status).clone() }</div>
                    </div>
                </div>
            </section>

            <section class="grid">
                <div class="card section">
                    <h2>{ "Episode Input" }</h2>

                    <div class="field">
                        <label for="episode-title">{ "Episode title" }</label>
                        <input
                            id="episode-title"
                            type="text"
                            value={(*episode_title).clone()}
                            oninput={on_episode_title}
                            placeholder="Example: The Library That Came Alive"
                        />
                    </div>

                    <div class="row">
                        <div class="field">
                            <label for="target-seconds">{ "Target preview length (seconds)" }</label>
                            <input
                                id="target-seconds"
                                type="number"
                                min="15"
                                max="90"
                                value={(*target_seconds).clone()}
                                oninput={on_target_seconds}
                            />
                        </div>

                        <div class="field">
                            <label for="wpm">{ "Reading pace (words/minute)" }</label>
                            <input
                                id="wpm"
                                type="number"
                                min="100"
                                max="220"
                                value={(*words_per_minute).clone()}
                                oninput={on_wpm}
                            />
                        </div>
                    </div>

                    <div class="field">
                        <label for="preview-style">{ "Preview style" }</label>
                        <select id="preview-style" onchange={on_style} value={(*preview_style).clone()}>
                            <option value="Hooky">{ "Hooky" }</option>
                            <option value="Emotional">{ "Emotional" }</option>
                            <option value="Cinematic">{ "Cinematic" }</option>
                            <option value="Informative">{ "Informative" }</option>
                        </select>
                    </div>

                    <div class="field">
                        <label for="transcript">{ "Episode transcript or narration script" }</label>
                        <textarea
                            id="transcript"
                            value={(*transcript).clone()}
                            oninput={on_transcript}
                            placeholder="Paste your full episode script or transcript here..."
                        />
                    </div>

                    <div class="actions">
                        <button class="btn-primary" onclick={on_generate}>{ "Generate Preview Clip" }</button>
                        <button class="btn-secondary" onclick={copy_package}>{ "Copy Full Package" }</button>
                        <button class="btn-secondary" onclick={export_json}>{ "Export JSON" }</button>
                        <button class="btn-ghost" onclick={on_clear}>{ "Clear" }</button>
                    </div>

                    <p class="footer-note">
                        { "Tip: for spoken previews, 30–60 seconds often works best. This tool estimates duration from your chosen words-per-minute pace." }
                    </p>
                </div>

                <div class="card section">
                    <h2>{ "Generated Preview Package" }</h2>

                    {
                        if let Some(r) = &*result {
                            html! {
                                <>
                                    <div class="score">
                                        <span>{ "Preview score" }</span>
                                        <strong>{ format!("{}/100", r.score) }</strong>
                                        <span class="small">{ format!("{} words • ~{} sec", r.words, r.estimated_seconds) }</span>
                                    </div>

                                    <div class="output-block">
                                        <div class="output-title">{ "Hook Title" }</div>
                                        <div class="output-text">{ r.title_hook.clone() }</div>
                                    </div>

                                    <div class="output-block">
                                        <div class="output-title">{ "Short Description" }</div>
                                        <div class="output-text">{ r.short_description.clone() }</div>
                                    </div>

                                    <div class="output-block">
                                        <div class="output-title">{ "Promo Caption" }</div>
                                        <div class="output-text">{ r.promo_caption.clone() }</div>
                                    </div>

                                    <div class="output-block">
                                        <div class="output-title">{ "Suggested Preview Script" }</div>
                                        <div class="output-text">{ r.selected_preview.clone() }</div>
                                    </div>

                                    <div class="actions">
                                        <button class="btn-primary" onclick={copy_preview}>{ "Copy Preview Script" }</button>
                                    </div>
                                </>
                            }
                        } else {
                            html! {
                                <div class="output-block">
                                    <div class="output-title">{ "Nothing generated yet" }</div>
                                    <div class="output-text">
                                        { "Your preview package will appear here after you generate it." }
                                    </div>
                                </div>
                            }
                        }
                    }

                    <p class="footer-note">
                        { "The selection logic favors lines with strong hooks, contrast, urgency, emotion, and story momentum while trying to stay near your target duration." }
                    </p>
                </div>
            </section>

            <div class="mike">{ "MikeGyver Studio" }</div>
        </div>
    }
}

fn generate_preview(
    title: &str,
    source: &str,
    target_seconds: usize,
    wpm: usize,
    style: &str,
) -> ClipResult {
    let safe_wpm = wpm.max(1);
    let words_target = ((target_seconds as f64 / 60.0) * safe_wpm as f64).round() as usize;
    let paragraphs = split_paragraphs(source);

    let mut best_text = String::new();
    let mut best_score = 0usize;
    let mut best_words = 0usize;

    for i in 0..paragraphs.len() {
        let mut combined = String::new();

        for j in i..paragraphs.len() {
            if !combined.is_empty() {
                combined.push_str("\n\n");
            }
            combined.push_str(&paragraphs[j]);

            let wc = count_words_str(&combined);
            if wc > words_target + 35 {
                break;
            }

            let score = score_candidate(&combined, words_target, style);
            if score > best_score && wc >= (words_target / 2).max(20) {
                best_score = score;
                best_text = combined.clone();
                best_words = wc;
            }
        }
    }

    if best_text.trim().is_empty() {
        let fallback = truncate_to_words(source, words_target.max(40));
        best_words = count_words_str(&fallback);
        best_score = score_candidate(&fallback, words_target, style);
        best_text = fallback;
    }

    let estimated_seconds = ((best_words as f64 / safe_wpm as f64) * 60.0).round() as usize;
    let title_hook = make_hook_title(title, &best_text, style);
    let short_description = make_short_description(title, &best_text, estimated_seconds, style);
    let promo_caption = make_promo_caption(title, &best_text, style);

    ClipResult {
        title_hook,
        short_description,
        selected_preview: best_text,
        promo_caption,
        words: best_words,
        estimated_seconds,
        score: best_score.min(100),
    }
}

fn split_paragraphs(source: &str) -> Vec<String> {
    let cleaned = source.replace("\r\n", "\n");

    let mut parts: Vec<String> = cleaned
        .split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    if parts.is_empty() {
        parts = cleaned
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
    }

    if parts.is_empty() {
        vec![source.trim().to_string()]
    } else {
        parts
    }
}

fn score_candidate(text: &str, target_words: usize, style: &str) -> usize {
    let lower = text.to_lowercase();
    let word_count = count_words_str(text);

    let hook_terms = [
        "but",
        "and yet",
        "suddenly",
        "then",
        "because",
        "until",
        "one day",
        "every second",
        "what happened",
        "the moment",
        "didn't",
        "never",
        "first",
        "finally",
        "before",
        "after",
        "mystery",
        "secret",
        "danger",
        "question",
    ];

    let emotion_terms = [
        "hope", "fear", "love", "courage", "loss", "wonder", "dream", "heart", "believe",
        "afraid", "joy", "pain", "grace", "light", "dark",
    ];

    let cinematic_terms = [
        "moon", "sky", "light", "shadow", "door", "wind", "stars", "city", "library",
        "house", "machine", "echo", "silence", "voice", "fire",
    ];

    let informative_terms = [
        "during",
        "when",
        "why",
        "how",
        "mission",
        "computer",
        "because",
        "history",
        "lesson",
        "truth",
        "meant",
        "decision",
        "problem",
        "answer",
    ];

    let mut score = 0usize;

    for t in hook_terms {
        if lower.contains(t) {
            score += 8;
        }
    }

    for t in emotion_terms {
        if lower.contains(t) {
            score += match style {
                "Emotional" => 7,
                _ => 4,
            };
        }
    }

    for t in cinematic_terms {
        if lower.contains(t) {
            score += match style {
                "Cinematic" => 7,
                _ => 4,
            };
        }
    }

    for t in informative_terms {
        if lower.contains(t) {
            score += match style {
                "Informative" => 7,
                _ => 4,
            };
        }
    }

    if text.contains('?') {
        score += 10;
    }
    if text.contains('…') || text.contains("...") {
        score += 8;
    }
    if text.contains('!') {
        score += 6;
    }

    let diff = word_count.abs_diff(target_words);
    if diff <= 8 {
        score += 22;
    } else if diff <= 18 {
        score += 16;
    } else if diff <= 28 {
        score += 10;
    } else {
        score += 4;
    }

    if word_count >= 35 {
        score += 6;
    }
    if word_count >= 55 {
        score += 4;
    }
    if word_count > target_words + 30 {
        score = score.saturating_sub(10);
    }

    score
}

fn make_hook_title(title: &str, preview: &str, style: &str) -> String {
    let preview_clean = preview.replace('\n', " ");
    let first = first_sentence(&preview_clean);

    if !title.trim().is_empty() {
        match style {
            "Emotional" => format!("{} — A Moment You’ll Feel", title.trim()),
            "Cinematic" => format!("{} — The Scene That Pulls You In", title.trim()),
            "Informative" => format!("{} — The Part That Explains Everything", title.trim()),
            _ => format!("{} — The Hook", title.trim()),
        }
    } else if first.len() > 60 {
        format!("{}...", &first[..60])
    } else {
        first
    }
}

fn make_short_description(title: &str, preview: &str, seconds: usize, style: &str) -> String {
    let opener = match style {
        "Emotional" => "An emotionally resonant preview from",
        "Cinematic" => "A cinematic preview moment from",
        "Informative" => "A sharp preview segment from",
        _ => "A compelling preview clip from",
    };

    let subject = if title.trim().is_empty() {
        "this episode".to_string()
    } else {
        format!("“{}”", title.trim())
    };

    let summary_seed = summarize_preview(preview, 24);

    format!(
        "{} {} — about {} seconds of story, tension, and momentum. {}",
        opener, subject, seconds, summary_seed
    )
}

fn make_promo_caption(title: &str, preview: &str, style: &str) -> String {
    let lead = match style {
        "Emotional" => "This moment hits hard.",
        "Cinematic" => "This is one of the most cinematic moments in the episode.",
        "Informative" => "This clip captures the key idea fast.",
        _ => "This preview gives you the heartbeat of the episode.",
    };

    let subject = if title.trim().is_empty() {
        "the episode".to_string()
    } else {
        format!("“{}”", title.trim())
    };

    let tease = summarize_preview(preview, 18);

    format!(
        "{} Here’s a short preview from {}. {}",
        lead, subject, tease
    )
}

fn summarize_preview(text: &str, max_words: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();

    if words.is_empty() {
        return "A short, polished preview clip.".to_string();
    }

    let take = words
        .iter()
        .take(max_words)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ");

    if words.len() > max_words {
        format!("{}...", take)
    } else {
        take
    }
}

fn first_sentence(text: &str) -> String {
    for sep in ['.', '!', '?'] {
        if let Some(idx) = text.find(sep) {
            return text[..=idx].trim().to_string();
        }
    }
    text.trim().to_string()
}

fn truncate_to_words(text: &str, max_words: usize) -> String {
    text.split_whitespace()
        .take(max_words)
        .collect::<Vec<_>>()
        .join(" ")
}

fn count_words_str(text: &str) -> usize {
    text.split_whitespace()
        .filter(|w| !w.trim().is_empty())
        .count()
}

fn copy_to_clipboard(text: &str) {
    if let Some(window) = web_sys::window() {
        let clipboard = window.navigator().clipboard();
        let _ = clipboard.write_text(text);
    }
}

fn download_text_file(filename: &str, content: &str, mime_type: &str) {
    let mut bag = BlobPropertyBag::new();
    bag.set_type(mime_type);

    let parts = {
        let a = Array::new();
        a.push(&JsValue::from_str(content));
        a
    };

    let blob = match Blob::new_with_str_sequence_and_options(&parts, &bag) {
        Ok(b) => b,
        Err(_) => return,
    };

    let url = match Url::create_object_url_with_blob(&blob) {
        Ok(u) => u,
        Err(_) => return,
    };

    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };

    let document = match window.document() {
        Some(d) => d,
        None => {
            let _ = Url::revoke_object_url(&url);
            return;
        }
    };

    let element = match document.create_element("a") {
        Ok(e) => e,
        Err(_) => {
            let _ = Url::revoke_object_url(&url);
            return;
        }
    };

    let anchor: HtmlAnchorElement = match element.dyn_into() {
        Ok(a) => a,
        Err(_) => {
            let _ = Url::revoke_object_url(&url);
            return;
        }
    };

    anchor.set_href(&url);
    anchor.set_download(filename);
    let _ = anchor.set_attribute("hidden", "true");

    if let Some(body) = document.body() {
        let _ = body.append_child(&anchor);
        anchor.click();
        let _ = body.remove_child(&anchor);
    }

    let _ = Url::revoke_object_url(&url);
}