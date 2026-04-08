use yew::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum MissionStage {
    Briefing,
    Reentry,
    Chutes,
    Splashdown,
    Success,
    Failure,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MissionResult {
    Perfect,
    Good,
    Rough,
    Failed,
}

struct App {
    stage: MissionStage,
    heat: i32,
    angle: i32,
    fuel: i32,
    chutes_deployed: bool,
    score: i32,
    status_text: String,
    tips_text: String,
    log_text: String,
    result: Option<MissionResult>,
}

enum Msg {
    StartMission,
    FireThrusters,
    AngleLeft,
    AngleRight,
    DeployChutes,
    AttemptSplashdown,
    ResetGame,
}

impl App {
    fn new_game() -> Self {
        Self {
            stage: MissionStage::Briefing,
            heat: 72,
            angle: 50,
            fuel: 4,
            chutes_deployed: false,
            score: 0,
            status_text: "Mission briefing ready. Your capsule is heading home.".to_string(),
            tips_text: "Keep the angle in the green zone before you deploy parachutes.".to_string(),
            log_text: "Welcome to Mission Control. Start when ready.".to_string(),
            result: None,
        }
    }

    fn angle_badge_class(&self) -> &'static str {
        if (44..=56).contains(&self.angle) {
            "good"
        } else if (36..=64).contains(&self.angle) {
            "warn"
        } else {
            "danger"
        }
    }

    fn heat_badge_class(&self) -> &'static str {
        if self.heat <= 55 {
            "good"
        } else if self.heat <= 78 {
            "warn"
        } else {
            "danger"
        }
    }

    fn fuel_badge_class(&self) -> &'static str {
        if self.fuel >= 2 {
            "good"
        } else if self.fuel == 1 {
            "warn"
        } else {
            "danger"
        }
    }

    fn angle_summary(&self) -> &'static str {
        if (44..=56).contains(&self.angle) {
            "Perfect reentry angle"
        } else if self.angle < 44 {
            "Too shallow"
        } else {
            "Too steep"
        }
    }

    fn can_use_reentry_controls(&self) -> bool {
        self.stage == MissionStage::Reentry
    }

    fn can_deploy_chutes(&self) -> bool {
        self.stage == MissionStage::Chutes && !self.chutes_deployed
    }

    fn can_attempt_splashdown(&self) -> bool {
        self.stage == MissionStage::Splashdown
    }

    fn evaluate_reentry_transition(&mut self) {
        if self.fuel == 0 {
            self.status_text = "Out of fuel. Do your best with the angle you have.".to_string();
            self.log_text = "Fuel tanks empty. Mission Control: Stay calm and guide the capsule.".to_string();
        }

        if self.heat >= 95 {
            self.stage = MissionStage::Failure;
            self.result = Some(MissionResult::Failed);
            self.status_text = "Heat shield overload. The capsule got too hot.".to_string();
            self.tips_text = "Too steep creates dangerous heating. Keep the angle closer to the center.".to_string();
            self.log_text = "Mission failed: reentry heating went critical.".to_string();
            return;
        }

        if (44..=56).contains(&self.angle) {
            self.stage = MissionStage::Chutes;
            self.score += 40;
            self.status_text = "Beautiful reentry. The capsule is stable and slowing down.".to_string();
            self.tips_text = "Now deploy the parachutes at the right moment.".to_string();
            self.log_text = "Reentry success. Capsule stable. Prepare for parachute sequence.".to_string();
        } else if (36..=64).contains(&self.angle) {
            self.stage = MissionStage::Chutes;
            self.score += 20;
            self.status_text = "A little bumpy, but still recoverable.".to_string();
            self.tips_text = "You can still save this. Deploy chutes before splashdown.".to_string();
            self.log_text = "Reentry acceptable, but not perfect. Guidance recommends caution.".to_string();
        } else {
            self.stage = MissionStage::Failure;
            self.result = Some(MissionResult::Failed);
            if self.angle < 36 {
                self.status_text = "Too shallow. The capsule skipped away from the safest path.".to_string();
                self.tips_text = "Next time, point a little steeper before the final descent.".to_string();
                self.log_text = "Mission failed: shallow reentry caused atmospheric skip.".to_string();
            } else {
                self.status_text = "Too steep. The capsule hit the atmosphere too hard.".to_string();
                self.tips_text = "Next time, keep the angle closer to the green center zone.".to_string();
                self.log_text = "Mission failed: steep reentry caused an unsafe descent.".to_string();
            }
        }
    }

    fn evaluate_final_result(&mut self) {
        if !self.chutes_deployed {
            self.stage = MissionStage::Failure;
            self.result = Some(MissionResult::Failed);
            self.status_text = "Splashdown failed. You forgot the parachutes!".to_string();
            self.tips_text = "Every capsule needs help slowing down before hitting the ocean.".to_string();
            self.log_text = "Mission failed: parachutes were not deployed.".to_string();
            return;
        }

        self.stage = MissionStage::Success;

        if (46..=54).contains(&self.angle) && self.heat <= 70 {
            self.result = Some(MissionResult::Perfect);
            self.score += 60;
            self.status_text = "Perfect splashdown! Mission Control is cheering!".to_string();
            self.tips_text = "That was a smooth, textbook return to Earth.".to_string();
            self.log_text = "Mission complete: perfect reentry, parachutes nominal, splashdown successful.".to_string();
        } else if (42..=58).contains(&self.angle) && self.heat <= 82 {
            self.result = Some(MissionResult::Good);
            self.score += 45;
            self.status_text = "Safe splashdown! Great job bringing the crew home.".to_string();
            self.tips_text = "Nice work. The capsule made it home in one piece.".to_string();
            self.log_text = "Mission complete: safe recovery and successful ocean landing.".to_string();
        } else {
            self.result = Some(MissionResult::Rough);
            self.score += 30;
            self.status_text = "Rough landing, but the crew is home!".to_string();
            self.tips_text = "A safe mission counts. Next try, aim closer to the green zone.".to_string();
            self.log_text = "Mission complete: rough but survivable splashdown.".to_string();
        }
    }

    fn marker_left_percent(&self) -> f64 {
        self.angle.clamp(0, 100) as f64
    }

    fn step_done(&self, stage: MissionStage) -> bool {
        match stage {
            MissionStage::Briefing => self.stage != MissionStage::Briefing,
            MissionStage::Reentry => matches!(
                self.stage,
                MissionStage::Chutes | MissionStage::Splashdown | MissionStage::Success | MissionStage::Failure
            ),
            MissionStage::Chutes => matches!(
                self.stage,
                MissionStage::Splashdown | MissionStage::Success | MissionStage::Failure
            ),
            MissionStage::Splashdown => matches!(self.stage, MissionStage::Success | MissionStage::Failure),
            MissionStage::Success | MissionStage::Failure => false,
        }
    }

    fn result_label(&self) -> &'static str {
        match self.result {
            Some(MissionResult::Perfect) => "Perfect Landing",
            Some(MissionResult::Good) => "Safe Return",
            Some(MissionResult::Rough) => "Rough Return",
            Some(MissionResult::Failed) => "Mission Failed",
            None => "In Progress",
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Self::new_game()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StartMission => {
                self.stage = MissionStage::Reentry;
                self.status_text = "The capsule is entering the atmosphere. Adjust angle and manage heat.".to_string();
                self.tips_text = "Use thrusters carefully. Too steep gets hot. Too shallow can skip off course.".to_string();
                self.log_text = "Mission start: reentry guidance is live.".to_string();
                true
            }
            Msg::FireThrusters => {
                if !self.can_use_reentry_controls() {
                    return false;
                }

                if self.fuel <= 0 {
                    self.status_text = "No fuel left for thrusters.".to_string();
                    self.log_text = "Thruster command rejected: fuel depleted.".to_string();
                    return true;
                }

                self.fuel -= 1;

                if self.angle > 50 {
                    self.angle -= 6;
                    self.status_text = "Thrusters fired. Angle nudged toward safer reentry.".to_string();
                    self.log_text = "Thruster burn complete. Capsule nose raised slightly.".to_string();
                } else if self.angle < 50 {
                    self.angle += 6;
                    self.status_text = "Thrusters fired. Angle corrected toward center.".to_string();
                    self.log_text = "Thruster burn complete. Guidance adjusted toward ideal corridor.".to_string();
                } else {
                    self.status_text = "Thrusters fired. You were already near the perfect angle.".to_string();
                    self.log_text = "Thruster pulse used to maintain stable attitude.".to_string();
                }

                self.heat += 4;
                self.score += 8;

                if self.heat >= 95 {
                    self.stage = MissionStage::Failure;
                    self.result = Some(MissionResult::Failed);
                    self.status_text = "Too much heat! The shield overheated.".to_string();
                    self.tips_text = "Thrusters help, but they also add stress. Use them carefully.".to_string();
                    self.log_text = "Mission failed: heat shield exceeded safe limit.".to_string();
                    return true;
                }

                self.evaluate_reentry_transition();
                true
            }
            Msg::AngleLeft => {
                if !self.can_use_reentry_controls() {
                    return false;
                }

                self.angle -= 5;
                self.angle = self.angle.clamp(0, 100);

                if self.angle < 44 {
                    self.status_text = "Angle is getting shallow.".to_string();
                    self.tips_text = "Too shallow can make the capsule bounce away from the safe path.".to_string();
                } else {
                    self.status_text = "Angle adjusted left.".to_string();
                    self.tips_text = "You are getting closer to the target corridor.".to_string();
                }

                self.log_text = "Manual guidance input: angle shifted shallower.".to_string();
                self.heat = (self.heat - 3).max(18);
                self.score += 5;

                self.evaluate_reentry_transition();
                true
            }
            Msg::AngleRight => {
                if !self.can_use_reentry_controls() {
                    return false;
                }

                self.angle += 5;
                self.angle = self.angle.clamp(0, 100);

                if self.angle > 56 {
                    self.status_text = "Warning: angle is getting steep.".to_string();
                    self.tips_text = "Too steep means hotter, rougher reentry.".to_string();
                } else {
                    self.status_text = "Angle adjusted right.".to_string();
                    self.tips_text = "Stay near the green zone for the smoothest return.".to_string();
                }

                self.log_text = "Manual guidance input: angle shifted steeper.".to_string();
                self.heat += 7;
                self.score += 5;

                self.evaluate_reentry_transition();
                true
            }
            Msg::DeployChutes => {
                if !self.can_deploy_chutes() {
                    return false;
                }

                self.chutes_deployed = true;
                self.stage = MissionStage::Splashdown;
                self.score += 25;
                self.heat = (self.heat - 10).max(10);
                self.status_text = "Parachutes deployed! The capsule is slowing down.".to_string();
                self.tips_text = "Great timing. Now complete the ocean landing.".to_string();
                self.log_text = "Parachute deployment confirmed. Main canopies are open.".to_string();
                true
            }
            Msg::AttemptSplashdown => {
                if !self.can_attempt_splashdown() {
                    return false;
                }

                self.evaluate_final_result();
                true
            }
            Msg::ResetGame => {
                *self = Self::new_game();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let marker_style = format!("left: {}%;", self.marker_left_percent());

        html! {
            <div class="app-shell">
                <section class="hero">
                    <div class="hero-inner">
                        <div class="eyebrow">{ "Carpool Lane Mission" }</div>
                        <h1 class="title">{ "Mission Control: Can You Get Home?" }</h1>
                        <p class="subtitle">
                            { "Guide the capsule back to Earth. Keep the angle safe, manage the heat, deploy the parachutes, and bring the crew home." }
                        </p>
                    </div>
                </section>

                <div class="grid">
                    <section class="card panel">
                        <div class="status-line">
                            <span class={classes!("badge", self.angle_badge_class())}>
                                { "Angle: " }{ self.angle }{ "°" }
                            </span>
                            <span class={classes!("badge", self.heat_badge_class())}>
                                { "Heat: " }{ self.heat }{ "%" }
                            </span>
                            <span class={classes!("badge", self.fuel_badge_class())}>
                                { "Fuel: " }{ self.fuel }
                            </span>
                            <span class="badge">
                                { self.result_label() }
                            </span>
                        </div>

                        <div class="hud">
                            <div class="hud-item">
                                <div class="hud-label">{ "Mission Stage" }</div>
                                <div class="hud-value small">
                                    {
                                        match self.stage {
                                            MissionStage::Briefing => "Briefing",
                                            MissionStage::Reentry => "Reentry",
                                            MissionStage::Chutes => "Parachutes",
                                            MissionStage::Splashdown => "Splashdown",
                                            MissionStage::Success => "Mission Complete",
                                            MissionStage::Failure => "Mission Failed",
                                        }
                                    }
                                </div>
                            </div>

                            <div class="hud-item">
                                <div class="hud-label">{ "Guidance Readout" }</div>
                                <div class="hud-value small">{ self.angle_summary() }</div>
                            </div>

                            <div class="hud-item">
                                <div class="hud-label">{ "Parachutes" }</div>
                                <div class="hud-value small">
                                    { if self.chutes_deployed { "Deployed" } else { "Not Yet" } }
                                </div>
                            </div>

                            <div class="hud-item">
                                <div class="hud-label">{ "Mission Score" }</div>
                                <div class="hud-value">{ self.score }</div>
                            </div>
                        </div>

                        <div class="track-wrap">
                            <div class="track-label">{ "Reentry Corridor" }</div>
                            <div class="track">
                                <div
                                    class={classes!("marker", if self.stage == MissionStage::Reentry { Some("flash") } else { None })}
                                    style={marker_style}
                                />
                            </div>
                            <div class="small-note" style="margin-top: 8px;">
                                { "Red is dangerous. Yellow is risky. Green is the smooth corridor." }
                            </div>
                        </div>

                        <div class="mission-box">
                            <div class="mission-label">{ "Mission Control" }</div>
                            <div class="mission-text">{ &self.status_text }</div>
                            <div class="mission-sub">{ &self.tips_text }</div>
                        </div>

                        <div class="controls">
                            <button
                                class="btn primary"
                                onclick={ctx.link().callback(|_| Msg::StartMission)}
                                disabled={self.stage != MissionStage::Briefing}
                            >
                                { "🚀 Start Mission" }
                            </button>

                            <button
                                class="btn"
                                onclick={ctx.link().callback(|_| Msg::FireThrusters)}
                                disabled={!self.can_use_reentry_controls()}
                            >
                                { "🔥 Fire Thrusters" }
                            </button>

                            <button
                                class="btn warn"
                                onclick={ctx.link().callback(|_| Msg::AngleLeft)}
                                disabled={!self.can_use_reentry_controls()}
                            >
                                { "↙️ Angle Left" }
                            </button>

                            <button
                                class="btn warn"
                                onclick={ctx.link().callback(|_| Msg::AngleRight)}
                                disabled={!self.can_use_reentry_controls()}
                            >
                                { "↘️ Angle Right" }
                            </button>

                            <button
                                class="btn success"
                                onclick={ctx.link().callback(|_| Msg::DeployChutes)}
                                disabled={!self.can_deploy_chutes()}
                            >
                                { "🪂 Deploy Chutes" }
                            </button>

                            <button
                                class="btn danger"
                                onclick={ctx.link().callback(|_| Msg::AttemptSplashdown)}
                                disabled={!self.can_attempt_splashdown()}
                            >
                                { "🌊 Splash Down" }
                            </button>
                        </div>

                        <div class="center-actions">
                            <button
                                class="btn"
                                onclick={ctx.link().callback(|_| Msg::ResetGame)}
                            >
                                { "🔁 Reset Mission" }
                            </button>
                        </div>
                    </section>

                    <aside class="card panel">
                        <h2>{ "Mission Checklist" }</h2>
                        <div class="checklist">
                            <div class={classes!("step", if self.step_done(MissionStage::Briefing) { Some("done") } else { None })}>
                                <div class="step-icon">{ if self.step_done(MissionStage::Briefing) { "✓" } else { "1" } }</div>
                                <div>
                                    <div class="step-title">{ "Start the mission" }</div>
                                    <div class="step-desc">{ "Mission Control wakes up. Your capsule begins its return to Earth." }</div>
                                </div>
                            </div>

                            <div class={classes!("step", if self.step_done(MissionStage::Reentry) { Some("done") } else { None })}>
                                <div class="step-icon">{ if self.step_done(MissionStage::Reentry) { "✓" } else { "2" } }</div>
                                <div>
                                    <div class="step-title">{ "Guide reentry" }</div>
                                    <div class="step-desc">{ "Keep the angle near the safe green zone. Avoid too steep or too shallow." }</div>
                                </div>
                            </div>

                            <div class={classes!("step", if self.step_done(MissionStage::Chutes) { Some("done") } else { None })}>
                                <div class="step-icon">{ if self.step_done(MissionStage::Chutes) { "✓" } else { "3" } }</div>
                                <div>
                                    <div class="step-title">{ "Deploy parachutes" }</div>
                                    <div class="step-desc">{ "Once stable, open the chutes so the capsule slows down safely." }</div>
                                </div>
                            </div>

                            <div class={classes!("step", if self.step_done(MissionStage::Splashdown) { Some("done") } else { None })}>
                                <div class="step-icon">{ if self.step_done(MissionStage::Splashdown) { "✓" } else { "4" } }</div>
                                <div>
                                    <div class="step-title">{ "Complete splashdown" }</div>
                                    <div class="step-desc">{ "Land in the ocean and bring the crew home." }</div>
                                </div>
                            </div>
                        </div>

                        <div class="score-box">
                            <div class="score-title">{ "Flight Log" }</div>
                            <div class="small-note">{ &self.log_text }</div>
                        </div>

                        <div class="kid-tip">
                            <strong>{ "Kid Science Tip: " }</strong>
                            { "A spacecraft returning to Earth cannot come in at just any angle. Too steep makes it dangerously hot. Too shallow can make it skip off the atmosphere. Real missions need a safe reentry corridor." }
                        </div>

                        <div class="footer-row">
                            <span class="badge">{ "👨‍👩‍👧‍👦 Parent-friendly" }</span>
                            <span class="badge">{ "🧠 STEM mini lesson" }</span>
                            <span class="badge">{ "📱 Mobile first" }</span>
                        </div>
                    </aside>
                </div>
            </div>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}