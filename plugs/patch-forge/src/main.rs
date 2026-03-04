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

fn clean_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn svg_for_patch(ring_text: &str, crew_raw: &str, motto: &str, icon: &IconSet) -> String {
    let ring_text = clean_text(ring_text.trim());
    let motto = clean_text(motto.trim());

    let crew_lines: Vec<String> = crew_raw
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(clean_text)
        .collect();

    let bg = "#0b1020";
    let ring = "#1a2b5e";
    let ring_hi = "#7aa8ff";
    let text = "#e9eefc";
    let muted = "#9fb0d9";
    let gold = "#f4d06f";

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

    // IMPORTANT: no XML header. Just the <svg> element so HTML parsing keeps it.
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

  <circle cx="0" cy="0" r="230" fill="{bg}" />
  <circle cx="0" cy="0" r="220" fill="none" stroke="{ring}" stroke-width="18" />
  <circle cx="0" cy="0" r="220" fill="none" stroke="{ring_hi}" stroke-width="2" opacity="0.6" />

  <circle cx="0" cy="0" r="178" fill="{bg}" stroke="{ring}" stroke-width="6" opacity="0.9" />

  <g filter="url(#softGlow)">
    <text font-family="ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Arial"
          font-size="20" fill="{text}" letter-spacing="2.2">
      <textPath href="#ringTopArc" startOffset="50%" text-anchor="middle">{ring_text}</textPath>
    </text>
    <text font-family="ui-sans-serif, system-ui, -apple-system, Segoe UI, Roboto, Arial"
          font-size="14" fill="{muted}" letter-spacing="2.0" opacity="0.95">
      <textPath href="#ringBottomArc" startOffset="50%" text-anchor="middle">PATCHFORGE</textPath>
    </text>
  </g>

  <g transform="translate(0,-18)">
    {icon_markup}
  </g>

  <g transform="translate(0,34)">
    {crew_block}
    {motto_block}
  </g>

  <g opacity="0.75">
    <circle cx="-206" cy="0" r="5" fill="{gold}"/>
    <circle cx="206" cy="0" r="5" fill="{gold}"/>
  </g>
</svg>"##,
        ring_text = ring_text,
        icon_markup = icon_markup,
        crew_block = crew_block,
        motto_block = motto_block,
        bg = bg,
        ring = ring,
        ring_hi = ring_hi,
        text = text,
        muted = muted,
        gold = gold,
    )
}

fn download_text_file(filename: &str, contents: &str, mime: &str) -> Result<(), JsValue> {
    let bag = {
        let mut b = web_sys::BlobPropertyBag::new();
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
    let ring_text = use_state(|| "MISSION • ARTEMIS • LUNAR".to_string());
    let crew = use_state(|| "Colin\nLuan\nClark\nSoham".to_string());
    let motto = use_state(|| "From Bow to Booster".to_string());
    let icon_set = use_state(|| IconSet::Moon);

    let svg = {
        let ring_text = (*ring_text).clone();
        let crew = (*crew).clone();
        let motto = (*motto).clone();
        let icon_set = (*icon_set).clone();
        use_memo((ring_text, crew, motto, icon_set), |(rt, cr, mo, ic)| {
            svg_for_patch(rt, cr, mo, ic)
        })
    };

    let on_ring = {
        let ring_text = ring_text.clone();
        Callback::from(move |e: InputEvent| {
            let v = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
            ring_text.set(v);
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

    let on_icon = {
        let icon_set = icon_set.clone();
        Callback::from(move |e: Event| {
            let v = e
                .target_unchecked_into::<web_sys::HtmlSelectElement>()
                .value();
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

    let icon_options = IconSet::all()
        .into_iter()
        .map(|ic| {
            let label = ic.label().to_string();
            html! { <option value={label.clone()} selected={*icon_set == ic}>{label}</option> }
        })
        .collect::<Html>();

    let svg_node = Html::from_html_unchecked(AttrValue::from((*svg).clone()));

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

                        <div class="btnbar">
                            <button class="primary" onclick={download_svg}>{ "Download SVG" }</button>
                            <button onclick={download_png}>{ "Download PNG" }</button>
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
                        { "Next upgrades: color pickers, ring thickness, inner arc text, or icon position sliders." }
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