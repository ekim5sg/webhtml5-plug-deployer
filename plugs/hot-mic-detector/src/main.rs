use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
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

    // Base heuristics
    let exclam = text.matches('!').count() as i32;
    let caps = text.chars().filter(|c| c.is_ascii_uppercase()).count() as i32;
    let len = text.chars().count() as i32;

    score += clamp(exclam * 3, 0, 15);
    score += clamp(caps / 10, 0, 10);
    if len > 220 { score += 8; }
    if len > 420 { score += 10; }

    // Phrase hits
    for h in hits_catalog() {
        if t.contains(h.phrase) {
            score += h.points;
            found.push(h);
        }
    }

    // Mild bump if it looks like a rant
    if t.contains("??") || t.contains("!!!") { score += 8; }
    if t.contains("everyone") && t.contains("always") { score += 10; }

    let final_score = clamp(score, 0, 100);
    (final_score, found)
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

    // General corporate polish swaps
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

    if out.chars().count() < 140 && !out.ends_with('.') {
        out.push('.');
    }
    if !out.to_lowercase().contains("thanks") {
        out.push_str(" Thanks!");
    }

    out
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
    let targets = vec![
        needle.to_string(),
        needle.to_uppercase(),
        capitalize_words(needle),
    ];
    for t in targets {
        out = out.replace(&t, repl);
    }
    out
}

async fn copy_to_clipboard(s: String) -> Result<(), JsValue> {
    let w = window().ok_or_else(|| JsValue::from_str("no window"))?;
    let nav = w.navigator();
    // ✅ FIX: navigator.clipboard() returns Clipboard (not Option)
    let cb = nav.clipboard();
    cb.write_text(&s).await
}

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
    // ✅ FIX: avoid HtmlElement.style() feature issues; set style attribute directly
    if let Some(doc) = window().and_then(|w| w.document()) {
        if let Some(el) = doc.get_element_by_id(id) {
            let _ = el.set_attribute("style", &format!("width:{}%", pct));
        }
    }
}

fn enable(id: &str, on: bool) {
    if let Some(doc) = window().and_then(|w| w.document()) {
        if let Some(el) = doc.get_element_by_id(id) {
            if on {
                let _ = el.remove_attribute("disabled");
            } else {
                let _ = el.set_attribute("disabled", "disabled");
            }
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

fn risk_label(score: i32) -> (&'static str, &'static str) {
    if score <= 24 {
        ("LOW RISK ✅", "risktag low")
    } else if score <= 59 {
        ("MEDIUM RISK 😬", "risktag med")
    } else {
        ("HIGH RISK 🫨", "risktag high")
    }
}

fn render_findings(score: i32, found: &[Hit]) -> String {
    let (tag, _) = risk_label(score);
    let risk_badge_class = if score <= 24 { "badge low" } else if score <= 59 { "badge med" } else { "badge high" };

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

fn muted_version(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() { return "".to_string(); }
    format!("Sorry—think I was muted for a second. Quick recap: {}", s)
}

fn corporate_translate(s: &str) -> String {
    let s = s.trim();
    if s.is_empty() { return "".to_string(); }
    format!("Quick alignment note: {} If helpful, I can draft next steps and owners.", s)
}

#[wasm_bindgen(start)]
pub fn main() {
    let w = window().expect("no window");
    let doc = w.document().expect("no document");

    let input_el: HtmlTextAreaElement = doc
        .get_element_by_id("input").unwrap()
        .dyn_into().unwrap();

    // Example chips helper
    let set_example = |text: &'static str, input_el: HtmlTextAreaElement| {
        Closure::<dyn FnMut()>::new(move || {
            input_el.set_value(text);
            run_analysis();
        })
    };

    if let Some(b) = doc.get_element_by_id("ex1") {
        let c = set_example("Real quick… I’m not saying but this is going nowhere.", input_el.clone());
        b.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }
    if let Some(b) = doc.get_element_by_id("ex2") {
        let c = set_example("Off the record… who hired this vendor? No offense, but wow.", input_el.clone());
        b.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }
    if let Some(b) = doc.get_element_by_id("ex3") {
        let c = set_example("This is a disaster. Obviously. Per my last email!!!", input_el.clone());
        b.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }
    if let Some(b) = doc.get_element_by_id("ex4") {
        let c = set_example("Let’s circle back after lunch. Between us, I hate this plan.", input_el.clone());
        b.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }

    // Analyze button
    if let Some(btn) = doc.get_element_by_id("analyze") {
        let c = Closure::<dyn FnMut()>::new(move || run_analysis());
        btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }

    // Clear button
    if let Some(btn) = doc.get_element_by_id("clear") {
        let input_for_clear = input_el.clone();
        let c = Closure::<dyn FnMut()>::new(move || {
            input_for_clear.set_value("");
            set_text("scoreBig", "—");
            set_text("riskTag", "Paste text to analyze");
            set_class("riskTag", "risktag neutral");
            set_style_width("meterFill", 0);
            set_html("findings", r#"<div class="empty-state"><div class="emoji">🫣</div><div class="empty-title">No findings yet</div><div class="empty-sub">Run an analysis to see risk triggers and suggested fixes.</div></div>"#);
            set_class("findings", "findings empty");
            set_html("rewrite", r#"<div class="empty-state"><div class="emoji">🧼</div><div class="empty-title">Awaiting corporate polish</div><div class="empty-sub">Your “rewrite” will show up here.</div></div>"#);
            set_class("rewrite", "rewrite empty");
            enable("copyRewrite", false);
            enable("corporate", false);
            enable("muted", false);
        });
        btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }

    // Copy input
    if let Some(btn) = doc.get_element_by_id("copyInput") {
        let input_for_copy = input_el.clone();
        let c = Closure::<dyn FnMut()>::new(move || {
            let s = input_for_copy.value();
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

    // Corporate translate
    if let Some(btn) = doc.get_element_by_id("corporate") {
        let c = Closure::<dyn FnMut()>::new(move || {
            if let Some(doc) = window().and_then(|w| w.document()) {
                if let Some(el) = doc.get_element_by_id("rewrite") {
                    let txt = el.text_content().unwrap_or_default();
                    let new_txt = corporate_translate(&txt);
                    set_html("rewrite", &format!(r#"<div class="kv"><div class="item"><div class="k">Corporate Translate</div><div class="v">{}</div></div></div>"#, escape_html(&new_txt)));
                    set_class("rewrite", "rewrite");
                }
            }
        });
        btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }

    // Muted version
    if let Some(btn) = doc.get_element_by_id("muted") {
        let c = Closure::<dyn FnMut()>::new(move || {
            if let Some(doc) = window().and_then(|w| w.document()) {
                if let Some(el) = doc.get_element_by_id("rewrite") {
                    let txt = el.text_content().unwrap_or_default();
                    let new_txt = muted_version(&txt);
                    set_html("rewrite", &format!(r#"<div class="kv"><div class="item"><div class="k">Muted Version</div><div class="v">{}</div></div></div>"#, escape_html(&new_txt)));
                    set_class("rewrite", "rewrite");
                }
            }
        });
        btn.add_event_listener_with_callback("click", c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }

    // ✅ FIX: don't move `input_el` into the closure used for the listener
    {
        let input_for_listener = input_el.clone();
        let c = Closure::<dyn FnMut()>::new(move || {
            if !input_for_listener.value().trim().is_empty() {
                run_analysis();
            }
        });
        input_el
            .add_event_listener_with_callback("input", c.as_ref().unchecked_ref())
            .unwrap();
        c.forget();
    }

    // Initial state
    set_html("findings", r#"<div class="empty-state"><div class="emoji">🫣</div><div class="empty-title">No findings yet</div><div class="empty-sub">Run an analysis to see risk triggers and suggested fixes.</div></div>"#);
    set_html("rewrite", r#"<div class="empty-state"><div class="emoji">🧼</div><div class="empty-title">Awaiting corporate polish</div><div class="empty-sub">Your “rewrite” will show up here.</div></div>"#);
}

fn run_analysis() {
    let w = window().expect("no window");
    let doc = w.document().expect("no document");
    let input_el: HtmlTextAreaElement = doc
        .get_element_by_id("input").unwrap()
        .dyn_into().unwrap();

    let text = input_el.value();
    let (score, found) = compute_risk(&text);

    set_text("scoreBig", &format!("{}", score));
    let (tag, cls) = risk_label(score);
    set_text("riskTag", tag);
    set_class("riskTag", cls);
    set_style_width("meterFill", score);

    let findings_html = render_findings(score, &found);
    set_html("findings", &findings_html);
    set_class("findings", "findings");

    let rewrite = rewrite_safer(&text, &found);
    if rewrite.trim().is_empty() {
        set_html("rewrite", r#"<div class="empty-state"><div class="emoji">🧼</div><div class="empty-title">Awaiting corporate polish</div><div class="empty-sub">Paste something to rewrite.</div></div>"#);
        set_class("rewrite", "rewrite empty");
        enable("copyRewrite", false);
        enable("corporate", false);
        enable("muted", false);
    } else {
        set_html(
            "rewrite",
            &format!(
                r#"<div class="kv">
                     <div class="item">
                       <div class="k">Suggested rewrite</div>
                       <div class="v">{}</div>
                     </div>
                   </div>"#,
                escape_html(&rewrite)
            ),
        );
        set_class("rewrite", "rewrite");
        enable("copyRewrite", true);
        enable("corporate", true);
        enable("muted", true);
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}