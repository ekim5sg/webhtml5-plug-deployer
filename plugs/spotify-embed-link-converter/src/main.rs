use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::prelude::*;

#[derive(Clone, PartialEq, Debug)]
struct ParsedSpotifyLink {
    kind: String,
    id: String,
    embed_url: String,
    original_url: String,
}

fn extract_spotify_embed(input: &str) -> Result<ParsedSpotifyLink, String> {
    let raw = input.trim();

    if raw.is_empty() {
        return Err("Paste a Spotify share link first.".to_string());
    }

    let normalized = raw
        .replace("spotify.link/", "open.spotify.com/")
        .replace("play.spotify.com/", "open.spotify.com/");

    let marker = "open.spotify.com/";
    let start = normalized
        .find(marker)
        .ok_or_else(|| "That does not look like a Spotify open URL.".to_string())?;

    let after_domain = &normalized[start + marker.len()..];

    let path_only = after_domain
        .split('?')
        .next()
        .unwrap_or("")
        .trim_matches('/');

    let parts: Vec<&str> = path_only.split('/').filter(|s| !s.is_empty()).collect();

    if parts.len() < 2 {
        return Err("Could not determine the Spotify content type and ID.".to_string());
    }

    let supported = ["track", "album", "playlist", "artist", "show", "episode"];
    let kind = parts[0].to_lowercase();

    if !supported.contains(&kind.as_str()) {
        return Err(format!(
            "Unsupported Spotify type: '{}'. Supported: {}",
            kind,
            supported.join(", ")
        ));
    }

    let id = parts[1].trim().to_string();

    if id.is_empty() {
        return Err("The Spotify ID appears to be missing.".to_string());
    }

    let embed_url = format!("https://open.spotify.com/embed/{}/{}", kind, id);

    Ok(ParsedSpotifyLink {
        kind,
        id,
        embed_url,
        original_url: raw.to_string(),
    })
}

#[function_component(App)]
fn app() -> Html {
    let input_value = use_state(String::new);
    let result = use_state(|| None as Option<ParsedSpotifyLink>);
    let error = use_state(|| None as Option<String>);
    let copied = use_state(|| false);

    let oninput = {
        let input_value = input_value.clone();
        let copied = copied.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlTextAreaElement = e.target_unchecked_into();
            input_value.set(input.value());
            copied.set(false);
        })
    };

    let on_convert = {
        let input_value = input_value.clone();
        let result = result.clone();
        let error = error.clone();
        let copied = copied.clone();

        Callback::from(move |_| {
            copied.set(false);
            match extract_spotify_embed(&input_value) {
                Ok(parsed) => {
                    result.set(Some(parsed));
                    error.set(None);
                }
                Err(msg) => {
                    result.set(None);
                    error.set(Some(msg));
                }
            }
        })
    };

    let on_clear = {
        let input_value = input_value.clone();
        let result = result.clone();
        let error = error.clone();
        let copied = copied.clone();

        Callback::from(move |_| {
            input_value.set(String::new());
            result.set(None);
            error.set(None);
            copied.set(false);
        })
    };

    let on_copy = {
        let result = result.clone();
        let copied = copied.clone();

        Callback::from(move |_| {
            if let Some(parsed) = (*result).clone() {
                if let Some(window) = web_sys::window() {
                    let clipboard = window.navigator().clipboard();
                    let _ = clipboard.write_text(&parsed.embed_url);
                    copied.set(true);
                }
            }
        })
    };

    let sample_links = vec![
        "https://open.spotify.com/track/4LJ10MnuPM8iTeTaDmLspQ?si=3da5443f144e4101",
        "https://open.spotify.com/album/0411KRvEqh6e0N5M1uWdoH?si=EbdxzjzPRj-K_e7we33Pjw",
        "https://open.spotify.com/playlist/37i9dQZF1DXcBWIGoYBM5M?si=abc123",
    ];

    html! {
        <div style="
            min-height: 100vh;
            background: linear-gradient(180deg, #0b1020 0%, #121a33 100%);
            color: #f5f7ff;
            font-family: Arial, Helvetica, sans-serif;
            padding: 32px 16px;
        ">
            <div style="
                max-width: 900px;
                margin: 0 auto;
                background: rgba(255,255,255,0.06);
                border: 1px solid rgba(255,255,255,0.10);
                border-radius: 24px;
                padding: 24px;
                box-shadow: 0 18px 50px rgba(0,0,0,0.35);
            ">
                <h1 style="margin: 0 0 8px 0; font-size: 2rem;">{"Spotify Embed URL Converter"}</h1>
                <p style="margin: 0 0 20px 0; color: #c9d4ff; line-height: 1.6;">
                    {"Paste a Spotify share link and generate the official embed URL instantly."}
                </p>

                <div style="
                    display: grid;
                    gap: 12px;
                ">
                    <textarea
                        rows="5"
                        value={(*input_value).clone()}
                        oninput={oninput}
                        placeholder="Paste a Spotify URL here..."
                        style="
                            width: 100%;
                            border-radius: 16px;
                            border: 1px solid rgba(255,255,255,0.14);
                            background: #0f1730;
                            color: #ffffff;
                            padding: 16px;
                            resize: vertical;
                            box-sizing: border-box;
                            font-size: 1rem;
                            line-height: 1.5;
                        "
                    />

                    <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                        <button
                            onclick={on_convert}
                            style="
                                border: none;
                                border-radius: 14px;
                                padding: 12px 18px;
                                font-weight: 700;
                                cursor: pointer;
                                background: #1db954;
                                color: #08110b;
                            "
                        >
                            {"Convert"}
                        </button>

                        <button
                            onclick={on_clear}
                            style="
                                border: 1px solid rgba(255,255,255,0.14);
                                border-radius: 14px;
                                padding: 12px 18px;
                                font-weight: 700;
                                cursor: pointer;
                                background: #17213f;
                                color: #ffffff;
                            "
                        >
                            {"Clear"}
                        </button>
                    </div>
                </div>

                <div style="margin-top: 20px;">
                    <h2 style="font-size: 1.05rem; margin-bottom: 10px;">{"Examples"}</h2>
                    <div style="display: grid; gap: 8px;">
                        {
                            sample_links.iter().map(|link| {
                                let link_string = (*link).to_string();
                                let input_value = input_value.clone();
                                let result = result.clone();
                                let error = error.clone();
                                let copied = copied.clone();

                                let onclick = Callback::from(move |_| {
                                    input_value.set(link_string.clone());
                                    copied.set(false);
                                    match extract_spotify_embed(&link_string) {
                                        Ok(parsed) => {
                                            result.set(Some(parsed));
                                            error.set(None);
                                        }
                                        Err(msg) => {
                                            result.set(None);
                                            error.set(Some(msg));
                                        }
                                    }
                                });

                                html! {
                                    <button
                                        {onclick}
                                        style="
                                            text-align: left;
                                            border: 1px solid rgba(255,255,255,0.10);
                                            border-radius: 14px;
                                            background: #10182f;
                                            color: #dce5ff;
                                            padding: 12px 14px;
                                            cursor: pointer;
                                            word-break: break-all;
                                        "
                                    >
                                        {link.to_string()}
                                    </button>
                                }
                            }).collect::<Html>()
                        }
                    </div>
                </div>

                {
                    if let Some(msg) = &*error {
                        html! {
                            <div style="
                                margin-top: 22px;
                                border-radius: 16px;
                                padding: 14px 16px;
                                background: rgba(255, 90, 90, 0.12);
                                border: 1px solid rgba(255, 90, 90, 0.28);
                                color: #ffd4d4;
                            ">
                                {msg.clone()}
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }

                {
                    if let Some(parsed) = &*result {
                        let iframe_html = format!(
                            "<iframe src=\"{}\" width=\"100%\" height=\"352\" frameborder=\"0\" allowfullscreen=\"\" allow=\"autoplay; clipboard-write; encrypted-media; fullscreen; picture-in-picture\" loading=\"lazy\"></iframe>",
                            parsed.embed_url
                        );

                        html! {
                            <div style="
                                margin-top: 24px;
                                padding: 18px;
                                border-radius: 18px;
                                background: #0e162d;
                                border: 1px solid rgba(255,255,255,0.10);
                            ">
                                <h2 style="margin-top: 0;">{"Result"}</h2>

                                <div style="display: grid; gap: 12px;">
                                    <div>
                                        <div style="font-size: 0.9rem; color: #9fb0e8; margin-bottom: 4px;">{"Type"}</div>
                                        <div style="font-weight: 700; text-transform: capitalize;">{parsed.kind.clone()}</div>
                                    </div>

                                    <div>
                                        <div style="font-size: 0.9rem; color: #9fb0e8; margin-bottom: 4px;">{"Spotify ID"}</div>
                                        <div style="word-break: break-all;">{parsed.id.clone()}</div>
                                    </div>

                                    <div>
                                        <div style="font-size: 0.9rem; color: #9fb0e8; margin-bottom: 4px;">{"Embed URL"}</div>
                                        <input
                                            readonly=true
                                            value={parsed.embed_url.clone()}
                                            style="
                                                width: 100%;
                                                border-radius: 12px;
                                                border: 1px solid rgba(255,255,255,0.12);
                                                background: #121d3c;
                                                color: #ffffff;
                                                padding: 12px;
                                                box-sizing: border-box;
                                            "
                                        />
                                    </div>

                                    <div style="display: flex; gap: 10px; flex-wrap: wrap;">
                                        <button
                                            onclick={on_copy}
                                            style="
                                                border: none;
                                                border-radius: 14px;
                                                padding: 12px 18px;
                                                font-weight: 700;
                                                cursor: pointer;
                                                background: #7aa2ff;
                                                color: #081224;
                                            "
                                        >
                                            {"Copy Embed URL"}
                                        </button>

                                        <a
                                            href={parsed.embed_url.clone()}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            style="
                                                text-decoration: none;
                                                display: inline-block;
                                                border-radius: 14px;
                                                padding: 12px 18px;
                                                font-weight: 700;
                                                background: #1a2547;
                                                color: #ffffff;
                                                border: 1px solid rgba(255,255,255,0.10);
                                            "
                                        >
                                            {"Open Embed"}
                                        </a>
                                    </div>

                                    {
                                        if *copied {
                                            html! {
                                                <div style="color: #8df0b2; font-weight: 700;">
                                                    {"Copied to clipboard."}
                                                </div>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }

                                    <div>
                                        <div style="font-size: 0.9rem; color: #9fb0e8; margin-bottom: 4px;">{"Iframe HTML"}</div>
                                        <textarea
                                            readonly=true
                                            rows="4"
                                            value={iframe_html}
                                            style="
                                                width: 100%;
                                                border-radius: 12px;
                                                border: 1px solid rgba(255,255,255,0.12);
                                                background: #121d3c;
                                                color: #ffffff;
                                                padding: 12px;
                                                box-sizing: border-box;
                                                resize: vertical;
                                            "
                                        />
                                    </div>

                                    <div>
                                        <div style="font-size: 0.9rem; color: #9fb0e8; margin-bottom: 10px;">{"Live Preview"}</div>
                                        <iframe
                                            src={parsed.embed_url.clone()}
                                            width="100%"
                                            height={if parsed.kind == "track" || parsed.kind == "episode" { "152" } else { "352" }}
                                            frameborder="0"
                                            allow="autoplay; clipboard-write; encrypted-media; fullscreen; picture-in-picture"
                                            loading="lazy"
                                            style="border-radius: 16px;"
                                        />
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}