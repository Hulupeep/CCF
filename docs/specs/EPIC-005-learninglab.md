# EPIC-005: LearningLab - The Education Kit

**"AI you can touch"**

## Overview

Transform mBot2 + RuVector into an educational platform for understanding artificial intelligence and emergent behavior. Instead of abstract concepts, students interact with a physical system that demonstrates AI principles in real-time.

## User Value

- **Tangible AI**: Touch and see AI concepts in action
- **Experimentation**: Safely modify parameters and see results
- **Understanding**: Grasp complex ideas through play
- **Curriculum**: Ready-made lesson plans for educators

---

## Architecture Requirements

### ARCH-LEARN-001 (MUST)
All visualizations must update in real-time (< 100ms latency).

### ARCH-LEARN-002 (MUST)
Parameter adjustments must be immediately visible in behavior.

### ARCH-LEARN-003 (MUST)
Educational mode must never allow harmful parameter combinations.

### ARCH-LEARN-004 (MUST)
All data visualized must be explainable to a 10-year-old.

---

## Feature Requirements

### LEARN-001 (MUST): Nervous System Visualizer
Real-time display of the robot's "brain" activity.

**Displays:**
- Current reflex mode (with icon)
- Tension/Coherence/Energy as animated gauges
- Sensor inputs as live values
- Motor outputs as directional arrows
- DAG nodes lighting up as they process

**Gherkin:**
```gherkin
@LEARN-001
Scenario: Visualizer shows real-time updates
  Given the nervous system visualizer is open
  When the robot detects an object
  Then the ultrasonic sensor display should update
  And tension gauge should increase
  And reflex mode should potentially change
  All within 100ms
```

### LEARN-002 (MUST): Personality Mixer
Adjust personality parameters with sliders, see immediate results.

**Controls:**
- All personality parameters as sliders
- Preset buttons for quick comparison
- "Randomize" for exploration
- Real-time behavior change

**Gherkin:**
```gherkin
@LEARN-002
Scenario: Parameter change causes immediate behavior change
  Given the personality mixer is open
  And energy_baseline is at 0.5
  When user moves energy_baseline to 0.9
  Then the robot should visibly increase activity
  Within 2 seconds of slider release
```

### LEARN-003 (MUST): Reflex Lab
Experiment with stimulus → response relationships.

**Experiments:**
1. **Startle Response**: Adjust sensitivity, test with stimuli
2. **Approach/Avoid**: Modify curiosity vs fear thresholds
3. **Recovery Time**: Watch tension return to baseline
4. **Mode Transitions**: See what triggers each reflex mode

**Gherkin:**
```gherkin
@LEARN-003
Scenario: Startle sensitivity experiment
  Given reflex lab is open on "Startle Response"
  When user sets startle_sensitivity to maximum
  And produces a small stimulus
  Then robot should have large startle response
  And student can observe cause and effect
```

### LEARN-004 (SHOULD): Emotion Timeline
Chart the robot's emotional journey over time.

**Features:**
- Line graph of tension/coherence/energy
- Event markers (stimuli, mode changes)
- Zoom in/out on time periods
- Export for analysis

**Gherkin:**
```gherkin
@LEARN-004
Scenario: Timeline records emotional history
  Given a 5-minute observation session
  When the session ends
  Then the timeline should show continuous mood data
  And all stimulus events should be marked
  And mode changes should be highlighted
```

### LEARN-005 (SHOULD): Compare Mode
Run two personalities side-by-side (simulated or two robots).

**Gherkin:**
```gherkin
@LEARN-005
Scenario: Compare personality responses
  Given compare mode with "Curious Cleo" and "Nervous Nellie"
  When identical stimulus is applied
  Then both responses should be shown
  And differences should be highlighted
```

### LEARN-006 (SHOULD): Lesson Plans
Pre-built curriculum for educators.

**Topics:**
1. What is AI? (Introduction)
2. Sensors and Perception (Input)
3. The Nervous System (Processing)
4. Emergent Behavior (Output)
5. Personality and Parameters (Customization)
6. Building Your Own AI (Project)

### LEARN-007 (MAY): Data Export
Export session data for classroom analysis.

### LEARN-008 (MAY): Class Mode
Teacher controls multiple robots for demonstrations.

---

## Journey: J-LEARN-FIRST-EXPERIMENT

**DOD Criticality: CRITICAL**

Student's first hands-on experiment with AI.

### Preconditions
- Companion app with LearningLab module
- Robot connected and active
- Nervous system visualizer open

### Steps

| Step | Action | Robot/App Response | Learning Outcome |
|------|--------|-------------------|------------------|
| 1 | Opens visualizer | Real-time nervous system display | "The robot has a 'brain'" |
| 2 | Observes idle robot | Calm, steady gauges | "It has a baseline state" |
| 3 | Waves hand near robot | Spike in tension, mode change | "It detected me!" |
| 4 | Removes hand | Tension gradually decreases | "It calms down over time" |
| 5 | Opens personality mixer | Slider controls appear | "I can adjust its 'personality'" |
| 6 | Increases startle sensitivity | - | "Let's see what happens" |
| 7 | Waves hand again | Much larger startle response | "Parameters change behavior!" |
| 8 | Creates custom personality | Saves with name | "I designed an AI" |

### Expected Outcome
Student understands: sensors → processing → behavior, and that parameters affect the system.

---

## Journey: J-LEARN-CLASS-DEMO

**DOD Criticality: IMPORTANT**

Teacher demonstrates AI concepts to a class.

### Preconditions
- Large display connected to companion app
- Robot visible to all students
- LearningLab in "demo mode"

### Steps

| Step | Teacher Action | Display Shows | Class Sees |
|------|---------------|---------------|------------|
| 1 | "Let's meet our AI" | Robot intro, visualizer | Friendly robot |
| 2 | "Watch its 'nervous system'" | Real-time gauges | Living system |
| 3 | "What happens if I scare it?" | Tension spike, mode change | Cause and effect |
| 4 | "Let's make it brave" | Adjusts startle sensitivity down | Parameter control |
| 5 | "Now watch" | Same stimulus, smaller response | Scientific method |
| 6 | "Your turn to experiment" | Hands off to students | Active learning |

### Expected Outcome
Class understands AI basics and is excited to experiment.

---

## Data Contracts

### VisualizerState
```typescript
interface VisualizerState {
  // Real-time nervous system state
  reflex_mode: 'calm' | 'active' | 'spike' | 'protect';
  tension: number;
  coherence: number;
  energy: number;

  // Sensor readings
  sensors: {
    ultrasonic_cm: number;
    light_level: number;
    sound_level: number;
    quad_rgb: number[][];
  };

  // Motor outputs
  motors: {
    left: number;
    right: number;
    led_color: number[];
    pen_angle: number;
  };

  // Timing
  timestamp: number;
  tick_count: number;
}
```

### ExperimentSession
```typescript
interface ExperimentSession {
  id: string;
  student_name?: string;
  started_at: number;
  ended_at?: number;

  // Time series data
  states: VisualizerState[];

  // Events
  stimuli: { time: number; type: string; magnitude: number }[];
  parameter_changes: { time: number; param: string; value: number }[];
  mode_transitions: { time: number; from: string; to: string }[];

  // Analysis
  observations?: string[];
  conclusions?: string[];
}
```

### LessonPlan
```typescript
interface LessonPlan {
  id: string;
  title: string;
  grade_level: string;  // "K-2", "3-5", "6-8", "9-12"
  duration_minutes: number;

  objectives: string[];
  materials: string[];

  activities: {
    name: string;
    duration: number;
    instructions: string;
    discussion_questions: string[];
  }[];

  assessment: string;
  extensions: string[];
}
```

---

## Stories

### STORY-LEARN-001: Real-Time Visualizer
**Points:** 8
**Covers:** LEARN-001, ARCH-LEARN-001

As a student, I need to see the robot's brain activity so I understand how it works.

**Tasks:**
- [ ] WebSocket real-time data streaming
- [ ] Reflex mode display with animations
- [ ] Gauge components for tension/coherence/energy
- [ ] Sensor value displays
- [ ] Motor output visualization
- [ ] < 100ms update latency

### STORY-LEARN-002: Personality Mixer UI
**Points:** 5
**Covers:** LEARN-002, ARCH-LEARN-002

As a student, I need to adjust personality parameters and see immediate results.

**Tasks:**
- [ ] Slider components for all parameters
- [ ] Real-time parameter transmission
- [ ] Preset personality buttons
- [ ] "Randomize" button
- [ ] Parameter bounds enforcement

### STORY-LEARN-003: Reflex Lab Experiments
**Points:** 5
**Covers:** LEARN-003

As a student, I need guided experiments to understand stimulus-response.

**Tasks:**
- [ ] Startle sensitivity experiment
- [ ] Approach/avoid experiment
- [ ] Recovery time experiment
- [ ] Mode transition explorer
- [ ] Guided instructions for each

### STORY-LEARN-004: Emotion Timeline
**Points:** 5
**Covers:** LEARN-004

As a student, I need to see the robot's emotional history over time.

**Tasks:**
- [ ] Time series chart component
- [ ] Event marker system
- [ ] Zoom/pan controls
- [ ] Session recording
- [ ] Export functionality

### STORY-LEARN-005: Lesson Plan Framework
**Points:** 3
**Covers:** LEARN-006

As an educator, I need structured lesson plans to teach AI concepts.

**Tasks:**
- [ ] Lesson plan data structure
- [ ] 6 initial lesson plans
- [ ] PDF export for printing
- [ ] Discussion questions
- [ ] Assessment rubrics

### STORY-LEARN-006: First Experiment Journey Test
**Points:** 5
**Covers:** J-LEARN-FIRST-EXPERIMENT

As a tester, I need to verify the first experiment flow works for students.

**Tasks:**
- [ ] Journey test file
- [ ] UI interaction tests
- [ ] Learning outcome verification
- [ ] Accessibility testing
- [ ] Document with video

---

## Dependencies

- **Requires:** EPIC-002 (Personality) for parameter system
- **Requires:** Web dashboard (companion app)
- **Soft dependency:** Large display for classroom use

---

## Educational Alignment

### Standards Coverage
- **NGSS**: Science and Engineering Practices
- **CSTA**: Computing Systems, Algorithms
- **ISTE**: Computational Thinking, Innovative Designer

### Age Appropriateness
| Feature | Ages 5-8 | Ages 9-12 | Ages 13+ |
|---------|----------|-----------|----------|
| Visualizer | Simplified | Full | Full + data |
| Mixer | 3 sliders | All sliders | + custom params |
| Timeline | Picture only | Basic graph | Full analysis |
| Lessons | Play-based | Guided | Self-directed |

---

## Open Questions

1. How do we assess learning outcomes?
2. Should there be "achievements" for experiments?
3. Can students share their custom personalities online?
4. What accessibility features are needed for diverse learners?
