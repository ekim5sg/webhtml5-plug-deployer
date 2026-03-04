use serde_json::Value;
use std::collections::BTreeMap;
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
    expect: Option<String>,
    line_no: usize,
    role: String,
}

#[derive(Debug, Clone)]
struct RuleResult {
    line_no: usize,
    raw: String,
    status: String, // PASS/FAIL/ERROR
    detail: String,
    role: String,
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

// Role header format: [EECOM] / [GUIDO] etc.
fn parse_role_header(line: &str) -> Option<String> {
    let t = line.trim();
    if t.starts_with('[') && t.ends_with(']') && t.len() >= 3 {
        let inner = t[1..t.len() - 1].trim();
        if inner.is_empty() {
            None
        } else {
            Some(inner.to_string())
        }
    } else {
        None
    }
}

fn parse_rules(text: &str) -> Vec<Rule> {
    let mut out = Vec::new();
    let mut current_role = "LD".to_string(); // default bucket

    for (i, line) in text.lines().enumerate() {
        let line_no = i + 1;
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(role) = parse_role_header(trimmed) {
            current_role = role;
            continue;
        }

        let raw = trimmed.to_string();
        let parts: Vec<&str> = raw.split_whitespace().collect();

        if parts.len() < 2 {
            out.push(Rule {
                raw,
                path: "".into(),
                op: Op::Exists,
                expect: None,
                line_no,
                role: current_role.clone(),
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
                role: current_role.clone(),
            });
            continue;
        }
        let op = op.unwrap();

        let expect = if matches!(op, Op::Exists | Op::NotExists) {
            None
        } else if parts.len() >= 3 {
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
            role: current_role.clone(),
        });
    }

    out
}

// dot paths + [index]
fn get_by_path<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = root;

    for seg in path.split('.') {
        if seg.is_empty() {
            return None;
        }

        let mut s = seg;

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

        while s.starts_with('[') {
            let close = s.find(']')?;
            let idx_str = &s[1..close];
            let idx: usize = idx_str.parse().ok()?;
            cur = cur.get(idx)?;
            s = &s[close + 1..];
        }

        if !s.is_empty() {
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
    if r.path.is_empty() {
        return RuleResult {
            line_no: r.line_no,
            raw: r.raw.clone(),
            status: "ERROR".into(),
            detail: "Rule format: <path> <op> <value?>. Example: power.bus_voltage_v >= 27.0".into(),
            role: r.role.clone(),
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
                detail: if ok { "Path exists" } else { "Path missing" }.into(),
                role: r.role.clone(),
            }
        }
        Op::NotExists => {
            let ok = actual.is_none();
            RuleResult {
                line_no: r.line_no,
                raw: r.raw.clone(),
                status: if ok { "PASS" } else { "FAIL" }.into(),
                detail: if ok { "Path not present" } else { "Path exists" }.into(),
                role: r.role.clone(),
            }
        }
        _ => {
            let Some(actual) = actual else {
                return RuleResult {
                    line_no: r.line_no,
                    raw: r.raw.clone(),
                    status: "FAIL".into(),
                    detail: "Path missing".into(),
                    role: r.role.clone(),
                };
            };

            let Some(expect_raw) = r.expect.clone() else {
                return RuleResult {
                    line_no: r.line_no,
                    raw: r.raw.clone(),
                    status: "ERROR".into(),
                    detail: "Missing expected value after operator.".into(),
                    role: r.role.clone(),
                };
            };

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
                    detail: format!("actual={a} expected={expect_clean}"),
                    role: r.role.clone(),
                };
            }

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
                    role: r.role.clone(),
                };
            }

            RuleResult {
                line_no: r.line_no,
                raw: r.raw.clone(),
                status: "ERROR".into(),
                detail: "Non-numeric compare requires numeric actual & expected, or use == / != for strings/bools.".into(),
                role: r.role.clone(),
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

fn role_board(results: &[RuleResult]) -> Vec<(String, String, String)> {
    // role -> (pass, fail, err)
    let mut m: BTreeMap<String, (usize, usize, usize)> = BTreeMap::new();

    for r in results {
        let entry = m.entry(r.role.clone()).or_insert((0, 0, 0));
        match r.status.as_str() {
            "PASS" => entry.0 += 1,
            "FAIL" => entry.1 += 1,
            _ => entry.2 += 1,
        }
    }

    // Return sorted by role name (BTreeMap order)
    let mut out = Vec::new();
    for (role, (p, f, e)) in m {
        let status = if e > 0 {
            "ERROR"
        } else if f > 0 {
            "NO-GO"
        } else {
            "GO"
        };
        let detail = format!("PASS {p} / FAIL {f} / ERR {e}");
        out.push((role, status.into(), detail));
    }
    out
}

#[function_component(App)]
fn app() -> Html {
    let telemetry = use_state(|| {
        "{\n  \"power\": {\"bus_voltage_v\": 28.2, \"bus_current_a\": 41.7},\n  \"thermal\": {\"avionics_temp_c\": 41.3},\n  \"propulsion\": {\"chamber_pressure_psi\": 295.0},\n  \"gnc\": {\"mode\": \"AUTO\"}\n}\n"
            .to_string()
    });

    let rules_text = use_state(|| {
        "# Group by role using [ROLE] headers\n\
[LD]\n\
power.bus_voltage_v >= 27.0\n\
faults not_exists\n\
\n\
[EECOM]\n\
power.bus_voltage_v >= 27.0\n\
power.bus_current_a <= 48.0\n\
thermal.avionics_temp_c < 55.0\n\
\n\
[PROP]\n\
propulsion.chamber_pressure_psi <= 310.0\n\
\n\
[GUIDO]\n\
gnc.mode == \"AUTO\"\n"
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
    let (telemetry_ok, telemetry_val, telemetry_err) = match telemetry_parsed {
        Ok(v) => (true, Some(v), String::new()),
        Err(e) => (false, None, e),
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

    let board = role_board(&results);

    html! {
      <div class="wrap">
        <header class="top">
          <div>
            <div class="h1">{ "Go No Go — Rules Evaluator" }</div>
            <div class="sub">{ "Paste telemetry JSON + grouped launch commit rules. Outputs GO/NO-GO plus a Launch Director board." }</div>
          </div>

          <div class="badgeRow">
            <div class={badge_class}>{ status }</div>
            <div class="badge">{ status_detail }</div>
            <div class="badge">{ format!("PASS: {pass_count}") }</div>
            <div class="badge">{ format!("FAIL: {fail_count}") }</div>
            <div class="badge">{ format!("ERROR: {err_count}") }</div>
          </div>
        </header>

        <section class="results" style="margin-top:0;">
          <div class="h2">{ "Launch Director Board" }</div>
          {
            if !telemetry_ok {
              html!{ <div class="muted">{ "Telemetry invalid — fix JSON to populate the board." }</div> }
            } else if board.is_empty() {
              html!{ <div class="muted">{ "No rules found. Add role headers like [EECOM] and rules beneath them." }</div> }
            } else {
              html!{
                <table class="tbl">
                  <thead>
                    <tr>
                      <th>{ "Console" }</th>
                      <th>{ "Status" }</th>
                      <th>{ "Counts" }</th>
                    </tr>
                  </thead>
                  <tbody>
                    { for board.iter().map(|(role, st, detail)| {
                      let row_class = if st == "GO" { "pass" } else if st == "NO-GO" { "fail" } else { "err" };
                      html!{
                        <tr class={row_class}>
                          <td class="mono">{ role }</td>
                          <td>{ st }</td>
                          <td class="mono">{ detail }</td>
                        </tr>
                      }
                    })}
                  </tbody>
                </table>
              }
            }
          }
        </section>

        <section class="grid" style="margin-top:12px;">
          <div class="panel">
            <div class="label">{ "Telemetry (JSON)" }</div>
            <textarea value={(*telemetry).clone()} oninput={on_telemetry} rows="18" spellcheck="false" />
            <div class="badgeRow">
              {
                if telemetry_ok {
                  html!{ <div class="badge go">{ "Telemetry: OK" }</div> }
                } else {
                  html!{ <div class="badge warn">{ telemetry_err }</div> }
                }
              }
            </div>
          </div>

          <div class="panel">
            <div class="label">{ "Rules (one per line) — group using [ROLE] headers" }</div>
            <textarea value={(*rules_text).clone()} oninput={on_rules} rows="18" spellcheck="false" />
            <div class="badgeRow">
              <div class="badge">{ "Ops: > >= < <= == != exists not_exists" }</div>
              <div class="badge">{ "Strings: use quotes \"AUTO\"" }</div>
              <div class="badge">{ "Comments: # ignored" }</div>
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
                      <th>{ "Role" }</th>
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
                          <td class="mono">{ &r.role }</td>
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
            { "Tip: Add headers like [EECOM] then rules beneath. Paths support dot notation and [index] arrays (e.g., sensors[0].temp_c)." }
          </div>
        </footer>
      </div>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}