[package]
name = "midnight-monsters"
version = "1.0.0"
edition = "2021"

[dependencies]
yew = { version = "0.21", features = ["csr"] }
gloo-timers = "0.3"
js-sys = "0.3"
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = [
    "Document",
    "Element",
    "HtmlAudioElement",
    "HtmlMediaElement",
    "Location",
    "Window"
] }

[profile.release]
lto = true
opt-level = "s"