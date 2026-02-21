use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct SpecLine {
    k: &'static str,
    v: String,
}

#[function_component(App)]
fn app() -> Html {
    let title = "Plug Creator".to_string();
    let subtitle = "Built in the carpool lane ðŸš—ðŸ’¨".to_string();

    // "Carpool lane" spec snapshot area â€” you can paste what you describe here quickly,
    // then later we turn it into real UI + logic.
    let specs = vec![
        SpecLine { k: "Plug name", v: "plug-creator".to_string() },
        SpecLine { k: "Intent", v: "Describe it â†’ ship it (Rust/Yew/WASM)".to_string() },
        SpecLine { k: "Next action", v: "Tell ChatGPT what the app should do; it returns the code delta.".to_string() },
        SpecLine { k: "Deploy URL", v: format!("https://www.webhtml5.info/{}/", "plug-creator") },
    ];

    let on_primary = Callback::from(|_| {
        web_sys::window()
            .and_then(|w| w.alert_with_message("Primary action hooked âœ… (next: replace with real behavior)").ok());
    });

    let on_secondary = Callback::from(|_| {
        web_sys::window()
            .and_then(|w| w.alert_with_message("Secondary action hooked âœ…").ok());
    });

    html! {
        <>
          <div class="bg" aria-hidden="true"></div>
          <a id="top"></a>

          <div class="wrap">
            <header class="hero">
              <div class="badge">{ "MikeGyver Studio â€¢ webhtml5 plug" }</div>
              <h1 class="h1">{ title }</h1>
              <p class="sub">{ subtitle }</p>

              <nav class="nav" aria-label="Quick navigation">
                <a class="chip" href="#overview">{ "Overview" }</a>
                <a class="chip" href="#actions">{ "Actions" }</a>
                <a class="chip" href="#spec">{ "Spec" }</a>
              </nav>
            </header>

            <section id="overview" class="grid">
              <div class="card">
                <div class="card-h">
                  <h2 class="card-t">{ "Overview" }</h2>
                  <p class="card-p">{ "This starter is hard-locked dark mode, mobile-first, and designed for fast iteration while you describe features on the go." }</p>
                </div>
                <div class="card-b">
                  <div class="kv">
                    { for specs.iter().map(|s| html!{
                      <div class="k">
                        <div class="label">{ s.k }</div>
                        <div class="value">{ s.v.clone() }</div>
                      </div>
                    })}
                  </div>
                </div>
              </div>

              <div class="card">
                <div class="card-h">
                  <h2 class="card-t" id="actions">{ "Actions" }</h2>
                  <p class="card-p">{ "These are placeholders. Tell me what buttons should do; Iâ€™ll wire the logic and UI." }</p>
                </div>
                <div class="card-b">
                  <div class="row">
                    <button class="btn" onclick={on_primary}>{ "Start" }</button>
                    <button class="btn btn2" onclick={on_secondary}>{ "Settings" }</button>
                  </div>
                  <div class="footer">
                    <span>{ "Tip: Keep the spec short; ship often." }</span>
                    <span>{ "Hard-locked dark mode âœ…" }</span>
                  </div>
                </div>
              </div>
            </section>

            <section id="spec" class="card" style="margin-top:14px;">
              <div class="card-h">
                <h2 class="card-t">{ "Carpool-lane spec" }</h2>
                <p class="card-p">
                  { "Message format to send me: â€œPlug: " }{ "plug-creator" }{ " â€” Users canâ€¦, Mustâ€¦, Buttonsâ€¦, Dataâ€¦, Deploy folderâ€¦, Done.â€" }
                </p>
              </div>
              <div class="card-b">
                <div class="k">
                  <div class="label">{ "Copy/paste this into ChatGPT when youâ€™re in the lane" }</div>
                  <div class="value" style="white-space:pre-wrap;">
{format!(
"Plug: {plug}\nGoal: \nUsers can:\n- \nMust:\n- \nButtons:\n- Primary:\n- Secondary:\nData:\n- \nDeploy:\n- https://www.webhtml5.info/{plug}/\n",
plug = "plug-creator"
)}
                  </div>
                </div>
              </div>
            </section>
          </div>

          <a class="backtop" href="#top" aria-label="Back to top">{ "â†‘ Top" }</a>
        </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
