use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
      <>
        <div class="bg" aria-hidden="true"></div>
        <main>
          <div class="card">
            <h1>{ "Rust iPhone Compiler Demo" }</h1>
            <p>{ "Plug scaffold is live. Replace this content with your real app." }</p>
            <p style="margin-top:10px;">
              <a href="https://www.webhtml5.info/rust-iphone-compiler-demo/" target="_blank">
                { "Open deployed URL" }
              </a>
            </p>
          </div>
        </main>
      </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}