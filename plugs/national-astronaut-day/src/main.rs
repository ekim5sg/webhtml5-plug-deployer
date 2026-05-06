use web_sys::HtmlAudioElement;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct Role {
    name: &'static str,
    call_sign: &'static str,
    focus: &'static str,
}

fn play_song(audio_ref: &NodeRef) -> bool {
    if let Some(audio) = audio_ref.cast::<HtmlAudioElement>() {
        audio.set_loop(true);
        audio.set_volume(0.55);

        let _ = audio.play();
        return true;
    }

    false
}

#[function_component(App)]
fn app() -> Html {
    let started = use_state(|| false);
    let selected_role = use_state(|| 0usize);
    let mission_step = use_state(|| 0usize);
    let music_started = use_state(|| false);
    let audio_ref = use_node_ref();

    {
        let audio_ref = audio_ref.clone();
        let music_started = music_started.clone();

        use_effect_with((), move |_| {
            if play_song(&audio_ref) {
                music_started.set(true);
            }

            || ()
        });
    }

    let roles = vec![
        Role {
            name: "Commander",
            call_sign: "Astra One",
            focus: "Lead the crew through launch and orbital decisions.",
        },
        Role {
            name: "Pilot",
            call_sign: "Vector Two",
            focus: "Guide the spacecraft through alignment and approach.",
        },
        Role {
            name: "Flight Engineer",
            call_sign: "Orbit Three",
            focus: "Watch the systems, fuel, power, and mission health.",
        },
        Role {
            name: "Mission Specialist",
            call_sign: "Nova Four",
            focus: "Solve the science challenge and complete the mission.",
        },
    ];

    let steps = vec![
        "Complete the launch countdown and confirm all systems are green.",
        "Enter low Earth orbit and stabilize the spacecraft attitude.",
        "Run the lunar approach burn with steady fuel control.",
        "Recover the deep-space signal and transmit mission success.",
    ];

    let role = roles[*selected_role].clone();
    let step_text = steps[*mission_step];
    let progress = ((*mission_step + 1) * 25).to_string();

    let start_music = {
        let audio_ref = audio_ref.clone();
        let music_started = music_started.clone();

        Callback::from(move |_| {
            if play_song(&audio_ref) {
                music_started.set(true);
            }
        })
    };

    let start_mission = {
        let started = started.clone();
        let audio_ref = audio_ref.clone();
        let music_started = music_started.clone();

        Callback::from(move |_| {
            if play_song(&audio_ref) {
                music_started.set(true);
            }

            started.set(true);
        })
    };

    let reset_mission = {
        let started = started.clone();
        let mission_step = mission_step.clone();

        Callback::from(move |_| {
            mission_step.set(0);
            started.set(false);
        })
    };

    let next_step = {
        let mission_step = mission_step.clone();

        Callback::from(move |_| {
            let next = if *mission_step >= 3 { 0 } else { *mission_step + 1 };
            mission_step.set(next);
        })
    };

    html! {
        <main class="app">
            <audio
                ref={audio_ref.clone()}
                src="./assets/audio/starlight-runaway.mp3"
                preload="auto"
                loop=true
            />

            <div class="stars"></div>

            <section class="shell">
                if !*started {
                    <div class="hero">
                        <div>
                            <div class="badge">{"🚀 May 5 • National Astronaut Day"}</div>
                            <h1>{"Astronaut For A Day"}</h1>
                            <p class="subtitle">
                                {"A cinematic Rust/Yew WASM mission experience celebrating the dreamers, builders, pilots, engineers, and explorers who help humanity reach beyond the stars."}
                            </p>
                            <p class="studio">{"MikeGyver Studio"}</p>

                            <div class="actions">
                                <button class="primary" onclick={start_mission.clone()}>
                                    {"Begin Mission"}
                                </button>

                                <button class="secondary" onclick={start_music.clone()}>
                                    {"Start Music"}
                                </button>

                                <a class="secondary" href="national-astronaut-day.zip">
                                    {"Download Source Zip"}
                                </a>
                            </div>

                            <p class="music-status">
                                {
                                    if *music_started {
                                        "🎵 Starlight Runaway is playing."
                                    } else {
                                        "🎵 Music will autoplay when allowed, or tap Start Music."
                                    }
                                }
                            </p>
                        </div>
                    </div>
                } else {
                    <div class="panel-grid">
                        <section class="card">
                            <div class="badge">{"Mission Control Online"}</div>
                            <h2>{"Choose Your Astronaut Role"}</h2>

                            <div class="role-grid">
                                {
                                    roles.iter().enumerate().map(|(index, item)| {
                                        let selected_role = selected_role.clone();
                                        let active = *selected_role == index;

                                        html! {
                                            <button
                                                class={classes!("role-button", active.then_some("active"))}
                                                onclick={Callback::from(move |_| selected_role.set(index))}
                                            >
                                                <strong>{item.name}</strong>
                                                <br />
                                                <span>{item.call_sign}</span>
                                            </button>
                                        }
                                    }).collect::<Html>()
                                }
                            </div>

                            <div class="telemetry">
                                <div class="row">
                                    <span>{"Selected Role"}</span>
                                    <strong>{role.name}</strong>
                                </div>
                                <div class="row">
                                    <span>{"Call Sign"}</span>
                                    <strong>{role.call_sign}</strong>
                                </div>
                                <div class="row">
                                    <span>{"Mission Focus"}</span>
                                    <strong>{role.focus}</strong>
                                </div>
                            </div>

                            <div class="actions">
                                <button class="secondary" onclick={start_music.clone()}>
                                    {"Start Music"}
                                </button>
                            </div>

                            <p class="music-status">
                                {
                                    if *music_started {
                                        "🎵 Starlight Runaway is playing."
                                    } else {
                                        "🎵 Tap Start Music if your browser blocked autoplay."
                                    }
                                }
                            </p>
                        </section>

                        <section class="card">
                            <h2>{"Astronaut Challenge"}</h2>

                            <div class="mission-screen">
                                <div class="orbit">
                                    <div class="craft">{"🛰️"}</div>
                                </div>
                                <div class="earth"></div>
                            </div>

                            <p class="challenge">{step_text}</p>

                            <div class="progress">
                                <div class="bar" style={format!("--w: {}%;", progress)}></div>
                            </div>

                            <div class="telemetry">
                                <div class="row">
                                    <span>{"Mission Progress"}</span>
                                    <strong>{format!("{}%", progress)}</strong>
                                </div>
                                <div class="row">
                                    <span>{"Crew Status"}</span>
                                    <strong>{"Inspired"}</strong>
                                </div>
                                <div class="row">
                                    <span>{"Spacecraft"}</span>
                                    <strong>{"MG-Studio Explorer"}</strong>
                                </div>
                            </div>

                            <div class="actions">
                                <button class="primary" onclick={next_step}>
                                    {"Complete Challenge"}
                                </button>

                                <button class="secondary" onclick={reset_mission}>
                                    {"Reset Mission"}
                                </button>
                            </div>
                        </section>

                        <section class="card">
                            <h3>{"Astronaut Candidate Badge"}</h3>
                            <p>
                                {"Congratulations, "}
                                <strong>{role.call_sign}</strong>
                                {". You are mission ready."}
                            </p>

                            <div class="telemetry">
                                <div class="row">
                                    <span>{"Badge"}</span>
                                    <strong>{"ASTRONAUT CANDIDATE"}</strong>
                                </div>
                                <div class="row">
                                    <span>{"Studio"}</span>
                                    <strong>{"MIKEGYVER STUDIO"}</strong>
                                </div>
                                <div class="row">
                                    <span>{"Mission Day"}</span>
                                    <strong>{"National Astronaut Day"}</strong>
                                </div>
                            </div>
                        </section>

                        <section class="card">
                            <h3>{"Why This Day Matters"}</h3>
                            <p>
                                {"National Astronaut Day is a reminder that exploration begins with imagination. This little app gives kids, families, and lifelong space fans a chance to step into the mission mindset for a few minutes."}
                            </p>
                            <p>
                                {"Today, the mission is yours."}
                            </p>
                        </section>
                    </div>
                }

                <footer class="footer">
                    {"Built with Rust, Yew, WebAssembly, and MikeGyver Studio imagination."}
                </footer>
            </section>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}