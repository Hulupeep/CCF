# mBot RuVector - Master Setup Guide

**Give your robot a nervous system. Watch it come alive.**

This guide walks you through setting up mBot2 with RuVector AI, from unboxing to running your first application.

---

## Table of Contents

1. [Hardware Requirements](#hardware-requirements)
2. [Software Prerequisites](#software-prerequisites)
3. [Hardware Setup](#hardware-setup)
4. [Connecting to CyberPi](#connecting-to-cyberpi)
5. [Building Firmware](#building-firmware)
6. [Downloading to Robot](#downloading-to-robot)
7. [Running Applications](#running-applications)
8. [Web Dashboard](#web-dashboard)
9. [Troubleshooting](#troubleshooting)
10. [Next Steps](#next-steps)

---

## Hardware Requirements

### Essential Components

| Item | Description | Source |
|------|-------------|--------|
| **mBot2** | Makeblock mBot2 robot kit with CyberPi controller | [Makeblock Store](https://www.makeblock.com/steam-kits/mbot2) |
| **USB-C Cable** | For connecting CyberPi to laptop | Usually included with mBot2 |
| **Computer** | Windows 10+, macOS 10.15+, or Ubuntu 20.04+ | - |

### Optional Components

| Item | Purpose | Used By |
|------|---------|---------|
| **Servo Motor** | Drawing pen control | ArtBot |
| **Pen/Marker** | Creating artwork | ArtBot |
| **LEGO Bricks** | Sorting demonstrations | LEGOSorter |
| **Colored Objects** | Color detection tests | HelperBot |
| **Flat Surface** | Drawing canvas | ArtBot |

---

## Software Prerequisites

### 1. Install Rust Toolchain

Rust is required to build the mBot RuVector firmware.

#### Windows

Download and run [rustup-init.exe](https://rustup.rs/)

```powershell
# Verify installation
rustc --version
cargo --version
```

#### macOS

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

#### Linux (Ubuntu/Debian)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install system dependencies
sudo apt update
sudo apt install -y build-essential pkg-config libdbus-1-dev libudev-dev

# Verify installation
rustc --version
cargo --version
```

### 2. Install Node.js (for Web Dashboard)

Download from [nodejs.org](https://nodejs.org/) (LTS version recommended)

```bash
# Verify installation
node --version
npm --version
```

### 3. Clone Repository

```bash
git clone https://github.com/Hulupeep/mbot_ruvector.git
cd mbot_ruvector
```

### 4. Install Web Dependencies

```bash
cd web
npm install
cd ..
```

---

## Hardware Setup

### Basic mBot2 Assembly

1. **Assemble the chassis** following the [official mBot2 guide](https://www.makeblock.com/pages/mbot2-support)
2. **Install batteries** in the CyberPi controller
3. **Power on** the CyberPi (press power button)
4. Verify the screen displays and the robot responds to test movements

### Optional: Pen Servo for Drawing

For ArtBot functionality, attach a servo motor with pen holder:

```
      mBot2
    ┌─────────┐
    │ CyberPi │
    │         │
    │  [pen]  │  ← Servo on Port 1
    │    │    │     Angle 45° = pen up
    │    ▼    │     Angle 90° = pen down
    └─────────┘
```

**Connection:**
1. Connect servo to **Port 1** on CyberPi
2. Attach pen holder to servo arm
3. Secure pen with rubber band or clip
4. Test servo range before drawing

---

## Connecting to CyberPi

### USB-C Connection (Recommended)

This is the most reliable connection method for firmware upload.

#### 1. Connect the Cable

1. Power on the CyberPi
2. Connect USB-C cable from CyberPi to computer
3. Wait for the connection sound/notification

#### 2. Find the Serial Port

**Linux:**
```bash
ls /dev/ttyUSB* /dev/ttyACM*
# Usually: /dev/ttyUSB0 or /dev/ttyACM0
```

**macOS:**
```bash
ls /dev/tty.usb*
# Usually: /dev/tty.usbserial-* or /dev/tty.usbmodem*
```

**Windows:**
```powershell
# Open Device Manager
# Look under "Ports (COM & LPT)"
# Usually: COM3, COM4, etc.
```

#### 3. Set Permissions (Linux Only)

```bash
# Add yourself to dialout group
sudo usermod -a -G dialout $USER

# Logout and login for changes to take effect
# OR for immediate access:
sudo chmod 666 /dev/ttyUSB0
```

### Bluetooth Connection (Alternative)

**Note:** Bluetooth has higher latency and is less reliable for firmware uploads. Use USB-C when possible.

#### 1. Enable Bluetooth on CyberPi

1. Navigate to Settings on CyberPi screen
2. Enable Bluetooth
3. Set device to discoverable

#### 2. Pair from Computer

**Linux:**
```bash
bluetoothctl
scan on
# Find mBot2-XXXX
pair XX:XX:XX:XX:XX:XX
trust XX:XX:XX:XX:XX:XX
connect XX:XX:XX:XX:XX:XX
```

**macOS:**
```bash
# Use System Preferences → Bluetooth
# Find and pair with mBot2-XXXX
```

**Windows:**
```powershell
# Use Settings → Bluetooth & devices
# Add device → Bluetooth
# Select mBot2-XXXX
```

---

## Building Firmware

### Development Build (Faster)

For testing and rapid iteration:

```bash
# Build companion app with simulation
cargo build --bin mbot-companion

# Build specific application binary
cargo build --bin draw
cargo build --bin tictactoe
```

### Release Build (Optimized)

For actual robot deployment:

```bash
# Build optimized firmware
cargo build --release --bin mbot-companion

# Build time: ~2-5 minutes (first build)
# Subsequent builds: ~30 seconds
```

### Build Verification

```bash
# Check that build succeeded
ls target/release/mbot-companion
# OR
ls target/debug/mbot-companion
```

**Expected output:**
- File size: 5-15 MB (debug), 2-5 MB (release)
- No error messages during build
- Executable permissions set

---

## Downloading to Robot

### Method 1: USB Serial Upload (Recommended)

This is the primary method for getting code onto the CyberPi.

```bash
# Find your serial port (see "Connecting" section above)
export MBOT_PORT=/dev/ttyUSB0  # Linux/Mac
# OR
set MBOT_PORT=COM3             # Windows

# Upload firmware (this compiles and sends)
cargo run --release --bin mbot-companion -- --serial $MBOT_PORT
```

**What happens:**
1. Rust compiles the code
2. Binary is sent via serial to CyberPi
3. CyberPi reboots with new firmware
4. Application starts automatically

### Method 2: Bluetooth Upload (Alternative)

**Warning:** Bluetooth upload is slower and less reliable. Use for remote updates only.

```bash
# Upload via Bluetooth
cargo run --release --features bluetooth --bin mbot-companion -- --bluetooth
```

### Method 3: DFU Mode (Recovery)

If normal upload fails, use Device Firmware Update mode:

#### Enter DFU Mode

1. Power off CyberPi
2. Hold **Reset** button
3. Press **Power** button while holding Reset
4. Release Reset after 3 seconds
5. Screen should show "DFU Mode"

#### Upload in DFU Mode

```bash
# Linux - requires dfu-util
sudo apt install dfu-util
dfu-util -d 0483:df11 -a 0 -s 0x08000000 -D target/release/mbot-companion

# macOS - install via Homebrew
brew install dfu-util
dfu-util -d 0483:df11 -a 0 -s 0x08000000 -D target/release/mbot-companion
```

### Upload Verification

**Success indicators:**
- ✅ "Upload complete" message
- ✅ CyberPi reboots automatically
- ✅ Screen shows application UI
- ✅ No error messages

**Failure indicators:**
- ❌ Timeout errors
- ❌ Permission denied
- ❌ Device not found
- ❌ CyberPi doesn't reboot

See [Troubleshooting](#troubleshooting) section if upload fails.

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
