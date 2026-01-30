# Product Requirements Document: mBot RuVector

## The Magic "What If?"

**What if your little robot wasn't just following code... but actually *feeling* its way through the world?**

What if it got nervous when things got too close? What if it got bored and started doodling? What if it had a personality that emerged from how it experienced its environment - not from a script someone wrote?

This is **mBot RuVector** - where we give a $100 educational robot the nervous system of something that's actually alive.

---

## Vision

Transform the Makeblock mBot2 from a "follow the code" robot into a **living toy** with emergent personality, real-time emotional responses, and the ability to surprise you. Using RuVector's DAG-based nervous system, we create robots that aren't programmed to be fun - they become fun through how they experience the world.

**For everyone. All ages. Pure play.**

---

## The No Bad Stuff Manifesto

This project exists for joy. Period.

### We Build For:
- **Wonder** - That moment when the robot does something unexpected and delightful
- **Learning** - Understanding AI through play, not textbooks
- **Connection** - Robots that respond to you, not just commands
- **Creativity** - Tools for expression, not just execution
- **Inclusion** - Accessible to kids, fun for adults, safe for everyone

### We Never Build:
- Weapons or anything that could harm
- Surveillance or tracking of people
- Manipulation or deceptive behaviors
- Anything that excludes or demeans
- "Creepy" behaviors (we test with the "would this scare a kid?" rule)

### The Kitchen Table Test
Every feature must pass: *"Would I be happy if my 7-year-old niece played with this unsupervised while grandma watched?"*

If no → we don't build it.

---

## Core Concept: The Nervous System

Traditional robots: `IF sensor THEN action`

RuVector robots: `Sensor → Nervous System → Emergent Behavior`

### The Four Reflex Modes

| Mode | Trigger | Behavior | Personality Expression |
|------|---------|----------|----------------------|
| **Calm** 😌 | Low tension, stable | Gentle movements, soft lights | Content humming, slow doodles |
| **Active** 🔍 | Curiosity spike | Exploring, seeking | Excited wiggles, bright colors |
| **Spike** ⚡ | Sudden change | Quick reactions, alert | Surprised beeps, fast movements |
| **Protect** 🛡️ | Threat detected | Defensive, cautious | Backing away, warning sounds |

### Homeostasis = Personality

The robot constantly seeks balance between:
- **Tension** (arousal/stress level)
- **Coherence** (how "together" it feels)
- **Energy** (activity budget)

Different personalities emerge from different balance points:

| Personality | Tension Baseline | Coherence Need | Energy Style |
|-------------|-----------------|----------------|--------------|
| **Curious George** | Low | Low | Burst explorer |
| **Nervous Nellie** | High | High | Cautious creeper |
| **Chill Charlie** | Very Low | Medium | Slow and steady |
| **Bouncy Betty** | Medium | Low | Constant motion |

---

## Product Lines

### 1. ArtBot - The Drawing Companion

**"It draws what it feels"**

A robot with a pen that creates art based on its emotional state:
- Calm = smooth spirals
- Excited = energetic scribbles
- Scared = tight, defensive marks
- Happy = big, sweeping curves

**Features:**
- Attach any pen/marker
- Real-time mood-to-art translation
- "Teach it to draw" - show it a shape, it incorporates into its vocabulary
- Collaborative drawing - you start, it finishes (in its style)
- Emotional art journaling - record its day as a drawing

### 2. GameBot - The Play Partner

**"Finally, a robot that actually plays"**

Interactive games where the robot is a real opponent/partner:
- **Tic-Tac-Toe** - It thinks, it strategizes, it celebrates (or sulks)
- **Chase** - Tag where the robot actually tries to win
- **Hide and Seek** - Uses sensors to find or hide
- **Simon Says** - But Simon has a personality
- **Dance Battle** - Learns your moves, creates its own

### 3. HelperBot - The Useful Friend

**"Chores become adventures"**

Practical tasks with personality:
- **LEGO Sorter** - Sorts by color, but gets excited about rare pieces
- **Desk Patrol** - Pushes things back to their spots, judges your mess
- **Plant Waterer** - Checks soil, waters with care, celebrates growth
- **Snack Delivery** - Brings you things, expects appreciation

### 4. PersonalityPets - Character Toys

**"Grumpy Bear was just the beginning"**

Distinct personalities in physical form:
- **Grumpy Gus** - Everything annoys him, but secretly loves attention
- **Anxious Andy** - Nervous about everything, needs reassurance
- **Sleepy Sam** - Just wants to rest, grumpy when woken
- **Hyper Heidi** - Cannot. Stop. Moving. Ever.
- **Curious Cleo** - Must investigate EVERYTHING

Each personality = different RuVector configuration. Same hardware, wildly different behaviors.

### 5. LearningLab - The Education Kit

**"AI you can touch"**

For classrooms and curious minds:
- **Neuron Visualizer** - See the nervous system firing in real-time
- **Personality Mixer** - Adjust parameters, see behavior change
- **Reflex Lab** - Understand stimulus → response
- **Emotion Mapping** - Chart how the robot "feels" over time

---

## Roadmap

### Phase 0: Foundation (Current)
- [x] Core nervous system (mbot-core)
- [x] Laptop companion app
- [x] Simulation mode
- [x] Basic transport (Bluetooth/Serial)

### Phase 1: ArtBot MVP
- [ ] Pen holder attachment design
- [ ] Basic mood-to-movement translation
- [ ] Spiral drawing in Calm mode
- [ ] Scribble drawing in Active mode
- [ ] Tic-Tac-Toe game (first complete game)

### Phase 2: Personality System
- [ ] Configurable personality parameters
- [ ] 5 preset personalities
- [ ] Personality persistence (remembers who it is)
- [ ] Mood history and patterns

### Phase 3: Games & Interaction
- [ ] Chase game with real pursuit AI
- [ ] Sound-reactive dancing
- [ ] Follow-the-leader mode
- [ ] Collaborative drawing

### Phase 4: Helper Functions
- [ ] Color sorting (LEGO sorter)
- [ ] Object pushing/herding
- [ ] Patrol patterns
- [ ] Simple delivery tasks

### Phase 5: Learning & Education
- [ ] Real-time nervous system dashboard
- [ ] Parameter adjustment UI
- [ ] Classroom lesson plans
- [ ] "Build your own personality" workshop

### Phase 6: Community & Sharing
- [ ] Personality sharing/marketplace
- [ ] Drawing gallery
- [ ] Game high scores
- [ ] Community challenges

---

## Technical Architecture

### Three-Layer System

```
┌─────────────────────────────────────────────────┐
│              Personality Layer                   │
│  (Configurable traits, mood baselines, quirks)  │
└─────────────────────────────────────────────────┘
                      │
┌─────────────────────────────────────────────────┐
│            RuVector Nervous System               │
│  (DAG processing, homeostasis, reflex modes)    │
└─────────────────────────────────────────────────┘
                      │
┌─────────────────────────────────────────────────┐
│              Hardware Abstraction                │
│  (mBot2, CyberPi, sensors, motors, LEDs)        │
└─────────────────────────────────────────────────┘
```

### Data Flow

```
Sensors → Perception → Nervous System → Behavior → Actuators
   ↑                         ↓
   └──── Feedback Loop ──────┘
```

### Deployment Options

| Option | Where Brain Runs | Latency | Use Case |
|--------|-----------------|---------|----------|
| **Companion** | Laptop via BT/Serial | ~50ms | Development, full features |
| **Embedded** | CyberPi (ESP32) | ~5ms | Standalone, portable |
| **Hybrid** | CyberPi + Cloud | ~100ms | Advanced AI, learning |

---

## Success Metrics

### Joy Metrics (Primary)
- **Smile Rate** - Do people smile when they see it?
- **"Show Someone" Rate** - Do people want to share it?
- **Return Rate** - Do people come back to play more?
- **Surprise Moments** - Unexpected delightful behaviors per session

### Technical Metrics (Secondary)
- Response latency < 50ms
- Battery life > 2 hours active play
- Personality consistency > 90%
- No "uncanny valley" moments

### Safety Metrics (Non-negotiable)
- Zero harmful behaviors
- Zero scary moments for target age
- 100% Kitchen Table Test pass rate

---

## Target Users

### Primary: Kids (7-14)
- First exposure to robotics/AI
- Want a toy that surprises them
- Love customization
- Share with friends

### Secondary: Makers/Hobbyists
- Want to understand AI through play
- Modify and extend
- Build new personalities
- Create custom games

### Tertiary: Educators
- Teach AI concepts tangibly
- Classroom activities
- STEM curriculum integration
- Assessment through play

### Quaternary: Families
- Multi-generational play
- Living room entertainment
- Conversation starter
- Tech-positive family time

---

## Constraints

### Hardware Limitations (mBot2)
- 2x DC motors (tank drive)
- 1x Ultrasonic sensor
- 1x Quad RGB sensor (line following)
- 8x RGB LEDs
- 1x Buzzer
- Optional: Servo for pen

### Design Constraints
- Must work with unmodified mBot2
- Must be safe for unsupervised play
- Must work offline (no cloud required)
- Must be explainable to a 10-year-old

### Ethical Constraints
- No data collection beyond device
- No connection to social media
- No competitive "pay to win" mechanics
- No addictive engagement patterns

---

## Open Questions

1. **Personality Persistence** - Should the robot "grow" over time or reset?
2. **Learning from Play** - Can it learn your preferences without being creepy?
3. **Multi-Robot Interaction** - What happens when two RuVector bots meet?
4. **Sound Design** - What does a "nervous" robot sound like?
5. **Physical Mods** - Official pen holder? LEGO-compatible attachments?

---

## Appendix: The Magic IF Moments

These are the moments we're designing for:

1. "Wait, did it just... get *nervous*?"
2. "It's drawing a spiral but then something startled it and the line went crazy!"
3. "My robot is totally different from yours even though they're the same!"
4. "It actually tries to win at tic-tac-toe, and it gets mad when it loses!"
5. "It sorted all my LEGOs but kept the rare ones for itself!"
6. "I left it alone and when I came back it had drawn a whole picture!"
7. "It recognized me! It does this little wiggle when I come back!"
8. "Watch - if I move too fast it gets scared, but if I'm gentle it comes to me."

**These moments don't come from code. They emerge from the nervous system.**

That's the magic.
