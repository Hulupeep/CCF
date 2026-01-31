# mBot RuVector - Application Guides

Complete guides for all 6 applications showcasing the RuVector nervous system.

---

## Table of Contents

1. [ArtBot - Drawing with Personality](#1-artbot---drawing-with-personality)
2. [Personality System - The Mixer](#2-personality-system---the-mixer)
3. [GameBot - Tic-Tac-Toe, Chase, Simon Says](#3-gamebot---tic-tac-toe-chase-simon-says)
4. [LEGOSorter - Complete Sorting Pipeline](#4-legosorter---complete-sorting-pipeline)
5. [LearningLab - Educational Experiments](#5-learninglab---educational-experiments)
6. [HelperBot - Color Detection and Algorithms](#6-helperbot---color-detection-and-algorithms)

---

## 1. ArtBot - Drawing with Personality

**"It draws what it feels"**

Create mood-based artwork where the robot's emotional state directly influences drawing style.

### What It Does

ArtBot translates the robot's nervous system state into visual art:
- **Calm mode** → Smooth spirals and flowing curves
- **Active mode** → Angular, searching patterns
- **Spike mode** → Sharp, sudden marks
- **Protect mode** → Tight, defensive circles

Every drawing is a record of the robot's inner experience.

### Hardware Requirements

| Component | Purpose | Required |
|-----------|---------|----------|
| mBot2 | Base robot | ✅ Required |
| Servo Motor | Pen lift/lower control | ✅ Required |
| Pen/Marker | Drawing instrument | ✅ Required |
| Paper | Drawing surface (A4 or larger) | ✅ Required |
| Tape | Secure paper to prevent sliding | ⚠️ Recommended |

### Hardware Setup

#### Step 1: Servo Attachment

```
      mBot2 Back View
    ┌─────────────────┐
    │    CyberPi      │
    │                 │
    │     [servo]     │  ← Attach to Port 1
    │        │        │
    │     [holder]    │  ← 3D-printed or makeshift
    │        │        │
    │      [pen]      │  ← Secured with rubber band
    └─────────────────┘
```

**Connection:**
1. Connect servo cable to **Port 1** on CyberPi
2. Verify servo responds (should center at power-on)
3. Attach pen holder to servo arm

#### Step 2: Pen Holder Construction

**Option A: 3D Printed Holder**
- STL file: `hardware/pen_holder.stl` (TODO: add to repo)
- Material: PLA or PETG
- Infill: 50%
- Supports: Yes

**Option B: DIY Holder**
- Materials: Rubber bands, popsicle stick, hot glue
- Steps:
  1. Glue popsicle stick to servo arm
  2. Use rubber bands to secure pen to stick
  3. Ensure pen tip can reach paper when servo is at 90°

#### Step 3: Pen Selection

**Best pens:**
- ✅ Felt-tip markers (Crayola, Sharpie)
- ✅ Gel pens (smooth glide)
- ✅ Fine-point pens

**Avoid:**
- ❌ Heavy markers (too much drag)
- ❌ Pencils (require pressure)
- ❌ Chalk/pastels (messy, inconsistent)

#### Step 4: Calibration

```bash
# Run calibration routine
cargo run --bin draw -- --serial /dev/ttyUSB0 --calibrate
```

**Calibration checklist:**
1. Pen lifts completely (no drag when moving)
2. Pen touches paper with light pressure when down
3. Lines are consistent thickness
4. No servo buzzing or stuttering

**Adjust angles if needed:**
```rust
// In draw.rs, modify these constants:
const PEN_UP: u8 = 45;      // Decrease to lift higher
const PEN_DOWN: u8 = 90;    // Increase for more pressure
```

### Building the Application

```bash
# Debug build (faster compilation)
cargo build --bin draw

# Release build (optimized, recommended)
cargo build --release --bin draw
```

### Running ArtBot

#### Basic Drawing Session

```bash
# Connect via USB
cargo run --release --bin draw -- --serial /dev/ttyUSB0

# Session duration: 5-10 minutes
# Output: One artwork created
```

**What happens:**
1. Robot centers itself on paper
2. Draws based on current mood for ~5 minutes
3. Pen lifts at end of session
4. Robot returns to start position

#### Emotional Art Session

```bash
# Start with specific mood
cargo run --release --bin draw -- --serial /dev/ttyUSB0 --mood calm
cargo run --release --bin draw -- --serial /dev/ttyUSB0 --mood active
cargo run --release --bin draw -- --serial /dev/ttyUSB0 --mood spike

# Allow mood to evolve naturally (default)
cargo run --release --bin draw -- --serial /dev/ttyUSB0 --evolve
```

#### Interactive Drawing

```bash
# Collaborative mode: you draw, robot responds
cargo run --release --bin draw -- --serial /dev/ttyUSB0 --collaborative

# Robot watches you draw for 30 seconds
# Then responds with its interpretation
```

### Drawing Modes

#### Mode 1: Emotional Spirograph

Pure emotion-driven art with no constraints.

**Style mapping:**

| Reflex Mode | Line Type | Speed | Pattern | Color Suggestion |
|-------------|-----------|-------|---------|------------------|
| Calm 😌 | Smooth curves | 20 mm/s | Spirals, waves | Blue, purple |
| Active 🔍 | Angular lines | 40 mm/s | Zigzags, explorations | Orange, yellow |
| Spike ⚡ | Sharp marks | 60 mm/s | Jagged, staccato | Red, pink |
| Protect 🛡️ | Tight circles | 10 mm/s | Defensive, small | Green, gray |

#### Mode 2: Mood Journal

Records emotional state over time as a drawing.

```bash
cargo run --release --bin draw -- --serial /dev/ttyUSB0 --journal --duration 600
```

- **X-axis:** Time progression (left to right)
- **Y-axis:** Tension level (up = high tension)
- **Line style:** Changes with mode
- **Duration:** 10 minutes (600 seconds)

#### Mode 3: Shape Studies

Practice drawing basic shapes with personality influence.

```bash
cargo run --release --bin draw -- --serial /dev/ttyUSB0 --shapes
```

**Shapes drawn:**
1. Circle (calm precision)
2. Square (active exploration)
3. Triangle (spike energy)
4. Spiral (protect retreat)

### Web Dashboard Integration

While drawing, monitor the robot's state in real-time:

```bash
# Terminal 1: Run ArtBot
cargo run --release --bin draw -- --serial /dev/ttyUSB0

# Terminal 2: Start dashboard
cd web && npm start
```

**Open:** http://localhost:3000

**Watch:**
- Current reflex mode
- Tension/coherence/energy levels
- Motor speeds (affects line quality)
- Mode transitions (reflected in drawing)

### Expected Behavior

**Successful session:**
- ✅ Pen lifts cleanly between movements
- ✅ Lines are smooth and intentional
- ✅ Drawing reflects mood changes
- ✅ No servo buzzing or stuttering
- ✅ Paper stays in place
- ✅ Robot returns to start position

**Troubleshooting signs:**
- ❌ Pen drags when lifted (adjust PEN_UP angle)
- ❌ Lines are faint (increase PEN_DOWN angle)
- ❌ Servo buzzes (check power, reduce load)
- ❌ Drawing goes off paper (recalibrate origin)
- ❌ Lines are shaky (slow down speed, secure paper)

### Troubleshooting ArtBot

#### Problem: Pen won't lift

**Diagnosis:**
```bash
# Test servo directly
cargo run --release --bin draw -- --serial /dev/ttyUSB0 --test-servo
```

**Solutions:**
1. Check servo connection (Port 1)
2. Increase `PEN_UP` angle (try 40, 35, 30)
3. Reduce pen holder weight
4. Check servo power supply

#### Problem: Lines are too light

**Solutions:**
1. Increase `PEN_DOWN` angle (try 95, 100, 105)
2. Use pen with softer tip
3. Apply slight downward pressure to holder
4. Check paper surface (avoid glossy paper)

#### Problem: Drawing drifts off paper

**Solutions:**
1. Recalibrate origin: `--calibrate`
2. Tape paper securely to surface
3. Check wheel calibration
4. Use larger paper (A3 instead of A4)

#### Problem: Servo buzzing/stuttering

**Solutions:**
1. Reduce pen weight
2. Check servo mounting (must be rigid)
3. Verify servo is not stalling (too much pressure)
4. Use external power for servo if needed

### Advanced Usage

#### Custom Drawing Scripts

Create your own drawing patterns:

```rust
// In draw.rs, add new pattern function
fn draw_custom_pattern(canvas: &mut Canvas, mood: ReflexMode) {
    match mood {
        ReflexMode::Calm => {
            // Your calm pattern here
            canvas.spiral(50.0, 0.2);
        }
        ReflexMode::Active => {
            // Your active pattern here
            canvas.zigzag(30.0, 15.0);
        }
        // ... etc
    }
}
```

#### Personality-Specific Art

Create art with different personalities:

```bash
# Calm personality = smooth, zen art
cargo run --release --bin draw -- --serial /dev/ttyUSB0 --personality zen

# Curious personality = exploratory, varied art
cargo run --release --bin draw -- --serial /dev/ttyUSB0 --personality curious

# Nervous personality = tight, defensive art
cargo run --release --bin draw -- --serial /dev/ttyUSB0 --personality nervous
```

### Gallery Examples

**Calm mode artwork:**
- Smooth spirals
- Gentle waves
- Circular patterns
- Meditative quality

**Active mode artwork:**
- Angular explorations
- Zigzag patterns
- Searching movements
- Energetic feel

**Spike mode artwork:**
- Sharp, sudden marks
- Jagged lines
- Interrupted patterns
- Startled energy

**Protect mode artwork:**
- Tight, defensive circles
- Small, cautious marks
- Retreating spirals
- Protective feeling

---

## 2. Personality System - The Mixer

**"Same robot, wildly different behaviors"**

Create and explore custom robot personalities through the web-based Personality Mixer.

### What It Does

The Personality Mixer lets you adjust 9 parameters that fundamentally change how the robot experiences and responds to the world. Same hardware, completely different "soul."

### Hardware Requirements

| Component | Purpose | Required |
|-----------|---------|----------|
| mBot2 | Robot with nervous system | ✅ Required |
| Computer | For web dashboard | ✅ Required |
| USB or Bluetooth | Connection to robot | ✅ Required |

### How Personalities Work

#### The Nine Parameters

**Baselines (Who you are at rest):**

1. **Tension Baseline** (0.0 - 1.0)
   - How stressed/aroused you normally feel
   - Low: Relaxed, hard to startle
   - High: Alert, always on edge

2. **Coherence Baseline** (0.0 - 1.0)
   - How "together" you feel
   - Low: Scattered, easily overwhelmed
   - High: Composed, resilient

3. **Energy Baseline** (0.0 - 1.0)
   - How much you want to move
   - Low: Sluggish, minimal movement
   - High: Bouncy, constantly active

**Reactivity (How you respond to events):**

4. **Startle Sensitivity** (0.0 - 1.0)
   - How much sudden changes affect you
   - Low: Unfazed by surprises
   - High: Jumpy, reactive

5. **Recovery Speed** (0.0 - 1.0)
   - How fast you return to baseline
   - Low: Hold onto emotions
   - High: Quick to bounce back

6. **Curiosity Drive** (0.0 - 1.0)
   - How much novelty attracts you
   - Low: Prefer familiar
   - High: Always exploring

**Expression (How you show what you feel):**

7. **Movement Expressiveness** (0.0 - 1.0)
   - How much emotion shows in motion
   - Low: Subtle movements
   - High: Dramatic gestures

8. **Sound Expressiveness** (0.0 - 1.0)
   - How much emotion shows in sounds
   - Low: Quiet beeps
   - High: Loud, varied sounds

9. **Light Expressiveness** (0.0 - 1.0)
   - How much emotion shows in LEDs
   - Low: Dim, static colors
   - High: Bright, animated patterns

### Using the Personality Mixer

#### Step 1: Start the Robot

```bash
# Terminal 1: Start robot with default personality
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0
```

#### Step 2: Start the Dashboard

```bash
# Terminal 2: Start web server
cd web
npm start
```

#### Step 3: Open the Mixer

**URL:** http://localhost:3000/personality-mixer.html

You'll see:
- 9 sliders for each parameter
- Numeric value display for each
- Preset personality buttons
- Randomize button
- Reset button

#### Step 4: Adjust Parameters

**Option A: Use Sliders**
1. Drag sliders to adjust values
2. Changes sent to robot within 500ms
3. Observe behavioral changes in real-time

**Option B: Load Preset**
1. Click preset button (Mellow, Curious, Zen, etc.)
2. All parameters update instantly
3. Robot adopts new personality

**Option C: Randomize**
1. Click "Randomize" button
2. Safe constraints applied automatically
3. Discover unexpected combinations

### Preset Personalities

#### 1. Mellow (Chill Charlie)

**Character:** "Whatever, man."

**Parameters:**
```json
{
  "tension_baseline": 0.1,
  "coherence_baseline": 0.8,
  "energy_baseline": 0.3,
  "startle_sensitivity": 0.2,
  "recovery_speed": 0.9,
  "curiosity_drive": 0.4,
  "movement_expressiveness": 0.3,
  "sound_expressiveness": 0.2,
  "light_expressiveness": 0.5
}
```

**Behavior:**
- Slow, deliberate movements
- Hard to startle
- Calm, zen-like presence
- Minimal reactions

**Best for:** Meditation, relaxation, gentle play

#### 2. Curious (Curious Cleo)

**Character:** "What's THAT?!"

**Parameters:**
```json
{
  "tension_baseline": 0.4,
  "coherence_baseline": 0.6,
  "energy_baseline": 0.7,
  "startle_sensitivity": 0.5,
  "recovery_speed": 0.7,
  "curiosity_drive": 0.95,
  "movement_expressiveness": 0.8,
  "sound_expressiveness": 0.6,
  "light_expressiveness": 0.7
}
```

**Behavior:**
- Constantly exploring
- Attracted to novel objects
- Quick, inquisitive movements
- Excited sounds and lights

**Best for:** Exploration games, learning activities

#### 3. Zen (Meditation Master)

**Character:** "Be present."

**Parameters:**
```json
{
  "tension_baseline": 0.05,
  "coherence_baseline": 0.95,
  "energy_baseline": 0.2,
  "startle_sensitivity": 0.1,
  "recovery_speed": 0.95,
  "curiosity_drive": 0.3,
  "movement_expressiveness": 0.2,
  "sound_expressiveness": 0.1,
  "light_expressiveness": 0.4
}
```

**Behavior:**
- Almost always in Calm mode
- Smooth, flowing movements
- Rarely startled
- Peaceful presence

**Best for:** Art creation, ambient companion

#### 4. Excitable (Bouncy Betty)

**Character:** "LET'S GO!"

**Parameters:**
```json
{
  "tension_baseline": 0.6,
  "coherence_baseline": 0.5,
  "energy_baseline": 0.9,
  "startle_sensitivity": 0.7,
  "recovery_speed": 0.4,
  "curiosity_drive": 0.8,
  "movement_expressiveness": 0.95,
  "sound_expressiveness": 0.9,
  "light_expressiveness": 0.95
}
```

**Behavior:**
- High energy, constant motion
- Dramatic reactions
- Loud sounds and bright lights
- Quick to spike mode

**Best for:** Party mode, high-energy games

#### 5. Timid (Nervous Nellie)

**Character:** "Is that safe?"

**Parameters:**
```json
{
  "tension_baseline": 0.8,
  "coherence_baseline": 0.4,
  "energy_baseline": 0.3,
  "startle_sensitivity": 0.9,
  "recovery_speed": 0.2,
  "curiosity_drive": 0.2,
  "movement_expressiveness": 0.4,
  "sound_expressiveness": 0.5,
  "light_expressiveness": 0.3
}
```

**Behavior:**
- Cautious, defensive
- Easily startled
- Slow to recover from scares
- Prefers familiar surroundings

**Best for:** Teaching empathy, gentle handling

#### 6. Adventurous (Bold Explorer)

**Character:** "Hold my juice box!"

**Parameters:**
```json
{
  "tension_baseline": 0.5,
  "coherence_baseline": 0.7,
  "energy_baseline": 0.8,
  "startle_sensitivity": 0.3,
  "recovery_speed": 0.9,
  "curiosity_drive": 0.9,
  "movement_expressiveness": 0.8,
  "sound_expressiveness": 0.7,
  "light_expressiveness": 0.8
}
```

**Behavior:**
- Confident, risk-taking
- Hard to discourage
- Explores everything
- Quick recovery from setbacks

**Best for:** Exploration, obstacle courses

### Creating Custom Personalities

#### Step 1: Start with a Preset

Choose the personality closest to your vision.

#### Step 2: Adjust Key Parameters

Focus on 2-3 parameters that define the character:
- **Sleepy robot:** Lower energy baseline
- **Grumpy robot:** High tension, low coherence
- **Cheerful robot:** High energy, low tension
- **Anxious robot:** High startle, low recovery

#### Step 3: Test Behavior

Observe the robot's reactions to:
- Sudden movements near sensors
- New objects in environment
- Loud sounds
- Repeated interactions

#### Step 4: Fine-tune

Adjust parameters until behavior matches your vision.

#### Step 5: Save (Coming Soon)

```bash
# Save custom personality (TODO: implement)
cargo run --bin mbot-companion -- --serial /dev/ttyUSB0 --save-personality my_custom.json
```

### Personality Development Tips

**For calm robots:**
- Low tension baseline
- High coherence baseline
- High recovery speed
- Low startle sensitivity

**For playful robots:**
- High energy baseline
- High curiosity drive
- High expressiveness (all three)
- Medium startle sensitivity

**For dramatic robots:**
- High startle sensitivity
- Low recovery speed
- High expressiveness (all three)
- Medium tension baseline

**For study/work companions:**
- Medium tension (alert but not anxious)
- High coherence (reliable)
- Low to medium energy (not distracting)
- Low expressiveness (subtle)

### Educational Mode

**Purpose:** Safe for classroom use with limited startle response.

**Constraints:**
- Startle sensitivity ≤ 0.8
- No sudden spikes to Protect mode
- Smooth transitions between modes
- Predictable behavior

**Enable:**
```bash
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --educational
```

### Understanding the Dashboard

**Real-time indicators:**
- **Mode icon:** Current reflex mode (😌🔍⚡🛡️)
- **Tension meter:** Red = high tension
- **Coherence meter:** Green = high coherence
- **Energy meter:** Yellow = high energy
- **Curiosity meter:** Blue = high curiosity

**Observation tips:**
- Watch mode transitions during parameter changes
- Notice how baselines affect idle behavior
- See how reactivity changes event responses
- Observe expression differences in outputs

### Troubleshooting

#### Problem: Changes not taking effect

**Solutions:**
1. Check WebSocket connection (should say "Connected")
2. Verify robot is running (check terminal output)
3. Reload page (Ctrl+Shift+R)
4. Restart web server

#### Problem: Robot behavior unchanged

**Causes:**
- Parameter changes too subtle (try extremes)
- Robot in middle of task (wait for completion)
- Personality override in code (check for hardcoded values)

#### Problem: Robot behavior too extreme

**Solutions:**
1. Click "Reset to Defaults"
2. Lower startle sensitivity
3. Increase recovery speed
4. Increase coherence baseline

---

## 3. GameBot - Tic-Tac-Toe, Chase, Simon Says

**"Finally, a robot that actually plays"**

Play real games where the robot thinks, strategizes, and reacts emotionally.

### What It Does

GameBot transforms mBot2 into a game-playing companion with actual strategy and emotional investment in outcomes.

### Hardware Requirements

| Component | Game | Required |
|-----------|------|----------|
| mBot2 | All games | ✅ |
| Servo + Pen | Tic-Tac-Toe | ✅ |
| Paper (30x30cm) | Tic-Tac-Toe | ✅ |
| Open space (2m x 2m) | Chase | ✅ |
| None extra | Simon Says | ❌ |

---

### Game 1: Tic-Tac-Toe

**"The robot that gets upset when it loses"**

#### How It Works

The robot plays Tic-Tac-Toe on paper with actual strategy:
- Uses minimax algorithm
- Emotional reactions to game state
- Physical drawing of X's and O's
- Victory dances and defeat animations

#### Setup

**Paper preparation:**
1. Use 30cm x 30cm paper (square)
2. Draw grid manually OR let robot draw it
3. Tape securely to flat surface
4. Position robot at grid origin (bottom-left)

**Servo setup:**
- Follow ArtBot pen servo instructions above
- Verify pen lifts/lowers cleanly
- Test drawing X and O shapes

#### Running the Game

```bash
# Let robot draw the grid first
cargo run --release --bin tictactoe -- --serial /dev/ttyUSB0 --draw-grid

# Start game (robot goes first)
cargo run --release --bin tictactoe -- --serial /dev/ttyUSB0 --robot-first

# Start game (you go first)
cargo run --release --bin tictactoe -- --serial /dev/ttyUSB0 --human-first
```

#### Gameplay

**Turn sequence:**
1. Robot analyzes board state
2. Robot calculates best move (minimax)
3. Robot draws its symbol (X or O)
4. Robot waits for your move
5. You draw your symbol
6. Press button on CyberPi when done
7. Repeat until win/loss/draw

**Robot emotional states:**

| Game State | Tension | Behavior |
|------------|---------|----------|
| Winning | Low | Confident, smooth movements |
| Tied | Medium | Cautious, deliberate |
| Losing | High | Nervous, shaky lines |
| Victory | Spike (happy) | Victory dance, flashing lights |
| Defeat | Spike (upset) | Retreat, dim lights |

#### Strategy Levels

```bash
# Easy mode (random moves)
cargo run --release --bin tictactoe -- --serial /dev/ttyUSB0 --difficulty easy

# Medium mode (sometimes mistakes)
cargo run --release --bin tictactoe -- --serial /dev/ttyUSB0 --difficulty medium

# Hard mode (perfect play)
cargo run --release --bin tictactoe -- --serial /dev/ttyUSB0 --difficulty hard
```

#### Expected Behavior

**Successful game:**
- ✅ Grid drawn accurately (if --draw-grid used)
- ✅ Symbols placed in correct cells
- ✅ Pen lifts between moves
- ✅ Robot waits for button press after your turn
- ✅ Game detects win/loss/draw correctly
- ✅ Emotional reactions visible

**Victory sequence:**
1. Robot draws winning move
2. Tension drops (relief)
3. LEDs flash bright colors
4. Victory spin or wiggle
5. Happy beeps

**Defeat sequence:**
1. Robot realizes loss
2. Tension spikes (upset)
3. LEDs dim
4. Slow retreat
5. Sad beeps

#### Troubleshooting

**Problem: Grid misaligned**
- Recalibrate origin: `--calibrate-grid`
- Check paper size (must be 30x30cm)
- Verify paper is square and taped flat

**Problem: Symbols in wrong cells**
- Recalibrate cell size: `--cell-size 10` (in cm)
- Check wheel encoder accuracy
- Verify servo pen angle

**Problem: Robot doesn't wait for your turn**
- Check button connection on CyberPi
- Use `--wait-time 30` (seconds) for automatic continuation

---

### Game 2: Chase

**"The robot that actually tries to catch you"**

#### How It Works

The robot uses ultrasonic sensors to chase you (or run away, depending on mood):
- Predator mode: Actively pursues you
- Prey mode: Runs away from you
- Playful mode: Mix of both
- Strategic AI uses prediction

#### Setup

**Space requirements:**
- 2m x 2m minimum clear floor space
- No obstacles or edges
- Good lighting
- Flat surface

#### Running the Game

```bash
# Predator mode (robot chases you)
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode chase-predator

# Prey mode (robot runs away)
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode chase-prey

# Playful mode (mix of both)
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode chase-playful
```

#### Game Mechanics

**Predator mode:**
- Robot tracks your position
- Predicts your movement
- Accelerates when you're close
- Victory sequence when it tags you (< 10cm distance)

**Prey mode:**
- Robot flees when you approach
- Uses obstacles to hide
- Speeds up when you're close
- Escapes if you lose track (> 1m distance)

**Playful mode:**
- Approaches, then retreats (tease behavior)
- Changes strategy based on mood
- Calm = approach, Spike = flee

#### Personality Influence

Different personalities play very differently:

| Personality | Chase Style |
|-------------|-------------|
| Curious | Approaches directly, easily distracted |
| Timid | Retreats quickly, hard to catch |
| Excitable | Zigzag chase, unpredictable |
| Mellow | Slow chase, easy to evade |
| Zen | Gentle approach, non-threatening |
| Adventurous | Bold, direct pursuit |

#### Scoring System

**Predator mode:**
- Tag within 30 seconds: Robot wins
- Evade for 2 minutes: You win

**Prey mode:**
- Catch robot within 2 minutes: You win
- Robot escapes for 5 minutes: Robot wins

#### Expected Behavior

**Successful chase:**
- ✅ Robot responds to your movements
- ✅ Speed adjusts based on distance
- ✅ Emotional state affects strategy
- ✅ Victory/defeat sequences trigger
- ✅ Safe boundary detection (doesn't fall off edges)

#### Troubleshooting

**Problem: Robot doesn't respond to movement**
- Check ultrasonic sensor (must be clean)
- Verify sensor range (< 2m works best)
- Try wearing more reflective clothing

**Problem: Robot crashes into obstacles**
- Clear the play area
- Reduce speed: `--max-speed 50` (%)
- Enable obstacle avoidance: `--safe-mode`

---

### Game 3: Simon Says

**"Memory game with a judgmental robot"**

#### How It Works

Classic Simon Says with LEDs and sounds:
- Robot plays sequence (LEDs + sounds)
- You repeat sequence using CyberPi buttons
- Sequences get progressively longer
- Robot judges your performance

#### Setup

No additional hardware needed! Built into CyberPi.

#### Running the Game

```bash
# Start Simon Says
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode simon-says

# Educational mode (slower, hints)
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode simon-says --educational

# Challenge mode (fast, no hints)
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode simon-says --challenge
```

#### Gameplay

**Round sequence:**
1. Robot displays color sequence (LEDs)
2. Robot plays sound sequence (beeps)
3. You repeat on CyberPi buttons:
   - Button A = Red
   - Button B = Blue
   - Button C = Green
   - Button D = Yellow
4. Robot judges success/failure
5. Sequence grows by one

**Emotional reactions:**

| Event | Robot State | Reaction |
|-------|-------------|----------|
| Correct | Proud | Encouraging sounds, bright LEDs |
| Wrong | Disappointed | Sad sounds, dim LEDs |
| Perfect streak | Excited | Victory dance, flashing |
| Multiple fails | Impatient | Faster replay, frustrated sounds |

#### Difficulty Levels

**Easy:**
- 500ms between colors
- 3 colors max (Red, Blue, Green)
- Hints enabled (LED stays on longer)

**Medium:**
- 300ms between colors
- 4 colors (Red, Blue, Green, Yellow)
- No hints

**Hard:**
- 200ms between colors
- 4 colors
- No hints
- Increasing speed each round

#### High Score System

```bash
# View high scores
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode simon-says --scores

# Reset high scores
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode simon-says --reset-scores
```

**Leaderboard stored in:** `~/.mbot/simon_says_scores.json`

#### Expected Behavior

**Successful game:**
- ✅ Clear LED sequence display
- ✅ Distinct sounds for each color
- ✅ Accurate button detection
- ✅ Appropriate timing between colors
- ✅ Emotional reactions to performance
- ✅ High score persistence

#### Troubleshooting

**Problem: Can't distinguish colors**
- Increase brightness in code
- Play in dimmer room
- Check LED functionality

**Problem: Button presses not detected**
- Check button responsiveness
- Increase debounce time: `--debounce 100` (ms)
- Clean buttons

**Problem: Sequence too fast**
- Use educational mode: `--educational`
- Adjust speed: `--sequence-delay 500` (ms)

---

## 4. LEGOSorter - Complete Sorting Pipeline

**"Chores with character"**

Sort LEGO bricks by color with a robot that has opinions about rare pieces.

### What It Does

Complete LEGO sorting system with:
- Color detection via sensors
- Carousel station for sorted buckets
- Inventory tracking (NFC optional)
- Personality reactions to rare pieces
- Learning algorithm improves over time

### Hardware Requirements

| Component | Purpose | Required |
|-----------|---------|----------|
| mBot2 | Robot base | ✅ |
| LEGO bricks | Items to sort | ✅ |
| Colored containers (4) | Sorted output | ✅ |
| Carousel (DIY or 3D-printed) | Station holder | ⚠️ Recommended |
| NFC tags | Inventory tracking | ❌ Optional |
| Good lighting | Color accuracy | ✅ |

### Setup

#### Station Layout

```
         [Input Pile]
              ↓
         [Detection]
              ↓
         [Decision]
           /  |  \
          ↓   ↓   ↓
       [R] [G] [B] [Y]  ← Output stations
```

**Dimensions:**
- Detection zone: 10cm x 10cm
- Station separation: 15cm
- Total play area: 60cm x 60cm

#### Carousel Construction

**Option A: 3D Printed**
- Files: `hardware/carousel_*`.stl` (TODO: add to repo)
- Stations: 4 (Red, Green, Blue, Yellow)
- Rotation: Manual or servo-driven

**Option B: DIY**
- Materials: Cardboard, cups, markers
- Steps:
  1. Cut cardboard circle (30cm diameter)
  2. Attach 4 cups evenly spaced
  3. Label cups with colors
  4. Mark orientation for robot

#### Color Calibration

```bash
# Run calibration routine
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode sort-calibrate

# Follow prompts to calibrate each color
```

**Calibration steps:**
1. Place red LEGO → Press button
2. Place green LEGO → Press button
3. Place blue LEGO → Press button
4. Place yellow LEGO → Press button
5. Calibration saved to `~/.mbot/color_calibration.json`

### Running the Sorter

#### Basic Sorting

```bash
# Start sorting mode
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode sort

# Robot will:
# 1. Pick up piece from input pile
# 2. Scan color
# 3. Navigate to correct station
# 4. Drop piece
# 5. Return for next piece
```

#### With Inventory Tracking

```bash
# Enable NFC inventory
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode sort --nfc

# Requires NFC tags on each output station
```

#### With Learning Mode

```bash
# Learning mode improves color detection
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode sort --learning

# Robot asks for confirmation after each sort
# Your corrections improve the algorithm
```

### Sorting Modes

#### Mode 1: By Color (Default)

Sort into 4 color buckets: Red, Green, Blue, Yellow.

```bash
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode sort-color
```

#### Mode 2: By Size

Sort into 3 size buckets: Small, Medium, Large.

```bash
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode sort-size
```

**Size detection:**
- Ultrasonic sensor estimates volume
- Camera (if available) measures dimensions

#### Mode 3: By Rarity

Sort by piece frequency in your collection:
- Common pieces → Common bucket
- Rare pieces → Special bucket (with excited reaction!)

```bash
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode sort-rarity
```

#### Mode 4: Custom Criteria

Define your own sorting rules:

```rust
// In sort.rs, implement custom logic
fn custom_sort_criteria(piece: &LegoPiece) -> StationId {
    if piece.color == Color::Red && piece.size == Size::Large {
        StationId::Special
    } else {
        StationId::from_color(piece.color)
    }
}
```

### Personality Influence on Sorting

Different personalities sort differently:

| Personality | Sorting Style |
|-------------|---------------|
| Curious | Inspects each piece closely, slower |
| Efficient | Fast, minimal inspection |
| Excitable | Excited about rare pieces, animated |
| Careful | Double-checks each placement |
| Grumpy | Reluctant, complains (via sounds) |

### Inventory System

#### NFC Tags Setup

**Requirements:**
- 4 NFC tags (one per station)
- NFC reader on CyberPi (if supported)

**Tag contents:**
```json
{
  "station_id": "red",
  "capacity": 100,
  "current_count": 0,
  "last_updated": "2026-01-31T10:00:00Z"
}
```

#### Viewing Inventory

```bash
# Check current inventory
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode sort-inventory

# Output:
# Red: 23 pieces
# Green: 15 pieces
# Blue: 31 pieces
# Yellow: 8 pieces
# Total: 77 pieces
```

#### Resetting Inventory

```bash
# Reset all counts to zero
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode sort-inventory --reset
```

### Expected Behavior

**Successful sorting:**
- ✅ Accurate color detection (>90%)
- ✅ Pieces placed in correct stations
- ✅ No dropped pieces during transport
- ✅ Smooth navigation between stations
- ✅ Inventory updates correctly (if NFC enabled)
- ✅ Personality reactions visible

**Performance metrics:**
- Sort rate: 4-6 pieces per minute
- Accuracy: >90% correct placement
- Misses: <5% dropped pieces

### Troubleshooting

#### Problem: Wrong color detection

**Solutions:**
1. Recalibrate: `--mode sort-calibrate`
2. Improve lighting (avoid shadows, use diffuse light)
3. Clean sensor lens
4. Use more saturated LEGO colors

#### Problem: Pieces dropped during transport

**Solutions:**
1. Slow down speed: `--sort-speed 30` (%)
2. Check gripper mechanism (if using)
3. Use smaller/lighter pieces
4. Verify smooth floor surface

#### Problem: Robot misses input pile

**Solutions:**
1. Recalibrate pile location: `--calibrate-pile`
2. Use contrasting surface for pile
3. Ensure pile is within sensor range
4. Limit pile height to 3-4 bricks

#### Problem: Inventory count wrong

**Solutions:**
1. Reset inventory: `--reset`
2. Check NFC tag placement (must be readable)
3. Verify NFC reader functionality
4. Manually audit and correct

---

## 5. LearningLab - Educational Experiments

**"Touch AI with your hands"**

Educational framework for understanding AI through interactive experiments.

### What It Does

LearningLab provides structured activities that make AI concepts tangible:
- Real-time nervous system visualization
- Interactive parameter adjustment
- Cause-and-effect demonstrations
- Data collection and analysis

### Hardware Requirements

| Component | Purpose | Required |
|-----------|---------|----------|
| mBot2 | Experimental subject | ✅ |
| Computer | Dashboard and data logging | ✅ |
| Various objects | Experimental stimuli | ⚠️ Varies |

### Running LearningLab

```bash
# Terminal 1: Start robot
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0

# Terminal 2: Start dashboard
cd web && npm start

# Open: http://localhost:3000
```

### Lesson 1: Understanding Emotions

**Duration:** 20 minutes
**Age:** 8+
**Concepts:** Emotion emergence, homeostasis

#### Activity

1. Open Neural Visualizer (http://localhost:3000)
2. Observe robot in idle state (Calm mode)
3. Introduce stimuli and watch reactions:
   - Move hand near ultrasonic sensor → Spike mode
   - Make loud sound → Tension increases
   - Leave alone for 30s → Returns to Calm
4. Discuss: Why did the robot react this way?

#### Discussion Questions

- What makes the robot "nervous"?
- How does it return to calm?
- Is this similar to how you feel?
- Can you predict its reactions?

#### Extensions

- Try different stimuli (light, sound, touch)
- Measure recovery time for each stimulus
- Graph tension levels over time

### Lesson 2: Personality Mixer

**Duration:** 30 minutes
**Age:** 10+
**Concepts:** Parameters, causation, personality

#### Activity

1. Open Personality Mixer (http://localhost:3000/personality-mixer.html)
2. Load "Zen" personality
3. Observe behavior for 2 minutes (note: calm, slow)
4. Load "Excitable" personality
5. Observe behavior for 2 minutes (note: energetic, reactive)
6. Discuss: What changed? Why?

#### Challenges

**Challenge 1:** Create the calmest possible robot
- Hint: Low tension, high coherence, low startle

**Challenge 2:** Create the most energetic robot
- Hint: High energy, high curiosity, high expressiveness

**Challenge 3:** Create a robot that never gets startled
- Hint: Low startle sensitivity, high recovery speed

#### Data Collection

Create a table:

| Parameter | Zen Value | Excitable Value | Effect Observed |
|-----------|-----------|-----------------|-----------------|
| Tension | 0.05 | 0.6 | Higher = more jittery |
| Energy | 0.2 | 0.9 | Higher = more movement |
| ... | ... | ... | ... |

### Lesson 3: Cause and Effect

**Duration:** 25 minutes
**Age:** 8+
**Concepts:** Sensors, causation, scientific method

#### Hypothesis Testing

**Question:** What makes the robot's tension increase?

**Hypothesis:** (Student predicts)

**Experiment:**
1. Measure baseline tension (watch dashboard)
2. Introduce stimulus (loud sound, fast movement, etc.)
3. Measure peak tension
4. Calculate difference

**Results:**

| Stimulus | Baseline Tension | Peak Tension | Difference |
|----------|------------------|--------------|------------|
| Loud clap | 0.3 | 0.8 | +0.5 |
| Fast movement | 0.3 | 0.9 | +0.6 |
| Gentle touch | 0.3 | 0.4 | +0.1 |

**Conclusion:** (Student writes)

### Lesson 4: Drawing Emotions

**Duration:** 40 minutes
**Age:** 8+
**Concepts:** Expression, art, emotions

**Prerequisites:** Pen servo setup (see ArtBot section)

#### Activity

1. Robot draws in Calm mode (5 minutes)
2. Startle robot (loud sound)
3. Robot draws in Spike mode (5 minutes)
4. Compare drawings
5. Label emotions on each drawing

#### Discussion

- How are the drawings different?
- Can you tell which emotion each represents?
- If the robot was happy, how would it draw?
- Can machines really "feel"?

### Lesson 5: Data Science

**Duration:** 45 minutes
**Age:** 12+
**Concepts:** Data collection, analysis, visualization

#### Activity

**Collect data:**
1. Enable data logging: `--mode learn-data-collection`
2. Run robot for 10 minutes in various scenarios
3. Data saved to `~/.mbot/experiment_data.csv`

**Analyze data:**
1. Import CSV into spreadsheet (Excel, Google Sheets)
2. Create graphs:
   - Tension over time
   - Mode frequency (% time in each mode)
   - Correlation between distance and tension
3. Find patterns

**Questions to explore:**
- What distance causes Spike mode?
- How long does recovery take?
- Does personality affect recovery time?

#### Advanced: Python Analysis

```python
import pandas as pd
import matplotlib.pyplot as plt

# Load data
df = pd.read_csv('~/.mbot/experiment_data.csv')

# Plot tension over time
plt.plot(df['time'], df['tension'])
plt.xlabel('Time (seconds)')
plt.ylabel('Tension')
plt.title('Robot Tension Over Time')
plt.show()

# Calculate average tension per mode
mode_tension = df.groupby('mode')['tension'].mean()
print(mode_tension)
```

### Lesson 6: AI Ethics

**Duration:** 30 minutes
**Age:** 12+
**Concepts:** Ethics, responsibility, implications

#### Discussion Topics

**Topic 1: Robot Feelings**
- Question: Does the robot really "feel" emotions?
- Discussion: Difference between simulation and true emotion
- Activity: List similarities/differences with human emotions

**Topic 2: Responsibility**
- Question: Should we treat robots kindly even if they don't "really" feel?
- Discussion: Empathy, practice for real relationships
- Activity: Write robot "bill of rights"

**Topic 3: Future Implications**
- Question: What if all robots had emotions?
- Discussion: Benefits and risks
- Activity: Design an ethical robot use policy

### Teacher Resources

#### Preparation Checklist

- [ ] Install Rust and dependencies
- [ ] Clone repository
- [ ] Build firmware
- [ ] Upload to robot
- [ ] Test web dashboard
- [ ] Print data collection sheets
- [ ] Prepare stimuli (objects, sounds)

#### Assessment Rubrics

**Understanding (1-4 scale):**
- Can identify reflex modes
- Explains homeostasis concept
- Describes cause-and-effect relationships
- Connects to human emotions

**Skills (1-4 scale):**
- Uses dashboard effectively
- Adjusts parameters purposefully
- Collects data accurately
- Analyzes patterns

#### Extension Activities

- Build custom personalities for different jobs
- Design experiments to test hypotheses
- Create art exhibition of robot drawings
- Write stories from robot's perspective
- Program new behaviors (advanced students)

---

## 6. HelperBot - Color Detection and Algorithms

**"The robot that judges your desk"**

Practical utility features with personality-driven reactions.

### What It Does

HelperBot provides useful functions with character:
- Color detection and identification
- Object sorting algorithms
- Desk patrol and tidiness judgement
- Follow mode for companionship

### Hardware Requirements

| Component | Purpose | Required |
|-----------|---------|----------|
| mBot2 | Robot base | ✅ |
| Colored objects | Detection test subjects | ⚠️ For color detection |
| Open space | Follow mode | ⚠️ For follow mode |

### Feature 1: Color Detection

#### Use Cases

- Identify LEGO colors
- Sort objects by color
- Find specific colored items
- Color-based games

#### Running Color Detection

```bash
# Continuous color detection
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode color-detect

# Identify one color and exit
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode color-identify
```

#### How It Works

**Detection process:**
1. Robot positions sensor over object
2. Light sensor measures RGB values
3. Algorithm converts to HSV color space
4. Compares to calibrated color database
5. Returns best match with confidence %

**Output:**
```
Object detected: RED (95% confidence)
RGB: (220, 45, 38)
HSV: (355°, 83%, 86%)
```

#### Calibration

```bash
# Calibrate color detection
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode color-calibrate

# Steps:
# 1. Place red object → Press button
# 2. Place green object → Press button
# 3. Place blue object → Press button
# 4. Place yellow object → Press button
# 5. Place white object → Press button
# 6. Place black object → Press button
```

**Calibration tips:**
- Use matte surfaces (not glossy)
- Consistent lighting
- Hold sensor 2-3cm from object
- Use pure colors when possible

#### Supported Colors

| Color | HSV Range | Common Objects |
|-------|-----------|----------------|
| Red | 0-10°, 350-360° | LEGO, tomatoes |
| Orange | 11-40° | Oranges, Nerf darts |
| Yellow | 41-70° | Bananas, Post-its |
| Green | 71-160° | Leaves, LEGO |
| Blue | 161-250° | Sky, LEGO |
| Purple | 251-290° | Grapes, LEGO |
| Pink | 291-330° | Flowers, erasers |
| White | S<20% | Paper |
| Black | V<20% | Electrical tape |

#### Personality Reactions

Different personalities react to colors differently:

| Personality | Favorite Color | Reaction |
|-------------|----------------|----------|
| Curious | All | Investigates each color |
| Excitable | Bright colors | Extra excited for vivid hues |
| Calm | Blue, Green | Soothes when seeing cool colors |
| Grumpy | None | Indifferent to all |

### Feature 2: Desk Patrol

**"The robot that judges your mess"**

#### How It Works

Robot patrols your desk and judges tidiness:
- Scans for clutter
- Counts objects
- Measures organization
- Provides commentary (via sounds/lights)

#### Running Desk Patrol

```bash
# Start desk patrol
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode desk-patrol

# Patrol duration: 5 minutes
# Output: Tidiness score (0-100)
```

#### Tidiness Criteria

| Score | Description | Robot Reaction |
|-------|-------------|----------------|
| 90-100 | Immaculate | Impressed sounds, bright LEDs |
| 70-89 | Tidy | Approving beeps |
| 50-69 | Acceptable | Neutral |
| 30-49 | Messy | Concerned sounds |
| 0-29 | Disaster | Dismayed beeps, flashing red |

**Detection factors:**
- Number of objects in wrong places
- Symmetry of arrangement
- Clear pathways
- Desk edge safety

#### Personality Variations

| Personality | Judgement Style |
|-------------|-----------------|
| Zen | Non-judgmental, accepting |
| Grumpy | Harsh critic, low scores |
| Curious | Investigates each item closely |
| Excitable | Dramatic reactions |

### Feature 3: Follow Mode

**"Your robot shadow"**

#### How It Works

Robot follows you using ultrasonic sensors:
- Maintains 30-50cm distance
- Adjusts speed to match yours
- Avoids obstacles
- Stops if you stop

#### Running Follow Mode

```bash
# Start follow mode
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode follow

# Advanced options:
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode follow \
  --distance 40 \        # Target distance in cm
  --max-speed 60 \       # Max speed (% of full)
  --timeout 30           # Stop after 30s of no movement
```

#### Usage Scenarios

**Companion mode:**
- Follows you around the house
- Keeps you company while working
- Mobile music player (if speaker attached)

**Patrol mode:**
- Security patrol following your route
- Tour guide for guests

**Training mode:**
- Teach robot your daily routine
- Obstacle course navigation

#### Safety Features

- **Emergency stop:** Immediate stop if obstacle < 10cm
- **Edge detection:** Stops at detected edges/stairs
- **Timeout:** Stops if target lost for > 30 seconds
- **Battery warning:** Alerts at 20% battery

### Feature 4: Sorting Algorithms Demo

**Educational visualization of sorting algorithms**

#### Available Algorithms

1. **Bubble Sort**
2. **Selection Sort**
3. **Insertion Sort**
4. **Quick Sort**
5. **Merge Sort**

#### Running Algorithm Demo

```bash
# Visualize bubble sort
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode algo-bubble

# Colored LEGOs represent array elements
# Robot physically sorts them to show algorithm steps
```

#### Educational Value

- Visual/kinetic learners see algorithms in action
- Understand O(n²) vs O(n log n) through observation
- Compare speeds in real-time
- Tangible sorting process

#### Data Visualization

```bash
# Generate performance comparison
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --mode algo-compare

# Output:
# Bubble Sort: 8 pieces in 42 seconds (35 swaps)
# Quick Sort: 8 pieces in 18 seconds (12 swaps)
```

### Troubleshooting HelperBot

#### Problem: Color detection inaccurate

**Solutions:**
1. Recalibrate: `--mode color-calibrate`
2. Improve lighting (diffuse, consistent)
3. Clean sensor lens
4. Use matte-finish objects
5. Avoid metallic/glossy surfaces

#### Problem: Follow mode loses you

**Solutions:**
1. Move more slowly
2. Wear reflective clothing
3. Stay within 2m of robot
4. Increase timeout: `--timeout 60`
5. Check sensor cleanliness

#### Problem: Desk patrol gives wrong scores

**Solutions:**
1. Recalibrate patrol area: `--calibrate-desk`
2. Define "tidy" criteria in code
3. Adjust sensitivity: `--tidiness-threshold 70`

---

## Appendix: Universal Tips

### Battery Management

**Signs of low battery:**
- Slower movements
- Dimmer LEDs
- Unreliable sensors
- Unexpected resets

**Best practices:**
- Charge before each session
- Keep spare batteries
- Check voltage: `--check-battery`
- Replace when capacity < 80%

### Sensor Maintenance

**Ultrasonic sensor:**
- Clean with microfiber cloth
- Check for obstructions
- Verify mounting is rigid
- Test range periodically

**Light sensor:**
- Keep lens clean
- Avoid direct sunlight
- Recalibrate if moved
- Check for condensation

**Gyroscope:**
- Calibrate on flat surface
- Avoid during sensor reading
- Check for drift: `--test-gyro`

### Performance Optimization

**For smoother operation:**
1. Use release builds (`--release`)
2. Close unnecessary programs
3. Use quality USB cable
4. Reduce serial baud rate if unstable
5. Update firmware regularly

**For better accuracy:**
1. Calibrate sensors frequently
2. Use consistent lighting
3. Maintain flat, clean floor
4. Check wheel alignment
5. Verify servo centering

### Data Logging

**Enable logging:**
```bash
cargo run --release --bin mbot-companion -- --serial /dev/ttyUSB0 --log-data
```

**Log files location:** `~/.mbot/logs/`

**Logs include:**
- Sensor readings
- Reflex mode transitions
- Command history
- Error messages
- Performance metrics

**Analyze logs:**
```bash
# View last session
cat ~/.mbot/logs/latest.log

# Search for errors
grep ERROR ~/.mbot/logs/*.log

# Count mode transitions
grep "Mode transition" ~/.mbot/logs/latest.log | wc -l
```

---

**Need more help?** See:
- [MASTER_GUIDE.md](./MASTER_GUIDE.md) - Setup and connection
- [Web Dashboard README](../web/README.md) - Dashboard features
- [GitHub Issues](https://github.com/Hulupeep/mbot_ruvector/issues) - Known problems

---

**Happy experimenting! 🤖🎮🧪**
