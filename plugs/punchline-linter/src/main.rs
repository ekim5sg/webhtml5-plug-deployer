use regex::Regex;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::window;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq)]
struct LintResult {
    pun_density: u8,
    groan_factor: u8,
    kid_safe: &'static str,
    messages: Vec<String>,
}

fn tokenize(s: &str) -> Vec<String> {
    let re = Regex::new(r"[A-Za-z0-9']+").unwrap();
    re.find_iter(s).map(|m| m.as_str().to_lowercase()).collect()
}

fn overlap(a: &[String], b: &[String]) -> f32 {
    use std::collections::HashSet;
    let sa: HashSet<&str> = a.iter().map(|x| x.as_str()).collect();
    let sb: HashSet<&str> = b.iter().map(|x| x.as_str()).collect();
    let inter = sa.intersection(&sb).count() as f32;
    let uni = sa.union(&sb).count() as f32;
    if uni == 0.0 { 0.0 } else { inter / uni }
}

fn lint(setup: &str, punch: &str) -> LintResult {
    let a = tokenize(setup);
    let b = tokenize(punch);

    let reuse = (overlap(&a, &b) * 100.0) as i32;
    let pun_density = reuse.clamp(0, 100) as u8;

    let mut groan = 30;
    if b.len() <= 8 && a.len() >= 20 {
        groan += 25;
    }
    if punch.trim().ends_with("...") {
        groan += 10;
    }
    let groan_factor = groan.min(100) as u8;

    // Super simple kid-safe gate for MVP
    let lower = format!("{} {}", setup, punch).to_lowercase();
    let kid_safe = if lower.contains("kill") || lower.contains("suicide") || lower.contains("porn") {
        "FAIL"
    } else {
        "G"
    };

    let mut messages = vec![];
    if setup.trim().is_empty() {
        messages.push("error[INPUT001]: Missing setup".into());
    }
    if punch.trim().is_empty() {
        messages.push("error[INPUT002]: Missing punchline".into());
    }

    if reuse > 10 {
        messages.push("info[PUN001]: Twist reuses setup keywords".into());
    } else {
        messages.push("warning[PUN000]: Low keyword reuse".into());
    }
    if b.len() <= 8 && !punch.trim().is_empty() {
        messages.push("info[GROAN001]: Short punchline boosts groan factor".into());
    }

    LintResult { pun_density, groan_factor, kid_safe, messages }
}

async fn copy_to_clipboard(text: String) -> Result<(), String> {
    let win = window().ok_or("No window available")?;
    let cb = win.navigator().clipboard(); // Clipboard (NOT Option)
    let promise = cb.write_text(&text);
    JsFuture::from(promise)
        .await
        .map_err(|_| "Clipboard write rejected".to_string())?;
    Ok(())
}

fn safe_lines(s: &str) -> Vec<String> {
    let trimmed = s.trim_end_matches('\n');
    if trimmed.trim().is_empty() {
        vec![]
    } else {
        trimmed.lines().map(|x| x.to_string()).collect()
    }
}

fn pretty_git_like(setup: &str, punch: &str) -> String {
    let old_lines = safe_lines(setup);
    let new_lines = safe_lines(punch);

    let old_n = old_lines.len().max(1);
    let new_n = new_lines.len().max(1);

    let mut out = String::new();
    out.push_str("diff --git a/joke.txt b/joke.txt\n");
    out.push_str("index dad000..groan999 100644\n");
    out.push_str("--- a/joke.txt  (setup)\n");
    out.push_str("+++ b/joke.txt  (punchline)\n");
    out.push_str(&format!("@@ -1,{} +1,{} @@\n", old_n, new_n));

    if old_lines.is_empty() {
        out.push_str("- (empty)\n");
    } else {
        for l in old_lines {
            out.push_str("- ");
            out.push_str(&l);
            out.push('\n');
        }
    }

    if new_lines.is_empty() {
        out.push_str("+ (empty)\n");
    } else {
        for l in new_lines {
            out.push_str("+ ");
            out.push_str(&l);
            out.push('\n');
        }
    }

    out
}

#[function_component(App)]
fn app() -> Html {
    let setup = use_state(|| "".to_string());
    let punch = use_state(|| "".to_string());
    let result = use_state(|| lint("", ""));
    let output = use_state(|| "".to_string());
    let copy_status = use_state(|| "".to_string());

    let on_lint = {
        let setup = setup.clone();
        let punch = punch.clone();
        let result = result.clone();
        Callback::from(move |_| {
            result.set(lint(&setup, &punch));
        })
    };

    let on_pretty = {
        let setup = setup.clone();
        let punch = punch.clone();
        let output = output.clone();
        Callback::from(move |_| {
            output.set(pretty_git_like(&setup, &punch));
        })
    };

    let on_copy = {
        let output = output.clone();
        let copy_status = copy_status.clone();
        Callback::from(move |_| {
            let txt = (*output).clone();
            if txt.trim().is_empty() {
                copy_status.set("Nothing to copy yet.".into());
                return;
            }
            copy_status.set("Copying…".into());
            let copy_status2 = copy_status.clone();
            spawn_local(async move {
                match copy_to_clipboard(txt).await {
                    Ok(_) => copy_status2.set("Copied ✅".into()),
                    Err(e) => copy_status2.set(format!("Copy failed: {}", e)),
                }
            });
        })
    };

    // Build line-aligned diff view
    let left_lines = safe_lines(&setup);
    let right_lines = safe_lines(&punch);
    let max_lines = left_lines.len().max(right_lines.len()).max(1);

    let diff_rows = (0..max_lines).map(|i| {
        let ln = (i + 1).to_string();
        let ltxt = left_lines.get(i).cloned().unwrap_or_default();
        let rtxt = right_lines.get(i).cloned().unwrap_or_default();

        let l_is_empty = ltxt.trim().is_empty();
        let r_is_empty = rtxt.trim().is_empty();

        html! {
            <div class="diffrow">
                <div class="cell minus">
                    <div class="line">
                        <div class="ln">{ ln.clone() }</div>
                        <div class="gutter">{ "-" }</div>
                        <div class={classes!(if l_is_empty { "empty" } else { "" })}>
                            { if left_lines.is_empty() { "(empty)".to_string() } else if ltxt.is_empty() { " ".to_string() } else { ltxt } }
                        </div>
                    </div>
                </div>
                <div class="cell plus">
                    <div class="line">
                        <div class="ln">{ ln }</div>
                        <div class="gutter">{ "+" }</div>
                        <div class={classes!(if r_is_empty { "empty" } else { "" })}>
                            { if right_lines.is_empty() { "(empty)".to_string() } else if rtxt.is_empty() { " ".to_string() } else { rtxt } }
                        </div>
                    </div>
                </div>
            </div>
        }
    });

    html! {
        <div class="wrap">
            <div class="topbar">
                <div class="brand">
                    <h1>{"Punchline Linter"}</h1>
                    <p>{"Treat dad jokes like code — lint, refactor, diff, and minify."}</p>
                </div>
                <div class="badges">
                    <div class="badge">{"Rust + Yew"}</div>
                    <div class="badge">{"WASM Dev Tool"}</div>
                </div>
            </div>

            <div class="grid">
                <div class="card">
                    <div class="hd">
                        <h3>{"Input"}</h3>
                        <button class="primary" onclick={on_lint}>{"Lint"}</button>
                    </div>
                    <div class="bd">
                        <textarea
                            placeholder="Setup..."
                            value={(*setup).clone()}
                            oninput={{
                                let setup = setup.clone();
                                Callback::from(move |e: InputEvent| {
                                    let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                                    setup.set(v);
                                })
                            }}
                        />
                        <textarea
                            placeholder="Punchline..."
                            value={(*punch).clone()}
                            oninput={{
                                let punch = punch.clone();
                                Callback::from(move |e: InputEvent| {
                                    let v = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                                    punch.set(v);
                                })
                            }}
                        />
                    </div>
                </div>

                <div class="card">
                    <div class="hd">
                        <h3>{"Scores"}</h3>
                        <button onclick={on_pretty}>{"Pretty Diff"}</button>
                        <button onclick={on_copy}>{"Copy"}</button>
                    </div>
                    <div class="bd">
                        <div class="metric">
                            <div>{"Pun Density"}<div class="n">{result.pun_density}</div></div>
                        </div>
                        <div class="metric">
                            <div>{"Groan Factor"}<div class="n">{result.groan_factor}</div></div>
                        </div>
                        <div class="metric">
                            <div>{"Kid Safe"}<div class="n">{result.kid_safe}</div></div>
                        </div>

                        <div class="log">
                            { for result.messages.iter().map(|m| html!{ <div>{m}</div> }) }
                        </div>

                        <div class="log" style="margin-top:10px;">
                            { (*output).clone() }
                        </div>

                        <div class="small" style="margin-top:8px;">
                            { (*copy_status).clone() }
                        </div>
                    </div>
                </div>
            </div>

            <div class="card diffcard">
                <div class="hd"><h3>{"Side-by-Side Diff"}</h3></div>
                <div class="bd diffgrid">
                    <div class="diffhdr">
                        <div>{"Setup (-)"}</div>
                        <div>{"Punchline (+)"}</div>
                    </div>
                    { for diff_rows }
                </div>
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}