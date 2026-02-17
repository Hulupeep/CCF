//! mBot2 Companion - Runs RuVector AI on laptop, controls mBot2 via Bluetooth/Serial
//!
//! Usage:
//!   mbot-companion --bluetooth           # Connect via Bluetooth
//!   mbot-companion --serial /dev/ttyUSB0 # Connect via USB serial
//!   mbot-companion --simulate            # Run without hardware (testing)

use anyhow::{Context, Result};
use clap::Parser;
use mbot_core::{HomeostasisState, MBotBrain, MBotSensors, MotorCommand, ReflexMode};
use mbot_core::coherence::{CoherenceField, PresenceDetector, ContextKey, SocialPhase, PresenceSignature};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{info, warn, Level};

mod protocol;
mod transport;

#[cfg(feature = "brain")]
mod brain;

use transport::{MBotTransport, TransportType};

#[derive(Parser, Debug)]
#[command(name = "mbot-companion")]
#[command(about = "RuVector AI companion for mBot2", long_about = None)]
struct Args {
    /// Connect via Bluetooth
    #[arg(long)]
    bluetooth: bool,

    /// Connect via serial port
    #[arg(long)]
    serial: Option<String>,

    /// Simulate without hardware
    #[arg(long)]
    simulate: bool,

    /// Control loop frequency in Hz
    #[arg(long, default_value = "20")]
    freq: u32,

    /// Enable drawing mode (pen attached)
    #[arg(long)]
    draw: bool,

    /// Enable brain layer (LLM-powered reasoning)
    #[cfg(feature = "brain")]
    #[arg(long)]
    brain: bool,

    /// Enable voice pipeline
    #[cfg(feature = "voice")]
    #[arg(long)]
    voice: bool,

    /// Enable autonomous room exploration (requires --brain)
    #[cfg(feature = "brain")]
    #[arg(long)]
    explore: bool,

    /// Start HTTP voice API server for phone-based voice interaction
    #[cfg(feature = "voice-api")]
    #[arg(long)]
    voice_api: bool,

    /// Voice API server port (default 8080)
    #[cfg(feature = "voice-api")]
    #[arg(long, default_value = "8080")]
    voice_port: u16,

    /// Run sensor diagnostics on startup
    #[arg(long)]
    diagnose: bool,

    /// Test R2-D2 voice with a phrase
    #[arg(long)]
    speak: Option<String>,

    /// Verbose output (default: on, use --quiet to suppress)
    #[arg(short, long, default_value_t = true)]
    verbose: bool,

    /// Quiet mode - suppress debug output
    #[arg(short, long)]
    quiet: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present (silently ignore if missing)
    dotenvy::dotenv().ok();

    let args = Args::parse();

    // Setup logging
    let log_level = if args.quiet { Level::WARN } else if args.verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    info!("🤖 mBot2 RuVector Companion starting...");

    // Determine transport type
    let transport_type = if args.simulate {
        info!("📡 Running in SIMULATION mode");
        TransportType::Simulated
    } else if args.bluetooth {
        #[cfg(feature = "bluetooth")]
        {
            info!("📡 Connecting via Bluetooth...");
            TransportType::Bluetooth
        }
        #[cfg(not(feature = "bluetooth"))]
        {
            anyhow::bail!("Bluetooth support not compiled. Rebuild with: cargo build --features bluetooth");
        }
    } else if let Some(port) = &args.serial {
        #[cfg(feature = "serial")]
        {
            info!("📡 Connecting via Serial: {}", port);
            TransportType::Serial(port.clone())
        }
        #[cfg(not(feature = "serial"))]
        {
            let _ = port;
            anyhow::bail!("Serial support not compiled. Rebuild with: cargo build --features serial");
        }
    } else {
        info!("📡 No connection specified, running in SIMULATION mode");
        info!("   Use --bluetooth or --serial <port> for real hardware");
        TransportType::Simulated
    };

    // Create transport
    let mut transport = MBotTransport::connect(transport_type).await?;

    // Run sensor diagnostics if requested (works for both serial and bluetooth)
    if args.diagnose {
        let health = transport.run_diagnostics().await?;
        info!("Diagnostics complete: {}/{} core sensors OK", health.ok_count(), health.total());
        // Stop motors after diagnostics to ensure robot is stationary
        transport.send_command(&MotorCommand::default()).await?;
        info!("Diagnostics-only mode. Exiting. (Remove --diagnose to run the main loop.)");
        return Ok(());
    }

    // Test R2-D2 voice if requested
    if let Some(ref phrase) = args.speak {
        info!("R2-D2 voice test: \"{}\"", phrase);
        transport.robot_speak(phrase).await?;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        info!("Voice test complete.");
        return Ok(());
    }

    let transport = Arc::new(Mutex::new(transport));

    // Start voice API server if requested
    #[cfg(feature = "voice-api")]
    let voice_shared: Option<Arc<Mutex<brain::voice_api::SharedVoiceState>>> = if args.voice_api {
        let shared = Arc::new(Mutex::new(brain::voice_api::SharedVoiceState::default()));
        let stt = Arc::new(brain::stt::SttChain::from_env());
        info!("STT providers: {:?}", stt.provider_names());
        let _handle = brain::voice_api::start_voice_api(
            shared.clone(),
            transport.clone(),
            stt,
            args.voice_port,
        ).await;
        info!("Voice API server started on port {}", args.voice_port);
        Some(shared)
    } else {
        None
    };

    // Create brain (deterministic nervous system)
    let brain = Arc::new(Mutex::new(MBotBrain::new()));

    // Initialize higher-order brain layer if enabled
    #[cfg(feature = "brain")]
    let brain_layer = if args.brain {
        info!("Initializing higher-order brain layer...");
        let config = brain::BrainConfig::default();
        match brain::BrainLayer::new(config).await {
            Ok(mut layer) => {
                // Set up LLM provider chain
                let llm_config = brain::llm::config::LlmConfig::from_env()
                    .unwrap_or_default();

                let mut chain = brain::llm::ProviderChain::new();

                // Try Claude first (primary)
                if llm_config.has_claude() {
                    match brain::llm::claude::ClaudeProvider::new(&llm_config) {
                        Ok(provider) => {
                            info!("Claude API provider configured ({})", llm_config.claude_model);
                            chain = chain.add_provider(std::sync::Arc::new(provider));
                        }
                        Err(e) => warn!("Failed to init Claude provider: {}", e),
                    }
                }

                // Ollama fallback
                match brain::llm::ollama::OllamaProvider::new(&llm_config) {
                    Ok(provider) => {
                        info!("Ollama provider configured ({})", llm_config.ollama_model);
                        chain = chain.add_provider(std::sync::Arc::new(provider));
                    }
                    Err(e) => warn!("Failed to init Ollama provider: {}", e),
                }

                if let Err(e) = layer.init(chain).await {
                    warn!("Failed to init brain planner: {}", e);
                }

                if let Err(e) = layer.init_autonomy().await {
                    warn!("Failed to init autonomy engine: {}", e);
                }

                info!("Brain layer initialized");
                Some(Arc::new(Mutex::new(layer)))
            }
            Err(e) => {
                warn!("Failed to create brain layer: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Create personality
    let personality = Arc::new(Mutex::new(mbot_core::personality::Personality::default()));

    // Create exploration engine if requested
    #[cfg(feature = "brain")]
    let explorer = if args.explore {
        if !args.brain {
            anyhow::bail!("--explore requires --brain");
        }
        info!("Autonomous exploration: ON");
        Some(Arc::new(Mutex::new(brain::autonomy::explore::ExploreAction::new())))
    } else {
        None
    };

    // Run main loop
    run_main_loop(
        transport,
        brain,
        args.freq,
        args.draw,
        #[cfg(feature = "brain")]
        brain_layer,
        personality,
        #[cfg(feature = "voice-api")]
        voice_shared,
        #[cfg(feature = "brain")]
        explorer,
    ).await
}

async fn run_main_loop(
    transport: Arc<Mutex<MBotTransport>>,
    brain: Arc<Mutex<MBotBrain>>,
    freq: u32,
    draw_mode: bool,
    #[cfg(feature = "brain")] brain_layer: Option<Arc<Mutex<brain::BrainLayer>>>,
    personality: Arc<Mutex<mbot_core::personality::Personality>>,
    #[cfg(feature = "voice-api")] voice_shared: Option<Arc<Mutex<brain::voice_api::SharedVoiceState>>>,
    #[cfg(feature = "brain")] explorer: Option<Arc<Mutex<brain::autonomy::explore::ExploreAction>>>,
) -> Result<()> {
    let tick_duration = Duration::from_secs_f64(1.0 / freq as f64);
    let mut last_tick = Instant::now();
    let mut tick_count = 0u64;

    // Stats tracking
    let mut total_loop_time = Duration::ZERO;
    let mut max_loop_time = Duration::ZERO;
    let mut slow_tick_count = 0u64;
    let mut last_slow_warn = Instant::now();

    // Contextual Coherence Fields
    let mut presence_detector = PresenceDetector::new();
    let mut coherence_field = CoherenceField::new_with_personality(
        personality.lock().await.curiosity_drive()
    );
    let mut social_phase = SocialPhase::ShyObserver;

    // Print a welcome banner so the user knows what they're looking at
    println!();
    println!("==========================================================");
    println!("  mBot2 RuVector - Nervous System Running");
    println!("==========================================================");
    println!();
    println!("  The robot's nervous system is now active!");
    println!("  A status display will update below every second.");
    println!();
    println!("  MOOD is the robot's emotional state. It changes on its own");
    println!("  based on what it senses. You did not program these moods -");
    println!("  they emerge from the nervous system.");
    println!();
    println!("  The four moods:");
    println!("    CALM    = relaxed, nothing interesting happening");
    println!("    ACTIVE  = curious, something caught its attention");
    println!("    SPIKE   = startled! something sudden happened");
    println!("    PROTECT = scared, backing away from a threat");
    println!();
    println!("  Tension  = how stressed the robot feels (0-100%)");
    println!("  Coherence= how clearly it is thinking  (0-100%)");
    println!("  Energy   = how much energy it has left  (0-100%)");
    println!("  Curiosity= how interested it is         (0-100%)");
    println!();
    println!("  Sensors read from hardware (via serial):");
    println!("    Distance = ultrasonic sensor (cm to nearest object)");
    println!("    Sound    = onboard microphone level (0-100%)");
    println!("    Light    = onboard light sensor (0-100%)");
    println!("    Gyro Z   = rotation rate (degrees/second)");
    println!("    Quad RGB = color sensor (R, G, B values)");
    println!();
    println!("  TRY THIS: put your hand in front of the robot's 'eyes'");
    println!("  (the two round tubes on the front). Move it closer and");
    println!("  farther away. Watch Distance change and the mood react.");
    println!();
    println!("  TIP: use --diagnose to test each sensor individually.");
    println!();
    if draw_mode {
        println!("  Drawing mode: ON (pen servo active)");
    }
    #[cfg(feature = "brain")]
    if brain_layer.is_some() {
        println!("  Brain layer: ON (LLM reasoning active)");
    }
    #[cfg(feature = "brain")]
    if explorer.is_some() {
        println!("  Exploration: ON (autonomous room explorer active)");
        println!("  The robot will scan, navigate, learn, and narrate its discoveries.");
    }
    #[cfg(feature = "voice-api")]
    if voice_shared.is_some() {
        println!("  Voice API: ON - open http://<laptop-ip>:8080 on your phone");
    }
    println!("  Press Ctrl+C to stop.");
    println!();
    println!("----------------------------------------------------------");

    loop {
        let loop_start = Instant::now();

        // Read sensors
        let sensors = {
            let mut t = transport.lock().await;
            t.read_sensors().await?
        };

        // Process through deterministic nervous system (always runs)
        let (mut state, mut cmd) = {
            let mut b = brain.lock().await;
            b.tick(&sensors)
        };

        // --- Contextual Coherence Fields (post-tick processing) ---
        {
            // 1. Update presence detector with ultrasonic reading
            let presence = presence_detector.update(sensors.ultrasonic_cm);

            // 2. Compute context key from current sensors
            let accel_mag = (sensors.accel[0] * sensors.accel[0]
                + sensors.accel[1] * sensors.accel[1]
                + sensors.accel[2] * sensors.accel[2])
                .sqrt();
            let motors_active = cmd.left != 0 || cmd.right != 0;
            let context_key = ContextKey::from_sensors(
                sensors.light_level,
                sensors.sound_level,
                presence,
                accel_mag,
                motors_active,
                0.0, // roll — wire from CyberPi when available
                0.0, // pitch — wire from CyberPi when available
            );

            // 3. Update accumulator for current context
            let alone = presence == PresenceSignature::Absent;
            let is_positive = state.tension < 0.5;
            {
                let pers = personality.lock().await;
                let acc = coherence_field.get_or_create(&context_key);
                if is_positive {
                    acc.positive_interaction(pers.recovery_speed(), tick_count, alone);
                } else if state.tension > 0.7 {
                    acc.negative_interaction(pers.startle_sensitivity(), tick_count);
                }
            }

            // 4. Set fallback (degraded mode) from instant coherence
            coherence_field.set_fallback(Some(state.coherence));

            // 5. Compute effective coherence (asymmetric gate)
            let eff_coherence = coherence_field.effective_coherence(state.coherence, &context_key);

            // 6. Classify social phase with hysteresis
            social_phase = SocialPhase::classify(eff_coherence, state.tension, social_phase);

            // 7. Override state coherence with effective value for downstream consumers
            state.coherence = eff_coherence;
            state.social_phase = social_phase;
            state.context_coherence = coherence_field.context_coherence(&context_key);

            // 8. Periodic decay (every 100 ticks)
            if tick_count % 100 == 0 {
                coherence_field.decay_all(100);
            }
        }

        // Higher-order brain layer (advisory, runs alongside deterministic system)
        #[cfg(feature = "brain")]
        if let Some(ref bl) = brain_layer {
            let mut layer = bl.lock().await;
            if layer.is_enabled() {
                let pers = personality.lock().await;
                match layer.on_tick(&state, &sensors, &pers).await {
                    Ok(actions) => {
                        for action in actions {
                            match action {
                                brain::planner::BrainAction::Motor(m) => cmd = m,
                                brain::planner::BrainAction::Speak(text) => {
                                    tracing::debug!("Brain says: {}", text);
                                    // Route to channels/voice in future
                                }
                                brain::planner::BrainAction::PersonalityAdjust { parameter, delta } => {
                                    tracing::debug!("Brain adjusts {}: {:+.2}", parameter, delta);
                                    // Apply personality adjustments in future
                                }
                                brain::planner::BrainAction::StartActivity(name) => {
                                    tracing::debug!("Brain starts activity: {}", name);
                                }
                                brain::planner::BrainAction::Explore(explore_cmd) => {
                                    tracing::debug!("Brain exploration command: {:?}", explore_cmd);
                                }
                                brain::planner::BrainAction::Noop => {}
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Brain layer error: {}", e);
                    }
                }
            }
        }

        // Exploration engine: autonomous room exploration (runs after brain, before voice override)
        // Priority: Protect mode > Voice Override > Explorer > Brain LLM > Default
        #[cfg(feature = "brain")]
        if let Some(ref exp) = explorer {
            // Only let explorer drive when NOT in Protect mode
            if state.reflex != ReflexMode::Protect {
                let mut exp_guard = exp.lock().await;
                let heading_deg = {
                    let b = brain.lock().await;
                    b.heading() * 180.0 / std::f32::consts::PI
                };
                if let Some(explore_cmd) = exp_guard.tick(
                    state.curiosity,
                    state.energy,
                    state.tension,
                    sensors.ultrasonic_cm,
                    heading_deg,
                    tick_count,
                ) {
                    cmd = explore_cmd;
                }

                // Process exploration events for narration
                let events = exp_guard.take_events();
                for event in &events {
                    match event {
                        mbot_core::exploration::ExplorationEvent::CellDiscovered { x, y, .. } => {
                            tracing::info!("Exploration: discovered cell ({}, {})", x, y);
                        }
                        mbot_core::exploration::ExplorationEvent::ObstacleFound { distance_cm, heading_deg } => {
                            tracing::info!("Exploration: obstacle at {:.0}cm, heading {:.0}°", distance_cm, heading_deg);
                        }
                        mbot_core::exploration::ExplorationEvent::ScanComplete { sectors_updated } => {
                            tracing::debug!("Exploration: scan complete, {} sectors updated", sectors_updated);
                        }
                        mbot_core::exploration::ExplorationEvent::TargetChosen { sector, heading_deg } => {
                            tracing::info!("Exploration: heading to sector {} ({:.0}°)", sector, heading_deg);
                        }
                        mbot_core::exploration::ExplorationEvent::Arrived { x, y } => {
                            tracing::info!("Exploration: arrived at ({}, {})", x, y);
                        }
                        mbot_core::exploration::ExplorationEvent::ReflectionPause => {
                            tracing::info!("Exploration: pausing for reflection...");
                        }
                    }
                }

                // Update voice API shared state with exploration data
                #[cfg(feature = "voice-api")]
                if let Some(ref vs) = voice_shared {
                    let mut vs_guard = vs.lock().await;
                    let grid_summary = exp_guard.grid_map.summary();
                    let sector_summary = exp_guard.sector_map.summary();
                    let metrics = exp_guard.learning_metrics();
                    vs_guard.exploration = Some(brain::voice_api::ExplorationState {
                        phase: exp_guard.phase_name().to_string(),
                        sectors_mapped: sector_summary.mapped,
                        grid_visited: grid_summary.visited,
                        discovery_count: exp_guard.discovery_count,
                        episode_count: exp_guard.episode_count,
                        nav_confidence: metrics.convergence_score,
                        grid: exp_guard.grid_map.cells.iter().flatten().map(|c| match c.occupancy {
                            mbot_core::exploration::Occupancy::Unknown => 0,
                            mbot_core::exploration::Occupancy::Free => 1,
                            mbot_core::exploration::Occupancy::Obstacle => 2,
                            mbot_core::exploration::Occupancy::Interesting => 3,
                        }).collect(),
                        robot_pos: [grid_summary.robot_x, grid_summary.robot_y],
                        robot_heading: grid_summary.heading_deg,
                        sector_distances: exp_guard.sector_map.sectors.iter().map(|s| s.min_distance_cm).collect(),
                        narration_log: Vec::new(), // filled by narration system
                        reflection: None,
                        avg_reward: metrics.average_reward,
                    });
                }
            }
        }

        // Voice API: voice commands can override motors.
        // When no voice override is active, let the explorer (or homeostasis) keep driving.
        #[cfg(feature = "voice-api")]
        if let Some(ref vs) = voice_shared {
            let mut vs_guard = vs.lock().await;
            if let Some(ref mut ov) = vs_guard.motor_override {
                // Start timer on first read (avoids TTS/speak eating into duration)
                if ov.expires_at.is_none() {
                    ov.expires_at = Some(Instant::now() + Duration::from_millis(ov.duration_ms));
                }
                match ov.expires_at {
                    Some(t) if Instant::now() < t => {
                        // Voice override takes priority over explorer
                        cmd = ov.cmd.clone();
                    }
                    _ => {
                        // Override expired, clear it (explorer/homeostasis cmd stays)
                        vs_guard.motor_override = None;
                    }
                }
            }
        }

        // Override pen state if not in draw mode
        if !draw_mode {
            cmd.pen_angle = 45; // Keep pen up
        }

        // Send motor commands
        {
            let mut t = transport.lock().await;
            t.send_command(&cmd).await?;
        }

        // Voice API: update robot state for connected phones
        #[cfg(feature = "voice-api")]
        if let Some(ref vs) = voice_shared {
            let mut vs_guard = vs.lock().await;
            vs_guard.robot.mood = format!("{:?}", state.reflex);
            vs_guard.robot.tension = state.tension;
            vs_guard.robot.energy = state.energy;
            vs_guard.robot.coherence = state.coherence;
            vs_guard.robot.curiosity = state.curiosity;
            vs_guard.robot.social_phase = format!("{:?}", state.social_phase);
            vs_guard.robot.context_coherence = state.context_coherence;
            vs_guard.robot.context_count = coherence_field.context_count();
        }

        // Track timing
        let loop_time = loop_start.elapsed();
        total_loop_time += loop_time;
        max_loop_time = max_loop_time.max(loop_time);
        tick_count += 1;

        // Print status periodically (every second)
        if tick_count % (freq as u64) == 0 {
            print_status(&sensors, &state, tick_count, total_loop_time, max_loop_time,
                         coherence_field.context_count());
        }

        // Maintain loop timing - only warn about slow loops occasionally, not every tick.
        // Serial communication is naturally slower than the tick rate and that is fine.
        let elapsed = last_tick.elapsed();
        if elapsed < tick_duration {
            tokio::time::sleep(tick_duration - elapsed).await;
        } else if elapsed > tick_duration * 2 {
            slow_tick_count += 1;
            // Only print a warning at most once every 30 seconds to avoid flooding
            if last_slow_warn.elapsed() > Duration::from_secs(30) {
                tracing::debug!(
                    "Loop slower than target: {:.1}ms avg (target {:.1}ms) - {} slow ticks. \
                     This is normal for serial connections.",
                    elapsed.as_secs_f64() * 1000.0,
                    tick_duration.as_secs_f64() * 1000.0,
                    slow_tick_count,
                );
                slow_tick_count = 0;
                last_slow_warn = Instant::now();
            }
        }
        last_tick = Instant::now();
    }
}

fn print_status(
    sensors: &MBotSensors,
    state: &HomeostasisState,
    tick_count: u64,
    _total_time: Duration,
    _max_time: Duration,
    context_count: usize,
) {
    // Describe the emotional mode in plain language
    let (mode_name, mode_desc) = match state.reflex {
        ReflexMode::Calm => ("CALM", "relaxed, content"),
        ReflexMode::Active => ("ACTIVE", "curious, exploring"),
        ReflexMode::Spike => ("SPIKE", "startled, alert!"),
        ReflexMode::Protect => ("PROTECT", "scared, backing away"),
    };

    let (phase_name, phase_desc) = match state.social_phase {
        SocialPhase::ShyObserver => ("SHY", "cautious, earning trust"),
        SocialPhase::StartledRetreat => ("STARTLED", "withdrawn, protective"),
        SocialPhase::QuietlyBeloved => ("BELOVED", "expressive, trusted"),
        SocialPhase::ProtectiveGuardian => ("GUARDIAN", "protective, familiar"),
    };

    // Build simple bar: 5 chars wide, filled proportionally
    fn bar(value: f32) -> String {
        let filled = (value * 5.0).round() as usize;
        let filled = filled.min(5);
        format!("[{}{}]", "#".repeat(filled), "-".repeat(5 - filled))
    }

    // Describe values in human terms
    fn describe_01(value: f32) -> &'static str {
        if value > 0.8 { "very high" }
        else if value > 0.6 { "high" }
        else if value > 0.4 { "medium" }
        else if value > 0.2 { "low" }
        else { "very low" }
    }

    fn describe_distance(cm: f32) -> &'static str {
        if cm < 5.0 { "TOUCHING!" }
        else if cm < 15.0 { "very close" }
        else if cm < 40.0 { "nearby" }
        else if cm < 100.0 { "in range" }
        else { "nothing near" }
    }

    println!();
    println!("  MOOD:  {}  ({})", mode_name, mode_desc);
    println!("  PHASE: {}  ({})", phase_name, phase_desc);
    println!("  -----------------------------------------------");
    println!(
        "  Tension:   {} {:.0}%  {}",
        bar(state.tension),
        state.tension * 100.0,
        match describe_01(state.tension) {
            "very high" => "<-- very stressed!",
            "high" => "<-- on edge",
            "medium" => "<-- a bit tense",
            "low" => "<-- relaxed",
            _ => "<-- totally chill",
        }
    );
    println!(
        "  Coherence: {} {:.0}%  {}",
        bar(state.coherence),
        state.coherence * 100.0,
        match describe_01(state.coherence) {
            "very high" => "<-- thinking clearly",
            "high" => "<-- focused",
            "medium" => "<-- ok",
            "low" => "<-- scattered",
            _ => "<-- confused",
        }
    );
    println!(
        "  Energy:    {} {:.0}%  {}",
        bar(state.energy),
        state.energy * 100.0,
        match describe_01(state.energy) {
            "very high" => "<-- full of energy!",
            "high" => "<-- energetic",
            "medium" => "<-- doing ok",
            "low" => "<-- getting tired",
            _ => "<-- exhausted",
        }
    );
    println!(
        "  Curiosity: {} {:.0}%  {}",
        bar(state.curiosity),
        state.curiosity * 100.0,
        match describe_01(state.curiosity) {
            "very high" => "<-- fascinated!",
            "high" => "<-- interested",
            "medium" => "<-- mildly curious",
            "low" => "<-- not very interested",
            _ => "<-- bored",
        }
    );
    println!("  -----------------------------------------------");
    println!(
        "  Distance:  {:.0} cm  ({})",
        sensors.ultrasonic_cm,
        describe_distance(sensors.ultrasonic_cm),
    );
    println!(
        "  Sound:     {} {:.0}%  ({})",
        bar(sensors.sound_level),
        sensors.sound_level * 100.0,
        describe_01(sensors.sound_level),
    );
    println!(
        "  Light:     {} {:.0}%  ({})",
        bar(sensors.light_level),
        sensors.light_level * 100.0,
        describe_01(sensors.light_level),
    );
    println!(
        "  Gyro Z:    {:.1} deg/s",
        sensors.gyro_z,
    );
    println!(
        "  RGB:       [{}, {}, {}]",
        sensors.quad_rgb[0][0],
        sensors.quad_rgb[0][1],
        sensors.quad_rgb[0][2],
    );
    println!(
        "  Context:   {} {:.0}%  (familiarity with current situation)",
        bar(state.context_coherence),
        state.context_coherence * 100.0,
    );
    println!(
        "  Contexts:  {} situations learned",
        context_count,
    );
    println!("  -----------------------------------------------");
    println!("  Tick: {}  |  Press Ctrl+C to stop", tick_count);
}
