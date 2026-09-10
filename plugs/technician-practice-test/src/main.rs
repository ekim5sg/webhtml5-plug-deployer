use yew::prelude::*;

#[derive(Clone, Copy)]
struct Question {
    id: &'static str,
    topic: &'static str,
    prompt: &'static str,
    answers: [&'static str; 4],
    correct: usize,
    explanation: &'static str,
}

const QUESTIONS: [Question; 35] = [
    Question { id:"T1A02", topic:"Commission's Rules", prompt:"Which agency regulates and enforces the rules for the Amateur Radio Service in the United States?", answers:["ARRL","Homeland Security","The FCC","All these choices are correct"], correct:2, explanation:"The Federal Communications Commission regulates and enforces the Amateur Radio Service rules in the United States." },
    Question { id:"T1B02", topic:"Commission's Rules", prompt:"Which of the following U.S. amateur radio operators are allowed to contact the International Space Station (ISS) on VHF bands?", answers:["Only amateurs with a General class or higher license","Any amateur with a Technician class or higher license","Only amateurs with a General class or higher license, and NASA approval","Any amateurs with a Technician class or higher license, and NASA approval"], correct:1, explanation:"A Technician license includes the VHF privileges needed to contact the ISS; separate NASA approval is not required for an ordinary amateur contact." },
    Question { id:"T1C01", topic:"Commission's Rules", prompt:"For which classes of amateur radio licenses does the FCC currently issue new licenses?", answers:["Novice, Technician, General, Amateur Extra","Technician, Technician Plus, General, Amateur Extra","Novice, Technician Plus, General, Advanced","Technician, General, Amateur Extra"], correct:3, explanation:"The FCC currently issues three classes: Technician, General, and Amateur Extra." },
    Question { id:"T1D04", topic:"Commission's Rules", prompt:"Under what conditions is an amateur station authorized to transmit music using a phone emission?", answers:["When incidental to an authorized retransmission of manned spacecraft communications","When the music produces no spurious emissions","When transmissions are limited to less than three minutes per hour","When the music is transmitted above 1280 MHz"], correct:0, explanation:"Music is generally prohibited, with the narrow exception of music incidental to an authorized retransmission of manned spacecraft communications." },
    Question { id:"T1E05", topic:"Commission's Rules", prompt:"What is an amateur station’s control point?", answers:["The location of the station’s transmitting antenna","The location of the station’s transmitting apparatus","The location at which the control operator function is performed","The mailing address of the station licensee"], correct:2, explanation:"The control point is where the control operator function is performed, whether control is local or remote." },
    Question { id:"T1F03", topic:"Commission's Rules", prompt:"When are you required to transmit your assigned call sign?", answers:["At the beginning of each contact, and every 10 minutes thereafter","At least once during each transmission","At least every 15 minutes during and at the end of a communication","At least every 10 minutes during and at the end of a communication"], correct:3, explanation:"Station identification is required at least every 10 minutes during a communication and again at its end." },
    Question { id:"T2A01", topic:"Operating Procedures", prompt:"What is a common repeater frequency offset in the 2-meter band?", answers:["Plus or minus 5 MHz","Plus or minus 600 kHz","Plus or minus 500 kHz","Plus or minus 1 MHz"], correct:1, explanation:"The commonly used 2-meter repeater input/output separation is 600 kHz." },
    Question { id:"T2B01", topic:"Operating Procedures", prompt:"What is the purpose of the reverse function on a VHF/UHF transceiver?", answers:["To reduce power output","To increase power output","To listen on a repeater’s input frequency","To listen on a repeater’s output frequency"], correct:2, explanation:"Reverse swaps the programmed transmit and receive frequencies, allowing you to listen directly on the repeater input." },
    Question { id:"T2C01", topic:"Operating Procedures", prompt:"When do the FCC Part 97 Amateur Radio Service rules NOT apply to the operation of an amateur station?", answers:["When operating under RACES rules","When operating under FEMA rules","When operating under ARES rules","FCC rules always apply"], correct:3, explanation:"Part 97 continues to apply during RACES, ARES, FEMA-related, and emergency operations." },
    Question { id:"T3A01", topic:"Radio-Wave Propagation", prompt:"Why do VHF signal strengths sometimes vary greatly when the antenna is moved only a few feet?", answers:["The signal path encounters different concentrations of water vapor","VHF ionospheric propagation is very sensitive to path length","Multipath propagation cancels or reinforces signals","The Doppler effect causes slight frequency shifts which result in changes in signal strength"], correct:2, explanation:"Reflected signals can arrive by multiple paths and either reinforce or cancel each other at nearby locations." },
    Question { id:"T3B01", topic:"Radio-Wave Propagation", prompt:"What is the relationship between the electric and magnetic fields of an electromagnetic wave?", answers:["They travel at different speeds","They are in parallel","They revolve in opposite directions","They are at right angles"], correct:3, explanation:"The electric and magnetic fields are perpendicular to one another and to the direction of travel." },
    Question { id:"T3C01", topic:"Radio-Wave Propagation", prompt:"Why are simplex UHF signals rarely heard beyond their radio horizon?", answers:["They are too weak to go very far","FCC regulations prohibit them from going more than 50 miles","UHF signals are usually not propagated by the ionosphere","UHF signals are absorbed by the ionospheric D region"], correct:2, explanation:"UHF communication is normally line-of-sight because the ionosphere usually does not return UHF signals to Earth." },
    Question { id:"T4A01", topic:"Amateur Radio Practices", prompt:"Which of the following is an appropriate power supply rating for a typical 50-watt output mobile FM transceiver?", answers:["24.0 volts at 4 amperes","13.8 volts at 4 amperes","24.0 volts at 12 amperes","13.8 volts at 12 amperes"], correct:3, explanation:"A typical mobile transceiver uses a nominal 13.8-volt supply and requires roughly 12 amperes while transmitting." },
    Question { id:"T4B01", topic:"Amateur Radio Practices", prompt:"What is the effect of excessive microphone gain on SSB transmissions?", answers:["Frequency instability","Distorted transmitted audio","Increased SWR","Sideband inversion"], correct:1, explanation:"Too much microphone gain overdrives the transmitter audio stages and produces distortion." },
    Question { id:"T5A05", topic:"Electrical Principles", prompt:"A difference in which of the following causes electron flow?", answers:["Voltage","Ampere-hours","Capacitance","Inductance"], correct:0, explanation:"Voltage is electrical potential difference; that difference causes electrons to flow in a circuit." },
    Question { id:"T5B01", topic:"Electrical Principles", prompt:"How many milliamperes is 1.5 amperes?", answers:["0.0000015 milliamperes","0.0015 milliamperes","1500 milliamperes","1,500,000 milliamperes"], correct:2, explanation:"One ampere equals 1,000 milliamperes, so 1.5 amperes equals 1,500 milliamperes." },
    Question { id:"T5C01", topic:"Electrical Principles", prompt:"What describes the ability to store energy in an electric field?", answers:["Inductance","Resistance","Frequency","Capacitance"], correct:3, explanation:"Capacitance describes energy storage in an electric field; inductance describes storage in a magnetic field." },
    Question { id:"T5D01", topic:"Electrical Principles", prompt:"What formula is used to calculate current in a circuit?", answers:["I = E × R","I = E ÷ R","I = E² × R","I = E² ÷ R"], correct:1, explanation:"Ohm’s law gives current as voltage divided by resistance: I = E ÷ R." },
    Question { id:"T6A01", topic:"Components", prompt:"What electrical component opposes the flow of current in a DC circuit?", answers:["Inductor","Resistor","Inverter","Transformer"], correct:1, explanation:"A resistor provides resistance, opposing current flow in a DC circuit." },
    Question { id:"T6B02", topic:"Components", prompt:"What electronic component allows current to flow in only one direction?", answers:["Resistor","Fuse","Diode","Driven element"], correct:2, explanation:"A diode conducts primarily in its forward direction and blocks current in the reverse direction." },
    Question { id:"T6C01", topic:"Components", prompt:"What is an electrical diagram using standard component symbols called?", answers:["Connection chart","Instrumentation system","Schematic","Flow chart"], correct:2, explanation:"A schematic uses standardized symbols to show components and their electrical connections." },
    Question { id:"T6D01", topic:"Components", prompt:"Which of the following devices or circuits changes an alternating current into a varying direct current signal?", answers:["Transformer","Rectifier","Amplifier","Reflector"], correct:1, explanation:"A rectifier permits current in one direction, converting AC into pulsating or varying DC." },
    Question { id:"T7A01", topic:"Practical Circuits", prompt:"Which term describes the ability of a receiver to detect the presence of a signal?", answers:["RF gain","Sensitivity","Selectivity","Total Harmonic Distortion"], correct:1, explanation:"Sensitivity describes how weak a signal a receiver can successfully detect." },
    Question { id:"T7B01", topic:"Practical Circuits", prompt:"What can you do if you are told your FM handheld or mobile transceiver is over-deviating?", answers:["Talk louder into the microphone","Let the transceiver cool off","Change to a higher power level","Talk farther away from the microphone"], correct:3, explanation:"Speaking farther from the microphone reduces audio level and therefore reduces excessive FM deviation." },
    Question { id:"T7C01", topic:"Practical Circuits", prompt:"What is the primary purpose of a dummy load?", answers:["To prevent transmitting signals over the air when making tests","To prevent over-modulation of a transmitter","To improve the efficiency of an antenna","To improve the signal-to-noise ratio of a receiver"], correct:0, explanation:"A dummy load absorbs transmitter power so equipment can be tested without radiating an on-air signal." },
    Question { id:"T7D01", topic:"Practical Circuits", prompt:"Which instrument would you use to measure electric potential?", answers:["An ammeter","A voltmeter","A potentiometer","An ohmmeter"], correct:1, explanation:"Electric potential difference is voltage, so it is measured with a voltmeter." },
    Question { id:"T8A01", topic:"Signals and Emissions", prompt:"Which of the following is a form of amplitude modulation?", answers:["Spread spectrum","Packet radio","Single sideband","Phase shift keying (PSK)"], correct:2, explanation:"Single sideband is produced from amplitude modulation by suppressing the carrier and one sideband." },
    Question { id:"T8B01", topic:"Signals and Emissions", prompt:"What telemetry information is typically transmitted by satellite beacons?", answers:["The signal strength of received signals","Time of day accurate to plus or minus 1/10 second","Health and status of the satellite","All these choices are correct"], correct:2, explanation:"Satellite beacon telemetry normally reports spacecraft health and operating status." },
    Question { id:"T8C01", topic:"Signals and Emissions", prompt:"Which of the following methods is used to locate sources of noise interference or jamming?", answers:["Echolocation","Doppler radar","Radio direction finding","Phase locking"], correct:2, explanation:"Radio direction finding uses directional measurements from one or more locations to locate a transmitter or interference source." },
    Question { id:"T8D01", topic:"Signals and Emissions", prompt:"Which of the following is a digital communications mode?", answers:["Packet radio","IEEE 802.11","FT8","All these choices are correct"], correct:3, explanation:"Packet radio, IEEE 802.11 techniques, and FT8 are all forms of digital communication." },
    Question { id:"T9A01", topic:"Antennas and Feed Lines", prompt:"What is a beam antenna?", answers:["An antenna built from square aluminum beams","An omnidirectional antenna invented by Clarence Beam","An antenna that concentrates signals in one direction","An antenna that focuses the signal into two intense rays"], correct:2, explanation:"A beam antenna provides gain by concentrating transmitted and received signals in a preferred direction." },
    Question { id:"T9B01", topic:"Antennas and Feed Lines", prompt:"Which of the following connectors should be carefully taped for weather protection when used outdoors?", answers:["PL-259","BNC","Type N","All these choices are correct"], correct:3, explanation:"Outdoor coax connectors of all these types require careful weatherproofing to keep moisture out." },
    Question { id:"T0A10", topic:"Safety", prompt:"What hazard exists when rapidly charging or discharging an unprotected battery?", answers:["Overheating or out-gassing","Excess output ripple","Electric shock","Overvoltage"], correct:0, explanation:"High charge or discharge current can overheat an unprotected battery and cause hazardous gas release." },
    Question { id:"T0B01", topic:"Safety", prompt:"Which of the following is good practice when installing ground wires on a tower for lightning protection?", answers:["Put a drip loop in the ground connection to prevent water damage to the ground system","Make sure all ground wire bends are right angles","Ensure that connections are short and direct","All these choices are correct"], correct:2, explanation:"Lightning conductors should follow short, direct paths; sharp or right-angle bends increase impedance to lightning current." },
    Question { id:"T0C01", topic:"Safety", prompt:"What type of radiation are radio signals?", answers:["Gamma radiation","Ionizing radiation","Alpha radiation","Non-ionizing radiation"], correct:3, explanation:"Radio-frequency energy is non-ionizing radiation, although sufficient RF exposure can still heat body tissue." },
];

#[function_component(App)]
fn app() -> Html {
    let current = use_state(|| 0usize);
    let answers = use_state(|| vec![None::<usize>; QUESTIONS.len()]);
    let submitted = use_state(|| false);

    let answered_count = answers.iter().filter(|a| a.is_some()).count();
    let score = answers.iter().enumerate().filter(|(i, a)| **a == Some(QUESTIONS[*i].correct)).count();
    let question = QUESTIONS[*current];
    let letters = ["A", "B", "C", "D"];
    let percent = (answered_count as f64 / QUESTIONS.len() as f64) * 100.0;

    let choose = {
        let answers = answers.clone();
        let current = current.clone();
        let submitted = submitted.clone();
        Callback::from(move |choice: usize| {
            if !*submitted {
                let mut updated = (*answers).clone();
                updated[*current] = Some(choice);
                answers.set(updated);
            }
        })
    };

    let previous = { let current=current.clone(); Callback::from(move |_| if *current>0 { current.set(*current-1); }) };
    let next = { let current=current.clone(); Callback::from(move |_| if *current+1<QUESTIONS.len() { current.set(*current+1); }) };
    let submit = { let submitted=submitted.clone(); Callback::from(move |_| submitted.set(true)) };
    let restart = {
        let current=current.clone(); let answers=answers.clone(); let submitted=submitted.clone();
        Callback::from(move |_| { current.set(0); answers.set(vec![None; QUESTIONS.len()]); submitted.set(false); })
    };

    let answer_buttons = question.answers.iter().enumerate().map(|(index, text)| {
        let selected = answers[*current] == Some(index);
        let class = if *submitted && index == question.correct { "answer correct" }
            else if *submitted && selected { "answer wrong" }
            else if selected { "answer selected" } else { "answer" };
        let choose=choose.clone();
        html! { <button class={class} onclick={Callback::from(move |_| choose.emit(index))}><span class="letter">{letters[index]}</span><span>{*text}</span></button> }
    });

    html! {
      <main class="app">
        <header class="top"><div><p class="eyebrow">{"MIKEGYVER STUDIO • RUST EXAM LAB"}</p><h1>{"Technician Practice Test"}</h1><p class="subtitle">{"35 real questions • Current 2026–2030 Element 2 pool"}</p></div><span class="badge">{"RUST • YEW • WASM"}</span></header>
        <div class="statusbar"><div class="progress-track"><div class="progress-fill" style={format!("width:{percent}%")}></div></div><span class="count">{format!("{answered_count}/35 answered")}</span></div>
        <section class="layout">
          <article class="panel">
            <div class="question-meta"><span class="question-id">{question.id}</span><span>{question.topic}</span><span>{format!("{} of 35",*current+1)}</span></div>
            <h2>{question.prompt}</h2>
            <div class="answers">{for answer_buttons}</div>
            {if *submitted { html!{<div class="explanation"><strong>{format!("Answer {}: ",letters[question.correct])}</strong>{question.explanation}</div>} } else { Html::default() }}
            <div class="nav"><button disabled={*current==0} onclick={previous}>{"← Previous"}</button><button class="primary" disabled={*current+1==QUESTIONS.len()} onclick={next}>{"Next →"}</button></div>
          </article>
          <aside class="panel sidebar">
            {if *submitted { html!{
              <div class="result"><h3>{if score>=26{"You passed!"}else{"Keep studying"}}</h3><div class="score-ring" style={format!("--score:{}%",score as f64/35.0*100.0)}><div><strong class={if score>=26{"pass"}else{"fail"}}>{format!("{score}/35")}</strong><span>{format!("{:.0}%",score as f64/35.0*100.0)}</span></div></div><p>{if score>=26{"You reached the 26-correct passing threshold."}else{"Review the marked questions, then try again. You need 26 correct to pass."}}</p><button class="restart" onclick={restart}>{"Retake Test"}</button></div>
            } } else { html!{<><h3>{"Question navigator"}</h3><p class="sidebar-note">{"Answer every question before submitting."}</p></>} }}
            <div class="palette">{for (0..QUESTIONS.len()).map(|i|{let current_handle=current.clone();let mut class="dot".to_string();if answers[i].is_some(){class.push_str(" done");}if i==*current{class.push_str(" current");}if *submitted{class.push_str(if answers[i]==Some(QUESTIONS[i].correct){" good"}else{" bad"});}html!{<button class={class} onclick={Callback::from(move |_|current_handle.set(i))}>{i+1}</button>}})}</div>
            <div class="legend"><span>{"Unanswered"}</span><span class="answered">{"Answered"}</span></div>
            {if !*submitted { html!{<button class="submit primary" disabled={answered_count<QUESTIONS.len()} onclick={submit}>{if answered_count<QUESTIONS.len(){format!("Answer {} More",QUESTIONS.len()-answered_count)}else{"Submit Test".into()}}</button>} } else { Html::default() }}
            <p class="source">{"Practice Test A • One question from each of the 35 NCVEC examination groups • Passing target: 26 correct • Question pool effective July 1, 2026–June 30, 2030"}</p>
          </aside>
        </section>
      </main>
    }
}

fn main() { yew::Renderer::<App>::new().render(); }
