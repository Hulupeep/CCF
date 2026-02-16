//! HTTP Voice API Server for phone-based voice interaction
//!
//! Phone (Android) -> HTTP -> Laptop (STT + command parsing) -> BLE -> CyberPi
//!
//! # Endpoints
//! - `POST /api/voice` - Accept audio bytes, transcribe, parse command, execute
//! - `GET /api/state` - Return current robot state as JSON
//! - `GET /` - Serve the phone web UI
//! - `WebSocket /ws/state` - Real-time state updates (500ms interval)
//!
//! # Contract Compliance
//! - **VCONV-003**: Voice processing on laptop, not CyberPi
//! - **VCONV-004**: Voice commands pass through SafetyFilter
//! - **VCONV-005**: API keys from env vars only (I-BRAIN-008)
//! - **VCONV-009**: Motor commands from voice clamped [-100, 100]

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use futures::{SinkExt, StreamExt};
use warp::Filter;

use base64::Engine;
use crate::brain::stt::SttChain;
use crate::brain::planner::BrainAction;
use crate::brain::planner::safety::SafetyFilter;
use mbot_core::MotorCommand;
use crate::transport::MBotTransport;

/// Snapshot of the robot's current state, broadcast to connected clients.
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct RobotState {
    pub mood: String,
    pub tension: f32,
    pub energy: f32,
    pub coherence: f32,
    pub curiosity: f32,
    pub speaking: bool,
    pub last_response: String,
    pub last_transcript: String,
}

/// Motor override set by voice commands, consumed by the main loop.
pub struct MotorOverride {
    pub cmd: MotorCommand,
    /// Duration in ms. Timer starts when the main loop first reads this override.
    pub duration_ms: u64,
    /// Set by the main loop on first read; None = not started yet.
    pub expires_at: Option<Instant>,
}

/// Shared state between voice API and main loop.
pub struct SharedVoiceState {
    /// Robot state (updated by main loop, read by voice API)
    pub robot: RobotState,
    /// Motor override from voice commands (set by voice API, read by main loop)
    pub motor_override: Option<MotorOverride>,
}

impl Default for SharedVoiceState {
    fn default() -> Self {
        Self {
            robot: RobotState {
                mood: "CALM".into(),
                ..Default::default()
            },
            motor_override: None,
        }
    }
}

/// Thread-safe handle to shared state.
pub type SharedState = Arc<Mutex<SharedVoiceState>>;

/// JSON body accepted by `POST /api/text`.
#[derive(Deserialize)]
pub struct TextRequest {
    pub text: String,
}

/// JSON body returned from `POST /api/voice`.
#[derive(Serialize)]
pub struct VoiceResponse {
    /// What the robot heard (STT transcription)
    pub transcript: String,
    /// What the robot says back
    pub text: String,
    /// Actions the robot is performing
    pub actions: Vec<String>,
    /// Current mood
    pub mood: String,
    /// Base64-encoded audio URL for TTS playback on phone (ElevenLabs)
    pub audio_url: Option<String>,
}

// ---------------------------------------------------------------------------
// ElevenLabs TTS
// ---------------------------------------------------------------------------

/// ElevenLabs text-to-speech client.
///
/// Requires `ELEVENLABS_API_KEY` env var (I-VCONV-005).
/// Voice ID from `MBOT_VOICE_ID` env var or defaults to "Rachel".
pub struct ElevenLabsTts {
    client: reqwest::Client,
    api_key: String,
    voice_id: String,
}

impl ElevenLabsTts {
    /// Create from environment variables. Returns None if ELEVENLABS_API_KEY is not set.
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("ELEVENLABS_API_KEY").ok()?;
        let voice_id = std::env::var("MBOT_VOICE_ID")
            .unwrap_or_else(|_| "21m00Tcm4TlvDq8ikWAM".into()); // Rachel
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .ok()?;
        tracing::info!("ElevenLabs TTS configured (voice: {})", voice_id);
        Some(Self { client, api_key, voice_id })
    }

    /// Generate speech audio from text. Returns MP3 bytes or None on failure.
    pub async fn generate(&self, text: &str) -> Option<Vec<u8>> {
        if text.is_empty() {
            return None;
        }

        let url = format!(
            "https://api.elevenlabs.io/v1/text-to-speech/{}",
            self.voice_id
        );

        let body = serde_json::json!({
            "text": text,
            "model_id": "eleven_monolingual_v1",
            "voice_settings": {
                "stability": 0.5,
                "similarity_boost": 0.75
            }
        });

        let resp = self.client
            .post(&url)
            .header("xi-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                match r.bytes().await {
                    Ok(bytes) => {
                        tracing::debug!("ElevenLabs TTS: {} bytes of audio", bytes.len());
                        Some(bytes.to_vec())
                    }
                    Err(e) => {
                        tracing::warn!("ElevenLabs TTS: failed to read body: {}", e);
                        None
                    }
                }
            }
            Ok(r) => {
                let status = r.status();
                let body = r.text().await.unwrap_or_default();
                tracing::warn!("ElevenLabs TTS error ({}): {}", status, body);
                None
            }
            Err(e) => {
                tracing::warn!("ElevenLabs TTS request failed: {}", e);
                None
            }
        }
    }
}

/// Default port the voice API listens on.
pub const DEFAULT_PORT: u16 = 8080;

/// Start the HTTP voice API server.
///
/// The server runs on a dedicated tokio task. State is shared with the main
/// loop via `Arc<Mutex<SharedVoiceState>>`.
pub async fn start_voice_api(
    state: SharedState,
    transport: Arc<Mutex<MBotTransport>>,
    stt: Arc<SttChain>,
    port: u16,
) -> tokio::task::JoinHandle<()> {
    let tts: Arc<Option<ElevenLabsTts>> = Arc::new(ElevenLabsTts::from_env());
    if tts.is_some() {
        tracing::info!("ElevenLabs TTS enabled - phone will play synthesized speech");
    } else {
        tracing::info!("ElevenLabs TTS disabled (set ELEVENLABS_API_KEY to enable)");
    }

    let state_get = state.clone();
    let state_post = state.clone();
    let state_text = state.clone();
    let state_ws = state.clone();
    let transport_post = transport.clone();
    let transport_text = transport.clone();
    let stt_post = stt.clone();
    let tts_post = tts.clone();
    let tts_text = tts.clone();

    // GET /api/state
    let get_state = warp::path!("api" / "state")
        .and(warp::get())
        .and(warp::any().map(move || state_get.clone()))
        .and_then(handle_get_state);

    // POST /api/voice
    let post_voice = warp::path!("api" / "voice")
        .and(warp::post())
        .and(warp::body::bytes())
        .and(warp::any().map(move || state_post.clone()))
        .and(warp::any().map(move || transport_post.clone()))
        .and(warp::any().map(move || stt_post.clone()))
        .and(warp::any().map(move || tts_post.clone()))
        .and_then(handle_post_voice);

    // POST /api/text - text commands (no mic needed)
    let post_text = warp::path!("api" / "text")
        .and(warp::post())
        .and(warp::body::json::<TextRequest>())
        .and(warp::any().map(move || state_text.clone()))
        .and(warp::any().map(move || transport_text.clone()))
        .and(warp::any().map(move || tts_text.clone()))
        .and_then(handle_post_text);

    // WebSocket /ws/state
    let ws_state = warp::path!("ws" / "state")
        .and(warp::ws())
        .and(warp::any().map(move || state_ws.clone()))
        .map(|ws: warp::ws::Ws, state: SharedState| {
            ws.on_upgrade(move |socket| handle_ws(socket, state))
        });

    // GET / - phone UI
    let index = warp::path::end()
        .and(warp::get())
        .map(|| warp::reply::html(PHONE_UI_HTML));

    // CORS
    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST"])
        .allow_headers(vec!["content-type"]);

    let routes = index
        .or(get_state)
        .or(post_voice)
        .or(post_text)
        .or(ws_state)
        .with(cors);

    let handle = tokio::spawn(async move {
        tracing::info!("Voice API server starting on 0.0.0.0:{}", port);
        tracing::info!("Open http://<laptop-ip>:{} on your phone", port);
        warp::serve(routes)
            .run(([0, 0, 0, 0], port))
            .await;
    });

    handle
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn handle_get_state(
    state: SharedState,
) -> Result<impl warp::Reply, warp::Rejection> {
    let s = state.lock().await;
    Ok(warp::reply::json(&s.robot))
}

async fn handle_post_voice(
    body: warp::hyper::body::Bytes,
    state: SharedState,
    transport: Arc<Mutex<MBotTransport>>,
    stt: Arc<SttChain>,
    tts: Arc<Option<ElevenLabsTts>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let audio_size = body.len();
    tracing::info!("Received voice audio: {} bytes", audio_size);

    if audio_size < 100 {
        return Ok(warp::reply::json(&VoiceResponse {
            transcript: String::new(),
            text: "Audio too short. Try holding the button longer.".into(),
            actions: vec![],
            mood: state.lock().await.robot.mood.clone(),
            audio_url: None,
        }));
    }

    // 1. Transcribe audio via STT chain
    let transcript = match stt.transcribe(&body, "webm").await {
        Ok(text) => {
            tracing::info!("Transcribed: \"{}\"", text);
            text
        }
        Err(e) => {
            tracing::warn!("STT failed: {}", e);
            let mood = state.lock().await.robot.mood.clone();
            return Ok(warp::reply::json(&VoiceResponse {
                transcript: String::new(),
                text: format!("Sorry, speech recognition failed: {}", e),
                actions: vec![],
                mood,
                audio_url: None,
            }));
        }
    };

    // 2. Parse voice command
    let result = parse_voice_command(&transcript);

    // 3. Safety filter (VCONV-004, VCONV-009)
    let safety = SafetyFilter::new();
    let safe_actions: Vec<BrainAction> = result.actions.iter()
        .filter_map(|a| safety.check(a.clone()))
        .collect();

    // 4. Execute actions
    let mut action_descriptions = Vec::new();

    for action in &safe_actions {
        match action {
            BrainAction::Motor(cmd) => {
                let duration = result.motor_duration_ms.unwrap_or(3000);
                {
                    let mut s = state.lock().await;
                    s.motor_override = Some(MotorOverride {
                        cmd: cmd.clone(),
                        duration_ms: duration,
                        expires_at: None, // timer starts on first main-loop read
                    });
                }
                action_descriptions.push(format!(
                    "Moving (L:{} R:{}) for {:.1}s",
                    cmd.left, cmd.right, duration as f64 / 1000.0
                ));
            }
            BrainAction::Speak(text) => {
                action_descriptions.push(format!("Speaking: {}", text));
            }
            _ => {}
        }
    }

    // 5. Skip robot_speak (R2-D2 tones) entirely — it blocks CyberPi's ESP32
    //    for ~120ms/char which drops the BLE connection. ElevenLabs TTS handles
    //    speech output through the phone/Telegram instead.

    // 6. Generate TTS audio for phone speaker (ElevenLabs)
    let audio_url = if let Some(ref tts_client) = *tts {
        let tts_text = result.robot_speech.as_deref()
            .unwrap_or(&result.response_text);
        match tts_client.generate(tts_text).await {
            Some(audio_bytes) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&audio_bytes);
                Some(format!("data:audio/mpeg;base64,{}", b64))
            }
            None => None,
        }
    } else {
        None
    };

    // 7. Update shared state
    {
        let mut s = state.lock().await;
        s.robot.last_transcript = transcript.clone();
        s.robot.last_response = result.response_text.clone();
    }

    // 8. Return response
    let mood = state.lock().await.robot.mood.clone();
    Ok(warp::reply::json(&VoiceResponse {
        transcript,
        text: result.response_text,
        actions: action_descriptions,
        mood,
        audio_url,
    }))
}

async fn handle_post_text(
    req: TextRequest,
    state: SharedState,
    transport: Arc<Mutex<MBotTransport>>,
    tts: Arc<Option<ElevenLabsTts>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let transcript = req.text.trim().to_string();
    tracing::info!("Text command: \"{}\"", transcript);

    if transcript.is_empty() {
        let mood = state.lock().await.robot.mood.clone();
        return Ok(warp::reply::json(&VoiceResponse {
            transcript: String::new(),
            text: "Please type a command.".into(),
            actions: vec![],
            mood,
            audio_url: None,
        }));
    }

    // Same pipeline as voice, but skip STT
    let result = parse_voice_command(&transcript);

    let safety = SafetyFilter::new();
    let safe_actions: Vec<BrainAction> = result.actions.iter()
        .filter_map(|a| safety.check(a.clone()))
        .collect();

    let mut action_descriptions = Vec::new();

    for action in &safe_actions {
        match action {
            BrainAction::Motor(cmd) => {
                let duration = result.motor_duration_ms.unwrap_or(3000);
                {
                    let mut s = state.lock().await;
                    s.motor_override = Some(MotorOverride {
                        cmd: cmd.clone(),
                        duration_ms: duration,
                        expires_at: None, // timer starts on first main-loop read
                    });
                }
                action_descriptions.push(format!(
                    "Moving (L:{} R:{}) for {:.1}s",
                    cmd.left, cmd.right, duration as f64 / 1000.0
                ));
            }
            BrainAction::Speak(text) => {
                action_descriptions.push(format!("Speaking: {}", text));
            }
            _ => {}
        }
    }

    // Skip robot_speak — blocks CyberPi ESP32 and drops BLE connection.
    // ElevenLabs TTS handles speech through phone/Telegram instead.

    // TTS for phone
    let audio_url = if let Some(ref tts_client) = *tts {
        let tts_text = result.robot_speech.as_deref()
            .unwrap_or(&result.response_text);
        match tts_client.generate(tts_text).await {
            Some(audio_bytes) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(&audio_bytes);
                Some(format!("data:audio/mpeg;base64,{}", b64))
            }
            None => None,
        }
    } else {
        None
    };

    {
        let mut s = state.lock().await;
        s.robot.last_transcript = transcript.clone();
        s.robot.last_response = result.response_text.clone();
    }

    let mood = state.lock().await.robot.mood.clone();
    Ok(warp::reply::json(&VoiceResponse {
        transcript,
        text: result.response_text,
        actions: action_descriptions,
        mood,
        audio_url,
    }))
}

async fn handle_ws(ws: warp::ws::WebSocket, state: SharedState) {
    use tokio::time::{interval, Duration};

    let (mut tx, mut _rx) = ws.split();
    let mut tick = interval(Duration::from_millis(500));

    loop {
        tick.tick().await;
        let s = state.lock().await;
        let json = serde_json::to_string(&s.robot).unwrap_or_default();
        if tx.send(warp::ws::Message::text(json)).await.is_err() {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// Voice Command Parser
// ---------------------------------------------------------------------------

struct VoiceCommandResult {
    actions: Vec<BrainAction>,
    response_text: String,
    robot_speech: Option<String>,
    motor_duration_ms: Option<u64>,
}

fn parse_voice_command(text: &str) -> VoiceCommandResult {
    let lower = text.to_lowercase();
    let trimmed = lower.trim();

    // Stop (highest priority)
    if trimmed.contains("stop") || trimmed.contains("halt") || trimmed == "enough" {
        return VoiceCommandResult {
            actions: vec![BrainAction::Motor(MotorCommand::default())],
            response_text: "Stopping!".into(),
            robot_speech: Some("OK".into()),
            motor_duration_ms: None,
        };
    }

    // Circle / spin
    if trimmed.contains("circle") || trimmed.contains("spin") || trimmed.contains("turn around") {
        return VoiceCommandResult {
            actions: vec![BrainAction::Motor(MotorCommand { left: 40, right: -40, ..Default::default() })],
            response_text: "Going in a circle!".into(),
            robot_speech: Some("Wheee!".into()),
            motor_duration_ms: Some(4000),
        };
    }

    // Forward
    if trimmed.contains("forward") || trimmed.contains("go ahead") || trimmed.contains("go straight") {
        return VoiceCommandResult {
            actions: vec![BrainAction::Motor(MotorCommand { left: 50, right: 50, ..Default::default() })],
            response_text: "Moving forward!".into(),
            robot_speech: Some("Going!".into()),
            motor_duration_ms: Some(3000),
        };
    }

    // Backward
    if trimmed.contains("back") || trimmed.contains("reverse") {
        return VoiceCommandResult {
            actions: vec![BrainAction::Motor(MotorCommand { left: -50, right: -50, ..Default::default() })],
            response_text: "Reversing!".into(),
            robot_speech: Some("Beep beep!".into()),
            motor_duration_ms: Some(3000),
        };
    }

    // Turn left
    if trimmed.contains("turn left") || trimmed.contains("go left") || trimmed == "left" {
        return VoiceCommandResult {
            actions: vec![BrainAction::Motor(MotorCommand { left: -30, right: 30, ..Default::default() })],
            response_text: "Turning left!".into(),
            robot_speech: None,
            motor_duration_ms: Some(1500),
        };
    }

    // Turn right
    if trimmed.contains("turn right") || trimmed.contains("go right") || trimmed == "right" {
        return VoiceCommandResult {
            actions: vec![BrainAction::Motor(MotorCommand { left: 30, right: -30, ..Default::default() })],
            response_text: "Turning right!".into(),
            robot_speech: None,
            motor_duration_ms: Some(1500),
        };
    }

    // Dance — spin left (opposite of circle) at high speed
    if trimmed.contains("dance") {
        return VoiceCommandResult {
            actions: vec![BrainAction::Motor(MotorCommand { left: -60, right: 60, ..Default::default() })],
            response_text: "Dance time!".into(),
            robot_speech: Some("Dance party!".into()),
            motor_duration_ms: Some(4000),
        };
    }

    // Say something explicit
    if let Some(after) = trimmed.strip_prefix("say ") {
        let speech = after.trim();
        if !speech.is_empty() {
            return VoiceCommandResult {
                actions: vec![BrainAction::Speak(speech.to_string())],
                response_text: format!("Saying: \"{}\"", speech),
                robot_speech: Some(speech.to_string()),
                motor_duration_ms: None,
            };
        }
    }

    // Greetings
    if trimmed.starts_with("hello") || trimmed.starts_with("hi") || trimmed.starts_with("hey") {
        return VoiceCommandResult {
            actions: vec![BrainAction::Speak("Hello! Nice to meet you!".into())],
            response_text: "Hello there!".into(),
            robot_speech: Some("Hello! Nice to meet you!".into()),
            motor_duration_ms: None,
        };
    }

    // Identity
    if trimmed.contains("your name") || trimmed.contains("who are you") {
        return VoiceCommandResult {
            actions: vec![BrainAction::Speak("I am mBot!".into())],
            response_text: "I am mBot!".into(),
            robot_speech: Some("I am mBot!".into()),
            motor_duration_ms: None,
        };
    }

    // How are you
    if trimmed.contains("how are you") || trimmed.contains("how do you feel") {
        return VoiceCommandResult {
            actions: vec![BrainAction::Noop],
            response_text: "I'm doing great, thanks for asking!".into(),
            robot_speech: Some("I feel great!".into()),
            motor_duration_ms: None,
        };
    }

    // Thank you
    if trimmed.contains("thank") {
        return VoiceCommandResult {
            actions: vec![BrainAction::Noop],
            response_text: "You're welcome!".into(),
            robot_speech: Some("You're welcome!".into()),
            motor_duration_ms: None,
        };
    }

    // Default: echo back
    let short = if text.len() > 40 { &text[..40] } else { text };
    VoiceCommandResult {
        actions: vec![BrainAction::Noop],
        response_text: format!("I heard: \"{}\". Try: forward, circle, dance, stop", short),
        robot_speech: Some("Hmm".into()),
        motor_duration_ms: None,
    }
}

// ---------------------------------------------------------------------------
// Phone UI HTML
// ---------------------------------------------------------------------------

const PHONE_UI_HTML: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, user-scalable=no">
<title>mBot Voice</title>
<style>
*{box-sizing:border-box;margin:0;padding:0}
body{font-family:-apple-system,BlinkMacSystemFont,sans-serif;background:#0d1117;color:#e6edf3;
  min-height:100vh;display:flex;flex-direction:column;overflow-x:hidden}
.hdr{text-align:center;padding:12px;background:#161b22;border-bottom:1px solid #30363d}
.hdr h1{font-size:18px;font-weight:600}
.conn{display:inline-flex;align-items:center;gap:6px;font-size:11px;color:#8b949e;margin-top:2px}
.dot{width:8px;height:8px;border-radius:50%;background:#f85149}
.dot.on{background:#3fb950}
.mood-area{text-align:center;padding:16px 12px 8px}
.mood-emoji{font-size:56px;line-height:1}
.mood-label{font-size:14px;margin-top:4px;color:#8b949e}
.bars{display:flex;justify-content:center;gap:20px;margin-top:8px}
.bar-item{text-align:center;font-size:10px;color:#8b949e}
.bar-bg{height:4px;width:50px;background:#21262d;border-radius:2px;margin-top:3px;overflow:hidden}
.bar-fg{height:100%;border-radius:2px;transition:width .5s}
.bar-e .bar-fg{background:#3fb950}
.bar-t .bar-fg{background:#f85149}
.bar-c .bar-fg{background:#58a6ff}
.chat{flex:1;padding:12px;display:flex;flex-direction:column;gap:10px;overflow-y:auto;
  min-height:120px;max-height:40vh}
.msg{max-width:85%;padding:10px 14px;border-radius:14px;font-size:14px;line-height:1.4;
  animation:fadeIn .3s}
@keyframes fadeIn{from{opacity:0;transform:translateY(6px)}to{opacity:1}}
.msg.user{align-self:flex-end;background:#1f6feb;border-bottom-right-radius:4px}
.msg.bot{align-self:flex-start;background:#21262d;border:1px solid #30363d;border-bottom-left-radius:4px}
.msg .lbl{font-size:10px;color:#8b949e;margin-bottom:3px}
.msg .act{margin-top:6px;padding-top:6px;border-top:1px solid #30363d;font-size:12px;color:#58a6ff}
.ctrl{padding:16px 12px 28px;text-align:center;background:#161b22;border-top:1px solid #30363d}
#talk{width:76px;height:76px;border-radius:50%;border:3px solid #30363d;background:#21262d;
  color:#e6edf3;font-size:28px;cursor:pointer;transition:all .2s;
  display:inline-flex;align-items:center;justify-content:center;-webkit-user-select:none}
#talk:active,#talk.rec{background:#f85149;border-color:#f85149;transform:scale(1.12);
  box-shadow:0 0 24px rgba(248,81,73,.4)}
#talk.proc{background:#8957e5;border-color:#8957e5;animation:pulse 1s infinite}
@keyframes pulse{0%,100%{box-shadow:0 0 8px rgba(137,87,229,.3)}50%{box-shadow:0 0 24px rgba(137,87,229,.5)}}
.hint{font-size:11px;color:#8b949e;margin-top:6px}
#err{font-size:11px;color:#f85149;margin-top:6px}
.cmds{font-size:11px;color:#484f58;margin-top:8px}
</style>
</head>
<body>
<div class="hdr">
  <h1>mBot Voice</h1>
  <div class="conn"><span class="dot" id="dot"></span><span id="conn">Connecting...</span></div>
</div>
<div class="mood-area">
  <div class="mood-emoji" id="emoji">&#x1F916;</div>
  <div class="mood-label" id="mlbl">CALM</div>
  <div class="bars">
    <div class="bar-item bar-e"><span>Energy</span><div class="bar-bg"><div class="bar-fg" id="be" style="width:50%"></div></div></div>
    <div class="bar-item bar-t"><span>Tension</span><div class="bar-bg"><div class="bar-fg" id="bt" style="width:20%"></div></div></div>
    <div class="bar-item bar-c"><span>Curiosity</span><div class="bar-bg"><div class="bar-fg" id="bc" style="width:50%"></div></div></div>
  </div>
</div>
<div class="chat" id="chat">
  <div class="msg bot"><div class="lbl">mBot</div>Hold the mic button and talk to me!<div class="act">Try: "go forward", "circle", "dance", "stop"</div></div>
</div>
<div class="ctrl">
  <button id="talk" data-testid="talk-btn">&#x1F3A4;</button>
  <div class="hint" id="hint">Hold to talk</div>
  <div id="err"></div>
  <div style="display:flex;gap:8px;margin:12px auto 0;max-width:320px">
    <input id="txt" type="text" placeholder="Type command..." data-testid="text-input"
      style="flex:1;padding:10px 14px;border-radius:20px;border:1px solid #30363d;background:#0d1117;color:#e6edf3;font-size:14px;outline:none">
    <button id="send" data-testid="send-btn"
      style="width:44px;height:44px;border-radius:50%;border:none;background:#1f6feb;color:#fff;font-size:18px;cursor:pointer">&#x27A4;</button>
  </div>
  <div class="cmds">forward &bull; back &bull; left &bull; right &bull; circle &bull; spin &bull; dance &bull; stop &bull; say ...</div>
</div>
<script>
const O=window.location.origin;
let MR,chunks=[];
const talk=document.getElementById('talk'),
      chat=document.getElementById('chat'),
      hint=document.getElementById('hint'),
      err=document.getElementById('err'),
      emoji=document.getElementById('emoji'),
      mlbl=document.getElementById('mlbl'),
      dot=document.getElementById('dot'),
      conn=document.getElementById('conn');

const MOODS={CALM:'\u{1F916}',ACTIVE:'\u{1F914}',SPIKE:'\u{1F631}',PROTECT:'\u{1F628}'};

// WebSocket
let ws;
function wsConnect(){
  const u=O.replace('http','ws')+'/ws/state';
  ws=new WebSocket(u);
  ws.onopen=()=>{dot.classList.add('on');conn.textContent='Connected'};
  ws.onclose=()=>{dot.classList.remove('on');conn.textContent='Reconnecting...';setTimeout(wsConnect,2000)};
  ws.onmessage=e=>{try{
    const s=JSON.parse(e.data);
    const m=s.mood||'CALM';
    mlbl.textContent=m;
    emoji.textContent=MOODS[m]||'\u{1F916}';
    document.getElementById('be').style.width=(s.energy*100)+'%';
    document.getElementById('bt').style.width=(s.tension*100)+'%';
    document.getElementById('bc').style.width=(s.curiosity*100)+'%';
  }catch(x){}};
}
wsConnect();

function addMsg(who,text,action){
  const d=document.createElement('div');
  d.className='msg '+(who==='user'?'user':'bot');
  let h='<div class="lbl">'+(who==='user'?'You':'mBot')+'</div>'+text;
  if(action)h+='<div class="act">'+action+'</div>';
  d.innerHTML=h;
  chat.appendChild(d);
  chat.scrollTop=chat.scrollHeight;
}

// Voice recording
talk.addEventListener('touchstart',startRec,{passive:false});
talk.addEventListener('mousedown',startRec);
talk.addEventListener('touchend',stopRec,{passive:false});
talk.addEventListener('mouseup',stopRec);

async function startRec(e){
  e.preventDefault();err.textContent='';
  try{
    const stream=await navigator.mediaDevices.getUserMedia({audio:true});
    MR=new MediaRecorder(stream,{mimeType:'audio/webm;codecs=opus'});
    chunks=[];
    MR.ondataavailable=e=>chunks.push(e.data);
    MR.start();
    talk.classList.add('rec');
    hint.textContent='Listening...';
  }catch(x){err.textContent='Mic denied: '+x.message}
}

async function stopRec(e){
  e.preventDefault();
  if(!MR||MR.state!=='recording')return;
  talk.classList.remove('rec');
  talk.classList.add('proc');
  hint.textContent='Thinking...';
  MR.stop();
  MR.onstop=async()=>{
    const blob=new Blob(chunks,{type:'audio/webm'});
    try{
      const r=await fetch(O+'/api/voice',{method:'POST',body:blob});
      const d=await r.json();
      if(d.transcript)addMsg('user',d.transcript);
      let act=d.actions&&d.actions.length?d.actions.join(', '):null;
      addMsg('bot',d.text,act);
      if(d.mood){mlbl.textContent=d.mood;emoji.textContent=MOODS[d.mood]||'\u{1F916}'}
    }catch(x){err.textContent='Error: '+x.message}
    talk.classList.remove('proc');
    hint.textContent='Hold to talk';
    MR.stream.getTracks().forEach(t=>t.stop());
  };
}

// Text input
const txtIn=document.getElementById('txt'),sendBtn=document.getElementById('send');
async function sendText(){
  const t=txtIn.value.trim();
  if(!t)return;
  addMsg('user',t);
  txtIn.value='';
  sendBtn.disabled=true;
  try{
    const r=await fetch(O+'/api/text',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({text:t})});
    const d=await r.json();
    let act=d.actions&&d.actions.length?d.actions.join(', '):null;
    addMsg('bot',d.text,act);
    if(d.audio_url){try{new Audio(d.audio_url).play()}catch(x){}}
    if(d.mood){mlbl.textContent=d.mood;emoji.textContent=MOODS[d.mood]||'\u{1F916}'}
  }catch(x){err.textContent='Error: '+x.message}
  sendBtn.disabled=false;
}
sendBtn.addEventListener('click',sendText);
txtIn.addEventListener('keydown',e=>{if(e.key==='Enter')sendText()});
</script>
</body>
</html>
"##;
