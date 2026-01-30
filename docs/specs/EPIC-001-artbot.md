# EPIC-001: ArtBot - The Drawing Companion

**"It draws what it feels"**

## Overview

Transform mBot2 into a creative drawing robot that translates its emotional state into visual art. The robot's mood directly influences its drawing style, creating unique artwork that reflects its "inner experience."

## User Value

- **Wonder**: Watch a robot create art from feelings, not code
- **Creativity**: Collaborative art where human and robot co-create
- **Learning**: Understand AI emotions through visual output
- **Keepsake**: Physical artwork created by your robot companion

---

## Architecture Requirements

### ARCH-ART-001 (MUST)
Pen servo control must be abstracted from drawing logic.
```rust
trait PenControl {
    fn pen_up(&mut self);
    fn pen_down(&mut self);
    fn set_angle(&mut self, angle: u8);
}
```

### ARCH-ART-002 (MUST)
Drawing commands must be queued, not immediate, to allow smoothing.

### ARCH-ART-003 (MUST)
All drawing movements must respect physical paper bounds (configurable).

---

## Feature Requirements

### ART-001 (MUST): Mood-to-Movement Translation
The robot's reflex mode must influence drawing characteristics:

| Reflex Mode | Line Style | Speed | Pattern |
|-------------|-----------|-------|---------|
| Calm 😌 | Smooth curves | Slow | Spirals, flowing |
| Active 🔍 | Angular, searching | Medium | Zigzags, explorations |
| Spike ⚡ | Sharp, sudden | Fast | Jagged, interrupted |
| Protect 🛡️ | Tight, small | Very slow | Defensive circles |

**Gherkin:**
```gherkin
@ART-001
Scenario: Calm mode produces smooth spirals
  Given the robot is in Calm reflex mode
  And the pen is down on paper
  When the robot draws for 10 seconds
  Then the path should contain smooth curves
  And no sharp angles greater than 45 degrees
```

### ART-002 (MUST): Pen Servo Control
Attach a pen via servo motor for drawing control.

**Acceptance Criteria:**
- Pen lifts cleanly (no drag marks)
- Pen pressure consistent when down
- Servo responds within 100ms
- Default: pen up when not drawing

**Gherkin:**
```gherkin
@ART-002
Scenario: Pen lifts cleanly between shapes
  Given the robot has completed drawing a shape
  When it moves to a new location
  Then the pen must be in the up position
  And no marks appear during transit
```

### ART-003 (MUST): Basic Shapes Library
The robot can draw basic shapes that form the vocabulary of its art:

- Circle (various sizes)
- Spiral (inward/outward)
- Line (various lengths)
- Arc (various curvatures)
- Scribble (controlled chaos)

**Gherkin:**
```gherkin
@ART-003
Scenario: Robot draws recognizable circle
  Given the robot is commanded to draw a circle
  And the radius is 5cm
  When drawing completes
  Then the shape should close within 5mm of start point
  And the path should be roughly circular (variance < 10%)
```

### ART-004 (MUST): Tension-Responsive Art
Higher tension = more erratic drawing. Lower tension = more deliberate.

**Gherkin:**
```gherkin
@ART-004
Scenario: High tension causes erratic drawing
  Given the robot tension is above 0.7
  When it draws a line
  Then the line should have visible wobble
  And direction changes should occur every 1-3cm
```

### ART-005 (SHOULD): Emotional Art Session
A complete drawing session that captures the robot's emotional journey:

1. Start calm (spirals)
2. React to stimuli (style changes)
3. Return to calm (closing patterns)
4. Sign with a unique mark

**Gherkin:**
```gherkin
@ART-005
Scenario: Complete emotional art session
  Given the robot starts in Calm mode
  When a drawing session of 60 seconds runs
  And external stimuli occur during the session
  Then the artwork should show style transitions
  And the session should end with a closing pattern
```

### ART-006 (SHOULD): Drawing Memory
The robot remembers and can replay/modify previous drawings.

### ART-007 (MAY): Collaborative Drawing
Human starts a drawing, robot continues in its style.

---

## Journey: J-ART-FIRST-DRAWING

**DOD Criticality: CRITICAL**

A user's first experience creating art with their ArtBot.

### Preconditions
- mBot2 with pen servo attached
- Paper secured to flat surface
- Robot calibrated to paper bounds
- Companion app connected

### Steps

| Step | User Action | Robot Response | Required Elements |
|------|-------------|----------------|-------------------|
| 1 | Places robot on paper | LED indicates ready (blue pulse) | `[data-testid="status-ready"]` |
| 2 | Taps "Start Drawing" | Pen lowers, begins Calm spiral | `[data-testid="start-drawing"]` |
| 3 | Makes loud noise | Robot startles, drawing becomes jagged | Spike mode visible |
| 4 | Stays quiet | Robot calms, returns to smooth | Calm mode restored |
| 5 | Taps "Finish" | Robot signs and lifts pen | `[data-testid="finish-drawing"]` |
| 6 | Views result | Unique artwork with emotional journey | Physical paper output |

### Expected Outcome
User has a physical drawing that shows the robot's emotional response to their interaction.

---

## Data Contracts

### DrawingCommand
```typescript
interface DrawingCommand {
  type: 'move' | 'line' | 'arc' | 'pen_up' | 'pen_down';
  x?: number;  // mm from origin
  y?: number;  // mm from origin
  radius?: number;  // for arcs
  angle?: number;   // degrees
  speed?: number;   // 0-100
}
```

### DrawingSession
```typescript
interface DrawingSession {
  id: string;
  started_at: number;  // timestamp
  ended_at?: number;
  commands: DrawingCommand[];
  mood_snapshots: HomeostasisState[];
  stimuli_events: StimulusEvent[];
}
```

### ArtStyle
```typescript
interface ArtStyle {
  line_smoothness: number;  // 0-1
  speed_factor: number;     // 0.5-2.0
  pattern_complexity: number; // 0-1
  color_preference?: string; // for LED indication
}
```

---

## Stories

### STORY-ART-001: Implement Pen Servo Control
**Points:** 3
**Covers:** ART-002, ARCH-ART-001

As a robot, I need to control a pen servo so that I can draw on paper.

**Tasks:**
- [ ] Add servo command to protocol.rs
- [ ] Create PenControl trait
- [ ] Implement pen_up/pen_down with timing
- [ ] Add pen_angle to MotorCommand
- [ ] Test with physical servo

### STORY-ART-002: Mood-to-Line-Style Mapping
**Points:** 5
**Covers:** ART-001, ART-004

As a robot, I need to translate my mood into drawing characteristics so my art reflects my feelings.

**Tasks:**
- [ ] Define ArtStyle struct
- [ ] Map ReflexMode → ArtStyle
- [ ] Add tension influence on wobble
- [ ] Add coherence influence on smoothness
- [ ] Unit tests for mapping

### STORY-ART-003: Basic Shape Drawing
**Points:** 5
**Covers:** ART-003

As a robot, I need to draw basic shapes so I have a vocabulary for my art.

**Tasks:**
- [ ] Implement circle drawing (encoder-based)
- [ ] Implement spiral (increasing/decreasing radius)
- [ ] Implement line with length
- [ ] Implement arc with curvature
- [ ] Implement scribble pattern

### STORY-ART-004: Drawing Session Manager
**Points:** 3
**Covers:** ART-005, ART-006

As a robot, I need to manage complete drawing sessions so my art has narrative.

**Tasks:**
- [ ] Create DrawingSession struct
- [ ] Record commands and mood snapshots
- [ ] Implement session start/end
- [ ] Add signing behavior at end
- [ ] Save session for replay

### STORY-ART-005: First Drawing Journey Test
**Points:** 5
**Covers:** J-ART-FIRST-DRAWING

As a tester, I need to verify the first drawing journey works end-to-end.

**Tasks:**
- [ ] Create journey test file
- [ ] Implement stimulus injection for test
- [ ] Verify mood transitions in drawing
- [ ] Verify physical output (visual inspection protocol)
- [ ] Document journey with photos

---

## Test Requirements

### Contract Tests (Pre-build)
- `architecture.test.ts` - Verify ARCH-ART-* rules
- `artbot_features.test.ts` - Verify ART-* patterns

### Unit Tests
- Pen servo timing
- Mood-to-style mapping
- Shape geometry calculations

### Integration Tests
- Full drawing session simulation
- Transport layer with servo commands

### Journey Tests (Post-build, Playwright)
- `journey_first_drawing.spec.ts` - E2E first drawing experience

---

## Dependencies

- **Requires:** EPIC-000 (Core nervous system) ✅ Complete
- **Enables:** EPIC-003 (Tic-Tac-Toe game)
- **Hardware:** Servo motor, pen holder attachment

---

## Open Questions

1. What paper size should be the default bounds?
2. Should we support multiple pen colors (servo selection)?
3. How do we handle pen running out of ink?
4. Should drawings be photographed automatically?
