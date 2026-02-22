// src/main.rs
use gloo::console::log;
use gloo_file::callbacks::FileReader;
use gloo_file::File;
use gloo_storage::{LocalStorage, Storage};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlInputElement, Url};
use yew::prelude::*;

const STORAGE_KEY: &str = "mg_spotify_inventory_v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Song {
    title: String,
    cover_art_url: String,
    lyrics: String,
    length_mmss: String,
    spotify_url: String,
}

impl Default for Song {
    fn default() -> Self {
        Self {
            title: "".into(),
            cover_art_url: "".into(),
            lyrics: "".into(),
            length_mmss: "3:00".into(),
            spotify_url: "".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
struct Inventory {
    songs: Vec<Song>,
}

fn main() {
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    let inventory = use_state(load_from_storage);
    let selected_index = use_state(|| None::<usize>);
    let draft = use_state(Song::default);
    let log_text = use_state(|| String::from("Ready.\n"));

    // Keep FileReader alive (Yew pattern)
    let reader = use_state(|| None::<FileReader>);

    // Helper: append to log
    let push_log = {
        let log_text = log_text.clone();
        move |line: &str| {
            let mut next = (*log_text).clone();
            next.push_str(line);
            if !line.ends_with('\n') {
                next.push('\n');
            }
            log_text.set(next);
        }
    };

    // Autosave inventory to LocalStorage
    {
        let inventory = inventory.clone();
        let push_log = push_log.clone();
        use_effect_with(*inventory, move |_| {
            if let Err(e) = LocalStorage::set(STORAGE_KEY, &*inventory) {
                push_log(&format!("⚠️ Failed to save to LocalStorage: {e:?}"));
            }
            || ()
        });
    }

    // Select from list
    let on_select = {
        let inventory = inventory.clone();
        let selected_index = selected_index.clone();
        let draft = draft.clone();
        let push_log = push_log.clone();
        Callback::from(move |idx: usize| {
            selected_index.set(Some(idx));
            if let Some(song) = inventory.songs.get(idx) {
                draft.set(song.clone());
                push_log(&format!("Selected: {}", song.title));
            }
        })
    };

    // New
    let on_new = {
        let selected_index = selected_index.clone();
        let draft = draft.clone();
        let push_log = push_log.clone();
        Callback::from(move |_| {
            selected_index.set(None);
            draft.set(Song::default());
            push_log("New draft started.");
        })
    };

    // Delete selected
    let on_delete = {
        let inventory = inventory.clone();
        let selected_index = selected_index.clone();
        let draft = draft.clone();
        let push_log = push_log.clone();
        Callback::from(move |_| {
            if let Some(i) = *selected_index {
                let mut next = (*inventory).clone();
                if i < next.songs.len() {
                    let removed = next.songs.remove(i);
                    inventory.set(next);
                    selected_index.set(None);
                    draft.set(Song::default());
                    push_log(&format!("🗑️ Deleted: {}", removed.title));
                } else {
                    push_log("⚠️ Selected index out of range.");
                }
            } else {
                push_log("Nothing selected to delete.");
            }
        })
    };

    // Save (add or update)
    let on_save = {
        let inventory = inventory.clone();
        let selected_index = selected_index.clone();
        let draft = draft.clone();
        let push_log = push_log.clone();
        Callback::from(move |_| {
            let mut s = (*draft).clone();

            // Minimal validation/cleanup
            s.title = s.title.trim().to_string();
            s.length_mmss = s.length_mmss.trim().to_string();
            s.cover_art_url = s.cover_art_url.trim().to_string();
            s.spotify_url = s.spotify_url.trim().to_string();

            if s.title.is_empty() {
                push_log("⚠️ Title is required.");
                return;
            }

            let mut next = (*inventory).clone();
            match *selected_index {
                Some(i) if i < next.songs.len() => {
                    next.songs[i] = s.clone();
                    inventory.set(next);
                    push_log(&format!("✅ Updated: {}", s.title));
                }
                _ => {
                    next.songs.push(s.clone());
                    let new_len = next.songs.len();
                    inventory.set(next);
                    selected_index.set(Some(new_len - 1));
                    push_log(&format!("✅ Added: {}", s.title));

                    if new_len == 1 {
                        push_log("🧾 JSON is now ready. Click “Export JSON” to create the file.");
                    }
                }
            }
        })
    };

    // Export JSON download
    let on_export = {
        let inventory = inventory.clone();
        let push_log = push_log.clone();
        Callback::from(move |_| {
            if inventory.songs.is_empty() {
                push_log("⚠️ Add at least one entry before exporting JSON.");
                return;
            }

            match serde_json::to_string_pretty(&*inventory) {
                Ok(json) => match download_text_file("spotify_inventory.json", &json) {
                    Ok(()) => push_log("⬇️ Exported spotify_inventory.json"),
                    Err(e) => push_log(&format!("⚠️ Export failed: {e}")),
                },
                Err(e) => push_log(&format!("⚠️ Could not serialize JSON: {e}")),
            }
        })
    };

    // Import JSON file
    let on_import_change = {
        let reader = reader.clone();
        let inventory = inventory.clone();
        let selected_index = selected_index.clone();
        let draft = draft.clone();
        let push_log = push_log.clone();

        Callback::from(move |e: Event| {
            let input: HtmlInputElement = match e
                .target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            {
                Some(i) => i,
                None => {
                    push_log("⚠️ Could not access file input.");
                    return;
                }
            };

            let files = match input.files() {
                Some(f) => f,
                None => {
                    push_log("No file selected.");
                    return;
                }
            };

            if files.length() == 0 {
                push_log("No file selected.");
                return;
            }

            let file = files.get(0).unwrap();
            let file = File::from(file);
            push_log(&format!("📥 Reading file: {}", file.name()));

            let inv_set = inventory.clone();
            let sel_set = selected_index.clone();
            let draft_set = draft.clone();
            let push_log2 = push_log.clone();

            let task = gloo_file::callbacks::read_as_text(&file, move |res| match res {
                Ok(text) => match serde_json::from_str::<Inventory>(&text) {
                    Ok(inv) => {
                        let count = inv.songs.len();
                        inv_set.set(inv);
                        sel_set.set(None);
                        draft_set.set(Song::default());
                        push_log2(&format!("✅ Imported {count} song(s) from JSON."));
                    }
                    Err(e) => push_log2(&format!("⚠️ JSON parse error: {e}")),
                },
                Err(e) => push_log2(&format!("⚠️ File read error: {e:?}")),
            });

            reader.set(Some(task));
        })
    };

    // Clear everything
    let on_clear = {
        let inventory = inventory.clone();
        let selected_index = selected_index.clone();
        let draft = draft.clone();
        let push_log = push_log.clone();
        Callback::from(move |_| {
            let _ = LocalStorage::delete(STORAGE_KEY);
            inventory.set(Inventory::default());
            selected_index.set(None);
            draft.set(Song::default());
            push_log("🧹 Cleared inventory + LocalStorage.");
        })
    };

    // Field handlers
    let on_change_title = bind_input(draft.clone(), |song, v| song.title = v);
    let on_change_cover = bind_input(draft.clone(), |song, v| song.cover_art_url = v);
    let on_change_len = bind_input(draft.clone(), |song, v| song.length_mmss = v);
    let on_change_spotify = bind_input(draft.clone(), |song, v| song.spotify_url = v);

    let on_change_lyrics = {
        let draft = draft.clone();
        Callback::from(move |e: InputEvent| {
            let target = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok());
            if let Some(t) = target {
                let mut s = (*draft).clone();
                s.lyrics = t.value();
                draft.set(s);
            }
        })
    };

    let songs = inventory.songs.clone();
    let selected = *selected_index;
    let draft_song = (*draft).clone();

    let json_preview = if songs.is_empty() {
        String::new()
    } else {
        serde_json::to_string_pretty(&*inventory).unwrap_or_else(|_| "{}".into())
    };

    html! {
        <div style="
            font-family: system-ui, -apple-system, Segoe UI, Roboto, sans-serif;
            padding: 16px; max-width: 1100px; margin: 0 auto;
        ">
            <h1 style="margin: 0 0 12px 0;">{"Spotify Song Inventory"}</h1>

            <div style="display: grid; grid-template-columns: 320px 1fr; gap: 16px;">
                // Left: list
                <div style="border: 1px solid #ddd; border-radius: 12px; padding: 12px;">
                    <div style="display:flex; gap:8px; flex-wrap: wrap; margin-bottom: 10px;">
                        <button onclick={on_new} style={btn()}>{"New"}</button>
                        <button onclick={on_save} style={btn_primary()}>{"Save"}</button>
                        <button onclick={on_delete} style={btn_danger()}>{"Delete"}</button>
                    </div>

                    <div style="display:flex; gap:8px; flex-wrap: wrap; margin-bottom: 12px;">
                        {
                            if songs.is_empty() {
                                html! { <button disabled=true style={btn_disabled()}>{"Export JSON"}</button> }
                            } else {
                                html! { <button onclick={on_export} style={btn()}>{"Export JSON"}</button> }
                            }
                        }
                        <label style="display:inline-flex; align-items:center; gap:8px;">
                            <span style="font-size: 12px; opacity: 0.8;">{"Import JSON"}</span>
                            <input type="file" accept="application/json,.json" onchange={on_import_change} />
                        </label>
                        <button onclick={on_clear} style={btn()}>{"Clear"}</button>
                    </div>

                    <div style="display:flex; justify-content: space-between; align-items: baseline;">
                        <h3 style="margin: 0;">{"Songs"}</h3>
                        <span style="font-size: 12px; opacity: 0.7;">{format!("{} total", songs.len())}</span>
                    </div>

                    <div style="margin-top: 10px; display: flex; flex-direction: column; gap: 6px; max-height: 520px; overflow: auto;">
                        { for songs.iter().enumerate().map(|(i, s)| {
                            let is_sel = selected == Some(i);
                            let mut style = String::from(
                                "text-align:left; padding:10px; border-radius:10px; border:1px solid #e5e5e5; cursor:pointer; background:#fff;"
                            );
                            if is_sel {
                                style.push_str("border-color:#6aa6ff; box-shadow: 0 0 0 2px rgba(106,166,255,0.25);");
                            }
                            let on_select = on_select.clone();
                            html! {
                                <button style={style} onclick={Callback::from(move |_| on_select.emit(i))}>
                                    <div style="font-weight: 700; font-size: 14px; margin-bottom: 2px;">{ s.title.clone() }</div>
                                    <div style="font-size: 12px; opacity: 0.75;">{ format!("Length: {}", s.length_mmss) }</div>
                                </button>
                            }
                        }) }
                    </div>
                </div>

                // Right: editor + preview + log + JSON preview
                <div style="display:flex; flex-direction: column; gap: 16px;">
                    <div style="border: 1px solid #ddd; border-radius: 12px; padding: 12px;">
                        <h3 style="margin: 0 0 10px 0;">{"Song Details"}</h3>

                        <div style="display:grid; grid-template-columns: 1fr 1fr; gap: 10px;">
                            { field("Title", &draft_song.title, on_change_title, "Courage of the Last Light") }
                            { field("Length (MM:SS)", &draft_song.length_mmss, on_change_len, "3:42") }
                            { field("Cover Art URL", &draft_song.cover_art_url, on_change_cover, "https://...") }
                            { field("Spotify URL", &draft_song.spotify_url, on_change_spotify, "https://open.spotify.com/track/...") }
                        </div>

                        <div style="margin-top: 10px;">
                            <label style="display:block; font-size: 12px; opacity: 0.8; margin-bottom: 6px;">
                                {"Lyrics (optional)"}
                            </label>
                            <textarea
                                value={draft_song.lyrics}
                                oninput={on_change_lyrics}
                                rows="8"
                                style="width:100%; border:1px solid #e5e5e5; border-radius:10px; padding:10px; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', monospace;"
                                placeholder="Paste lyrics here..."
                            />
                        </div>

                        <div style="margin-top: 12px;">
                            { preview_card(&draft_song) }
                        </div>
                    </div>

                    <div style="border: 1px solid #ddd; border-radius: 12px; padding: 12px;">
                        <h3 style="margin: 0 0 10px 0;">{"Log"}</h3>
                        <textarea
                            readonly=true
                            value={(*log_text).clone()}
                            rows="8"
                            style="width:100%; border:1px solid #e5e5e5; border-radius:10px; padding:10px; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', monospace; background: #fafafa;"
                        />

                        <div style="margin-top: 12px;">
                            <h3 style="margin: 0 0 10px 0;">{"JSON Preview"}</h3>
                            {
                                if songs.is_empty() {
                                    html! { <div style="font-size: 12px; opacity: 0.75;">{"No entries yet — save your first song to generate JSON."}</div> }
                                } else {
                                    html! {
                                        <textarea
                                            readonly=true
                                            value={json_preview}
                                            rows="10"
                                            style="width:100%; border:1px solid #e5e5e5; border-radius:10px; padding:10px; font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, 'Liberation Mono', monospace; background: #fafafa;"
                                        />
                                    }
                                }
                            }
                        </div>

                        <div style="margin-top: 8px; font-size: 12px; opacity: 0.75;">
                            {"JSON “file” is created when you click Export (download). Import restores from a saved JSON file. Auto-saves to LocalStorage too."}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn field(label: &str, value: &str, oninput: Callback<InputEvent>, placeholder: &str) -> Html {
    html! {
        <div>
            <label style="display:block; font-size: 12px; opacity: 0.8; margin-bottom: 6px;">
                { label }
            </label>
            <input
                value={value.to_string()}
                {oninput}
                placeholder={placeholder.to_string()}
                style="width:100%; border:1px solid #e5e5e5; border-radius:10px; padding:10px;"
            />
        </div>
    }
}

fn bind_input(draft: UseStateHandle<Song>, mutator: fn(&mut Song, String)) -> Callback<InputEvent> {
    Callback::from(move |e: InputEvent| {
        let input = e
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
        if let Some(i) = input {
            let mut s = (*draft).clone();
            mutator(&mut s, i.value());
            draft.set(s);
        }
    })
}

fn preview_card(song: &Song) -> Html {
    let has_cover = !song.cover_art_url.trim().is_empty();
    let has_spotify = !song.spotify_url.trim().is_empty();
    let title = if song.title.trim().is_empty() {
        "Untitled"
    } else {
        song.title.trim()
    };

    html! {
        <div style="display:flex; gap:12px; align-items: flex-start; border:1px solid #eee; border-radius: 12px; padding: 12px; width: 100%; max-width: 680px;">
            <div style="width: 96px; height: 96px; border-radius: 12px; overflow:hidden; background: #f2f2f2; flex: 0 0 auto;">
                {
                    if has_cover {
                        html! { <img src={song.cover_art_url.clone()} style="width:100%; height:100%; object-fit: cover;" /> }
                    } else {
                        html! { <div style="display:flex; align-items:center; justify-content:center; height:100%; font-size: 12px; opacity: 0.6;">{"No cover"}</div> }
                    }
                }
            </div>
            <div style="flex:1;">
                <div style="font-weight:800; font-size: 16px; margin-bottom: 2px;">
                    { title }
                </div>
                <div style="font-size: 13px; opacity: 0.8; margin-bottom: 6px;">
                    { format!("Length: {}", song.length_mmss.trim()) }
                </div>
                {
                    if has_spotify {
                        html! {
                            <a href={song.spotify_url.clone()} target="_blank" style="font-size: 13px;">
                                {"Open in Spotify"}
                            </a>
                        }
                    } else {
                        html! { <div style="font-size: 13px; opacity: 0.6;">{"No Spotify URL yet."}</div> }
                    }
                }
            </div>
        </div>
    }
}

fn load_from_storage() -> Inventory {
    LocalStorage::get::<Inventory>(STORAGE_KEY).unwrap_or_default()
}

fn download_text_file(filename: &str, content: &str) -> Result<(), String> {
    let mut bag = BlobPropertyBag::new();
    bag.type_("application/json");

    let parts = js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(content));

    let blob = Blob::new_with_str_sequence_and_options(&parts, &bag)
        .map_err(|_| "Could not create Blob".to_string())?;

    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|_| "Could not create object URL".to_string())?;

    let window = web_sys::window().ok_or("No window".to_string())?;
    let document = window.document().ok_or("No document".to_string())?;
    let a = document
        .create_element("a")
        .map_err(|_| "Could not create <a> element".to_string())?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "Could not cast to HtmlAnchorElement".to_string())?;

    a.set_href(&url);
    a.set_download(filename);
    a.style().set_property("display", "none").ok();

    let body = document.body().ok_or("No body".to_string())?;
    body.append_child(&a)
        .map_err(|_| "Could not append link".to_string())?;
    a.click();
    body.remove_child(&a).ok();

    Url::revoke_object_url(&url).ok();
    log!(format!("Downloaded file: {filename}"));
    Ok(())
}

fn btn() -> String {
    "padding:10px 12px; border-radius:10px; border:1px solid #ddd; background:#fff; cursor:pointer;"
        .into()
}
fn btn_primary() -> String {
    "padding:10px 12px; border-radius:10px; border:1px solid #1b66ff; background:#1b66ff; color:#fff; cursor:pointer;"
        .into()
}
fn btn_danger() -> String {
    "padding:10px 12px; border-radius:10px; border:1px solid #d33; background:#fff; color:#d33; cursor:pointer;"
        .into()
}
fn btn_disabled() -> String {
    "padding:10px 12px; border-radius:10px; border:1px solid #ddd; background:#f3f3f3; color:#888; cursor:not-allowed;"
        .into()
}