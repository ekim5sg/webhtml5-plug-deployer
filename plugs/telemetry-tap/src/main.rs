// src/main.rs
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use yew::prelude::*;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
struct ThresholdRule {
    min: Option<f64>,
    max: Option<f64>,
}
type ThresholdMap = BTreeMap<String, ThresholdRule>;

#[derive(Debug, Clone)]
struct FlatEntry {
    path: String,
    value: Value,
}

#[derive(Debug, Clone)]
struct SchemaRow {
    path: String,
    ty: String,
    sample: String,
    n_min: Option<f64>,
    n_max: Option<f64>,
    n_mean: Option<f64>,
}

#[derive(Debug, Clone)]
struct DiffRow {
    path: String,
    status: String, // ADDED/REMOVED/CHANGED/SAME
    a: String,
    b: String,
}

fn copy_to_clipboard(text: &str) {
    if let Some(w) = web_sys::window() {
        let clip = w.navigator().clipboard();
        let _ = wasm_bindgen_futures::JsFuture::from(clip.write_text(text));
    }
}

fn json_type(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn compact_value(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            if s.len() > 64 {
                format!("\"{}…\"", &s[..61])
            } else {
                format!("\"{}\"", s)
            }
        }
        Value::Array(a) => format!("[…] (len {})", a.len()),
        Value::Object(o) => format!("{{…}} ({} keys)", o.len()),
    }
}

fn value_as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn flatten_json(root: &Value) -> Vec<FlatEntry> {
    fn rec(path: &str, v: &Value, out: &mut Vec<FlatEntry>) {
        match v {
            Value::Object(map) => {
                for (k, vv) in map.iter() {
                    let p = if path.is_empty() { k.clone() } else { format!("{}.{}", path, k) };
                    rec(&p, vv, out);
                }
            }
            Value::Array(arr) => {
                out.push(FlatEntry { path: path.to_string(), value: v.clone() });
                for (i, vv) in arr.iter().enumerate() {
                    let p = format!("{}[{}]", path, i);
                    rec(&p, vv, out);
                }
            }
            _ => out.push(FlatEntry { path: path.to_string(), value: v.clone() }),
        }
    }

    let mut out = vec![];
    rec("", root, &mut out);
    out.into_iter().filter(|e| !e.path.is_empty()).collect()
}

fn schema_summary(flat: &[FlatEntry]) -> Vec<SchemaRow> {
    #[derive(Default)]
    struct Acc {
        ty_set: BTreeSet<String>,
        sample: Option<String>,
        n_sum: f64,
        n_count: usize,
        n_min: Option<f64>,
        n_max: Option<f64>,
    }

    let mut acc: BTreeMap<String, Acc> = BTreeMap::new();

    for e in flat {
        let a = acc.entry(e.path.clone()).or_default();
        a.ty_set.insert(json_type(&e.value).to_string());
        if a.sample.is_none() {
            a.sample = Some(compact_value(&e.value));
        }
        if let Some(x) = value_as_f64(&e.value) {
            a.n_sum += x;
            a.n_count += 1;
            a.n_min = Some(a.n_min.map(|m| m.min(x)).unwrap_or(x));
            a.n_max = Some(a.n_max.map(|m| m.max(x)).unwrap_or(x));
        }
    }

    acc.into_iter()
        .map(|(path, a)| {
            let ty = if a.ty_set.len() == 1 {
                a.ty_set.iter().next().unwrap().clone()
            } else {
                format!("mixed({})", a.ty_set.into_iter().collect::<Vec<_>>().join(","))
            };
            let mean = if a.n_count > 0 { Some(a.n_sum / a.n_count as f64) } else { None };

            SchemaRow {
                path,
                ty,
                sample: a.sample.unwrap_or_else(|| "—".into()),
                n_min: a.n_min,
                n_max: a.n_max,
                n_mean: mean,
            }
        })
        .collect()
}

fn parse_json(text: &str) -> Result<Value, String> {
    let t = text.trim();
    if t.is_empty() {
        return Err("Empty JSON".into());
    }
    serde_json::from_str::<Value>(t).map_err(|e| format!("{}", e))
}

fn parse_thresholds(text: &str) -> Result<ThresholdMap, String> {
    let t = text.trim();
    if t.is_empty() {
        return Ok(BTreeMap::new());
    }
    serde_json::from_str::<ThresholdMap>(t).map_err(|e| format!("{}", e))
}

fn make_map(flat: Vec<FlatEntry>) -> BTreeMap<String, Value> {
    let mut m = BTreeMap::new();
    for e in flat {
        m.insert(e.path, e.value);
    }
    m
}

fn out_of_family(path: &str, v: &Value, rules: &ThresholdMap) -> Option<String> {
    let rule = rules.get(path)?;
    let x = value_as_f64(v)?;
    if let Some(min) = rule.min {
        if x < min {
            return Some(format!("LOW (< {})", min));
        }
    }
    if let Some(max) = rule.max {
        if x > max {
            return Some(format!("HIGH (> {})", max));
        }
    }
    None
}

fn diff_flat(a: &BTreeMap<String, Value>, b: &BTreeMap<String, Value>) -> Vec<DiffRow> {
    let mut paths: BTreeSet<String> = BTreeSet::new();
    for k in a.keys() { paths.insert(k.clone()); }
    for k in b.keys() { paths.insert(k.clone()); }

    let mut out = vec![];
    for p in paths {
        match (a.get(&p), b.get(&p)) {
            (None, Some(vb)) => out.push(DiffRow { path: p, status: "ADDED".into(), a: "—".into(), b: compact_value(vb) }),
            (Some(va), None) => out.push(DiffRow { path: p, status: "REMOVED".into(), a: compact_value(va), b: "—".into() }),
            (Some(va), Some(vb)) => out.push(DiffRow {
                path: p,
                status: if va == vb { "SAME" } else { "CHANGED" }.into(),
                a: compact_value(va),
                b: compact_value(vb),
            }),
            (None, None) => {}
        }
    }
    out
}

#[function_component(App)]
fn app() -> Html {
    let telemetry_a = use_state(|| "{\n  \"power\": {\"bus_voltage_v\": 28.2},\n  \"propulsion\": {\"chamber_pressure_psi\": 295.0},\n  \"thermal\": {\"avionics_temp_c\": 41.3}\n}\n".to_string());
    let telemetry_b = use_state(|| "{\n  \"power\": {\"bus_voltage_v\": 26.4},\n  \"propulsion\": {\"chamber_pressure_psi\": 318.9},\n  \"thermal\": {\"avionics_temp_c\": 41.3}\n}\n".to_string());
    let thresholds = use_state(|| "{\n  \"power.bus_voltage_v\": {\"min\": 27.0, \"max\": 29.5},\n  \"propulsion.chamber_pressure_psi\": {\"min\": 250.0, \"max\": 310.0},\n  \"thermal.avionics_temp_c\": {\"min\": -10.0, \"max\": 55.0}\n}\n".to_string());

    let active_tab = use_state(|| "analyze".to_string());

    let a_parsed = parse_json(&telemetry_a);
    let b_parsed = parse_json(&telemetry_b);
    let rules_parsed = parse_thresholds(&thresholds);
    let rules: ThresholdMap = rules_parsed.clone().unwrap_or_default();

    let (a_map, a_schema) = match &a_parsed {
        Ok(v) => {
            let flat = flatten_json(v);
            (Some(make_map(flat.clone())), Some(schema_summary(&flat)))
        }
        Err(_) => (None, None),
    };

    let b_map = match &b_parsed {
        Ok(v) => {
            let flat = flatten_json(v);
            Some(make_map(flat))
        }
        Err(_) => None,
    };

    let anomalies: Vec<(String, String, String)> = if let (Some(map), Ok(_)) = (&a_map, &rules_parsed) {
        map.iter()
            .filter_map(|(p, v)| out_of_family(p, v, &rules).map(|flag| (p.clone(), compact_value(v), flag)))
            .collect()
    } else {
        vec![]
    };

    let diffs: Vec<DiffRow> = match (&a_map, &b_map) {
        (Some(am), Some(bm)) => diff_flat(am, bm),
        _ => vec![],
    };

    let on_a: Callback<InputEvent> = {
        let telemetry_a = telemetry_a.clone();
        Callback::from(move |e: InputEvent| {
            let Some(t) = e.target_dyn_into::<web_sys::HtmlTextAreaElement>() else { return; };
            telemetry_a.set(t.value());
        })
    };
    let on_b: Callback<InputEvent> = {
        let telemetry_b = telemetry_b.clone();
        Callback::from(move |e: InputEvent| {
            let Some(t) = e.target_dyn_into::<web_sys::HtmlTextAreaElement>() else { return; };
            telemetry_b.set(t.value());
        })
    };
    let on_thr: Callback<InputEvent> = {
        let thresholds = thresholds.clone();
        Callback::from(move |e: InputEvent| {
            let Some(t) = e.target_dyn_into::<web_sys::HtmlTextAreaElement>() else { return; };
            thresholds.set(t.value());
        })
    };

    let set_analyze = {
        let active_tab = active_tab.clone();
        Callback::from(move |_| active_tab.set("analyze".into()))
    };
    let set_diff = {
        let active_tab = active_tab.clone();
        Callback::from(move |_| active_tab.set("diff".into()))
    };

    let copy_anom = {
        let anomalies = anomalies.clone();
        Callback::from(move |_| {
            let mut lines = vec!["path,value,flag".to_string()];
            for (p, v, f) in anomalies.iter() {
                lines.push(format!("{},{},{}", p, v.replace(',', ";"), f));
            }
            copy_to_clipboard(&lines.join("\n"));
        })
    };

    let copy_diff = {
        let diffs = diffs.clone();
        Callback::from(move |_| {
            let mut lines = vec!["status,path,a,b".to_string()];
            for d in diffs.iter() {
                lines.push(format!("{},{},{},{}", d.status, d.path, d.a.replace(',', ";"), d.b.replace(',', ";")));
            }
            copy_to_clipboard(&lines.join("\n"));
        })
    };

    // Render blocks as expressions returning Html (no raw if/else tags)
    let analyze_view: Html = {
        let anom_table: Html = if anomalies.is_empty() {
            html! { <div class="muted">{ "No out-of-family values (or no matching numeric paths / thresholds)." }</div> }
        } else {
            html! {
              <table class="tbl">
                <thead>
                  <tr><th>{ "Path" }</th><th>{ "Value" }</th><th>{ "Flag" }</th></tr>
                </thead>
                <tbody>
                  { for anomalies.iter().map(|(p,v,f)| html!{
                    <tr class="warnrow">
                      <td class="mono">{ p }</td>
                      <td class="mono">{ v }</td>
                      <td>{ f }</td>
                    </tr>
                  })}
                </tbody>
              </table>
            }
        };

        let schema_table: Html = if let Some(rows) = a_schema.clone() {
            html! {
              <table class="tbl">
                <thead>
                  <tr>
                    <th>{ "Path" }</th>
                    <th>{ "Type" }</th>
                    <th>{ "Sample" }</th>
                    <th>{ "Min" }</th>
                    <th>{ "Max" }</th>
                    <th>{ "Mean" }</th>
                  </tr>
                </thead>
                <tbody>
                  { for rows.iter().map(|r| {
                      let min = r.n_min.map(|x| format!("{:.4}", x)).unwrap_or_else(|| "—".into());
                      let max = r.n_max.map(|x| format!("{:.4}", x)).unwrap_or_else(|| "—".into());
                      let mean = r.n_mean.map(|x| format!("{:.4}", x)).unwrap_or_else(|| "—".into());
                      html!{
                        <tr>
                          <td class="mono">{ &r.path }</td>
                          <td>{ &r.ty }</td>
                          <td class="mono">{ &r.sample }</td>
                          <td class="mono">{ min }</td>
                          <td class="mono">{ max }</td>
                          <td class="mono">{ mean }</td>
                        </tr>
                      }
                  })}
                </tbody>
              </table>
            }
        } else {
            html! { <div class="muted">{ "Provide valid Telemetry A JSON to see schema and stats." }</div> }
        };

        html! {
          <section class="results">
            <div class="rowhead">
              <div class="h2">{ "Out-of-family highlights (A)" }</div>
              <button class="ghost" onclick={copy_anom}>{ "Copy CSV" }</button>
            </div>
            { anom_table }

            <div class="rowhead" style="margin-top:16px;">
              <div class="h2">{ "Schema summary + key stats (A)" }</div>
              <div class="muted">{ "Numeric stats shown when value parses as number." }</div>
            </div>
            { schema_table }
          </section>
        }
    };

    let diff_view: Html = {
        let body: Html = if diffs.is_empty() {
            html! { <div class="muted">{ "Provide valid JSON in both A and B to see a diff." }</div> }
        } else {
            html! {
              <table class="tbl">
                <thead>
                  <tr><th>{ "Status" }</th><th>{ "Path" }</th><th>{ "A" }</th><th>{ "B" }</th></tr>
                </thead>
                <tbody>
                  { for diffs.iter().map(|d| {
                      let cls = match d.status.as_str() {
                          "CHANGED" => "chg",
                          "ADDED" => "add",
                          "REMOVED" => "rem",
                          _ => "",
                      };
                      html!{
                        <tr class={cls}>
                          <td>{ &d.status }</td>
                          <td class="mono">{ &d.path }</td>
                          <td class="mono">{ &d.a }</td>
                          <td class="mono">{ &d.b }</td>
                        </tr>
                      }
                  })}
                </tbody>
              </table>
            }
        };

        html! {
          <section class="results">
            <div class="rowhead">
              <div class="h2">{ "Diff: Telemetry A vs B" }</div>
              <button class="ghost" onclick={copy_diff}>{ "Copy CSV" }</button>
            </div>
            { body }
          </section>
        }
    };

    html! {
      <div class="wrap">
        <header class="top">
          <div class="title">
            <div class="h1">{ "TelemetryTap" }</div>
            <div class="sub">{ "Paste JSON telemetry → schema + stats, threshold checks, and snapshot diff." }</div>
          </div>
          <div class="tabs">
            <button class={classes!("tab", if *active_tab == "analyze" { "on" } else { "" })} onclick={set_analyze}>{ "Analyze" }</button>
            <button class={classes!("tab", if *active_tab == "diff" { "on" } else { "" })} onclick={set_diff}>{ "Diff A vs B" }</button>
          </div>
        </header>

        <section class="grid">
          <div class="panel">
            <div class="label">{ "Telemetry A (JSON)" }</div>
            <textarea value={(*telemetry_a).clone()} oninput={on_a} rows="14" spellcheck="false" />
            {
              match &a_parsed {
                Ok(_) => html!{ <div class="ok">{ "A: OK" }</div> },
                Err(e) => html!{ <div class="err">{ format!("A parse error: {}", e) }</div> },
              }
            }
          </div>

          <div class="panel">
            <div class="label">{ "Telemetry B (JSON) — for diff" }</div>
            <textarea value={(*telemetry_b).clone()} oninput={on_b} rows="14" spellcheck="false" />
            {
              match &b_parsed {
                Ok(_) => html!{ <div class="ok">{ "B: OK" }</div> },
                Err(e) => html!{ <div class="err">{ format!("B parse error: {}", e) }</div> },
              }
            }
          </div>

          <div class="panel">
            <div class="label">{ "Threshold rules (path → {min,max})" }</div>
            <textarea value={(*thresholds).clone()} oninput={on_thr} rows="14" spellcheck="false" />
            {
              match &rules_parsed {
                Ok(_) => html!{ <div class="ok">{ "Thresholds: OK" }</div> },
                Err(e) => html!{ <div class="err">{ format!("Threshold parse error: {}", e) }</div> },
              }
            }
          </div>
        </section>

        { if *active_tab == "analyze" { analyze_view } else { diff_view } }

        <footer class="foot">
          <div class="muted">
            { "Tip: Threshold keys must match exact flattened paths (e.g., power.bus_voltage_v). Arrays show [index] paths." }
          </div>
        </footer>
      </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}