use gloo::utils::document;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
enum IconSet {
    Moon,
    Orion,
    Starfield,
    RocketPlume,
}

impl IconSet {
    fn label(&self) -> &'static str {
        match self {
            IconSet::Moon => "Moon",
            IconSet::Orion => "Orion",
            IconSet::Starfield => "Starfield",
            IconSet::RocketPlume => "Rocket plume",
        }
    }

    fn all() -> Vec<IconSet> {
        vec![
            IconSet::Moon,
            IconSet::Orion,
            IconSet::Starfield,
            IconSet::RocketPlume,
        ]
    }
}

#[derive(Clone, PartialEq)]
struct Palette {
    bg: String,
    ring: String,
    ring_hi: String,
    text: String,
    muted: String,
    gold: String,
}

impl Palette {
    fn default_dark() -> Self {
        Self {
            bg: "#0b1020".to_string(),
            ring: "#1a2b5e".to_string(),
            ring_hi: "#7aa8ff".to_string(),
            text: "#e9eefc".to_string(),
            muted: "#9fb0d9".to_string(),
            gold: "#f4d06f".to_string(),
        }
    }
}

fn clean_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// Build SVG (IMPORTANT: no XML header; return only the <svg> element)
fn svg_for_patch(
    ring_text: &str,
    bottom_arc_text: &str,
    crew_raw: &str,
    motto: &str,
    icon: &IconSet,
    palette: &Palette,
    ring_thickness: f32,
    icon_x: f32,
    icon_y: f32,
    icon_scale: f32,
) -> String {
    let ring_text = clean_text(ring_text.trim());
    let bottom_arc_text = clean_text(bottom_arc_text.trim());
    let motto = clean_text(motto.trim());

    let crew_lines: Vec<String> = crew_raw
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(clean_text)
        .collect();

    let bg = palette.bg.as_str();
    let ring = palette.ring.as_str();
    let ring_hi = palette.ring_hi.as_str();
    let text = palette.text.as_str();
    let muted = palette.muted.as_str();
    let gold = palette.gold.as_str();

    let icon_markup = match icon {
        IconSet::Moon => format!(
            r##"
            <g transform="translate(0,2)">
              <circle cx="0" cy="0" r="92" fill="{bg}" />
              <path d="M 40 -70
                       A 80 80 0 1 0 40 70
                       A 62 62 0 1 1 40 -70 Z"
                    fill="{ring_hi}" opacity="0.9"/>
              <circle cx="-18" cy="-18" r="6" fill="{muted}" opacity="0.55"/>
              <circle cx="-36" cy="20" r="4" fill="{muted}" opacity="0.45"/>
              <circle cx="12" cy="34" r="5" fill="{muted}" opacity="0.35"/>
            </g>
        "##,
        ),
        IconSet::Orion => format!(
            r##"
            <g>
              <circle cx="0" cy="0" r="92" fill="{bg}" />
              <g opacity="0.95" stroke="{ring_hi}" stroke-width="2" fill="none">
                <path d="M -35 -40 L 10 -55 L 40 -20 L 25 35 L -20 55 L -45 10 Z" opacity="0.65"/>
                <path d="M -18 -8 L 0 2 L 18 12" opacity="0.85"/>
              </g>
              <g fill="{gold}">
                <circle cx="-35" cy="-40" r="4"/>
                <circle cx="10" cy="-55" r="4"/>
                <circle cx="40" cy="-20" r="4"/>
                <circle cx="25" cy="35" r="4"/>
                <circle cx="-20" cy="55" r="4"/>
                <circle cx="-45" cy="10" r="4"/>
                <circle cx="-18" cy="-8" r="3"/>
                <circle cx="0" cy="2" r="3"/>
                <circle cx="18" cy="12" r="3"/>
              </g>
            </g>
        "##,
        ),
        IconSet::Starfield => format!(
            r##"
            <g>
              <circle cx="0" cy="0" r="92" fill="{bg}" />
              <g fill="{muted}" opacity="0.8">
                <circle cx="-44" cy="-44" r="2"/><circle cx="-10" cy="-58" r="1.6"/>
                <circle cx="26" cy="-46" r="1.8"/><circle cx="52" cy="-18" r="1.7"/>
                <circle cx="44" cy="26" r="1.9"/><circle cx="-52" cy="10" r="1.7"/>
                <circle cx="-28" cy="46" r="1.8"/><circle cx="6" cy="56" r="1.6"/>
                <circle cx="22" cy="18" r="1.6"/><circle cx="-6" cy="24" r="1.4"/>
              </g>
              <path d="M0-44 L10-10 L44 0 L10 10 L0 44 L-10 10 L-44 0 L-10-10 Z"
                    fill="{gold}" opacity="0.95"/>
            </g>
        "##,
        ),
        IconSet::RocketPlume => format!(
            r##"
            <g>
              <circle cx="0" cy="0" r="92" fill="{bg}" />
              <g transform="translate(0,-6)">
                <path d="M0 -62 C 18 -42 22 -18 0 20 C -22 -18 -18 -42 0 -62 Z"
                      fill="{ring_hi}" opacity="0.95"/>
                <circle cx="0" cy="-24" r="10" fill="{bg}" opacity="0.95"/>
                <path d="M -16 8 L -44 22 L -22 -4 Z" fill="{muted}" opacity="0.75"/>
                <path d="M 16 8 L 44 22 L 22 -4 Z" fill="{muted}" opacity="0.75"/>
                <path d="M0 20 C 10 30 14 40 0 58 C -14 40 -10 30 0 20 Z"
                      fill="{gold}" opacity="0.95"/>
                <path d="M0 26 C 6 34 8 40 0 52 C -8 40 -6 34 0 26 Z"
                      fill="{ring_hi}" opacity="0.75"/>
              </g>
            </g>
        "##,
        ),
    };

    let crew_block = if crew_lines.is_empty() {
        format!(
            r#"<text x="0" y="34" text-anchor="middle" font-size="12" fill="{muted}" opacity="0.9">Add crew names</text>"#
        )
    } else {
        let mut t = String::new();
        let start_y = 26 - ((crew_lines.len().saturating_sub(1) as i32) * 14 / 2);
        for (i, line) in crew_lines.iter().take(6).enumerate() {
            let y = start_y + (i as i32) * 14;
            t.push_str(&format!(
                r#"<text x="0" y="{y}" text-anchor="middle" font-size="14" fill="{text}" opacity="0.95">{line}</text>"#,
            ));
        }
        t
    };

    let motto_block = if motto.is_empty() {
        String::new()
    } else {
        format!(
            r#"<text x="0" y="68" text-anchor="middle" font-size="12" fill="{muted}" opacity="0.95">“{motto}”</text>"#
        )
    };

    // ring thickness is the stroke width for the outer main ring
    let ring_thickness = ring_thickness.clamp(6.0, 30.0);

    // icon transform knobs
    let icon_scale = icon_scale.clamp(0.6, 1.6);
    let icon_x = icon_x.clamp(-60.0, 60.0);
    let icon_y = icon_y.clamp(-80.0, 80.0);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="1024" viewBox="-256 -256 512 512">
  <defs>
    <path id="ringTopArc" d="M -180 0 A 180 180 0 0 1 180 0" />
    <path id="ringBottomArc" d="M 180 0 A 180 180 0 0 1 -180 0" />
    <filter id="softGlow" x="-40%" y="-40%" width="180%" height="180%">
      <feGaussianBlur stdDeviation="2.4" result="blur" />
      <feMerge>
        <feMergeNode in="blur"/>
        <feMergeNode in="SourceGraphic"/>
      </feMerge>
    </filter>
  </defs>

  <!-- Outer ring -->
  <circle cx="0" cy="0" r="230" fill="{bg}" />
  <circle cx="0" cy="0" r="220" fill="none" stroke="{ring}" stroke-width="{ring_thickness}" />
  <circle cx="0" cy="0" r="220" fill="none" stroke="{ring_hi}" stroke-width="2" opacity="0.6" />

  <!-- Inner disc -->
  <circle cx="0" cy="0" r="178" fill="{bg}" stroke="{ring}" stroke-width="6" opacity="0.9" />

  <!-- Ring text -->
  <g filter="url(#softGlow)">
    <text font-family="ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Arial"
          font-size="20" fill="{text}" letter-spacing="2.2">
      <textPath href="#ringTopArc" startOffset="50%" text-anchor="middle">{ring_text}</textPath>
    </text>
    <text font-family="ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Arial"
          font-size="14" fill="{muted}" letter-spacing="2.0" opacity="0.95">
      <textPath href="#ringBottomArc" startOffset="50%" text-anchor="middle">{bottom_arc_text}</textPath>
    </text>
  </g>

  <!-- Icon layer (position + scale controls) -->
  <g transform="translate({icon_x},{icon_y}) scale({icon_scale})">
    {icon_markup}
  </g>

  <!-- Crew + motto -->
  <g transform="translate(0,34)">
    {crew_block}
    {motto_block}
  </g>

  <!-- Separators -->
  <g opacity="0.75">
    <circle cx="-206" cy="0" r="5" fill="{gold}"/>
    <circle cx="206" cy="0" r="5" fill="{gold}"/>
  </g>
</svg>"##,
        ring_text = ring_text,
        bottom_arc_text = bottom_arc_text,
        icon_markup = icon_markup,
        crew_block = crew_block,
        motto_block = motto_block,
        bg = bg,
        ring = ring,
        ring_hi = ring_hi,
        text = text,
        muted = muted,
        gold = gold,
        ring_thickness = ring_thickness,
        icon_x = icon_x,
        icon_y = icon_y,
        icon_scale = icon_scale,
    )
}

fn download_text_file(filename: &str, contents: &str, mime: &str) -> Result<(), JsValue> {
    // ✅ no "mut" warning for the binding; we mutate only the temp var
    let bag = {
        let b = web_sys::BlobPropertyBag::new();
        b.set_type(mime);
        b
    };

    let parts = js_sys::Array::new();
    parts.push(&JsValue::from_str(contents));

    let blob = web_sys::Blob::new_with_str_sequence_and_options(&parts, &bag)?;
    let object_url = web_sys::Url::create_object_url_with_blob(&blob)?;

    let a: web_sys::HtmlAnchorElement = document()
        .create_element("a")?
        .dyn_into::<web_sys::HtmlAnchorElement>()?;

    a.set_href(&object_url);
    a.set_download(filename);

    document().body().unwrap().append_child(&a)?;
    a.click();
    a.remove();

    web_sys::Url::revoke_object_url(&object_url)?;
    Ok(())
}

fn download_png_from_svg(svg: String, filename: &str, size: u32) -> Result<(), JsValue> {
    let doc = document();

    let canvas: web_sys::HtmlCanvasElement = doc
        .create_element("canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    canvas.set_width(size);
    canvas.set_height(size);

    let ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let encoded = js_sys::encode_uri_component(&svg);
    let data_url = format!("data:image/svg+xml;charset=utf-8,{}", encoded);

    let img = web_sys::HtmlImageElement::new()?;
    let img_for_closure = img.clone();

    let filename = filename.to_string();
    let doc_for_closure = doc.clone();
    let canvas_for_closure = canvas.clone();
    let ctx_for_closure = ctx.clone();

    let onload = Closure::<dyn FnMut()>::new(move || {
        let _ = ctx_for_closure.clear_rect(0.0, 0.0, size as f64, size as f64);
        let _ = ctx_for_closure.draw_image_with_html_image_element_and_dw_and_dh(
            &img_for_closure,
            0.0,
            0.0,
            size as f64,
            size as f64,
        );

        if let Ok(png_url) = canvas_for_closure.to_data_url_with_type("image/png") {
            if let Ok(a) = doc_for_closure.create_element("a") {
                if let Ok(a) = a.dyn_into::<web_sys::HtmlAnchorElement>() {
                    a.set_href(&png_url);
                    a.set_download(&filename);
                    if let Some(body) = doc_for_closure.body() {
                        let _ = body.append_child(&a);
                        a.click();
                        a.remove();
                    }
                }
            }
        }
    });

    img.set_onload(Some(onload.as_ref().unchecked_ref()));
    img.set_src(&data_url);
    onload.forget();

    Ok(())
}

#[function_component(App)]
fn app() -> Html {
    // Text
    let ring_text = use_state(|| "MISSION • ARTEMIS • LUNAR".to_string());
    let bottom_arc_text = use_state(|| "PATCHFORGE".to_string());
    let crew = use_state(|| "Colin\nLuan\nClark\nSoham".to_string());
    let motto = use_state(|| "From Bow to Booster".to_string());

    // Icon controls
    let icon_set = use_state(|| IconSet::Moon);
    let icon_x = use_state(|| 0.0_f32);
    let icon_y = use_state(|| -18.0_f32);
    let icon_scale = use_state(|| 1.0_f32);

    // Ring thickness
    let ring_thickness = use_state(|| 18.0_f32);

    // Colors
    let palette = use_state(Palette::default_dark);

    // Build SVG
    let svg = {
        let ring_text = (*ring_text).clone();
        let bottom_arc_text = (*bottom_arc_text).clone();
        let crew = (*crew).clone();
        let motto = (*motto).clone();
        let icon_set = (*icon_set).clone();
        let palette = (*palette).clone();
        let ring_thickness = *ring_thickness;
        let icon_x = *icon_x;
        let icon_y = *icon_y;
        let icon_scale = *icon_scale;

        use_memo(
            (
                ring_text,
                bottom_arc_text,
                crew,
                motto,
                icon_set,
                palette,
                ring_thickness,
                icon_x,
                icon_y,
                icon_scale,
            ),
            |(rt, bt, cr, mo, ic, pal, th, ix, iy, sc)| {
                svg_for_patch(rt, bt, cr, mo, ic, pal, *th, *ix, *iy, *sc)
            },
        )
    };

    // Text handlers
    let on_ring = {
        let ring_text = ring_text.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            ring_text.set(v);
        })
    };

    let on_bottom_arc = {
        let bottom_arc_text = bottom_arc_text.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            bottom_arc_text.set(v);
        })
    };

    let on_crew = {
        let crew = crew.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
            crew.set(v);
        })
    };

    let on_motto = {
        let motto = motto.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            motto.set(v);
        })
    };

    // Icon selector
    let on_icon = {
        let icon_set = icon_set.clone();
        Callback::from(move |e: Event| {
            let v = e.target_unchecked_into::<web_sys::HtmlSelectElement>().value();
            let picked = match v.as_str() {
                "Moon" => IconSet::Moon,
                "Orion" => IconSet::Orion,
                "Starfield" => IconSet::Starfield,
                "Rocket plume" => IconSet::RocketPlume,
                _ => IconSet::Moon,
            };
            icon_set.set(picked);
        })
    };

    // Sliders
    let on_ring_thickness = {
        let ring_thickness = ring_thickness.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            if let Ok(f) = v.parse::<f32>() {
                ring_thickness.set(f);
            }
        })
    };

    let on_icon_x = {
        let icon_x = icon_x.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            if let Ok(f) = v.parse::<f32>() {
                icon_x.set(f);
            }
        })
    };

    let on_icon_y = {
        let icon_y = icon_y.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            if let Ok(f) = v.parse::<f32>() {
                icon_y.set(f);
            }
        })
    };

    let on_icon_scale = {
        let icon_scale = icon_scale.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            if let Ok(f) = v.parse::<f32>() {
                icon_scale.set(f);
            }
        })
    };

    // Color pickers (input[type=color] gives hex)
    let on_color = |field: &'static str,
                    palette: UseStateHandle<Palette>|
     -> Callback<InputEvent> {
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            let mut p = (*palette).clone();
            match field {
                "bg" => p.bg = v,
                "ring" => p.ring = v,
                "ring_hi" => p.ring_hi = v,
                "text" => p.text = v,
                "muted" => p.muted = v,
                "gold" => p.gold = v,
                _ => {}
            }
            palette.set(p);
        })
    };

    let icon_options = IconSet::all()
        .into_iter()
        .map(|ic| {
            let label = ic.label().to_string();
            html! { <option value={label.clone()} selected={*icon_set == ic}>{label}</option> }
        })
        .collect::<Html>();

    let svg_node = Html::from_html_unchecked(AttrValue::from((*svg).clone()));

    // Downloads
    let download_svg = {
        let svg = (*svg).clone();
        Callback::from(move |_| {
            let _ = download_text_file("patchforge.svg", &svg, "image/svg+xml;charset=utf-8");
        })
    };

    let download_png = {
        let svg = (*svg).clone();
        Callback::from(move |_| {
            let _ = download_png_from_svg(svg.clone(), "patchforge.png", 1024);
        })
    };

    let p = (*palette).clone();

    html! {
        <div class="wrap">
            <div class="header">
                <div class="brand">
                    <h1>{ "PatchForge" }</h1>
                    <p>{ "Mission patch SVG generator — export SVG or PNG." }</p>
                </div>
            </div>

            <div class="grid">
                <div class="card">
                    <div class="section-title">
                        <h2>{ "Controls" }</h2>
                        <span>{ "Live preview" }</span>
                    </div>

                    <div class="form">
                        <div class="row">
                            <label>{ "Ring text" }</label>
                            <input value={(*ring_text).clone()} oninput={on_ring} />
                        </div>

                        <div class="row">
                            <label>{ "Bottom arc text" }</label>
                            <input value={(*bottom_arc_text).clone()} oninput={on_bottom_arc} />
                        </div>

                        <div class="row">
                            <label>{ "Crew names" }</label>
                            <textarea value={(*crew).clone()} oninput={on_crew}></textarea>
                        </div>

                        <div class="row">
                            <label>{ "Mission motto" }</label>
                            <input value={(*motto).clone()} oninput={on_motto} />
                        </div>

                        <div class="row">
                            <label>{ "Icon set" }</label>
                            <select onchange={on_icon}>
                                { icon_options }
                            </select>
                        </div>

                        <div class="row">
                            <label>{ "Ring thickness" }</label>
                            <input type="range" min="6" max="30" step="1"
                                   value={ring_thickness.to_string()}
                                   oninput={on_ring_thickness} />
                        </div>

                        <div class="row">
                            <label>{ "Icon X" }</label>
                            <input type="range" min="-60" max="60" step="1"
                                   value={icon_x.to_string()}
                                   oninput={on_icon_x} />
                        </div>

                        <div class="row">
                            <label>{ "Icon Y" }</label>
                            <input type="range" min="-80" max="80" step="1"
                                   value={icon_y.to_string()}
                                   oninput={on_icon_y} />
                        </div>

                        <div class="row">
                            <label>{ "Icon scale" }</label>
                            <input type="range" min="0.6" max="1.6" step="0.05"
                                   value={icon_scale.to_string()}
                                   oninput={on_icon_scale} />
                        </div>

                        <div class="row">
                            <label>{ "BG" }</label>
                            <input type="color" value={p.bg.clone()} oninput={on_color("bg", palette.clone())} />
                        </div>
                        <div class="row">
                            <label>{ "Ring" }</label>
                            <input type="color" value={p.ring.clone()} oninput={on_color("ring", palette.clone())} />
                        </div>
                        <div class="row">
                            <label>{ "Highlight" }</label>
                            <input type="color" value={p.ring_hi.clone()} oninput={on_color("ring_hi", palette.clone())} />
                        </div>
                        <div class="row">
                            <label>{ "Text" }</label>
                            <input type="color" value={p.text.clone()} oninput={on_color("text", palette.clone())} />
                        </div>
                        <div class="row">
                            <label>{ "Muted" }</label>
                            <input type="color" value={p.muted.clone()} oninput={on_color("muted", palette.clone())} />
                        </div>
                        <div class="row">
                            <label>{ "Gold" }</label>
                            <input type="color" value={p.gold.clone()} oninput={on_color("gold", palette.clone())} />
                        </div>

                        <div class="btnbar">
                            <button class="primary" onclick={download_svg}>{ "Download SVG" }</button>
                            <button onclick={download_png}>{ "Download PNG" }</button>
                        </div>

                        <div class="smallnote">
                            { "Tip: Use icon sliders to center the emblem; ring thickness changes the outer band weight." }
                        </div>
                    </div>
                </div>

                <div class="card preview">
                    <div class="section-title">
                        <h2>{ "Patch" }</h2>
                        <span>{ "1024×1024 SVG" }</span>
                    </div>

                    <div class="patchbox">
                        { svg_node }
                    </div>

                    <div class="hint">
                        { "Upgrades installed: colors, ring thickness, inner arc text, and icon position/scale." }
                    </div>
                </div>
            </div>
        </div>
    }
}

#[wasm_bindgen(start)]
pub fn run() {
    yew::Renderer::<App>::new().render();
}