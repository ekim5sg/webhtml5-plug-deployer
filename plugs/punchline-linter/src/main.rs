use regex::Regex;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::{window, Clipboard};
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
    let pun_density = reuse.min(100).max(0) as u8;

    let mut groan = 30;
    if b.len() <= 8 && a.len() >= 20 { groan += 25; }
    if punch.trim().ends_with("...") { groan += 10; }
    let groan_factor = groan.min(100) as u8;

    let kid_safe = if punch.contains("kill") { "FAIL" } else { "G" };

    let mut messages = vec![];
    if reuse > 10 {
        messages.push("info[PUN001]: Twist reuses setup keywords".into());
    } else {
        messages.push("warning[PUN000]: Low keyword reuse".into());
    }

    if b.len() <= 8 {
        messages.push("info[GROAN001]: Short punchline boosts groan factor".into());
    }

    LintResult { pun_density, groan_factor, kid_safe, messages }
}

async fn copy(text: String) {
    if let Some(win) = window() {
        if let Some(clip) = win.navigator().clipboard() {
            let _ = clip.write_text(&text).await;
        }
    }
}

#[function_component(App)]
fn app() -> Html {
    let setup = use_state(|| "".to_string());
    let punch = use_state(|| "".to_string());
    let result = use_state(|| lint("", ""));
    let output = use_state(|| "".to_string());

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
            let diff = format!("- {}\n+ {}", *setup, *punch);
            output.set(diff);
        })
    };

    let on_copy = {
        let output = output.clone();
        Callback::from(move |_| {
            let txt = (*output).clone();
            spawn_local(copy(txt));
        })
    };

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
                    <div class="diffrow">
                        <div class="cell minus">
                            <div class="line">
                                <div class="ln">{"1"}</div>
                                <div class="gutter">{"-"}</div>
                                <div>{(*setup).clone()}</div>
                            </div>
                        </div>
                        <div class="cell plus">
                            <div class="line">
                                <div class="ln">{"1"}</div>
                                <div class="gutter">{"+"}</div>
                                <div>{(*punch).clone()}</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}