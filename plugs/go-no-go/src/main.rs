use serde_json::Value;
use yew::prelude::*;

#[derive(Debug, Clone, PartialEq)]
enum Op {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
    Ne,
    Exists,
    NotExists,
}

#[derive(Debug, Clone)]
struct Rule {
    raw: String,
    path: String,
    op: Op,
    expect: Option<String>, // raw expected token (number or string)
    line_no: usize,
}

#[derive(Debug, Clone)]
struct RuleResult {
    line_no: usize,
    raw: String,
    status: String, // PASS/FAIL/ERROR
    detail: String,
}

fn parse_json(text: &str) -> Result<Value, String> {
    let t = text.trim();
    if t.is_empty() {
        return Err("Telemetry JSON is empty.".into());
    }
    serde_json::from_str::<Value>(t).map_err(|e| format!("Telemetry JSON parse error: {e}"))
}

fn parse_op(s: &str) -> Option<Op> {
    match s {
        ">" => Some(Op::Gt),
        ">=" => Some(Op::Ge),
        "<" => Some(Op::Lt),
        "<=" => Some(Op::Le),
        "==" => Some(Op::Eq),
        "!=" => Some(Op::Ne),
        "exists" => Some(Op::Exists),
        "not_exists" => Some(Op::NotExists),
        _ => None,
    }
}

fn strip_quotes(s: &str) -> String {
    let t = s.trim();
    if (t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')) {
        t[1..t.len() - 1].to_string()
    } else {
        t.to_string()
    }
}

fn parse_rules(text: &str) -> Vec<Rule> {
    let mut out = Vec::new();

    for (i, line) in text.lines().enumerate() {
        let line_no = i + 1;
        let raw = line.trim().to_string();

        if raw.is_empty() || raw.starts_with('#') {
            continue;
        }

        // Split into: path op value?
        let parts: Vec<&str> = raw.split_whitespace().collect();
        if parts.len() < 2 {
            out.push(Rule {
                raw,
                path: "".into(),
                op: Op::Exists,
                expect: None,
                line_no,
            });
            continue;
        }

        let path = parts[0].to_string();
        let op_s = parts[1];
        let op = parse_op(op_s);

        if op.is_none() {
            out.push(Rule {
                raw,
                path,
                op: Op::Exists,
                expect: None,
                line_no,
            });
            continue;
        }
        let op = op.unwrap();

        let expect = if matches!(op, Op::Exists | Op::NotExists) {
            None
        } else if parts.len() >= 3 {
            // allow spaces in string by joining remainder
            Some(parts[2..].join(" "))
        } else {
            None
        };

        out.push(Rule {
            raw,
            path,
            op,
            expect,
            line_no,
        });
    }

    out
}

// Supports dot paths and optional [index] segments, e.g.
// power.bus_voltage_v
// sensors[0].temp_c
fn get_by_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = root;

    for seg in path.split('.') {
        if seg.is_empty() {
            return None;
        }

        // Handle possible [index] suffixes in the same segment
        let mut s = seg;

        // First, object key before any '['
        if let Some(bracket_pos) = s.find('[') {
            let key = &s[..bracket_pos];
            if !key.is_empty() {
                cur = cur.get(key)?;
            }
            s = &s[bracket_pos..];
        } else {
            cur = cur.get(s)?;
            continue;
        }

        // Now consume one or more [index]
        while s.starts_with('[') {
            let close = s.find(']')?;
            let idx_str = &s[1..close];
            let idx: usize = idx_str.parse().ok()?;
            cur = cur.get(idx)?;
            s = &s[close + 1..];
        }

        if !s.is_empty() {
            // leftover characters = invalid
            return None;
        }
    }

    Some(cur)
}

fn val_as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}

fn val_as_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::Null => Some("null".into()),
        _ => None,
    }
}

fn eval_rule(root: &Value, r: &Rule) -> RuleResult {
    // Validate parse errors we encoded as "best-effort" rules
    if r.path.is_empty() {
        return RuleResult {
            line_no: r.line_no,
            raw: r.raw.clone(),
            status: "ERROR".into(),
            detail: "Rule format: <path> <op> <value?>. Example: power.bus_voltage_v >= 27.0".into(),
        };
    }

    let actual = get_by_path(root, &r.path);

    match r.op {
        Op::Exists => {
            let ok = actual.is_some();
            RuleResult {
                line_no: r.line_no,
                raw: r.raw.clone(),
                status: if ok { "PASS" } else { "FAIL" }.into(),
                detail: if ok {
                    "Path exists".into()
                } else {
                    "Path missing".into()
                },
            }
        }
        Op::NotExists => {
            let ok = actual.is_none();
            RuleResult {
                line_no: r.line_no,
                raw: r.raw.clone(),
                status: if ok { "PASS" } else { "FAIL" }.into(),
                detail: if ok {
                    "Path not present".into()
                } else {
                    "Path exists".into()
                },
            }
        }
        _ => {
            let Some(actual) = actual else {
                return RuleResult {
                    line_no: r.line_no,
                    raw: r.raw.clone(),
                    status: "FAIL".into(),
                    detail: "Path missing".into(),
                };
            };

            let Some(expect_raw) = r.expect.clone() else {
                return RuleResult {
                    line_no: r.line_no,
                    raw: r.raw.clone(),
                    status: "ERROR".into(),
                    detail: "Missing expected value after operator.".into(),
                };
            };

            // Try numeric compare first
            let expect_clean = strip_quotes(&expect_raw);
            let expect_num = expect_clean.parse::<f64>().ok();
            let actual_num = val_as_f64(actual);

            if let (Some(a), Some(b)) = (actual_num, expect_num) {
                let ok = match r.op {
                    Op::Gt => a > b,
                    Op::Ge => a >= b,
                    Op::Lt => a < b,
                    Op::Le => a <= b,
                    Op::Eq => (a - b).abs() < 1e-12,
                    Op::Ne => (a - b).abs() >= 1e-12,
                    _ => false,
                };

                return RuleResult {
                    line_no: r.line_no,
                    raw: r.raw.clone(),
                    status: if ok { "PASS" } else { "FAIL" }.into(),
                    detail: format!("actual={a} expected={}", expect_clean),
                };
            }

            // Fallback to string compare for == / !=
            if matches!(r.op, Op::Eq | Op::Ne) {
                let a = val_as_string(actual).unwrap_or_else(|| "<non-scalar>".into());
                let b = expect_clean;
                let ok = match r.op {
                    Op::Eq => a == b,
                    Op::Ne => a != b,
                    _ => false,
                };
                return RuleResult {
                    line_no: r.line_no,
                    raw: r.raw.clone(),
                    status: if ok { "PASS" } else { "FAIL" }.into(),
                    detail: format!("actual={a} expected={b}"),
                };
            }

            RuleResult {
                line_no: r.line_no,
                raw: r.raw.clone(),
                status: "ERROR".into(),
                detail: "Non-numeric compare requires numeric actual & expected, or use == / != for strings/bools.".into(),
            }
        }
    }
}

fn overall_status(results: &[RuleResult], telemetry_ok: bool) -> (String, String) {
    if !telemetry_ok {
        return ("NO-GO".into(), "Telemetry JSON invalid.".into());
    }
    let errors = results.iter().any(|r| r.status == "ERROR");
    let fails = results.iter().any(|r| r.status == "FAIL");

    if errors {
        ("NO-GO".into(), "One or more rules have errors.".into())
    } else if fails {
        ("NO-GO".into(), "One or more rules failed.".into())
    } else {
        ("GO".into(), "All rules passed.".into())
    }
}

#[function_component(App)]
fn app() -> Html {
    let telemetry = use_state(|| {
        "{\n  \"power\": {\"bus_voltage_v\": 28.2, \"bus_current_a\": 41.7},\n  \"thermal\": {\"avionics_temp_c\": 41.3},\n  \"propulsion\": {\"chamber_pressure_psi\": 295.0},\n  \"gnc\": {\"mode\": \"AUTO\"}\n}\n"
            .to_string()
    });

    let rules_text = use_state(|| {
        "# One rule per line: <path> <op> <value?>\n\
power.bus_voltage_v >= 27.0\n\
power.bus_current_a <= 48.0\n\
thermal.avionics_temp_c < 55.0\n\
propulsion.chamber_pressure_psi <= 310.0\n\
gnc.mode == \"AUTO\"\n\
# Existence checks\n\
power.bus_voltage_v exists\n\
faults not_exists\n"
        .to_string()
    });

    let on_telemetry: Callback<InputEvent> = {
        let telemetry = telemetry.clone();
        Callback::from(move |e: InputEvent| {
            let Some(t) = e.target_dyn_into::<web_sys::HtmlTextAreaElement>() else { return; };
            telemetry.set(t.value());
        })
    };

    let on_rules: Callback<InputEvent> = {
        let rules_text = rules_text.clone();
        Callback::from(move |e: InputEvent| {
            let Some(t) = e.target_dyn_into::<web_sys::HtmlTextAreaElement>() else { return; };
            rules_text.set(t.value());
        })
    };

    let telemetry_parsed = parse_json(&telemetry);
    let (telemetry_ok, telemetry_val) = match telemetry_parsed {
        Ok(v) => (true, Some(v)),
        Err(_) => (false, None),
    };

    let rules = parse_rules(&rules_text);

    let results: Vec<RuleResult> = if let Some(v) = telemetry_val.as_ref() {
        rules.iter().map(|r| eval_rule(v, r)).collect()
    } else {
        Vec::new()
    };

    let (status, status_detail) = overall_status(&results, telemetry_ok);

    let pass_count = results.iter().filter(|r| r.status == "PASS").count();
    let fail_count = results.iter().filter(|r| r.status == "FAIL").count();
    let err_count = results.iter().filter(|r| r.status == "ERROR").count();

    let badge_class = if status == "GO" { "badge go" } else { "badge nogo" };

    html! {
      <div class="wrap">
        <header class="top">
          <div>
            <div class="h1">{ "Go No Go — Rules Evaluator" }</div>
            <div class="sub">
              { "Paste telemetry JSON + launch commit rules. Evaluates PASS/FAIL/ERROR and outputs GO/NO-GO." }
            </div>
          </div>

          <div class="badgeRow">
            <div class={badge_class}>{ status }</div>
            <div class="badge">{ status_detail }</div>
            <div class="badge">{ format!("PASS: {pass_count}") }</div>
            <div class="badge">{ format!("FAIL: {fail_count}") }</div>
            <div class="badge">{ format!("ERROR: {err_count}") }</div>
          </div>
        </header>

        <section class="grid">
          <div class="panel">
            <div class="label">{ "Telemetry (JSON)" }</div>
            <textarea value={(*telemetry).clone()} oninput={on_telemetry} rows="18" spellcheck="false" />
            {
              if telemetry_ok {
                html!{ <div class="badgeRow"><div class="badge go">{ "Telemetry: OK" }</div></div> }
              } else {
                let msg = match parse_json(&telemetry) {
                    Ok(_) => "Telemetry: OK".to_string(),
                    Err(e) => e,
                };
                html!{ <div class="badgeRow"><div class="badge warn">{ msg }</div></div> }
              }
            }
          </div>

          <div class="panel">
            <div class="label">{ "Rules (one per line)" }</div>
            <textarea value={(*rules_text).clone()} oninput={on_rules} rows="18" spellcheck="false" />
            <div class="badgeRow">
              <div class="badge">{ "Ops: > >= < <= == != exists not_exists" }</div>
              <div class="badge">{ "Strings: use quotes \"AUTO\"" }</div>
              <div class="badge">{ "Comments: lines starting with # are ignored" }</div>
            </div>
          </div>
        </section>

        <section class="results">
          <div class="h2">{ "Rule Results" }</div>
          {
            if !telemetry_ok {
              html!{ <div class="muted">{ "Fix telemetry JSON to evaluate rules." }</div> }
            } else if results.is_empty() {
              html!{ <div class="muted">{ "Add at least one rule." }</div> }
            } else {
              html!{
                <table class="tbl">
                  <thead>
                    <tr>
                      <th>{ "Status" }</th>
                      <th>{ "Line" }</th>
                      <th>{ "Rule" }</th>
                      <th>{ "Detail" }</th>
                    </tr>
                  </thead>
                  <tbody>
                    { for results.iter().map(|r| {
                      let row_class = if r.status == "PASS" { "pass" } else if r.status == "FAIL" { "fail" } else { "err" };
                      html!{
                        <tr class={row_class}>
                          <td>{ &r.status }</td>
                          <td class="mono">{ r.line_no }</td>
                          <td class="mono">{ &r.raw }</td>
                          <td class="mono">{ &r.detail }</td>
                        </tr>
                      }
                    })}
                  </tbody>
                </table>
              }
            }
          }
        </section>

        <footer class="foot">
          <div class="muted">
            { "Tip: Paths support dot notation and [index] arrays (e.g., sensors[0].temp_c). Missing paths FAIL unless you use not_exists." }
          </div>
        </footer>
      </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}