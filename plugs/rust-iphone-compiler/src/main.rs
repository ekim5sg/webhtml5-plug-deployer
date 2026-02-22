use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
        <main class="wrap">
          <section class="card">
            <div class="card-h">
              <div class="badge">{ "webhtml5 plug scaffold" }</div>
              <h1 class="h1">{ "Rust iPhone Compiler" }</h1>
              <p class="sub">{ "Plug scaffold is live. Replace this content with your real app." }</p>
            </div>
            <div class="card-b">
              <p class="sub">{ "https://www.webhtml5.info/rust-iphone-compiler/" }</p>
            </div>
          </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
