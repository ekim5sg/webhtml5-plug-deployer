use gloo::timers::callback::Interval;
use js_sys::Math;
use wasm_bindgen::JsCast;
use web_sys::{
    window, Blob, CanvasRenderingContext2d, HtmlAnchorElement, HtmlCanvasElement, HtmlInputElement,
    Url,
};
use yew::prelude::*;

const PI_DIGITS: &str = "314159265358979323846264338327950288419716939937510\
58209749445923078164062862089986280348253421170679\
82148086513282306647093844609550582231725359408128\
48111745028410270193852110555964462294895493038196\
44288109756659334461284756482337867831652712019091\
45648566923460348610454326648213393607260249141273\
72458700660631558817488152092096282925409171536436";

#[derive(Clone, PartialEq)]
enum ArtMode {
    Spiral,
    Orbit,
    Burst,
}

impl ArtMode {
    fn label(&self) -> &'static str {
        match self {
            ArtMode::Spiral => "Spiral",
            ArtMode::Orbit => "Orbit",
            ArtMode::Burst => "Burst",
        }
    }
}

fn palette_color(digit: u32) -> String {
    match digit {
        0 => "#7cc7ff".to_string(),
        1 => "#ffd166".to_string(),
        2 => "#ff8fab".to_string(),
        3 => "#8ce99a".to_string(),
        4 => "#caa8ff".to_string(),
        5 => "#ffb703".to_string(),
        6 => "#72efdd".to_string(),
        7 => "#ff7b7b".to_string(),
        8 => "#9bf6ff".to_string(),
        _ => "#f8f9fa".to_string(),
    }
}

fn get_canvas_and_ctx(
    canvas_ref: &NodeRef,
) -> Option<(HtmlCanvasElement, CanvasRenderingContext2d)> {
    let canvas: HtmlCanvasElement = canvas_ref.cast()?;
    let ctx = canvas
        .get_context("2d")
        .ok()
        .flatten()?
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()?;
    Some((canvas, ctx))
}

fn clear_canvas(ctx: &CanvasRenderingContext2d, width: f64, height: f64) {
    ctx.set_fill_style_str("#030811");
    ctx.fill_rect(0.0, 0.0, width, height);

    for _ in 0..60 {
        let x = Math::random() * width;
        let y = Math::random() * height;
        let alpha = 0.2 + Math::random() * 0.5;
        ctx.set_fill_style_str(&format!("rgba(255,255,255,{alpha:.2})"));
        ctx.begin_path();
        let _ = ctx.arc(x, y, 1.1 + Math::random() * 1.5, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();
    }
}

fn draw_art(
    ctx: &CanvasRenderingContext2d,
    width: f64,
    height: f64,
    total_digits: usize,
    mode: &ArtMode,
    progress: usize,
) {
    clear_canvas(ctx, width, height);

    let center_x = width / 2.0;
    let center_y = height / 2.0;

    let max_steps = progress.min(total_digits);
    let mut x = center_x;
    let mut y = center_y;

    for (i, ch) in PI_DIGITS.chars().cycle().take(max_steps).enumerate() {
        let digit = ch.to_digit(10).unwrap_or(0);
        let color = palette_color(digit);

        ctx.set_stroke_style_str(&color);
        ctx.set_fill_style_str(&color);

        match mode {
            ArtMode::Spiral => {
                let angle = (i as f64 * 0.17) + (digit as f64 * 0.23);
                let length = 4.0 + digit as f64 * 1.8;
                let new_x = x + angle.cos() * length;
                let new_y = y + angle.sin() * length;

                ctx.set_line_width(1.0 + (digit % 4) as f64 * 0.5);
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(new_x, new_y);
                ctx.stroke();

                x = new_x;
                y = new_y;

                if x < 20.0 || x > width - 20.0 || y < 20.0 || y > height - 20.0 {
                    x = center_x;
                    y = center_y;
                }
            }
            ArtMode::Orbit => {
                let orbit_r = 30.0 + (i as f64 * 0.65) % (height.min(width) * 0.4);
                let angle = (i as f64 * 0.11) + (digit as f64 * 0.42);
                let px = center_x + orbit_r * angle.cos();
                let py = center_y + orbit_r * angle.sin();

                ctx.set_line_width(1.0 + (digit % 3) as f64);
                ctx.begin_path();
                let _ = ctx.arc(px, py, 1.5 + digit as f64 * 0.4, 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();

                if i > 0 {
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    ctx.line_to(px, py);
                    ctx.stroke();
                }

                x = px;
                y = py;
            }
            ArtMode::Burst => {
                let angle = ((digit as f64) * 36.0 + i as f64 * 2.0).to_radians();
                let length = 25.0 + digit as f64 * 8.0 + (i as f64 % 60.0);
                let px = center_x + angle.cos() * length;
                let py = center_y + angle.sin() * length;

                ctx.set_line_width(0.8 + (digit % 5) as f64 * 0.5);
                ctx.begin_path();
                ctx.move_to(center_x, center_y);
                ctx.line_to(px, py);
                ctx.stroke();

                ctx.begin_path();
                let _ = ctx.arc(px, py, 1.2 + (digit % 4) as f64, 0.0, std::f64::consts::PI * 2.0);
                ctx.fill();
            }
        }
    }

    ctx.set_fill_style_str("rgba(255,255,255,0.75)");
    ctx.set_font("bold 20px Arial");
    let _ = ctx.fill_text("Pi Art Generator", 18.0, 32.0);

    ctx.set_fill_style_str("rgba(255,255,255,0.55)");
    ctx.set_font("14px Arial");
    let _ = ctx.fill_text(
        &format!("Mode: {} • Digits: {}", mode.label(), total_digits),
        18.0,
        54.0,
    );
}

#[function_component(App)]
fn app() -> Html {
    let canvas_ref = use_node_ref();

    let digits = use_state(|| 600usize);
    let progress = use_state(|| 600usize);
    let is_animating = use_state(|| false);
    let mode = use_state(|| ArtMode::Spiral);

    {
        let canvas_ref = canvas_ref.clone();
        let digits = digits.clone();
        let progress = progress.clone();
        let mode = mode.clone();

        use_effect_with(
            ((*digits), (*progress), (*mode).clone()),
            move |_| {
                if let Some((canvas, ctx)) = get_canvas_and_ctx(&canvas_ref) {
                    draw_art(
                        &ctx,
                        canvas.width() as f64,
                        canvas.height() as f64,
                        *digits,
                        &mode,
                        *progress,
                    );
                }
                || {}
            },
        );
    }

    {
        let is_animating = is_animating.clone();
        let progress = progress.clone();
        let digits = digits.clone();

        use_effect_with((*is_animating, *digits), move |_| {
            let interval = if *is_animating {
                Some(Interval::new(24, move || {
                    let next = (*progress + 12).min(*digits);
                    progress.set(next);
                    if next >= *digits {
                        // stop requested by state on next user interaction; visual completes here
                    }
                }))
            } else {
                None
            };

            move || drop(interval)
        });
    }

    let on_digits_input = {
        let digits = digits.clone();
        let progress = progress.clone();
        let is_animating = is_animating.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let value = input.value().parse::<usize>().unwrap_or(600).clamp(50, 5000);
            digits.set(value);
            progress.set(value);
            is_animating.set(false);
        })
    };

    let set_spiral = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(ArtMode::Spiral))
    };

    let set_orbit = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(ArtMode::Orbit))
    };

    let set_burst = {
        let mode = mode.clone();
        Callback::from(move |_| mode.set(ArtMode::Burst))
    };

    let generate_now = {
        let progress = progress.clone();
        let digits = digits.clone();
        let is_animating = is_animating.clone();
        Callback::from(move |_| {
            progress.set(*digits);
            is_animating.set(false);
        })
    };

    let animate_draw = {
        let progress = progress.clone();
        let is_animating = is_animating.clone();
        Callback::from(move |_| {
            progress.set(0);
            is_animating.set(true);
        })
    };

    let clear_art = {
        let canvas_ref = canvas_ref.clone();
        let is_animating = is_animating.clone();
        let progress = progress.clone();
        Callback::from(move |_| {
            is_animating.set(false);
            progress.set(0);
            if let Some((canvas, ctx)) = get_canvas_and_ctx(&canvas_ref) {
                clear_canvas(&ctx, canvas.width() as f64, canvas.height() as f64);
            }
        })
    };

    let download_png = {
        let canvas_ref = canvas_ref.clone();
        Callback::from(move |_| {
            let Some((canvas, _ctx)) = get_canvas_and_ctx(&canvas_ref) else {
                return;
            };

            let Ok(data_url) = canvas.to_data_url() else {
                return;
            };

            let Some(win) = window() else {
                return;
            };
            let Some(document) = win.document() else {
                return;
            };

            let Ok(element) = document.create_element("a") else {
                return;
            };
            let Ok(anchor) = element.dyn_into::<HtmlAnchorElement>() else {
                return;
            };

            anchor.set_href(&data_url);
            anchor.set_download("pi-art-generator.png");
            let _ = anchor.style().set_property("display", "none");

            if let Some(body) = document.body() {
                let _ = body.append_child(&anchor);
                anchor.click();
                let _ = body.remove_child(&anchor);
            }
        })
    };

    let displayed_digits = (*progress).min(*digits);

    html! {
        <div class="app-shell">
            <section class="hero">
                <div class="kicker">{ "MIKEGYVER STUDIO • PI DAY VISUAL BUILD" }</div>
                <h1>{ "Pi Art Generator v2" }</h1>
                <p>
                    { "Turn the digits of π into colorful generative artwork. Change the digit count, switch styles, animate the drawing, and export your creation as a PNG." }
                </p>
            </section>

            <section class="grid">
                <aside class="card">
                    <h2>{ "Art Controls" }</h2>

                    <div class="control-group">
                        <div class="control-label">
                            <span>{ "Digits Used" }</span>
                            <span class="control-value">{ *digits }</span>
                        </div>
                        <input
                            type="range"
                            min="50"
                            max="5000"
                            step="10"
                            value={digits.to_string()}
                            oninput={on_digits_input}
                        />
                    </div>

                    <h3>{ "Art Mode" }</h3>
                    <div class="mode-row">
                        <button
                            class={classes!("secondary", matches!(*mode, ArtMode::Spiral).then_some("active"))}
                            onclick={set_spiral}
                        >
                            { "Spiral" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*mode, ArtMode::Orbit).then_some("active"))}
                            onclick={set_orbit}
                        >
                            { "Orbit" }
                        </button>
                        <button
                            class={classes!("secondary", matches!(*mode, ArtMode::Burst).then_some("active"))}
                            onclick={set_burst}
                        >
                            { "Burst" }
                        </button>
                    </div>

                    <h3>{ "Actions" }</h3>
                    <div class="button-row">
                        <button class="primary" onclick={generate_now}>{ "Generate" }</button>
                        <button class="secondary" onclick={animate_draw}>{ "Animate" }</button>
                        <button class="secondary" onclick={clear_art}>{ "Clear" }</button>
                        <button class="secondary" onclick={download_png}>{ "Download PNG" }</button>
                    </div>

                    <div class="note-box">
                        { "Each digit of π influences direction, color, distance, and shape behavior. The result is a shareable visual artwork generated from math." }
                    </div>
                </aside>

                <main class="card preview-card">
                    <div class="canvas-wrap">
                        <canvas
                            ref={canvas_ref}
                            width="1000"
                            height="700"
                        />
                    </div>

                    <div class="stats">
                        <div class="stat">
                            <div class="stat-label">{ "Mode" }</div>
                            <div class="stat-value">{ mode.label() }</div>
                        </div>
                        <div class="stat">
                            <div class="stat-label">{ "Digits Drawn" }</div>
                            <div class="stat-value">{ displayed_digits }</div>
                        </div>
                        <div class="stat">
                            <div class="stat-label">{ "Animation" }</div>
                            <div class="stat-value">
                                { if *is_animating { "Running" } else { "Ready" } }
                            </div>
                        </div>
                    </div>

                    <div class="footer">
                        <strong>{ "Pi Day art mission:" }</strong>
                        { " Use π to paint something unexpected." }
                    </div>
                </main>
            </section>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}