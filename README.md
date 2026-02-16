# mBot RuVector

**Give your robot a nervous system. Watch it come alive.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com)

---

<img width="718" height="676" alt="image" src="https://github.com/user-attachments/assets/6093ccbc-df03-41b6-9d3d-720fc543f1b1" />

## The Big Why

Most educational robots follow scripts. You tell them what to do and they do it. There is no surprise, no personality, no life.

**mBot RuVector** is different. It gives a $100 Makeblock mBot2 a nervous system - a real one, modeled after biological homeostasis. The robot doesn't execute instructions. It *experiences* its environment and *reacts* based on its internal state. It gets nervous when you move too fast. Curious about new objects. Tired when it has been running too long. It has a personality that emerges from how it processes the world, not from a script someone wrote.

```
Traditional Robot:  IF distance < 10cm THEN reverse()
RuVector Robot:     Sensor --> Nervous System --> Emergent Behavior
```

The difference is **surprise**. The robot does things you didn't program.

---

## Where RuVector Fits In

[RuVector](https://github.com/ruvnet/ruvector) is the AI engine underneath. Think of it like this:

- **mBot2** is the body (motors, sensors, LEDs, wheels)
- **RuVector** is the nervous system (processes sensation into behavior)
- **mBot RuVector** is the complete creature (body + nervous system + personality)

RuVector provides a DAG-based processing pipeline that runs a homeostasis loop: sensors feed into a nervous system that maintains internal balance across tension, coherence, energy, and curiosity. Different balance points create different personalities. The same hardware behaves completely differently depending on how its nervous system is tuned.

The companion app on your laptop runs the RuVector nervous system and communicates with the physical mBot2 over USB serial or Bluetooth. An optional **brain layer** adds LLM reasoning (Claude or Ollama), voice interaction, chat channels, memory, and proactive behavior on top of the deterministic nervous system.

---

## What Can It Do

### ArtBot - It Draws What It Feels
Attach a pen. The robot draws art based on its emotional state. Calm produces spirals. Startled produces jagged lines. Every drawing is a snapshot of its inner experience.

### Personality Pets - Same Robot, Different Soul
15 preset personalities. **Curious Cleo** investigates everything. **Nervous Nellie** is scared of sudden movements. **Grumpy Gus** does NOT want to play (but secretly does). 9 personality quirks add extra flavor.

### GameBot - Real Play
Tic-tac-toe where the robot thinks. Chase where it tries to catch you. Games with actual emotional stakes.

### HelperBot - Chores With Character
LEGO sorter that gets excited about rare pieces. Tasks become entertainment.

### LearningLab - Touch AI
Watch the nervous system fire in real-time. Adjust parameters, see behavior change. AI education you can feel.

### Brain Layer (New)
Give your robot a mind. LLM-powered reasoning (Claude API or Ollama local), voice conversation (speak to it, hear it respond), chat channels (Telegram, Discord), persistent memory (it remembers yesterday), and proactive behavior (it says good morning, offers to play when bored).

### Voice API + Telegram (New)
Control your mBot2 from Telegram or any browser. Send text commands ("forward", "dance", "hello") and the robot moves. ElevenLabs gives it a realistic voice. No app install needed - just open a browser on your phone or message the Telegram bot.

---

## Quick Start: Bluetooth + Telegram (Most Fun Setup)

This is the fastest path to controlling your mBot2 from your phone via Telegram.
No soldering, no special hardware, no app install. Just the mBot2, your laptop, and Telegram.

**Time needed:** About 15 minutes (most of it is the first Rust compile).

### Prerequisites

| You need | How to get it |
|----------|---------------|
| Makeblock mBot2 (powered on, home screen showing) | Press the round power button. Wait 5 seconds for the home screen |
| A laptop running Ubuntu/Debian Linux | Other Linux works too. macOS is untested but should work |
| Rust installed | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` then reopen terminal |
| `libdbus-1-dev` installed | `sudo apt install libdbus-1-dev pkg-config` |
| Python 3 (for Telegram bridge) | Already installed on most Linux. Check: `python3 --version` |
| A Telegram bot token | Open Telegram, message @BotFather, send `/newbot`, follow prompts, copy the token |

### Step 1: Clone and Set Up .env

```bash
git clone https://github.com/Hulupeep/mbot_ruvector.git
cd mbot_ruvector
cp .env.example .env
```

Open `.env` in any text editor. Uncomment and fill in:

```bash
# REQUIRED for Telegram control:
MBOT_TELEGRAM_TOKEN=paste-your-bot-token-here

# OPTIONAL but recommended (gives the robot a voice):
ELEVENLABS_API_KEY=paste-your-elevenlabs-key-here
```

Save the file.

### Step 2: Build

```bash
cargo build --bin mbot-companion --features "brain,voice-api,bluetooth"
```

First build takes 2-5 minutes. Go get a coffee. Subsequent builds take about 3 seconds.

**If the build fails** with "libdbus" or "pkg-config" errors:
```bash
sudo apt install libdbus-1-dev pkg-config build-essential libssl-dev
```
Then try the build again.

### Step 3: Turn On the mBot2

1. Press the round power button on top of CyberPi
2. Wait about 5 seconds until the home screen appears (shows icons)
3. **Important:** Make sure no mBlock program is running. If you see a program running, press the Home button (round button) to go back to the home screen

### Step 4: Start the Companion App

Open a terminal and run:

```bash
cargo run --bin mbot-companion --features "brain,voice-api,bluetooth" -- \
  --bluetooth --voice-api --voice-port 8088
```

Watch the output. You should see these lines appear (in order):

```
Scanning for mBot2 via BLE...
```

Wait about 5-10 seconds...

```
BLE connected to Makeblock_LE...
CyberPi f3/f4 protocol ready
Voice API server started on port 8088
```

**If "Scanning" hangs for more than 30 seconds:**
- Is the mBot2 powered on and on the home screen?
- Is Bluetooth enabled on your laptop? Check: `bluetoothctl show | grep Powered`
- Try: `sudo systemctl restart bluetooth` then restart the app

**Leave this terminal running.** Open a second terminal for the next step.

### Step 5: Start the Telegram Bridge

In a new terminal:

```bash
cd mbot_ruvector
python3 -u tools/telegram_bridge.py
```

You should see:

```
Telegram bot @YourBotName connected!
Voice API: http://localhost:8088
Send messages to the bot in Telegram. Ctrl+C to stop.
```

### Step 6: Send Commands from Telegram

1. Open Telegram on your phone
2. Search for your bot name (the one you created with @BotFather)
3. Send `/start` to see the help message
4. Send `forward` - the robot drives forward for 3 seconds
5. Send `dance` - the robot spins
6. Send `hello` - the robot says hello (with ElevenLabs voice if configured)

### Available Telegram Commands

| Command | What the robot does |
|---------|-------------------|
| `forward` | Drives straight ahead for 3 seconds |
| `back` | Reverses for 3 seconds |
| `turn left` | Spins left for 1.5 seconds |
| `turn right` | Spins right for 1.5 seconds |
| `circle` | Spins right slowly for 4 seconds |
| `dance` | Spins left fast for 4 seconds |
| `stop` | Stops all movement |
| `hello` | Says "Hello! Nice to meet you!" |
| `say <anything>` | Says whatever you type after "say" |
| `how are you` | Robot tells you how it feels |

### Step 7: Control From Your Phone Browser (Alternative to Telegram)

You can also control the robot from any browser without Telegram:

1. Find your laptop's IP: `hostname -I | awk '{print $1}'`
2. On your phone, open a browser and go to: `http://<your-laptop-ip>:8088`
3. Type commands in the text box and press Send
4. If you have ElevenLabs configured, you'll hear the robot's voice responses

### Stopping Everything

1. In the Telegram bridge terminal: press `Ctrl+C`
2. In the companion terminal: press `Ctrl+C`
3. Turn off the mBot2 by holding the power button for 2 seconds

---

## Alternative: Getting Started Without Bluetooth

If you don't have Bluetooth or prefer a USB cable, see the full step-by-step below.

## Getting Started (Step by Step)

This section assumes you have never used Rust or the command line before. Every step is spelled out. Take your time.

### Step 1: Open a Terminal

**On Linux (Ubuntu/Debian):** Press `Ctrl + Alt + T`. A black window with white text appears. This is your terminal.

**On macOS:** Open Spotlight (Cmd + Space), type `Terminal`, press Enter.

**On Windows:** Press the Windows key, type `cmd`, press Enter. (Or install [Windows Terminal](https://aka.ms/terminal) for a better experience.)

You should see a blinking cursor. That is where you type commands.

### Step 2: Install Rust

Rust is the programming language this project is written in. Copy and paste this entire line into your terminal and press Enter:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

It will ask you a question. Type `1` and press Enter to accept the defaults.

When it finishes, close your terminal and open a new one (this loads Rust into your path).

Verify it worked by typing:

```bash
rustc --version
```

You should see something like `rustc 1.XX.X`. If you see "command not found", close and reopen the terminal.

### Step 3: Install System Libraries

These are packages your operating system needs so Rust can talk to USB devices and Bluetooth.

**Ubuntu/Debian Linux:**
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libdbus-1-dev libudev-dev libssl-dev
```

It will ask for your password. Type it (nothing will appear on screen, that is normal) and press Enter.

**macOS:**
```bash
xcode-select --install
```

**Windows:** Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/). Select "Desktop development with C++".

### Step 4: Download This Project

```bash
git clone https://github.com/Hulupeep/mbot_ruvector.git
cd mbot_ruvector
```

If you see "git: command not found", install git first:
- Linux: `sudo apt install git`
- macOS: It installs automatically when you run the command above
- Windows: Download from [git-scm.com](https://git-scm.com/)

### Step 5: Test Without a Robot (Simulation)

You do not need an mBot2 to try this. Run:

```bash
cargo run --bin mbot-companion -- --simulate
```

The first time takes a few minutes because Rust downloads and compiles everything. Subsequent runs are fast.

You should see a status display updating every second showing the robot's nervous system: reflex mode, tension, coherence, energy, and simulated sensor readings. Press `Ctrl + C` to stop.

### Step 6: Connect a Real Robot

1. Turn on your mBot2
2. Connect it to your computer with a USB-C cable
3. Find the serial port:

```bash
# Linux
ls /dev/ttyUSB* /dev/ttyACM*

# macOS
ls /dev/tty.usb*
```

4. On Linux, give yourself permission to use the port:

```bash
sudo chmod 666 /dev/ttyUSB0
```

5. Run the companion:

```bash
cargo run --features serial --bin mbot-companion -- --serial /dev/ttyUSB0
```

Replace `/dev/ttyUSB0` with whatever port you found in step 3.

### Step 7: Start the Web Dashboard (Optional)

```bash
cd web
npm install
npm start
```

Open `http://localhost:3000` in your browser to see the nervous system visualized in real-time.

---

## Enabling the Voice API (Without Full Brain)

The Voice API gives your robot a web UI and HTTP endpoint for text commands. It does not require any API keys for basic motor control.

```bash
# Bluetooth + Voice API (no API keys needed for motor commands)
cargo run --bin mbot-companion --features "brain,voice-api,bluetooth" -- \
  --bluetooth --voice-api --voice-port 8088
```

Open `http://localhost:8088` in your browser. Type `forward` and press Send.

To add ElevenLabs voice (so the robot speaks through your phone/browser):
1. Sign up at https://elevenlabs.io and copy your API key
2. Add to `.env`: `ELEVENLABS_API_KEY=your-key-here`
3. Restart the companion app

---

## Enabling the Brain Layer

The brain layer adds LLM reasoning on top of the deterministic nervous system. It is optional. Without it, the robot still has its full nervous system, personality, and all applications.

### Basic Brain (Ollama, runs locally, free)

1. Install [Ollama](https://ollama.ai/) and pull a model:

```bash
ollama pull llama3.2
```

2. Run with brain enabled:

```bash
cargo run --features "serial,brain" --bin mbot-companion -- --serial /dev/ttyUSB0 --brain
```

The robot will now occasionally "think" using the local LLM and generate speech, motor actions, or personality adjustments.

### Brain + Claude API (better reasoning)

1. Get an API key from [console.anthropic.com](https://console.anthropic.com/)
2. Set the environment variable:

```bash
export ANTHROPIC_API_KEY="sk-ant-your-key-here"
```

3. Run (Claude is tried first, Ollama is the fallback):

```bash
cargo run --features "serial,brain" --bin mbot-companion -- --serial /dev/ttyUSB0 --brain
```

### Brain + Voice (talk to your robot)

Requires API keys for speech services:

```bash
export OPENAI_API_KEY="sk-your-key"         # For Whisper speech-to-text
export ELEVENLABS_API_KEY="your-key"        # For text-to-speech
```

```bash
cargo run --features "serial,brain,voice" --bin mbot-companion -- --serial /dev/ttyUSB0 --brain --voice
```

Speak into your microphone. The robot transcribes what you say, thinks about it, and responds in a voice that matches its personality.

### Brain + Chat Channels

**Telegram:**
```bash
export MBOT_TELEGRAM_TOKEN="your-bot-token"
cargo run --features "serial,brain,telegram" --bin mbot-companion -- --serial /dev/ttyUSB0 --brain
```

**Discord:**
```bash
export MBOT_DISCORD_TOKEN="your-bot-token"
cargo run --features "serial,brain,discord" --bin mbot-companion -- --serial /dev/ttyUSB0 --brain
```

Chat with your robot from your phone. It responds in character, remembers your conversations, and its personality colors every reply.

---

## All Features at a Glance

| Feature Flag | What It Adds | System Dependencies |
|---|---|---|
| *(none)* | Simulation mode, full nervous system | None |
| `serial` | USB serial connection to mBot2 | `libudev-dev` on Linux |
| `bluetooth` | Bluetooth connection to mBot2 | `libdbus-1-dev` on Linux |
| `brain` | LLM reasoning, memory, autonomy engine | None (HTTP-based) |
| `voice` | Microphone input, speech output | Audio libraries (cpal/rodio) |
| `voice-api` | HTTP server + phone UI + REST API | Requires `brain` |
| `telegram` | Telegram chat bot | Requires `brain` |
| `discord` | Discord chat bot | Requires `brain` |

### CLI Flags

| Flag | Description |
|---|---|
| `--simulate` | Run without hardware (testing/demo) |
| `--serial <port>` | Connect via USB serial |
| `--bluetooth` | Connect via Bluetooth |
| `--freq <hz>` | Control loop frequency (default: 20) |
| `--draw` | Enable pen servo for drawing mode |
| `--brain` | Enable LLM-powered brain layer |
| `--voice` | Enable voice pipeline (mic + speaker) |
| `--voice-api` | Start HTTP voice API server (web UI + REST) |
| `--voice-port <port>` | Port for voice API (default: 8080) |
| `--speak "<text>"` | Test R2-D2 speech on CyberPi and exit |
| `-v, --verbose` | Show debug output |

### Environment Variables

| Variable | Used By | Purpose |
|---|---|---|
| `ANTHROPIC_API_KEY` | Brain (Claude) | Claude API authentication |
| `MBOT_CLAUDE_MODEL` | Brain (Claude) | Model name (default: claude-sonnet-4-20250514) |
| `OPENAI_API_KEY` | Voice (STT) | Whisper speech-to-text |
| `GROQ_API_KEY` | Voice (STT) | Groq Whisper (faster alternative) |
| `ELEVENLABS_API_KEY` | Voice (TTS) | ElevenLabs text-to-speech |
| `MBOT_OLLAMA_URL` | Brain (Ollama) | Ollama server URL (default: http://localhost:11434) |
| `MBOT_OLLAMA_MODEL` | Brain (Ollama) | Model name (default: llama3.2) |
| `MBOT_TELEGRAM_TOKEN` | Telegram | Bot token from BotFather |
| `MBOT_DISCORD_TOKEN` | Discord | Bot token from Discord Developer Portal |
| `MBOT_LLM_TIMEOUT_SECS` | Brain | Max LLM request time (default: 30, max: 30) |
| `MBOT_DB_PATH` | Memory | SQLite database location |

---

## The Four Reflex Modes

The robot's nervous system has four modes that emerge from experience:

| Mode | What Triggers It | How It Behaves |
|------|------------------|----------------|
| **Calm** | Low tension, stability | Gentle, flowing, content |
| **Active** | Curiosity, novelty | Exploring, seeking, alert |
| **Spike** | Sudden change | Quick reactions, startled |
| **Protect** | Threat detected | Defensive, cautious, retreating |

These are not programmed states. They **emerge** from the homeostasis system balancing tension, coherence, and energy.

---

## Personalities

Same robot. Wildly different behaviors. 15 presets are included:

| Personality | Vibe | Key Trait |
|-------------|------|-----------|
| **Curious Cleo** | "What's THAT?!" | High curiosity drive |
| **Nervous Nellie** | "Is that safe?" | High startle sensitivity |
| **Chill Charlie** | "Whatever." | Low reactivity |
| **Bouncy Betty** | "LET'S GO!" | High energy baseline |
| **Grumpy Gus** | "Ugh, fine." | Low coherence, reluctant |
| **Zen Master** | "Breathe." | Perfect calm |
| **Brave Explorer** | "Let's find out!" | High curiosity, low startle |
| **Drama Queen** | "THIS IS HUGE!" | Maximum expressiveness |
| **Shy Scholar** | "I'll just watch." | High curiosity, very reserved |
| **Party Animal** | "WOOOO!" | Maximum energy, maximum expression |

Plus 9 personality quirks: `VictoryDance`, `NervousHum`, `CuriousWiggle`, `StartlePause`, `HappySpin`, `SleepyDrift`, `ProtestGrumble`, `ExcitedBeep`, `CalmPurr`.

Create your own with the Personality Mixer in the web dashboard.

---

## Architecture

```
                    mBot2 RuVector System
 ================================================================

   mBot2 (CyberPi)              Companion App (Laptop)
  +-----------------+          +---------------------------+
  | Sensors:        |   USB/   | Deterministic Layer:      |
  |  - Ultrasonic   |   BT     |  - Homeostasis engine     |
  |  - Sound level  | <------> |  - Reflex mode selection  |
  |  - Light level  |          |  - Motor command output   |
  |  - Gyroscope    |          |  - Personality system     |
  |  - Encoders     |          |                           |
  |                 |          | Brain Layer (optional):   |
  | Actuators:      |          |  - LLM reasoning          |
  |  - Left motor   |          |  - SQLite memory          |
  |  - Right motor  |          |  - Autonomy engine        |
  |  - 8x RGB LEDs  |          |  - Voice pipeline         |
  |  - Buzzer       |          |  - Chat channels          |
  |  - Pen servo    |          |  - Personality narrator   |
  +-----------------+          +---------------------------+
                                          |
                               +----------+---------+
                               |  Web Dashboard     |
                               |  localhost:3000    |
                               |  - Neural viz      |
                               |  - Personality mix  |
                               +--------------------+
```

### Crate Structure

```
mbot_ruvector/
+-- crates/
|   +-- mbot-core/            # Nervous system (no_std, deterministic)
|   |   +-- homeostasis.rs    # Tension/coherence/energy balance
|   |   +-- personality/      # 15 presets, 9 quirks, bounded params
|   |   +-- learning/         # Q-learning, SONA adaptation
|   |   +-- drawing/          # Mood-to-movement art generation
|   |
|   +-- mbot-companion/       # Laptop app (full std, async)
|   |   +-- transport/        # Serial, Bluetooth, Simulated
|   |   +-- protocol/         # CyberPi communication
|   |   +-- brain/            # LLM integration (feature-gated)
|   |   |   +-- llm/          # Claude + Ollama providers
|   |   |   +-- planner/      # State -> LLM -> BrainAction
|   |   |   +-- memory/       # SQLite persistence
|   |   |   +-- channels/     # Telegram, Discord
|   |   |   +-- autonomy/     # Proactive behaviors
|   |   |   +-- voice/        # STT + TTS pipeline
|   |   |   +-- narrator/     # Personality-colored speech
|   |   +-- websocket_v2/     # Dashboard communication
|   |
|   +-- mbot-embedded/        # Direct ESP32 deployment (WIP)
|
+-- web/                      # Browser dashboard
+-- docs/                     # Specs, contracts, guides
```

### Safety Architecture

The deterministic nervous system in `mbot-core` **always runs**, even when the brain layer is active. The brain layer is advisory - it suggests actions, but a `SafetyFilter` checks every suggestion before execution:

- Motor speeds clamped to [-100, 100]
- Harmful speech blocked
- Personality changes limited to small deltas (max 0.1 per adjustment)
- LLM requests timeout at 30 seconds max
- If all LLM providers fail, the robot continues on its deterministic nervous system

This is the **Kitchen Table Test**: would you be happy if your 7-year-old played with this while grandma watched? Every feature must pass that test.

---

## Extending the System

### Adding a Camera / Video

The architecture supports adding video through the brain layer. You would:

1. Add a camera capture crate (e.g., `nokhwa` for webcam, or a Raspberry Pi camera library)
2. Create a new module `brain/vision/` alongside the existing `brain/voice/`
3. Feed frames to a vision model (Claude's vision API accepts images)
4. The planner already handles multi-modal input - add image context to the prompt builder

Example use cases: object recognition for smarter LEGO sorting, face detection for greeting behavior, obstacle mapping for navigation.

### Adding New Sensors

The `MBotSensors` struct in `mbot-core` defines what sensor data flows through the system. To add a new sensor:

1. Add the field to `MBotSensors` (e.g., `temperature: f32`)
2. Update the transport layer to read the new sensor
3. The homeostasis engine and brain layer automatically receive the new data
4. Optionally add it to the prompt builder so the LLM knows about it

Compatible sensors include: temperature, humidity, air quality, IR proximity arrays, GPS, IMU (6-axis), color cameras, LIDAR.

### Adding New LLM Providers

The `LlmProvider` trait is designed for extension. To add a new provider (e.g., Google Gemini, Mistral, local llama.cpp):

1. Create `brain/llm/your_provider.rs`
2. Implement the `LlmProvider` trait (4 methods: `complete`, `complete_streaming`, `model_name`, `is_available`)
3. Add it to the `ProviderChain` in `main.rs`
4. The chain tries providers in order and falls back automatically

### Adding New Chat Channels

The `ChatChannel` trait supports any messaging platform. To add one (e.g., Slack, Matrix, SMS):

1. Create `brain/channels/your_channel.rs`
2. Implement `ChatChannel` (methods: `start`, `stop`, `send`, `channel_type`)
3. Register with the `MessageRouter`
4. The rate limiter (20 msg/min per user) applies automatically

### Adding New Proactive Behaviors

The autonomy engine accepts any action that implements the pattern:

1. Create a struct with a `should_trigger()` method (checks context: time, idle duration, sensor state)
2. Register it with the `AutonomyEngine`
3. It will be evaluated on each tick and executed when conditions are met
4. Built-in cooldowns prevent spamming

Existing behaviors: `GoodMorning` (greets on startup), `InactivityCheck` (offers to play after 5 min idle), `IdleOffer` (suggests activities when it hears you nearby).

### Adding New Games

Games in mbot-companion are standalone binaries. To add a new game:

1. Create `src/bin/your_game.rs`
2. Use `mbot-core` for personality and nervous system
3. Use the transport layer for motor/sensor control
4. Add a `[[bin]]` entry in `Cargo.toml`

---

## The No Bad Stuff Manifesto

This project exists for **joy**. Period.

**We build for:** Wonder and surprise. Learning through play. Connection and companionship. Creative expression. All ages, all backgrounds.

**We never build:** Weapons or harm. Surveillance or tracking. Manipulation or deception. Anything that would scare a kid. "Creepy" behaviors.

---

## Complete Documentation

| Document | What It Covers |
|---|---|
| **[Master Setup Guide](docs/MASTER_GUIDE.md)** | Hardware setup, installation, connection, deployment |
| **[Application Guides](docs/APP_GUIDES.md)** | Detailed guide for each of the 6 applications |
| **[Product Vision](docs/PRD.md)** | Full product requirements and roadmap |
| **[Web Dashboard](web/README.md)** | Dashboard features and real-time visualization |

---

## Using an LLM to Build, Test, and Debug

This project is designed to be worked on with an LLM coding assistant (Claude Code, Cursor, Copilot, etc.). The LLM can read error messages, suggest fixes, run tests, and explain what's happening.

**Examples of things to ask your LLM:**

| What you want | What to ask |
|---------------|-------------|
| Build the project | "Build mbot-companion with bluetooth and voice-api features" |
| Run tests | "Run cargo test and explain any failures" |
| Fix a build error | Paste the error and ask "What's wrong and how do I fix it?" |
| Understand the code | "Explain how the motor_script function in protocol.rs works" |
| Add a new voice command | "Add a 'zigzag' command that alternates left and right turns" |
| Debug BLE connection | "The BLE scan hangs forever. What should I check?" |
| Set up Telegram | "Walk me through setting up the Telegram bot step by step" |
| Check sensor readings | "Read the companion logs and tell me what sensors are reporting" |

**Running tests with LLM help:**

```bash
# Ask your LLM to run this and interpret the results:
cargo test 2>&1

# Or for just the contract tests:
cargo test -- contracts 2>&1

# The LLM can read the output and tell you:
# - Which tests passed and failed
# - What the failures mean
# - How to fix them
```

**Tip:** If you're new to Rust, don't try to understand every error message yourself. Paste the full error into your LLM and ask "What does this mean and how do I fix it?" The LLM will explain it in plain English and give you the exact fix.

---

## Troubleshooting

### Build Problems

**"error: linker cc not found"**
```bash
sudo apt install build-essential
```

**"Could not find dbus" or "pkg-config" errors**
```bash
sudo apt install libdbus-1-dev pkg-config
```

**"Could not find openssl"**
```bash
sudo apt install libssl-dev
```

### Bluetooth Problems

**"Scanning for mBot2 via BLE..." hangs forever**
1. Is the mBot2 turned on? Press the power button and wait for the home screen
2. Is Bluetooth enabled on your laptop? `bluetoothctl show | grep Powered` should say "yes"
3. Restart Bluetooth: `sudo systemctl restart bluetooth`
4. Try again. BLE scanning sometimes needs 2-3 attempts

**"Permission denied" when scanning**
```bash
sudo usermod -aG bluetooth $USER
# Log out and log back in, then try again
```

**"BLE TX failed: Not connected"**
- The BLE connection dropped. Common causes:
  - mBot2 went to sleep (auto-off after inactivity)
  - Robot moved too far from laptop (BLE range is ~10 meters)
  - CyberPi was busy running a heavy script
- Fix: press Ctrl+C, turn mBot2 off and on, restart the companion

### Voice API Problems

**"Voice API error: connection refused" from Telegram bridge**
- The companion app isn't running, or is on a different port
- Make sure the companion is running with `--voice-api --voice-port 8088`
- The Telegram bridge connects to `http://localhost:8088` by default

**ElevenLabs returns 401 or "quota exceeded"**
- Check your ElevenLabs dashboard at https://elevenlabs.io for remaining credits
- The free tier has limited characters per month
- Motor commands (forward, dance) don't use TTS credits

**Phone can't reach the Voice API**
- Make sure you're using your laptop's IP, not `localhost`
- Check firewall: `sudo ufw allow 8088/tcp`
- Both devices must be on the same WiFi network

### Motor Problems

**Robot doesn't move but Telegram says "Moving forward!"**
- Check the companion terminal for "BLE TX failed" errors
- If BLE dropped, restart both the companion and Telegram bridge
- Make sure the mBot2 is on a flat surface with wheels touching the ground

**Robot goes in a circle instead of straight**
- Make sure you have the latest code. This was fixed by switching from `drive_speed()` to `mbot2.forward()`
- Rebuild: `cargo build --bin mbot-companion --features "brain,voice-api,bluetooth"`

---

## Want to Help?

This is a community project. Whether you are a roboticist, an AI enthusiast, a teacher, a designer, a kid who wants to play with robots, or a parent looking for screen-free tech time - there is a place for you.

1. Check the [Issues](https://github.com/Hulupeep/mbot_ruvector/issues) - we label things `good first issue` for newcomers
2. Open an issue with your idea
3. Send a PR
4. Share what you built

Contact: robots@floutlabs.com

---

## Built With

- **[RuVector](https://github.com/ruvnet/ruvector)** - The nervous system architecture
- **[Makeblock mBot2](https://www.makeblock.com/steam-kits/mbot2)** - The robot platform
- **Rust** - Performance and safety
- **Claude / Ollama** - LLM reasoning (brain layer)
- **ElevenLabs / Whisper** - Voice interaction (voice layer)

## License

MIT
