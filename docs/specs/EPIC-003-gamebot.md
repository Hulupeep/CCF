# EPIC-003: GameBot - The Play Partner

**"Finally, a robot that actually plays"**

## Overview

Transform mBot2 into a gaming companion that plays real games - not scripted routines, but actual gameplay with strategy, emotion, and the will to win (or the grace to lose). The robot's personality affects how it plays, making every game unique.

## User Value

- **Real Competition**: Play against something that actually tries
- **Emotional Stakes**: Robot celebrates wins, sulks at losses
- **Skill Levels**: Games that grow with you
- **Social Play**: Something to play with when friends aren't around

---

## Architecture Requirements

### ARCH-GAME-001 (MUST)
Game logic must be separate from nervous system - games query mood, don't control it.

### ARCH-GAME-002 (MUST)
All games must have timeout/forfeit mechanisms to prevent stuck states.

### ARCH-GAME-003 (MUST)
Game difficulty must never require impossible physical movements.

### ARCH-GAME-004 (MUST)
Games must be playable without companion app (physical-only mode).

---

## Feature Requirements

### GAME-001 (MUST): Tic-Tac-Toe
The classic game, drawn on paper with personality.

**Gameplay:**
- Robot draws grid
- Human marks X (or points to position)
- Robot "thinks" (personality affects thinking time/style)
- Robot draws O in chosen position
- Winner celebration or loser sulk

**AI Levels:**
| Level | Strategy | Personality |
|-------|----------|-------------|
| Easy | Random valid moves | Playful, doesn't care about winning |
| Medium | Block wins, take center | Competitive but fair |
| Hard | Minimax optimal | Focused, intense |

**Gherkin:**
```gherkin
@GAME-001
Scenario: Robot plays tic-tac-toe competently
  Given a new tic-tac-toe game on medium difficulty
  When the human plays center
  Then the robot should play a corner
  And show "thinking" behavior before moving
```

```gherkin
@GAME-001
Scenario: Robot celebrates winning
  Given the robot has won tic-tac-toe
  Then the robot should perform victory behavior
  And LEDs should flash green
  And a celebratory sound should play
```

```gherkin
@GAME-001
Scenario: Robot handles losing gracefully
  Given the human has won tic-tac-toe
  Then the robot should show disappointment
  And offer to play again
  And NOT become aggressive or refuse to play
```

### GAME-002 (MUST): Chase/Tag
Physical game of tag where the robot is "it" or runs away.

**Robot is "It" Mode:**
- Robot chases human hand/foot
- Uses ultrasonic to track
- Celebrates when it "tags" (gets within 5cm)

**Human is "It" Mode:**
- Robot flees from approaching objects
- Uses evasive maneuvers
- Personality affects fleeing style (nervous = erratic, chill = lazy dodges)

**Gherkin:**
```gherkin
@GAME-002
Scenario: Robot chases effectively
  Given the robot is in "chase" mode
  When an object is detected at 30cm
  Then the robot should move toward it
  And increase speed as distance decreases
  And declare "tag" when within 5cm
```

```gherkin
@GAME-002
Scenario: Robot evades pursuers
  Given the robot is in "flee" mode
  When an object approaches from the front
  Then the robot should turn and move away
  And evasion style should match personality
```

### GAME-003 (MUST): Simon Says
Robot issues commands (via LEDs/sounds), human must obey.

**Gameplay:**
- Robot shows color pattern
- Human repeats with colored cards or app buttons
- Patterns get longer
- Robot judges correctness (via app or assumed)

**Gherkin:**
```gherkin
@GAME-003
Scenario: Simon pattern increases difficulty
  Given a Simon Says game in progress
  When the human completes round 3
  Then round 4 should have 4 colors
  And the pattern should include previous colors plus one
```

### GAME-004 (SHOULD): Dance Battle
Robot and human take turns dancing. Robot learns from human!

**Gameplay:**
- Human "dances" (waves hand in patterns)
- Robot detects movement via ultrasonic
- Robot performs its own dance
- Turn-based, escalating complexity

**Gherkin:**
```gherkin
@GAME-004
Scenario: Robot performs unique dance moves
  Given a dance battle is in progress
  When it's the robot's turn
  Then the robot should perform a dance sequence
  And moves should be influenced by personality
  And include spins, wiggles, or bounces
```

### GAME-005 (SHOULD): Hide and Seek
Robot hides or seeks. Real hiding behaviors!

**Seeker Mode:**
- Robot counts (beeps)
- Then searches in pattern
- Uses ultrasonic to "find" hidden human
- Personality affects search style

**Hider Mode:**
- Robot drives to random location
- Stops and "hides" (turns off LEDs, silent)
- Reacts when found

**Gherkin:**
```gherkin
@GAME-005
Scenario: Robot seeks systematically
  Given the robot is in "seeker" mode
  And has finished counting
  Then it should search the area
  And react excitedly when detecting a close object
```

### GAME-006 (MAY): Racing
Two robots race. Personality affects racing style.

### GAME-007 (MAY): Obstacle Course
Robot navigates course. Time trials, personality-based style.

---

## Journey: J-GAME-FIRST-TICTACTOE

**DOD Criticality: CRITICAL**

First complete tic-tac-toe game against the robot.

### Preconditions
- ArtBot features working (drawing capability)
- Paper with grid OR robot draws grid
- Method to indicate human moves (point or app)

### Steps

| Step | User Action | Robot Response | Verification |
|------|-------------|----------------|--------------|
| 1 | Starts tic-tac-toe | Robot draws grid | 3x3 grid visible |
| 2 | User plays X (center) | Robot "thinks" with LEDs | Thinking animation |
| 3 | - | Robot draws O (corner) | Valid move made |
| 4 | User plays X | Robot evaluates board | No pause > 5s |
| 5 | Continue until end | Robot plays valid moves | Game completes |
| 6 | Game ends | Winner celebration or graceful loss | Emotional response |
| 7 | - | Robot offers rematch | "Play again?" behavior |

### Expected Outcome
User plays a complete, fair game of tic-tac-toe and experiences the robot as a real opponent with emotions.

---

## Journey: J-GAME-CHASE-FUN

**DOD Criticality: IMPORTANT**

Playing chase/tag with the robot.

### Preconditions
- Open floor space (1m x 1m minimum)
- Robot in Chase mode

### Steps

| Step | User Action | Robot Response | Verification |
|------|-------------|----------------|--------------|
| 1 | Enables Chase mode | Robot enters "stalking" pose | LED color change |
| 2 | Places hand 50cm away | Robot slowly approaches | Movement detected |
| 3 | Pulls hand back | Robot follows | Chase behavior |
| 4 | Hand at 5cm | Robot "tags" and celebrates | Victory behavior |
| 5 | Switches to Flee mode | Robot backs away | Mode change |
| 6 | Approaches robot | Robot evades | Evasion pattern |
| 7 | "Catches" robot | Robot acts defeated | Graceful loss |

### Expected Outcome
User has active physical play that feels like playing with a pet or friend.

---

## Data Contracts

### GameState
```typescript
interface GameState {
  game_type: 'tictactoe' | 'chase' | 'simon' | 'dance' | 'hide_seek';
  status: 'waiting' | 'playing' | 'won' | 'lost' | 'draw';
  difficulty: 'easy' | 'medium' | 'hard';
  turn: 'robot' | 'human';
  score: { robot: number; human: number };
  round: number;
}
```

### TicTacToeBoard
```typescript
interface TicTacToeBoard {
  cells: ('X' | 'O' | null)[];  // 9 cells
  winner: 'X' | 'O' | 'draw' | null;
  winning_line: number[] | null;  // Indices of winning cells
}
```

### ChaseState
```typescript
interface ChaseState {
  mode: 'chase' | 'flee';
  target_distance: number;  // cm
  target_angle: number;     // degrees
  tagged: boolean;
  evasion_count: number;
}
```

---

## Stories

### STORY-GAME-001: Tic-Tac-Toe Core Logic
**Points:** 5
**Covers:** GAME-001, ARCH-GAME-001

As a robot, I need to play valid tic-tac-toe moves so I can compete fairly.

**Tasks:**
- [ ] Implement TicTacToeBoard struct
- [ ] Add win/draw detection
- [ ] Implement easy AI (random)
- [ ] Implement medium AI (blocking)
- [ ] Implement hard AI (minimax)
- [ ] Unit tests for all scenarios

### STORY-GAME-002: Tic-Tac-Toe Physical Drawing
**Points:** 5
**Covers:** GAME-001

As a robot, I need to draw the tic-tac-toe board and my moves.

**Tasks:**
- [ ] Grid drawing routine
- [ ] X drawing at position
- [ ] O drawing at position
- [ ] Position calibration system
- [ ] Integration with ArtBot

### STORY-GAME-003: Game Emotional Responses
**Points:** 3
**Covers:** GAME-001

As a robot, I need to show emotions during games so play feels alive.

**Tasks:**
- [ ] Thinking behavior (LEDs, small movements)
- [ ] Victory celebration
- [ ] Graceful loss response
- [ ] Draw response (shrug equivalent)
- [ ] Personality affects intensity

### STORY-GAME-004: Chase Game Mechanics
**Points:** 5
**Covers:** GAME-002

As a robot, I need to chase and flee effectively.

**Tasks:**
- [ ] Ultrasonic-based target tracking
- [ ] Chase movement algorithm
- [ ] Flee/evasion patterns
- [ ] Tag detection (distance threshold)
- [ ] Mode switching

### STORY-GAME-005: Simon Says Implementation
**Points:** 3
**Covers:** GAME-003

As a robot, I need to play Simon Says with patterns.

**Tasks:**
- [ ] Pattern generation
- [ ] LED display of pattern
- [ ] Pattern storage for comparison
- [ ] Difficulty progression
- [ ] Win/lose conditions

### STORY-GAME-006: First Tic-Tac-Toe Journey Test
**Points:** 5
**Covers:** J-GAME-FIRST-TICTACTOE

As a tester, I need to verify complete tic-tac-toe experience.

**Tasks:**
- [ ] Journey test file
- [ ] Test all game outcomes
- [ ] Verify emotional responses
- [ ] Test difficulty levels
- [ ] Document with video

---

## Dependencies

- **Requires:** EPIC-001 (ArtBot) for drawing games
- **Requires:** EPIC-002 (Personality) for emotional responses
- **Hardware:** Open floor space for Chase

---

## Open Questions

1. How does user indicate moves in physical-only mode?
2. Should there be a "practice" mode against itself?
3. Can multiple games track cumulative scores over days?
4. Should losing make the robot better (learning)?
