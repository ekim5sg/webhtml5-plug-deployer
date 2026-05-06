use js_sys::Math;
use web_sys::HtmlAudioElement;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
struct Role {
    name: &'static str,
    call_sign: &'static str,
    focus: &'static str,
}

#[derive(Clone, PartialEq)]
struct Question {
    prompt: &'static str,
    choices: [&'static str; 4],
    correct: usize,
}

fn play_audio(audio_ref: &NodeRef, volume: f64) -> bool {
    if let Some(audio) = audio_ref.cast::<HtmlAudioElement>() {
        audio.set_volume(volume);
        let _ = audio.play();
        return true;
    }
    false
}

fn stop_audio(audio_ref: &NodeRef) {
    if let Some(audio) = audio_ref.cast::<HtmlAudioElement>() {
        let _ = audio.pause();
    }
}

fn random_question_index(len: usize) -> usize {
    (Math::random() * len as f64).floor() as usize
}

fn random_question_index_excluding_used(used: &[usize], len: usize) -> usize {
    let available: Vec<usize> = (0..len).filter(|i| !used.contains(i)).collect();

    if available.is_empty() {
        return random_question_index(len);
    }

    let pick = (Math::random() * available.len() as f64).floor() as usize;
    available[pick]
}

fn open_badge_svg(role: &Role) {
    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="1200" viewBox="0 0 1200 1200">
<defs>
  <radialGradient id="bg" cx="50%" cy="35%" r="70%">
    <stop offset="0%" stop-color="#1d4ed8"/>
    <stop offset="55%" stop-color="#07152f"/>
    <stop offset="100%" stop-color="#020617"/>
  </radialGradient>
</defs>
<rect width="1200" height="1200" fill="url(#bg)"/>
<circle cx="600" cy="430" r="260" fill="none" stroke="#8fe8ff" stroke-width="10" opacity="0.8"/>
<circle cx="600" cy="430" r="190" fill="none" stroke="#ffd166" stroke-width="6" opacity="0.7"/>
<text x="600" y="150" text-anchor="middle" fill="#ffd166" font-size="54" font-family="Arial" font-weight="800">MIKEGYVER STUDIO</text>
<text x="600" y="345" text-anchor="middle" fill="#ffffff" font-size="82" font-family="Arial" font-weight="900">ASTRONAUT</text>
<text x="600" y="430" text-anchor="middle" fill="#8fe8ff" font-size="74" font-family="Arial" font-weight="900">CANDIDATE</text>
<text x="600" y="540" text-anchor="middle" fill="#ffffff" font-size="46" font-family="Arial">Call Sign: {}</text>
<text x="600" y="610" text-anchor="middle" fill="#dbeafe" font-size="42" font-family="Arial">Role: {}</text>
<text x="600" y="760" text-anchor="middle" fill="#ffd166" font-size="42" font-family="Arial" font-weight="700">MISSION COMPLETE</text>
<text x="600" y="840" text-anchor="middle" fill="#ffffff" font-size="36" font-family="Arial">Mission Day: National Astronaut Day</text>
<text x="600" y="970" text-anchor="middle" fill="#9db4d6" font-size="32" font-family="Arial">Today, the mission was yours.</text>
</svg>"##,
        role.call_sign,
        role.name
    );

    let encoded = js_sys::encode_uri_component(&svg);
    let url = format!("data:image/svg+xml;charset=utf-8,{}", encoded);

    if let Some(window) = web_sys::window() {
        let _ = window.open_with_url(&url);
    }
}

#[function_component(App)]
fn app() -> Html {
    let started = use_state(|| false);
    let selected_role = use_state(|| 0usize);
    let mission_step = use_state(|| 0usize);
    let music_started = use_state(|| false);
    let music_enabled = use_state(|| true);
    let mission_complete = use_state(|| false);
    let selected_answer = use_state(|| None::<usize>);
    let feedback = use_state(|| String::new());
    let question_index = use_state(|| random_question_index(6));
    let used_questions = use_state(|| Vec::<usize>::new());

    let music_ref = use_node_ref();
    let applause_ref = use_node_ref();

    {
        let music_ref = music_ref.clone();
        let music_started = music_started.clone();
        let music_enabled = music_enabled.clone();

        use_effect_with((), move |_| {
            if *music_enabled && play_audio(&music_ref, 0.55) {
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
        "Launch Readiness Check",
        "Low Earth Orbit Stabilization",
        "Lunar Approach Burn",
        "Deep-Space Signal Recovery",
    ];

    let questions = vec![
        Question {
            prompt: "Who was the first American astronaut to travel into space?",
            choices: ["Neil Armstrong", "Alan Shepard", "John Glenn", "Buzz Aldrin"],
            correct: 1,
        },
        Question {
            prompt: "Which mission first landed humans on the Moon?",
            choices: ["Apollo 8", "Apollo 11", "Gemini 4", "Mercury-Redstone 3"],
            correct: 1,
        },
        Question {
            prompt: "What spacecraft carried astronauts to the Moon during Apollo missions?",
            choices: ["Orion", "Dragon", "Saturn V", "Command and Service Module"],
            correct: 3,
        },
        Question {
            prompt: "What does EVA stand for in astronaut missions?",
            choices: [
                "Earth Vehicle Arrival",
                "Extra-Vehicular Activity",
                "Emergency Velocity Alert",
                "External Vision Alignment",
            ],
            correct: 1,
        },
        Question {
            prompt: "What is the name of NASA’s modern spacecraft designed for deep-space crew missions?",
            choices: ["Orion", "Voyager", "Discovery", "Columbia"],
            correct: 0,
        },
        Question {
            prompt: "What city is famous for NASA’s Mission Control?",
            choices: ["Houston", "Cape Canaveral", "Los Angeles", "Seattle"],
            correct: 0,
        },
    ];

    let role = roles[*selected_role].clone();
    let current_question = questions[*question_index].clone();

    let progress = if *mission_complete {
        "100".to_string()
    } else {
        ((*mission_step + 1) * 25).to_string()
    };

    let toggle_music = {
        let music_ref = music_ref.clone();
        let music_started = music_started.clone();
        let music_enabled = music_enabled.clone();

        Callback::from(move |_| {
            if *music_enabled {
                stop_audio(&music_ref);
                music_enabled.set(false);
                music_started.set(false);
            } else {
                if play_audio(&music_ref, 0.55) {
                    music_started.set(true);
                }
                music_enabled.set(true);
            }
        })
    };

    let start_mission = {
        let started = started.clone();
        let music_ref = music_ref.clone();
        let music_started = music_started.clone();
        let music_enabled = music_enabled.clone();

        Callback::from(move |_| {
            if *music_enabled && play_audio(&music_ref, 0.55) {
                music_started.set(true);
            }
            started.set(true);
        })
    };

    let reset_mission = {
        let started = started.clone();
        let mission_step = mission_step.clone();
        let mission_complete = mission_complete.clone();
        let selected_answer = selected_answer.clone();
        let feedback = feedback.clone();
        let question_index = question_index.clone();
        let used_questions = used_questions.clone();

        Callback::from(move |_| {
            mission_step.set(0);
            mission_complete.set(false);
            selected_answer.set(None);
            feedback.set(String::new());
            used_questions.set(Vec::new());
            question_index.set(random_question_index(6));
            started.set(false);
        })
    };

    let complete_challenge = {
        let mission_step = mission_step.clone();
        let mission_complete = mission_complete.clone();
        let selected_answer = selected_answer.clone();
        let feedback = feedback.clone();
        let question_index = question_index.clone();
        let used_questions = used_questions.clone();
        let applause_ref = applause_ref.clone();
        let current_question = current_question.clone();

        Callback::from(move |_| {
            if *mission_complete {
                return;
            }

            match *selected_answer {
                None => {
                    feedback.set("Select an answer before completing the challenge.".to_string());
                }
                Some(answer) if answer == current_question.correct => {
                    let mut updated_used = (*used_questions).clone();
                    updated_used.push(*question_index);
                    used_questions.set(updated_used.clone());

                    if *mission_step >= 3 {
                        mission_complete.set(true);
                        feedback.set("Mission complete! Houston confirms success. 🎉".to_string());
                        let _ = play_audio(&applause_ref, 0.85);
                    } else {
                        mission_step.set(*mission_step + 1);
                        selected_answer.set(None);
                        feedback.set("Correct. Advancing to the next mission stage.".to_string());

                        let next_question = random_question_index_excluding_used(&updated_used, 6);
                        question_index.set(next_question);
                    }
                }
                Some(_) => {
                    feedback.set("Not quite. Try another answer to continue the mission.".to_string());
                }
            }
        })
    };

    let display_badge = {
        let role = role.clone();

        Callback::from(move |_| {
            open_badge_svg(&role);
        })
    };

    let music_status = if *music_enabled && *music_started {
        "🎵 Starlight Runaway is playing."
    } else if *music_enabled {
        "🎵 Music will autoplay when allowed, or tap Start Music."
    } else {
        "🔇 Music muted."
    };

    html! {
        <main class="app">
            <audio
                ref={music_ref.clone()}
                src="./assets/audio/starlight-runaway.mp3"
                preload="auto"
                loop=true
            />

            <audio
                ref={applause_ref.clone()}
                src="./assets/audio/mission-control-applause.mp3"
                preload="auto"
            />

            if *mission_complete {
                <div class="fireworks">
                    <div class="firework one"></div>
                    <div class="firework two"></div>
                    <div class="firework three"></div>
                </div>
            }

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

                                <button class="secondary" onclick={toggle_music.clone()}>
                                    {
                                        if *music_enabled {
                                            "Mute Music"
                                        } else {
                                            "Start Music"
                                        }
                                    }
                                </button>

                                <a class="secondary" href="national-astronaut-day.zip">
                                    {"Download Source Zip"}
                                </a>
                            </div>

                            <p class="music-status">{music_status}</p>
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
                                <button class="secondary" onclick={toggle_music.clone()}>
                                    {
                                        if *music_enabled {
                                            "Mute Music"
                                        } else {
                                            "Start Music"
                                        }
                                    }
                                </button>
                            </div>

                            <p class="music-status">{music_status}</p>
                        </section>

                        <section class="card">
                            <h2>{steps[*mission_step]}</h2>

                            <div class="mission-screen">
                                <div class="orbit">
                                    <div class="craft">{"🛰️"}</div>
                                </div>
                                <div class="earth"></div>
                            </div>

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
                                    <strong>
                                        {
                                            if *mission_complete {
                                                "Mission Complete"
                                            } else {
                                                "Awaiting Correct Answer"
                                            }
                                        }
                                    </strong>
                                </div>
                            </div>

                            <div class="question-box">
                                <h3>{current_question.prompt}</h3>

                                <div class="choice-grid">
                                    {
                                        current_question.choices.iter().enumerate().map(|(choice_index, choice)| {
                                            let selected_answer = selected_answer.clone();
                                            let active = *selected_answer == Some(choice_index);

                                            html! {
                                                <button
                                                    class={classes!("choice-button", active.then_some("active"))}
                                                    disabled={*mission_complete}
                                                    onclick={Callback::from(move |_| selected_answer.set(Some(choice_index)))}
                                                >
                                                    {choice}
                                                </button>
                                            }
                                        }).collect::<Html>()
                                    }
                                </div>

                                if !feedback.is_empty() {
                                    <p class="feedback">{(*feedback).clone()}</p>
                                }
                            </div>

                            <div class="actions">
                                <button
                                    class={classes!("primary", (*mission_complete).then_some("disabled"))}
                                    disabled={*mission_complete}
                                    onclick={complete_challenge}
                                >
                                    {
                                        if *mission_complete {
                                            "Mission Completed"
                                        } else {
                                            "Complete Challenge"
                                        }
                                    }
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
                                {
                                    if *mission_complete {
                                        ". You completed the mission."
                                    } else {
                                        ". Complete all mission gates to earn final clearance."
                                    }
                                }
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

                            <div class="actions">
                                <button
                                    class={classes!("primary", (!*mission_complete).then_some("disabled"))}
                                    disabled={!*mission_complete}
                                    onclick={display_badge}
                                >
                                    {"Display Astronaut Candidate Badge"}
                                </button>
                            </div>
                        </section>

                        <section class="card">
                            <h3>{"Why This Day Matters"}</h3>
                            <p>
                                {"National Astronaut Day is a reminder that exploration begins with imagination. This app now turns that inspiration into a small mission challenge."}
                            </p>
                            <p>{"Today, the mission is yours."}</p>
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