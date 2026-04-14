use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    html! {
      <>
        <main style="font-family:system-ui; padding:24px;">
          <h1>"Rust Again in 20 Minutes App"</h1>
          <p style="color:#667;">{"Plug scaffold is live. Replace this content with your real app."}</p>
          <p>"https://www.webhtml5.info/rust-again-in-20-minutes-app/"</p>
        </main>
      </>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
