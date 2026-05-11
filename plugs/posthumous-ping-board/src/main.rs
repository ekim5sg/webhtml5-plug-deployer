use js_sys::Math;
use wasm_bindgen::JsCast;
use web_sys::HtmlAudioElement;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
enum Track {
    Spooky,
    FunnyLyrics,
}

#[function_component(App)]
fn app() -> Html {
    let message = use_state(|| "Ask the board a question… then click a letter. The afterlife has excellent customer service.".to_string());
    let spelled = use_state(|| "".to_string());
    let audio_started = use_state(|| false);
    let audio_paused = use_state(|| true);
    let active_track = use_state(|| Track::Spooky);

    let spooky_ref = use_node_ref();
    let funny_ref = use_node_ref();

    let responses = vec![
        "PLEASE HOLD… YOUR HAUNT IS VERY IMPORTANT TO US.",
        "I WOULD TEXT BACK BUT MY PLAN EXPIRED IN 1997.",
        "YES. ALSO CHECK THE GARAGE.",
        "NO. BUT I ADMIRE THE CONFIDENCE.",
        "THE WIFI IS WEAK OVER HERE.",
        "TELL EVERYONE I LEFT THE GOOD SNACKS HIDDEN.",
        "I CAN HEAR YOU… BUT ONLY WHEN YOU OPEN A BAG OF CHIPS.",
        "ASK AGAIN AFTER COFFEE.",
        "THE PASSWORD IS STILL NOT ‘PASSWORD123’.",
        "I TRIED TO CALL BUT IT WENT STRAIGHT TO SPIRIT MAIL.",
        "LET’S STAY IN TOUCH… BUT MAYBE NOT DAILY.",
        "THE AFTERLIFE HAS NO MEETINGS. IT IS GLORIOUS.",
        "PLEASE STOP TOUCHING THE PLANCHETTE WITH CHEETO FINGERS.",
        "I HAVE BEEN TRYING TO REACH YOU ABOUT YOUR EXTENDED WARRANTY.",
    ];

    let letters = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect::<Vec<char>>();

    let pick_response = {
        let message = message.clone();
        Callback::from(move |_| {
            let index = (Math::random() * responses.len() as f64).floor() as usize;
            message.set(responses[index].to_string());
        })
    };

    let clear_board = {
        let spelled = spelled.clone();
        let message = message.clone();
        Callback::from(move |_| {
            spelled.set("".to_string());
            message.set("Board cleared. The spirits are pretending they were never here.".to_string());
        })
    };

    let start_or_resume_audio = {
        let spooky_ref = spooky_ref.clone();
        let funny_ref = funny_ref.clone();
        let audio_started = audio_started.clone();
        let audio_paused = audio_paused.clone();
        let active_track = active_track.clone();

        Callback::from(move |_| {
            let spooky = spooky_ref.cast::<HtmlAudioElement>();
            let funny = funny_ref.cast::<HtmlAudioElement>();

            if let (Some(spooky), Some(funny)) = (spooky, funny) {
                spooky.set_loop(true);
                funny.set_loop(true);
                spooky.set_volume(0.45);
                funny.set_volume(0.35);

                let _ = spooky.pause();
                let _ = funny.pause();

                match *active_track {
                    Track::Spooky => {
                        let _ = spooky.play();
                    }
                    Track::FunnyLyrics => {
                        let _ = funny.play();
                    }
                }

                audio_started.set(true);
                audio_paused.set(false);
            }
        })
    };

    let pause_audio = {
        let spooky_ref = spooky_ref.clone();
        let funny_ref = funny_ref.clone();
        let audio_paused = audio_paused.clone();

        Callback::from(move |_| {
            if let Some(spooky) = spooky_ref.cast::<HtmlAudioElement>() {
                let _ = spooky.pause();
            }
            if let Some(funny) = funny_ref.cast::<HtmlAudioElement>() {
                let _ = funny.pause();
            }
            audio_paused.set(true);
        })
    };

    let switch_track = {
        let spooky_ref = spooky_ref.clone();
        let funny_ref = funny_ref.clone();
        let active_track = active_track.clone();
        let audio_started = audio_started.clone();
        let audio_paused = audio_paused.clone();

        Callback::from(move |_| {
            let next_track = match *active_track {
                Track::Spooky => Track::FunnyLyrics,
                Track::FunnyLyrics => Track::Spooky,
            };

            active_track.set(next_track.clone());

            if *audio_started && !*audio_paused {
                let spooky = spooky_ref.cast::<HtmlAudioElement>();
                let funny = funny_ref.cast::<HtmlAudioElement>();

                if let (Some(spooky), Some(funny)) = (spooky, funny) {
                    let _ = spooky.pause();
                    let _ = funny.pause();

                    match next_track {
                        Track::Spooky => {
                            let _ = spooky.play();
                        }
                        Track::FunnyLyrics => {
                            let _ = funny.play();
                        }
                    }
                }
            }
        })
    };

    let current_track_label = match *active_track {
        Track::Spooky => "Spooky Instrumental",
        Track::FunnyLyrics => "Funny Lyrics Loop",
    };

    html! {
        <main class="app">
            <audio ref={spooky_ref} src="spooky-loop.mp3" preload="auto"></audio>
            <audio ref={funny_ref} src="funny-lyrics-loop.mp3" preload="auto"></audio>

            <section class="hero">
                <div class="studio-pill">{"MIKEGYVER STUDIO PRESENTS"}</div>
                <h1>{"Posthumous Ping Board"}</h1>
                <p class="subtitle">
                    {"The world’s least reliable online spirit board for people who said, “Let’s stay in touch!” a little too literally."}
                </p>

                <div class="audio-panel">
                    <p>{"iPhone-safe audio: tap Start Music once, then pause anytime."}</p>
                    <div class="audio-buttons">
                        <button onclick={start_or_resume_audio.clone()}>
                            { if *audio_started && *audio_paused { "Resume Music" } else { "Start Music" } }
                        </button>
                        <button onclick={pause_audio}>{"Pause"}</button>
                        <button onclick={switch_track}>{"Switch Track"}</button>
                    </div>
                    <small>{format!("Current track: {}", current_track_label)}</small>
                </div>
            </section>

            <section class="board-wrap">
                <div class="board">
                    <div class="board-top">
                        <span>{"YES"}</span>
                        <span>{"MAYBE AFTER COFFEE"}</span>
                        <span>{"NO"}</span>
                    </div>

                    <div class="message-box">
                        <div class="ghost-icon">{"☾"}</div>
                        <p>{(*message).clone()}</p>
                    </div>

                    <div class="planchette">
                        <div class="planchette-hole"></div>
                        <div class="planchette-moon">{"☽"}</div>
                    </div>

                    <div class="letters">
                        { for letters.iter().map(|letter| {
                            let spelled = spelled.clone();
                            let message = message.clone();
                            let ch = *letter;

                            html! {
                                <button
                                    class="letter"
                                    onclick={Callback::from(move |_| {
                                        let mut next = (*spelled).clone();
                                        next.push(ch);
                                        spelled.set(next);

                                        let tiny_messages = [
                                            "The planchette wiggles suspiciously.",
                                            "A draft enters the room. Or maybe the AC kicked on.",
                                            "The board says this is billable.",
                                            "The spirit pauses dramatically.",
                                            "Somewhere, a printer jams.",
                                        ];

                                        let index = (Math::random() * tiny_messages.len() as f64).floor() as usize;
                                        message.set(tiny_messages[index].to_string());
                                    })}
                                >
                                    {ch}
                                </button>
                            }
                        })}
                    </div>

                    <div class="spelled">
                        <span>{"Current transmission:"}</span>
                        <strong>{ if spelled.is_empty() { "—".to_string() } else { (*spelled).clone() } }</strong>
                    </div>

                    <div class="actions">
                        <button onclick={pick_response}>{"Ask the Beyond"}</button>
                        <button onclick={clear_board}>{"Clear Board"}</button>
                    </div>
                </div>
            </section>

            <section class="footer-card">
                <h2>{"Important Disclaimer"}</h2>
                <p>
                    {"This board does not actually contact the dead, ghosts, ancestors, extended warranty departments, or anyone who left you on read."}
                </p>
                <p class="tagline">
                    {"From the MikeGyver Studio… where stories don’t just live on pages… they come alive in your imagination."}
                </p>
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}