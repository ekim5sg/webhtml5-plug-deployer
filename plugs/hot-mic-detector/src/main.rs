use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{window, HtmlElement, HtmlTextAreaElement};

#[derive(Clone, Debug)]
struct Hit {
    phrase: &'static str,
    points: i32,
    why: &'static str,
    suggestion: &'static str,
}

fn hits_catalog() -> Vec<Hit> {
    vec![
        Hit { phrase: "real quick", points: 10, why: "Often precedes a 12-minute monologue.", suggestion: "Quick note:" },
        Hit { phrase: "circle back", points: 14, why: "Triggers meeting recursion.", suggestion: "Follow up" },
        Hit { phrase: "off the record", points: 28, why: "If you said it… it is now on the record.", suggestion: "For context" },
        Hit { phrase: "between us", points: 18, why: "Immediately becomes between everyone.", suggestion: "In general" },
        Hit { phrase: "i'm not saying but", points: 22, why: "You are absolutely saying it.", suggestion: "One consideration is" },
        Hit { phrase: "this is a disaster", points: 26, why: "May summon the calendar invite boss-fight.", suggestion: "We have an opportunity to improve" },
        Hit { phrase: "who hired", points: 30, why: "Speedrun to HR any%.", suggestion: "I'm looking for clarity on" },
        Hit { phrase: "this is going nowhere", points: 24, why: "A morale debuff in one sentence.", suggestion: "Let's align on next steps" },
        Hit { phrase: "obviously", points: 10, why: "Not obvious to at least one person in the call.", suggestion: "To clarify" },
        Hit { phrase: "per my last email", points: 20, why: "Passive-aggressive confetti cannon.", suggestion: "Following up on my previous note" },
        Hit { phrase: "just saying", points: 12, why: "Adds spice without adding value.", suggestion: "In my view" },
        Hit { phrase: "no offense", points: 18, why: "Usually followed by offense.", suggestion: "Respectfully" },
        Hit { phrase: "it's not my job", points: 22, why: "Summons managerial side-quests.", suggestion: "Let’s clarify ownership" },
        Hit { phrase: "they don't get it", points: 18, why: "May be true. Still risky on mic.", suggestion: "There may be a gap in context" },
        Hit { phrase: "i hate", points: 16, why: "Strong emotion detected. Mic is hot.", suggestion: "I’m concerned about" },
    ]
}

fn clamp(n: i32, lo: i32, hi: i32) -> i32 {
    if n < lo { lo } else if n > hi { hi } else { n }
}

fn normalize(s: &str) -> String {
    s.to_lowercase()
        .replace('’', "'")
        .replace("  ", " ")
}

fn compute_risk(text: &str) -> (i32, Vec<Hit>) {
    let t = normalize(text);
    let mut score: i32 = 0;
    let mut found: Vec<Hit> = vec![];

    let exclam = text.matches('!').count() as i32;
    let caps = text.chars().filter(|c| c.is_ascii_uppercase()).count() as i32;
    let len = text.chars().count() as i32;

    score += clamp(exclam * 3, 0, 15);
    score += clamp(caps / 10, 0, 10);
    if len > 220 { score += 8; }
    if len > 420 { score += 10; }

    for h in hits_catalog() {
        if t.contains(h.phrase) {
            score += h.points;
            found.push(h);
        }
    }

    if t.contains("??") || t.contains("!!!") { score += 8; }
    if t.contains("everyone") && t.contains("always") { score += 10; }

    (clamp(score, 0, 100), found)
}

fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut ch = w.chars();
            match ch.next() {
                None => "".to_string(),
                Some(f) => f.to_uppercase().collect::<String>() + ch.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn replace_word_loose(text: &str, needle: &str, repl: &str) -> String {
    let mut out = text.to_string();
    let targets = vec![needle.to_string(), needle.to_uppercase(), capitalize_words(needle)];
    for t in targets {
        out = out.replace(&t, repl);
    }
    out
}

fn rewrite_safer(text: &str, found: &[Hit]) -> String {
    let mut out = text.trim().to_string();
    if out.is_empty() { return out; }

    for h in found {
        let p = h.phrase;
        let s = h.suggestion;

        let variants = vec![
            p.to_string(),
            p.to_uppercase(),
            capitalize_words(p),
            p.replace("i'm", "I'm"),
        ];

        for v in variants {
            if out.contains(&v) {
                out = out.replace(&v, s);
            }
        }
    }

    let swaps = vec![
        ("disaster", "challenge"),
        ("hate", "have concerns about"),
        ("stupid", "not ideal"),
        ("terrible", "suboptimal"),
        ("worst", "tough"),
        ("blame", "root cause"),
        ("fault", "contributing factor"),
        ("angry", "frustrated"),
        ("annoyed", "a bit blocked"),
    ];
    for (a, b) in swaps {
        out = replace_word_loose(&out, a, b);
    }

    if out.chars().count() < 140 && !out.ends_with('.') { out.push('.'); }
    if !out.to_lowercase().contains("thanks") { out.push_str(" Thanks!"); }

    out
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

async fn copy_to_clipboard(s: String) -> Result<(), JsValue> {
    let w = window().ok_or_else(|| JsValue::from_str("no window"))?;
    let cb = w.navigator().clipboard();
    let promise = cb.write_text(&s);
    JsFuture::from(promise).await.map(|_| ())
}

/* -----------------------------
   URL Share State (#t=...)
----------------------------- */

fn encode_uri(s: &str) -> String {
    js_sys::encode_uri_component(s).as_string().unwrap_or_default()
}

fn decode_uri(s: &str) -> String {
    js_sys::decode_uri_component(s).ok().and_then(|v| v.as_string()).unwrap_or_default()
}

fn set_hash_for_text(text: &str) {
    if let Some(w) = window() {
        if let Ok(loc) = w.location().set_hash(&format!("t={}", encode_uri(text))) {
            let _ = loc;
        }
    }
}

fn read_text_from_hash() -> Option<String> {
    let w = window()?;
    let hash = w.location().hash().ok()?;
    // hash is like "#t=..."
    let h = hash.trim_start_matches('#');
    for part in h.split('&') {
        if let Some(rest) = part.strip_prefix("t=") {
            return Some(decode_uri(rest));
        }
    }
    None
}

/* -----------------------------
   Tone system
----------------------------- */

#[derive(Copy, Clone)]
enum Tone {
    Standard,
    Exec,
    Polite,
    Nasa,
}

fn tone_from_select_value(v: &str) -> Tone {
    match v {
        "exec" => Tone::Exec,
        "polite" => Tone::Polite,
        "nasa" => Tone::Nasa,
        _ => Tone::Standard,
    }
}

fn apply_tone(base: &str, tone: Tone) -> String {
    let s = base.trim();
    if s.is_empty() { return "".to_string(); }

    match tone {
        Tone::Standard => s.to_string(),

        Tone::Exec => format!(
            "Executive summary:\n• Situation: {}\n• Impact: Moderate\n• Ask: Align on next steps + owner\n• Next action: I can draft a 3-point plan.",
            s
        ),

        Tone::Polite => format!(
            "Respectfully sharing a quick thought: {} If I’m missing context, happy to adjust. Thanks!",
            s
        ),

        Tone::Nasa => format!(
            "Mission Control update:\nStatus: Stable.\nObservation: {}\nRecommendation: Confirm constraints, assign owner, proceed with next step.\nCopy: Roger that.",
            s
        ),
    }
}

fn random_tone() -> Tone {
    let n = (js_sys::Math::random() * 4.0).floor() as i32;
    match n {
        1 => Tone::Exec,
        2 => Tone::Polite,
        3 => Tone::Nasa,
        _ => Tone::Standard,
    }
}

fn meeting_survival(base: &str) -> String {
    let s = base.trim();
    if s.is_empty() { return "".to_string(); }

    format!(
        "Sorry—think I was muted for a second. Quick recap: {} If helpful, I’ll send next steps + owners right after this call.",
        s
    )
}

/* -----------------------------
   DOM helpers
----------------------------- */

fn set_text(id: &str, value: &str) {
    if let Some(doc) = window().and_then(|w| w.document()) {
        if let Some(el) = doc.get_element_by_id(id) {
            if let Some(h) = el.dyn_ref::<HtmlElement>() {
                h.set_inner_text(value);
            } else {
                el.set_text_content(Some(value));
            }
        }
    }
}

fn set_class(id: &str, class_name: &str) {
    if let Some(doc) = window().and_then(|w| w.document()) {
        if let Some(el) = doc.get_element_by_id(id) {
            let _ = el.set_attribute("class", class_name);
        }
    }
}

fn set_style_width(id: &str, pct: i32) {
    if let Some(doc) = window().and_then(|w| w.document()) {
        if let Some(el) = doc.get_element_by_id(id) {
            let _ = el.set_attribute("style", &format!("width:{}%", pct));
        }
    }
}

fn enable(id: &str, on: bool) {
    if let Some(doc) = window().and_then(|w| w.document()) {
        if let Some(el) = doc.get_element_by_id(id) {
            if on { let _ = el.remove_attribute("disabled"); }
            else { let _ = el.set_attribute("disabled", "disabled"); }
        }
    }
}

fn set_html(id: &str, html: &str) {
    if let Some(doc) = window().and_then(|w| w.document()) {
        if let Some(el) = doc.get_element_by_id(id) {
            if let Some(h) = el.dyn_ref::<HtmlElement>() {
                h.set_inner_html(html);
            }
        }
    }
}

fn get_select_value(id: &str) -> String {
    // We keep this simple: read attribute "value" from the <select> element via JS property
    if let Some(doc) = window().and_then(|w| w.document()) {
        if let Some(el) = doc.get_element_by_id(id) {
            // cast to HtmlElement and read "value" via get_attribute fallback
            // (works reliably enough for this environment)
            if let Some(v) = el.get_attribute("value") {
                return v;
            }
            // Better: use js_sys::Reflect to read property
            let v = js_sys::Reflect::get(&el, &JsValue::from_str("value"))
                .ok()
                .and_then(|x| x.as_string())
                .unwrap_or_else(|| "standard".to_string());
            return v;
        }
    }
    "standard".to_string()
}

fn set_select_value(id: &str, value: &str) {
    if let Some(doc) = window().and_then(|w| w.document()) {
        if let Some(el) = doc.get_element_by_id(id) {
            let _ = js_sys::Reflect::set(&el, &JsValue::from_str("value"), &JsValue::from_str(value));
        }
    }
}

/* -----------------------------
   Render helpers
----------------------------- */

fn risk_label(score: i32) -> (&'static str, &'static str) {
    if score <= 24 { ("LOW RISK ✅", "risktag low") }
    else if score <= 59 { ("MEDIUM RISK 😬", "risktag med") }
    else { ("HIGH RISK 🫨", "risktag high") }
}

fn render_findings(score: i32, found: &[Hit]) -> String {
    let (tag, _) = risk_label(score);
    let risk_badge_class =
        if score <= 24 { "badge low" } else if score <= 59 { "badge med" } else { "badge high" };

    let mut badges = String::new();
    badges.push_str(&format!(
        r#"<div class="badges">
            <div class="{cls}">Risk: <b>{tag}</b> ({score}/100)</div>
            <div class="badge">Triggers: <b>{n}</b></div>
          </div>"#,
        cls = risk_badge_class,
        tag = tag,
        score = score,
        n = found.len()
    ));

    if found.is_empty() {
        return format!(
            r#"{badges}
            <div class="kv">
              <div class="item">
                <div class="k">Finding</div>
                <div class="v">No obvious danger phrases detected. Still… breathe before you unmute.</div>
              </div>
            </div>"#
        );
    }

    let mut items = String::new();
    for h in found {
        items.push_str(&format!(
            r#"<div class="item">
                 <div class="k">Trigger</div>
                 <div class="v">“{p}” (+{pts}) — {why}<br/><span style="color:#aab3d6">Suggested swap:</span> <b>{s}</b></div>
               </div>"#,
            p = h.phrase,
            pts = h.points,
            why = h.why,
            s = h.suggestion
        ));
    }

    format!(r#"{badges}<div class="kv">{items}</div>"#)
}

/* -----------------------------
   Analysis pipeline
----------------------------- */

fn compute_and_render() {
    let w = window().expect("no window");
    let doc = w.document().expect("no document");
    let input_el: HtmlTextAreaElement = doc.get_element_by_id("input").unwrap().dyn_into().unwrap();

    let text = input_el.value();
    let (score, found) = compute_risk(&text);

    set_text("scoreBig", &format!("{}", score));
    let (tag, cls) = risk_label(score);
    set_text("riskTag", tag);
    set_class("riskTag", cls);
    set_style_width("meterFill", score);

    // Findings
    set_html("findings", &render_findings(score, &found));
    set_class("findings", "findings");

    // Base rewrite
    let base = rewrite_safer(&text, &found);

    if base.trim().is_empty() {
        set_html("rewrite", r#"<div class="empty-state"><div class="emoji">🧼</div><div class="empty-title">Awaiting corporate polish</div><div class="empty-sub">Paste something to rewrite.</div></div>"#);
        set_class("rewrite", "rewrite empty");

        enable("copyRewrite", false);
        enable("randomTone", false);
        enable("survival", false);
        enable("shareLink", false);
        return;
    }

    // Apply selected tone
    let tone_val = get_select_value("tone");
    let toned = apply_tone(&base, tone_from_select_value(&tone_val));

    set_html(
        "rewrite",
        &format!(
            r#"<div class="kv">
                 <div class="item">
                   <div class="k">Suggested rewrite</div>
                   <div class="v">{}</div>
                 </div>
               </div>"#,
            escape_html(&toned)
        ),
    );
    set_class("rewrite", "rewrite");

    // Enable buttons
    enable("copyRewrite", true);
    enable("randomTone", true);
    enable("survival", true);
    enable("shareLink", true);

    // Update share hash (stores the original message)
    set_hash_for_text(&text);
}

fn set_empty_panels() {
    set_text("scoreBig", "—");
    set_text("riskTag", "Paste text to analyze");
    set_class("riskTag", "risktag neutral");
    set_style_width("meterFill", 0);

    set_html("findings", r#"<div class="empty-state"><div class="emoji">🫣</div><div class="empty-title">No findings yet</div><div class="empty-sub">Run an analysis to see risk triggers and suggested fixes.</div></div>"#);
    set_class("findings", "findings empty");

    set_html("rewrite", r#"<div class="empty-state"><div class="emoji">🧼</div><div class="empty-title">Awaiting corporate polish</div><div class="empty-sub">Your “rewrite” will show up here.</div></div>"#);
    set_class("rewrite", "rewrite empty");

    enable("copyRewrite", false);
    enable("randomTone", false);
    enable("survival", false);
    enable("shareLink", false);
}

/* -----------------------------
   Entrypoints
----------------------------- */

#[wasm_bindgen(start)]
pub fn start() {
    let w = window().expect("no window");
    let doc = w.document().expect("no document");

    set_empty_panels();

    let input_el: HtmlTextAreaElement = doc.get_element_by_id("input").unwrap().dyn_into().unwrap();

    // Load from URL hash if present
    if let Some(t) = read_text_from_hash() {
        if !t.trim().is_empty() {
            input_el.set_value(&t);
            compute_and_render();
        }
    }

    // Example chips
    let bind_example = |btn_id: &str, text: &'static str| {
        if let Some(b) = doc.get_element_by_id(btn_id) {
            let input = input_el.clone();
            let c = Closure::<dyn FnMut()>::new(move || {
                input.set_value(text);
                compute_and_render();
            });
            b.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
            c.forget();
        }
    };
    bind_example("ex1", "Real quick… I’m not saying but this is going nowhere.");
    bind_example("ex2", "Off the record… who hired this vendor? No offense, but wow.");
    bind_example("ex3", "This is a disaster. Obviously. Per my last email!!!");
    bind_example("ex4", "Let’s circle back after lunch. Between us, I hate this plan.");

    // Analyze
    if let Some(btn) = doc.get_element_by_id("analyze") {
        let c = Closure::<dyn FnMut()>::new(move || compute_and_render());
        btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }

    // Clear
    if let Some(btn) = doc.get_element_by_id("clear") {
        let input = input_el.clone();
        let c = Closure::<dyn FnMut()>::new(move || {
            input.set_value("");
            set_hash_for_text("");
            set_empty_panels();
        });
        btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }

    // Auto analyze on input
    {
        let input_for_listener = input_el.clone();
        let c = Closure::<dyn FnMut()>::new(move || {
            if !input_for_listener.value().trim().is_empty() {
                compute_and_render();
            } else {
                set_hash_for_text("");
                set_empty_panels();
            }
        });
        input_el
            .add_event_listener_with_callback("input", c.as_ref().unchecked_ref())
            .unwrap();
        c.forget();
    }

    // Tone change -> recompute to apply tone to current rewrite
    if let Some(tone_el) = doc.get_element_by_id("tone") {
        let c = Closure::<dyn FnMut()>::new(move || compute_and_render());
        tone_el.add_event_listener_with_callback("change", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }

    // Random tone
    if let Some(btn) = doc.get_element_by_id("randomTone") {
        let c = Closure::<dyn FnMut()>::new(move || {
            let t = random_tone();
            let v = match t {
                Tone::Exec => "exec",
                Tone::Polite => "polite",
                Tone::Nasa => "nasa",
                Tone::Standard => "standard",
            };
            set_select_value("tone", v);
            compute_and_render();
        });
        btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }

    // Copy input
    if let Some(btn) = doc.get_element_by_id("copyInput") {
        let input = input_el.clone();
        let c = Closure::<dyn FnMut()>::new(move || {
            let s = input.value();
            spawn_local(async move { let _ = copy_to_clipboard(s).await; });
        });
        btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }

    // Copy rewrite
    if let Some(btn) = doc.get_element_by_id("copyRewrite") {
        let c = Closure::<dyn FnMut()>::new(move || {
            if let Some(doc) = window().and_then(|w| w.document()) {
                if let Some(el) = doc.get_element_by_id("rewrite") {
                    let txt = el.text_content().unwrap_or_default();
                    spawn_local(async move { let _ = copy_to_clipboard(txt).await; });
                }
            }
        });
        btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }

    // Copy share link
    if let Some(btn) = doc.get_element_by_id("shareLink") {
        let c = Closure::<dyn FnMut()>::new(move || {
            if let Some(w) = window() {
                if let Ok(href) = w.location().href() {
                    spawn_local(async move { let _ = copy_to_clipboard(href).await; });
                }
            }
        });
        btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }

    // Meeting survival
    if let Some(btn) = doc.get_element_by_id("survival") {
        let c = Closure::<dyn FnMut()>::new(move || {
            if let Some(doc) = window().and_then(|w| w.document()) {
                let input: HtmlTextAreaElement = doc.get_element_by_id("input").unwrap().dyn_into().unwrap();
                let (score, found) = compute_risk(&input.value());
                let base = rewrite_safer(&input.value(), &found);
                if base.trim().is_empty() {
                    return;
                }
                let survival = meeting_survival(&apply_tone(&base, tone_from_select_value(&get_select_value("tone"))));
                set_html(
                    "rewrite",
                    &format!(
                        r#"<div class="kv">
                             <div class="item">
                               <div class="k">Meeting survival script</div>
                               <div class="v">{}</div>
                             </div>
                           </div>"#,
                        escape_html(&survival)
                    ),
                );
                set_class("rewrite", "rewrite");
                enable("copyRewrite", true);

                // keep score UI updated (so the page doesn't feel like it "lost state")
                set_text("scoreBig", &format!("{}", score));
            }
        });
        btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }
}

// ✅ Bin crates still want a Rust main. wasm-bindgen will call `start()` for the WASM init.
fn main() {}