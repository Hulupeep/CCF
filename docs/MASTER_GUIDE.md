# mBot RuVector - Master Setup Guide

This guide assumes you have never done this before. Every step tells you exactly what to type, what you should see on screen, and what it means when something goes wrong.

**How it works:** The mBot2 robot is the body (wheels, sensors, LEDs). Your laptop runs the brain (the RuVector companion app). They talk to each other through a USB cable. You do not upload code to the robot. The robot keeps its factory software. Your laptop tells it what to do.

---

## Table of Contents

1. [What You Need to Buy](#what-you-need-to-buy)
2. [Putting the Robot Together](#putting-the-robot-together)
3. [Installing Software on Your Computer](#installing-software-on-your-computer)
4. [Try It Without a Robot First](#try-it-without-a-robot-first)
5. [Plugging In the Robot](#plugging-in-the-robot)
6. [Running With the Real Robot](#running-with-the-real-robot)
7. [Web Dashboard](#web-dashboard)
8. [Brain Layer (LLM Integration)](#brain-layer-llm-integration)
9. [When Things Go Wrong](#when-things-go-wrong)
10. [Next Steps](#next-steps)

---

## What You Need to Buy

### Required

| Item | What It Is | Where to Get It | Cost |
|------|-----------|-----------------|------|
| **Makeblock mBot2** | A small wheeled robot with a brain board called CyberPi. Comes with motors, sensors, LEDs, and a USB-C cable in the box. | [Makeblock Store](https://www.makeblock.com/steam-kits/mbot2) | ~$100 |
| **A computer** | Windows 10 or newer, macOS 10.15 or newer, or Ubuntu 20.04 or newer. A laptop works great. | You probably have one | - |

The USB-C cable you need comes in the mBot2 box. You do not need to buy one separately.

### Optional (for specific features)

| Item | What It Is For | Which Feature Uses It |
|------|---------------|----------------------|
| **A small servo motor** | Lifts and lowers a pen so the robot can draw | ArtBot drawing mode |
| **A pen or marker** | The actual drawing tool | ArtBot drawing mode |
| **LEGO bricks (assorted colors)** | Something for the robot to sort | LEGO Sorter |

You do not need any of the optional items to get started. Start with just the mBot2 and your computer.

---

## Putting the Robot Together

If you bought a new mBot2, it comes in a box with parts that need to be assembled.

### Step 1: Follow the Makeblock Instructions

The mBot2 box includes a printed instruction booklet. Follow it to snap the chassis together, attach the wheels, and connect the motors. It takes about 20-30 minutes. No tools required - everything snaps or screws together by hand.

If you lost the booklet, Makeblock has the instructions online: [mBot2 Assembly Guide](https://www.makeblock.com/pages/mbot2-support)

### Step 2: Find the Power Button

Look at the CyberPi board on top of the robot. It is the green circuit board with a small color screen. The power button is on the **left side** of the CyberPi board. It is a small physical button.

### Step 3: Turn It On

Press and hold the power button for about 2 seconds. Let go when the screen lights up.

**What you should see:** The small color screen on the CyberPi shows the Makeblock logo, then a home screen with icons. The screen is about the size of a postage stamp. If the screen stays dark, the battery is dead - charge it with the USB-C cable for 30 minutes and try again.

**What you should hear:** A short startup chime from the buzzer.

### Step 4: Make Sure It Works

On the CyberPi home screen, use the small joystick (the little nub next to the screen) to navigate to the "Drive" icon and click it. Now tilt the CyberPi forward - the robot should drive forward. Tilt it left - it turns left. This confirms the motors, sensors, and brain board are all working.

Press the home button (the round button below the screen) to go back to the home screen.

You are done with hardware setup. Leave the robot turned on for the next steps.

### Optional: Attaching a Pen for Drawing

Skip this section if you just want to try the nervous system first. Come back to it later when you want to try ArtBot.

You need a small hobby servo motor (the kind used in RC cars, about $5-10). The mBot2 does not come with one.

```
      Top view of mBot2
    ┌──────────────────┐
    │    [CyberPi]     │
    │                  │
    │   servo ← Port 1 is on the right side
    │    │             │    of the CyberPi board.
    │   pen            │    It is labeled "1".
    │    ▼             │
    └──────────────────┘
        wheels here
```

1. Plug the servo's 3-pin cable into **Port 1** on the CyberPi. Port 1 is labeled with a "1" printed on the board, on the right side. The cable only fits one way.
2. Tape or rubber-band the servo to the back of the robot chassis so the arm hangs over the rear.
3. Tape a pen or marker to the servo arm so the pen tip points down.
4. When the servo arm rotates to 45 degrees, the pen should lift off the paper. At 90 degrees, it should touch the paper. You will calibrate the exact angles later.

---

## Installing Software on Your Computer

You need to install two things: the Rust programming language (which builds the companion app) and some system libraries (which let Rust talk to USB devices).

### Step 1: Open a Terminal

A terminal is the text window where you type commands. It looks like a black or white window with a blinking cursor.

**Linux (Ubuntu/Debian):** Press `Ctrl + Alt + T` on your keyboard. A terminal window opens.

**macOS:** Press `Cmd + Space` to open Spotlight search. Type `Terminal` and press Enter.

**Windows:** Press the Windows key on your keyboard. Type `cmd` and press Enter. (For a better experience, install [Windows Terminal](https://aka.ms/terminal) from the Microsoft Store - it is free.)

You should see a blinking cursor waiting for you to type. Everything below that says "type this" means type it into this window and press Enter.

### Step 2: Install Rust

Rust is the programming language this project is written in. You need it to build the app that runs on your laptop.

**Linux or macOS - type this:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

It will show some text and then ask:
```
1) Proceed with standard installation (default - just press enter)
2) Customize installation
3) Cancel installation
```

Type `1` and press Enter.

When it says "Rust is installed now", **close your terminal window and open a new one.** This is important - the new terminal will know where Rust is. The old one will not.

**Windows:** Download and run [rustup-init.exe](https://rustup.rs/). Follow the prompts. When done, close and reopen your terminal.

**Verify it worked - type this in your new terminal:**
```bash
rustc --version
```

You should see something like `rustc 1.83.0 (some numbers)`. The exact version number does not matter. If you see "command not found" instead, close the terminal, open a new one, and try again. If it still says "command not found", the installation failed - go back to the curl command and try again.

### Step 3: Install System Libraries

These are extra packages your operating system needs so Rust can talk to USB devices and Bluetooth adapters. Without them, the build will fail with confusing errors about "libudev" or "libdbus".

**Linux (Ubuntu/Debian) - type this:**
```bash
sudo apt update && sudo apt install -y build-essential pkg-config libdbus-1-dev libudev-dev libssl-dev
```

It will ask for your password. When you type your password, **nothing appears on screen** - no dots, no stars, nothing. That is normal. Type your password and press Enter.

If it says "Unable to locate package", you might be on a non-Debian Linux. Search for equivalent packages for your distribution.

**macOS - type this:**
```bash
xcode-select --install
```

A popup window appears asking to install developer tools. Click "Install". Wait for it to finish (5-10 minutes).

**Windows:** Download and run [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/). In the installer, check the box next to "Desktop development with C++" and click Install. This is a large download (~2 GB).

### Step 4: Download This Project

**Type this:**
```bash
git clone https://github.com/Hulupeep/mbot_ruvector.git
```

This downloads the project into a folder called `mbot_ruvector` in whatever directory your terminal is currently in (usually your home folder).

If you see "git: command not found":
- Linux: type `sudo apt install git` then try again
- macOS: it will prompt you to install git automatically
- Windows: download from [git-scm.com](https://git-scm.com/), install it, reopen your terminal

Now go into the project folder:
```bash
cd mbot_ruvector
```

Your terminal prompt should now show `mbot_ruvector` somewhere in it. If it says "No such file or directory", the clone failed. Try the git clone command again.

---

## Try It Without a Robot First

Before connecting hardware, make sure the software works. Simulation mode runs the entire nervous system on your laptop without needing a robot.

**Type this:**
```bash
cargo run --bin mbot-companion -- --simulate
```

**The first time you run this**, Rust downloads and compiles all the dependencies. This takes 2-5 minutes. You will see lines like:
```
   Compiling serde v1.0.203
   Compiling tokio v1.38.0
   ...
```

This is normal. Wait for it to finish.

**What you should see when it starts:** A box made of lines that updates every second:

```
╔══════════════════════════════════════════════════════════════╗
║  😌 Calm  │  Tension: 0.30  │  Coherence: 0.70  │  Energy: 0.50
╠══════════════════════════════════════════════════════════════╣
║  📏 Distance:  100.0 cm  │  🔊 Sound: 0.00  │  💡 Light: 0.50
║  ⚙️  Encoders: L=    0 R=    0  │  🌀 Gyro:    0.0°/s
╠══════════════════════════════════════════════════════════════╣
║  ⏱️  Tick:     20  │  Avg:   50µs  │  Max:  120µs
╚══════════════════════════════════════════════════════════════╝
```

This is the robot's nervous system running. It is thinking, feeling, and reacting - just with simulated sensor data instead of real hardware. The mode (Calm/Active/Spike/Protect) will change over time as the internal state fluctuates.

**To stop it:** Press `Ctrl + C` (hold Control and press C).

If you see errors instead, jump to [When Things Go Wrong](#when-things-go-wrong).

---

## Plugging In the Robot

### Step 1: Get the USB-C Cable

Find the USB-C cable that came in the mBot2 box. One end is USB-C (small, oval-shaped connector). The other end is USB-A (the rectangular one) or USB-C depending on what came in your box.

### Step 2: Connect the Cable

1. Make sure the mBot2 is **turned on** (the small screen on the CyberPi should be showing something)
2. Plug the USB-C end into the CyberPi (the port is on the bottom edge of the CyberPi board)
3. Plug the other end into your computer

If you are using a USB hub, that works fine too.

### Step 3: Check That Your Computer Sees the Robot

Your computer creates a "serial port" when the robot is plugged in. You need to find its name.

**Linux - type this:**
```bash
ls /dev/ttyUSB* /dev/ttyACM*
```

You should see something like `/dev/ttyUSB0` or `/dev/ttyACM0`. That is your port name. Remember it.

If you see `No such file or directory` for both, your computer does not see the robot. Try:
- Unplug and replug the USB cable
- Try a different USB port on your computer
- Make sure the robot is turned on
- Try a different USB cable

**macOS - type this:**
```bash
ls /dev/tty.usb*
```

You should see something like `/dev/tty.usbserial-1420` or `/dev/tty.usbmodem14201`. That is your port name.

**Windows:** Open Device Manager (press Windows key, type "Device Manager", press Enter). Look for "Ports (COM & LPT)" in the list. Click the arrow to expand it. You should see something like "USB Serial Device (COM3)". The "COM3" part is your port name.

### Step 4: Give Yourself Permission to Use the Port (Linux Only)

Linux does not let regular users talk to USB devices by default. You need to fix this once.

**Quick fix (works right now, resets on reboot):**
```bash
sudo chmod 666 /dev/ttyUSB0
```

Replace `/dev/ttyUSB0` with whatever port name you found in Step 3.

**Permanent fix (survives reboots):**
```bash
sudo usermod -a -G dialout $USER
```

After running this, **log out of your computer and log back in** (or restart). The change does not take effect until you do.

---

## Running With the Real Robot

Now you will run the same nervous system, but connected to actual hardware. The robot will move, react to sounds, flash its LEDs, and respond to objects near its ultrasonic sensor.

**Type this** (replace `/dev/ttyUSB0` with your actual port name from the previous section):

```bash
cargo run --features serial --bin mbot-companion -- --serial /dev/ttyUSB0
```

**What you should see:** The same status box as simulation mode, but now the sensor values come from the real robot. Wave your hand in front of the ultrasonic sensor (the two "eyes" on the front of the mBot2) and watch the Distance value change. Clap your hands and watch the Sound value spike. The reflex mode may change from Calm to Spike when you clap.

**What you should see on the robot:** The LEDs may change color. The motors may turn briefly. The exact behavior depends on the robot's personality and current emotional state - that is the whole point. It is not following a script.

**To stop:** Press `Ctrl + C`. The robot will stop moving.

---

## Running Applications

mBot RuVector includes 6 major applications, each showcasing different aspects of the nervous system.

### Quick Reference

| App | Binary | Command | Duration |
|-----|--------|---------|----------|
| **Simulation** | `mbot-companion` | `cargo run --bin mbot-companion -- --simulate` | Continuous |
| **Drawing** | `draw` | `cargo run --features serial --bin draw -- --serial /dev/ttyUSB0` | 5-10 min |
| **Tic-Tac-Toe** | `tictactoe` | `cargo run --features serial --bin tictactoe -- --serial /dev/ttyUSB0` | 5-10 min |
| **Color Detect** | `mbot-companion` | `cargo run --features serial --bin mbot-companion -- --serial /dev/ttyUSB0 --mode color` | Continuous |
| **LEGO Sort** | `mbot-companion` | `cargo run --features serial --bin mbot-companion -- --serial /dev/ttyUSB0 --mode sort` | Continuous |
| **Personality** | `mbot-companion` | `cargo run --features serial --bin mbot-companion -- --serial /dev/ttyUSB0` | Continuous |

### Detailed Application Guides

For complete instructions on each application, see:
- [APP_GUIDES.md](./APP_GUIDES.md) - Individual application documentation

### General Usage Pattern

```bash
# 1. Connect robot via USB
# 2. Run application with serial port
cargo run --release --bin [app-name] -- --serial [port] [options]

# 3. Watch output in terminal
# 4. Robot executes autonomously
# 5. Press Ctrl+C to stop
```

---

## Web Dashboard

The web dashboard provides real-time visualization and control.

### Starting the Dashboard

```bash
cd web
npm start
```

**Output:**
```
Server running at http://localhost:3000
WebSocket server on ws://localhost:8081
```

### Available Pages

#### 1. Neural Visualizer (`http://localhost:3000`)

Real-time brain visualization showing:
- **Reflex Mode:** Current emotional state (Calm/Active/Spike/Protect)
- **Homeostasis Meters:** Tension, Coherence, Energy, Curiosity
- **Sensor Readings:** Distance, gyro, sound, light
- **Motor Outputs:** Left/right motor speeds

**Use for:** Understanding what the robot is "feeling" in real-time

#### 2. Personality Mixer (`http://localhost:3000/personality-mixer.html`)

Adjust personality parameters with sliders:
- **Baselines:** Tension, Coherence, Energy
- **Reactivity:** Startle sensitivity, Recovery speed
- **Expression:** Movement, Sound, Light expressiveness
- **Drives:** Curiosity drive

**Features:**
- 6 preset personalities (Mellow, Curious, Zen, etc.)
- Randomize button with safety constraints
- Educational mode (limited startle)
- Real-time parameter updates

**Use for:** Creating custom robot personalities

#### 3. Projects Navigator (`http://localhost:3000/projects.html`)

Overview of all 6 major projects with:
- Implementation status
- Story completion tracking
- Quick links to documentation
- Launch buttons (coming soon)

**Use for:** Exploring available applications

#### 4. Download Manager (`http://localhost:3000/download.html`)

Step-by-step firmware installation guide with:
- Prerequisites checklist
- Build instructions
- Connection detection
- Troubleshooting tips

**Use for:** First-time setup assistance

### Dashboard Architecture

```
┌─────────────┐     WebSocket      ┌──────────────┐     Serial/BT     ┌──────────┐
│   Browser   │ ←────8081─────────→ │  server.js   │ ←────────────────→ │ CyberPi  │
│  Dashboard  │                     │   (Node.js)  │                     │  mBot2   │
└─────────────┘                     └──────────────┘                     └──────────┘
```

**Data flow:**
1. Robot sends telemetry via serial (50ms interval)
2. Server broadcasts to browser via WebSocket (20Hz)
3. User adjusts personality in browser
4. Changes forwarded to robot via serial

---

## Troubleshooting

### Connection Issues

#### Problem: Device not found

**Linux:**
```bash
# Check if device is detected
dmesg | grep tty
lsusb

# Grant permissions
sudo chmod 666 /dev/ttyUSB0
sudo usermod -a -G dialout $USER
```

**macOS:**
```bash
# Check if device is detected
ls /dev/tty.*

# Install CH340 driver if needed
# Download from: https://www.wch-ic.com/downloads/CH341SER_MAC_ZIP.html
```

**Windows:**
```powershell
# Check Device Manager for yellow warnings
# Install CH340 driver: https://www.wch-ic.com/downloads/CH341SER_EXE.html
# Reboot after driver installation
```

#### Problem: Permission denied (Linux)

```bash
# Quick fix (one-time)
sudo chmod 666 /dev/ttyUSB0

# Permanent fix
sudo usermod -a -G dialout $USER
# Logout and login
```

#### Problem: Port busy

```bash
# Find what's using the port
lsof | grep ttyUSB0

# Kill the process
kill -9 [PID]

# Or disconnect and reconnect USB cable
```

### Build Issues

#### Problem: Rust not found

```bash
# Reinstall rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### Problem: Missing dependencies (Linux)

```bash
sudo apt install -y \
  build-essential \
  pkg-config \
  libdbus-1-dev \
  libudev-dev \
  libssl-dev
```

#### Problem: RuVector not found

```bash
# Check RuVector path in Cargo.toml
# Default: ../../../tooling/ruvector/crates/ruvector-dag

# Clone RuVector if missing
cd ../tooling
git clone https://github.com/ruvnet/ruvector.git
cd ruvector
cargo build
```

### Upload Issues

#### Problem: Upload timeout

**Solutions:**
1. Try slower baud rate: `--baud 57600`
2. Reset CyberPi before upload
3. Use shorter/better USB cable
4. Try different USB port
5. Enter DFU mode (see Method 3 above)

#### Problem: Robot doesn't reboot after upload

**Solutions:**
1. Manually reset CyberPi (press reset button)
2. Power cycle (turn off, wait 5s, turn on)
3. Check battery level (low battery = unreliable)
4. Re-upload in DFU mode

### Runtime Issues

#### Problem: Robot not responding

**Diagnostics:**
```bash
# Check if companion app is running
ps aux | grep mbot-companion

# Check serial connection
screen /dev/ttyUSB0 115200
# Press Ctrl+A, then K to exit screen
```

**Solutions:**
1. Restart companion app
2. Reconnect USB cable
3. Power cycle robot
4. Check battery level

#### Problem: Erratic behavior

**Causes:**
- Low battery (charge or replace)
- Sensor obstruction (clean sensors)
- Extreme personality parameters (reset to defaults)
- Motor calibration drift (recalibrate)

**Reset to defaults:**
```bash
# Load default personality
cargo run --bin mbot-companion -- --serial /dev/ttyUSB0 --reset-personality
```

#### Problem: Web dashboard not updating

**Diagnostics:**
```bash
# Check server status
curl http://localhost:3000
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" ws://localhost:8081

# Check browser console for errors (F12)
```

**Solutions:**
1. Restart web server: `npm start`
2. Refresh browser (Ctrl+Shift+R)
3. Check firewall blocking port 8081
4. Try different browser

### Drawing Issues (ArtBot)

#### Problem: Pen not lifting

**Solutions:**
1. Check servo connection (Port 1)
2. Verify servo angles in code:
   ```rust
   const PEN_UP: u8 = 45;
   const PEN_DOWN: u8 = 90;
   ```
3. Calibrate servo range (see APP_GUIDES.md)
4. Check servo power (may need external power for heavy pens)

#### Problem: Drawings are shaky

**Solutions:**
1. Use smoother surface (tape paper down)
2. Reduce movement speed in code
3. Check wheel calibration
4. Use lighter pen (less drag)

### Game Issues

#### Problem: Tic-Tac-Toe grid misaligned

**Solutions:**
1. Recalibrate grid origin
2. Check paper size (must be 30cm x 30cm)
3. Verify servo pen angle
4. Run calibration routine first

---

## Brain Layer (LLM Integration)

The brain layer adds LLM-powered reasoning, memory, voice, and chat on top of the deterministic nervous system. It is entirely optional - the robot works without it.

### Prerequisites

The brain layer has no extra system dependencies beyond what you already installed. It uses HTTP to talk to LLM APIs.

For local-only operation (no API keys needed):
1. Install [Ollama](https://ollama.ai/)
2. Pull a model: `ollama pull llama3.2`

### Building with Brain Features

```bash
# Brain only (LLM + memory + autonomy)
cargo build --features "serial,brain" --bin mbot-companion

# Brain + voice pipeline
cargo build --features "serial,brain,voice" --bin mbot-companion

# Brain + Telegram chat
cargo build --features "serial,brain,telegram" --bin mbot-companion

# Everything
cargo build --features "serial,brain,voice,telegram,discord" --bin mbot-companion
```

### Running with Brain

```bash
# With Ollama (local, free)
cargo run --features "serial,brain" --bin mbot-companion -- --serial /dev/ttyUSB0 --brain

# With Claude API (set key first)
export ANTHROPIC_API_KEY="sk-ant-your-key"
cargo run --features "serial,brain" --bin mbot-companion -- --serial /dev/ttyUSB0 --brain

# With voice (set keys first)
export OPENAI_API_KEY="sk-your-key"
export ELEVENLABS_API_KEY="your-key"
cargo run --features "serial,brain,voice" --bin mbot-companion -- --serial /dev/ttyUSB0 --brain --voice
```

### What the Brain Does

When enabled, the brain layer runs alongside the deterministic nervous system:

1. **Every tick**: The deterministic nervous system processes sensors and generates motor commands (always runs, cannot be overridden by LLM)
2. **Every N seconds** (configurable): The brain queries the LLM with the robot's current state and personality
3. **The LLM suggests**: Motor actions, speech, personality adjustments, or activities
4. **SafetyFilter checks**: Every suggestion before execution (motors clamped, harmful speech blocked)
5. **If LLM fails**: The robot continues on its deterministic nervous system - no degradation

### Brain Environment Variables

| Variable | Default | Purpose |
|---|---|---|
| `ANTHROPIC_API_KEY` | *(none)* | Claude API key |
| `MBOT_CLAUDE_MODEL` | `claude-sonnet-4-20250514` | Claude model |
| `MBOT_OLLAMA_URL` | `http://localhost:11434` | Ollama server |
| `MBOT_OLLAMA_MODEL` | `llama3.2` | Ollama model |
| `MBOT_LLM_TIMEOUT_SECS` | `30` | Max request time (capped at 30) |
| `MBOT_LLM_MAX_TOKENS` | `256` | Max response tokens |
| `MBOT_DB_PATH` | `mbot_memory.db` | SQLite database file |
| `OPENAI_API_KEY` | *(none)* | Whisper STT |
| `GROQ_API_KEY` | *(none)* | Groq Whisper STT |
| `ELEVENLABS_API_KEY` | *(none)* | ElevenLabs TTS |
| `MBOT_TELEGRAM_TOKEN` | *(none)* | Telegram bot token |
| `MBOT_DISCORD_TOKEN` | *(none)* | Discord bot token |

### Brain Troubleshooting

**"No providers available"**: Neither Claude API key nor Ollama is reachable. Check that Ollama is running (`ollama serve`) or that `ANTHROPIC_API_KEY` is set.

**"LLM timeout"**: The LLM took longer than 30 seconds. This can happen with large local models. Try a smaller Ollama model (`llama3.2` is faster than `llama3.1:70b`).

**"Brain layer error" in logs**: The brain gracefully degrades. The deterministic nervous system continues running. Check verbose output (`-v`) for details.

**Voice not working**: Make sure both `OPENAI_API_KEY` (for speech-to-text) and `ELEVENLABS_API_KEY` (for text-to-speech) are set. Check that your microphone is connected and working.

---

## Next Steps

### Learn More

- **[APP_GUIDES.md](./APP_GUIDES.md)** - Detailed guide for each application
- **[PRD.md](./PRD.md)** - Full product vision and architecture
- **[Web Dashboard README](../web/README.md)** - Dashboard API and customization
- **[Specflow Contracts](./contracts/)** - System contracts and invariants

### Experiment

1. **Try different personalities** in the Personality Mixer
2. **Create artwork** with different emotional states
3. **Play games** and observe strategy changes
4. **Sort LEGOs** and watch the inventory system
5. **Mix your own personality** with custom parameters

### Contribute

Found a bug? Have an idea? Want to add a feature?

1. Check [existing issues](https://github.com/Hulupeep/mbot_ruvector/issues)
2. Open a new issue with:
   - Clear description
   - Steps to reproduce (for bugs)
   - Expected vs actual behavior
   - System info (OS, Rust version)
3. Submit PRs following [Specflow compliance](../CLAUDE.md#rule-1-no-ticket--no-code)

### Join the Community

- **Email:** robots@floutlabs.com
- **GitHub:** https://github.com/Hulupeep/mbot_ruvector
- **Discussions:** [GitHub Discussions](https://github.com/Hulupeep/mbot_ruvector/discussions)

---

## Appendix

### System Requirements

**Minimum:**
- CPU: Dual-core 1.5 GHz
- RAM: 4 GB
- Storage: 2 GB free space
- OS: Windows 10, macOS 10.15, Ubuntu 20.04

**Recommended:**
- CPU: Quad-core 2.5 GHz
- RAM: 8 GB
- Storage: 5 GB free space
- OS: Latest stable version

### Port Settings

| Service | Port | Protocol | Purpose |
|---------|------|----------|---------|
| Web Server | 3000 | HTTP | Dashboard pages |
| WebSocket | 8081 | WS | Real-time telemetry |
| Serial | varies | UART | Robot communication |

### Default Personality Parameters

```json
{
  "tension_baseline": 0.3,
  "coherence_baseline": 0.7,
  "energy_baseline": 0.5,
  "startle_sensitivity": 0.4,
  "recovery_speed": 0.6,
  "curiosity_drive": 0.8,
  "movement_expressiveness": 0.7,
  "sound_expressiveness": 0.5,
  "light_expressiveness": 0.6
}
```

### Keyboard Shortcuts (Dashboard)

| Key | Action |
|-----|--------|
| `r` | Reset personality to default |
| `m` | Toggle mute (sound expressiveness) |
| `p` | Pause telemetry updates |
| `f` | Toggle fullscreen |
| `?` | Show help overlay |

### File Locations

```
mbot_ruvector/
├── crates/
│   ├── mbot-core/          # Core nervous system (no_std)
│   ├── mbot-companion/     # Laptop control app
│   │   └── src/bin/        # Application binaries
│   │       ├── draw.rs     # ArtBot drawing app
│   │       └── tictactoe.rs # Tic-Tac-Toe game
│   └── mbot-embedded/      # Direct ESP32 deployment (WIP)
├── web/                    # Web dashboard
│   ├── public/             # HTML pages
│   └── server.js           # WebSocket server
├── docs/                   # Documentation
└── examples/               # Example code
```

---

**Happy building! Let's make robots feel. 🤖❤️🧠**
