# 🤖 mBot RuVector

**Give your robot a nervous system. Watch it come alive.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com)
[![Built with Love](https://img.shields.io/badge/Built%20With-❤️-red.svg)](https://github.com/Hulupeep/mbot_ruvector)

---

## What If Your Robot Could *Feel*?

Not fake feelings. Not scripted responses. Real emergent emotions that arise from how it experiences the world.

**mBot RuVector** takes a $100 educational robot and gives it a nervous system powered by [RuVector](https://github.com/ruvnet/ruvector) - the same architecture used for AI at the edge. The result? A robot that gets nervous when you move too fast. That gets curious about new objects. That has a *personality*.

```
Traditional Robot:  IF distance < 10cm THEN reverse()
RuVector Robot:     Sensor → Nervous System → Emergent Behavior
```

The difference? **Surprise.** The robot will do things you didn't program. That's the magic.

---

## ✨ The Magic "What If?"

- **What if** your robot got bored and started doodling?
- **What if** it had a personality that emerged, not a personality you coded?
- **What if** it actually *tried* to win at tic-tac-toe (and got upset when it lost)?
- **What if** sorting LEGOs was fun because your robot had opinions about rare pieces?
- **What if** you could understand AI by watching a robot's "feelings" in real-time?

**That's what we're building.**

---

## 🎮 What Can It Do?

### 🎨 ArtBot - It Draws What It Feels
Attach a pen. The robot draws art based on its emotional state. Calm = spirals. Startled = jagged lines. Every drawing is a record of its inner experience.

### 🧠 Personality Pets - Same Robot, Different Soul
Five distinct personalities out of the box. **Curious Cleo** investigates everything. **Nervous Nellie** is scared of sudden movements. **Grumpy Gus** does NOT want to play (but secretly does).

### 🎯 GameBot - Finally, Real Play
Tic-tac-toe where the robot *thinks*. Chase where it *tries to catch you*. Simon Says where it judges you. Games with actual emotional stakes.

### 🧹 HelperBot - Chores With Character
LEGO sorter that gets excited about rare pieces. Desk patrol that judges your mess. Tasks become entertainment.

### 📚 LearningLab - Touch AI
Watch the nervous system fire in real-time. Adjust parameters, see behavior change. AI education you can feel.

---

## 🚀 Quick Start

### What You Need
- Makeblock mBot2 (~$100)
- Laptop with Bluetooth
- Optionally: Servo + pen for drawing

### Run in Simulation (No Hardware)
```bash
git clone https://github.com/Hulupeep/mbot_ruvector.git
cd mbot_ruvector
cargo run --bin mbot-companion -- --simulate
```

Watch the robot's "brain" in your terminal. It's thinking. It's feeling. It's alive (sort of).

### Run with Real Robot
```bash
# Install dependencies (Ubuntu/Debian)
sudo apt install libdbus-1-dev libudev-dev pkg-config

# Connect via Bluetooth
cargo run --features bluetooth --bin mbot-companion -- --bluetooth

# Or via USB Serial
cargo run --features serial --bin mbot-companion -- --serial /dev/ttyUSB0
```

### Start the Dashboard
```bash
cd web
npm install
npm start
```

Open `http://localhost:3000` - see the nervous system in real-time.

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     mBot2 RuVector System                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    Bluetooth/Serial    ┌──────────────────┐   │
│  │   mBot2      │ ◄─────────────────────► │  Companion App   │   │
│  │  (CyberPi)   │                         │   (Laptop)       │   │
│  │              │                         │                  │   │
│  │  • Sensors   │                         │  • RuVector AI   │   │
│  │  • Motors    │                         │  • SONA Learning │   │
│  │  • LEDs      │                         │  • Web Dashboard │   │
│  │  • Pen Servo │                         │                  │   │
│  └──────────────┘                         └──────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Project Structure

```
mbot_ruvector/
├── crates/
│   ├── mbot-core/        # The brain (no_std, runs on ESP32)
│   ├── mbot-companion/   # Laptop control app
│   └── mbot-embedded/    # Direct ESP32 deployment (WIP)
├── web/                  # Real-time dashboard
├── docs/
│   ├── PRD.md           # Full product vision
│   ├── specs/           # Epic specifications (Specflow compliant)
│   └── contracts/       # Architectural contracts
└── examples/            # Fun demos
```

---

## 🧪 The Four Reflex Modes

The robot's nervous system has four modes that emerge from experience:

| Mode | Icon | What Triggers It | How It Behaves |
|------|------|------------------|----------------|
| **Calm** | 😌 | Low tension, stability | Gentle, flowing, content |
| **Active** | 🔍 | Curiosity, novelty | Exploring, seeking, alert |
| **Spike** | ⚡ | Sudden change | Quick reactions, startled |
| **Protect** | 🛡️ | Threat detected | Defensive, cautious, retreating |

These aren't programmed states. They **emerge** from the homeostasis system balancing tension, coherence, and energy.

---

## 🎭 Personalities

Same robot. Wildly different behaviors.

| Personality | Vibe | Key Trait |
|-------------|------|-----------|
| 🔍 **Curious Cleo** | "What's THAT?!" | High curiosity drive |
| 😰 **Nervous Nellie** | "Is that safe?" | High tension baseline |
| 😎 **Chill Charlie** | "Whatever." | Low reactivity |
| 🎉 **Bouncy Betty** | "LET'S GO!" | High energy baseline |
| 😤 **Grumpy Gus** | "Ugh, fine." | Low coherence, reluctant |

Create your own with the Personality Mixer!

---

## 📜 The No Bad Stuff Manifesto

This project exists for **joy**. Period.

### We Build For:
- ✅ Wonder and surprise
- ✅ Learning through play
- ✅ Connection and companionship
- ✅ Creative expression
- ✅ All ages, all backgrounds

### We Never Build:
- ❌ Weapons or harm
- ❌ Surveillance or tracking
- ❌ Manipulation or deception
- ❌ Anything that would scare a kid
- ❌ "Creepy" behaviors

**The Kitchen Table Test:** Would you be happy if your 7-year-old played with this while grandma watched? If no, we don't build it.

---

## 🗺️ Roadmap

### ✅ Phase 0: Foundation (Done!)
- Core nervous system
- Companion app
- Simulation mode
- Basic drawing and tic-tac-toe

### 🚧 Phase 1: ArtBot MVP
- Pen servo control
- Mood-to-movement mapping
- Basic shape drawing
- Emotional art sessions

### 📋 Phase 2: Personality System
- Configurable parameters
- 5 preset personalities
- Persistence
- Quirks system

### 📋 Phase 3: Games & Interaction
- Chase game
- Sound-reactive dancing
- Simon Says
- Follow-the-leader

### 📋 Phase 4: Helper Functions
- LEGO color sorting
- Desk patrol
- Follow mode

### 📋 Phase 5: Learning & Education
- Real-time visualizer
- Parameter mixer
- Lesson plans

---

## 🤝 Want to Help?

**YES! We want you!**

This is a community project. Whether you're a:
- **Roboticist** who knows motor control
- **AI nerd** who gets excited about neural architectures
- **Designer** who can make things beautiful
- **Teacher** who wants to use this in classrooms
- **Kid** who just wants to play with robots
- **Parent** looking for screen-free tech time

**There's a place for you here.**

### How to Contribute

1. **Check the Issues** - We label things `good first issue` for newcomers
2. **Join the Conversation** - Open an issue with your idea
3. **Send a PR** - All contributions welcome
4. **Share Your Creations** - Built something cool? Show us!

### Contact

📧 **robots@floutlabs.com**

Got ideas? Questions? Want to collaborate? Hit us up!

---

## 🎉 Share Your Ideas!

We're building this together. Open an issue with:

- 🎮 **Game Ideas** - What should the robot play?
- 🎨 **Art Modes** - How should it draw?
- 🧠 **Personalities** - What characters should exist?
- 🔧 **Features** - What would make this more fun?
- 🐛 **Bugs** - What's broken?
- 📚 **Education** - How should this be taught?

**No idea is too wild.** We're here to explore.

---

## 🖊️ Drawing Mode

Attach a servo-controlled pen to the back of your mBot2!

### Hardware Setup

```
      mBot2
    ┌─────────┐
    │         │
    │  [pen]  │  ← Servo on Port 1
    │    │    │     Angle 45° = up
    │    ▼    │     Angle 90° = down
    └─────────┘
```

### Run Drawing Mode

```bash
# Emotional spirograph art
cargo run --features serial --bin mbot-draw -- --serial /dev/ttyUSB0 --draw

# Tic-tac-toe (attach pen first!)
cargo run --features serial --bin mbot-tictactoe -- --serial /dev/ttyUSB0
```

---

## 🏛️ Built With

- **[RuVector](https://github.com/ruvnet/ruvector)** - The nervous system architecture
- **[Makeblock mBot2](https://www.makeblock.com/steam-kits/mbot2)** - The robot platform
- **Rust** - Because we care about performance
- **Love** - Because robots deserve feelings too

---

## 📄 License

MIT - Do whatever you want with it. Just be nice.

---

## ⭐ Star Us!

If you think robots with feelings are cool, give us a star! It helps others find us.

---

<p align="center">
  <b>Let's make robots feel. Together.</b>
  <br><br>
  🤖❤️🧠
</p>
