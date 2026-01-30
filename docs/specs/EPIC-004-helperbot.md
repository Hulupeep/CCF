# EPIC-004: HelperBot - The Useful Friend

**"Chores become adventures"**

## Overview

Transform mBot2 into a helpful companion that performs practical tasks - but with personality. It's not just sorting LEGOs, it's a character who has *opinions* about which pieces are best. It's not just delivering snacks, it's a friend bringing you a gift.

## User Value

- **Utility**: Actual helpful tasks
- **Entertainment**: Chores become play
- **Companionship**: A helper that has personality
- **Learning**: Watch AI solve real problems

---

## Architecture Requirements

### ARCH-HELP-001 (MUST)
Helper tasks must have clear completion criteria - no infinite loops.

### ARCH-HELP-002 (MUST)
Object manipulation must account for robot's limited pushing ability.

### ARCH-HELP-003 (MUST)
Tasks must be interruptible - user can always stop mid-task.

### ARCH-HELP-004 (MUST)
Helper behaviors must not damage objects or surfaces.

---

## Feature Requirements

### HELP-001 (MUST): LEGO Sorter
Sort LEGO bricks by color using the quad RGB sensor.

**How it works:**
1. LEGOs spread on white paper
2. Robot approaches each piece
3. Reads color with RGB sensor
4. Pushes piece to corresponding zone
5. Has opinions about rare pieces (excited about special colors)

**Gherkin:**
```gherkin
@HELP-001
Scenario: Robot sorts LEGO by color
  Given LEGO pieces scattered on white surface
  When the robot starts sorting
  Then each piece should be pushed to the matching color zone
  And common colors sorted with normal demeanor
  And rare colors (gold, clear) trigger excitement
```

```gherkin
@HELP-001
Scenario: Robot expresses preference for rare pieces
  Given the robot detects a gold/special LEGO piece
  Then it should pause and "admire" the piece
  And LEDs should reflect excitement
  And it may "reluctantly" push it to the zone
```

### HELP-002 (MUST): Desk Patrol
Robot patrols desk, pushes stray items back toward center.

**How it works:**
1. Define desk boundary
2. Robot patrols perimeter
3. When object detected, gently pushes inward
4. Personality affects patrol style (diligent vs lazy)

**Gherkin:**
```gherkin
@HELP-002
Scenario: Robot patrols and nudges objects
  Given a desk with items near edges
  When the robot is in patrol mode
  Then it should circle the desk boundary
  And push edge items toward center
  And complete a full patrol within 5 minutes
```

```gherkin
@HELP-002
Scenario: Grumpy personality affects patrol
  Given the active personality is "Grumpy Gus"
  When patrolling
  Then the robot should patrol slower
  And occasionally stop to "complain" (sounds/LEDs)
  And push items with visible reluctance
```

### HELP-003 (SHOULD): Follow Mode
Robot follows the user around like a pet.

**How it works:**
- User walks, robot follows at set distance
- Maintains 30-50cm following distance
- Personality affects following style

**Gherkin:**
```gherkin
@HELP-003
Scenario: Robot follows user
  Given follow mode is active
  When user moves forward
  Then robot should move forward to maintain 40cm distance
  And stop when user stops
```

### HELP-004 (SHOULD): Snack Delivery
Robot delivers small items from point A to B.

**How it works:**
1. Place item on robot or in small trailer
2. Tell robot destination (or follow user)
3. Robot navigates to destination
4. "Presents" item with personality flourish

**Gherkin:**
```gherkin
@HELP-004
Scenario: Robot delivers item
  Given an item is placed on the robot
  And a destination is set
  When delivery is started
  Then robot should navigate to destination
  And announce arrival with sounds/lights
  And wait for item to be taken
```

### HELP-005 (SHOULD): Wake Up Call
Robot wakes you up with increasing persistence.

**How it works:**
1. Set wake time in app
2. Robot starts gentle (soft sounds, slow approach)
3. If no response, escalates (louder, closer)
4. Personality affects style (gentle vs chaotic)

**Gherkin:**
```gherkin
@HELP-005
Scenario: Robot provides gentle wake up
  Given wake up time has arrived
  Then robot should start with soft sounds
  And gradually approach the bed
  And escalate intensity if no response within 2 minutes
```

### HELP-006 (MAY): Plant Check
Robot checks soil moisture indicator, reports plant health.

### HELP-007 (MAY): Remote Presence
Robot acts as mobile webcam for remote family members.

---

## Journey: J-HELP-LEGO-SORT

**DOD Criticality: CRITICAL**

Sorting a pile of LEGO bricks by color.

### Preconditions
- White paper as sorting surface
- 10+ LEGO bricks (multiple colors)
- Colored paper zones for sorted piles
- Clear path for robot movement

### Steps

| Step | User Action | Robot Response | Verification |
|------|-------------|----------------|--------------|
| 1 | Spreads LEGOs on white paper | Robot observes area | Ready behavior |
| 2 | Starts sorting via app | Robot approaches first piece | Movement begins |
| 3 | - | Robot reads color (RGB sensor down) | LED shows detected color |
| 4 | - | Robot pushes piece to matching zone | Correct destination |
| 5 | - | Robot proceeds to next piece | Systematic coverage |
| 6 | Gold piece detected | Robot shows excitement | Special behavior |
| 7 | Sorting complete | Robot celebrates | Victory behavior |
| 8 | Reviews results | Pieces sorted by color | 90%+ accuracy |

### Expected Outcome
User watches robot sort LEGOs with personality, feeling entertained and helped.

---

## Journey: J-HELP-DESK-TIDY

**DOD Criticality: IMPORTANT**

Robot tidies a messy desk.

### Preconditions
- Defined desk area (tape markers or app configuration)
- Small pushable objects near edges
- Clear center zone

### Steps

| Step | User Action | Robot Response | Verification |
|------|-------------|----------------|--------------|
| 1 | Places robot on desk | Robot orients to boundary | Ready state |
| 2 | Starts patrol mode | Robot begins perimeter scan | Movement pattern |
| 3 | - | Robot detects edge object | Ultrasonic reading |
| 4 | - | Robot pushes object inward | Gentle movement |
| 5 | - | Robot continues patrol | Full coverage |
| 6 | Patrol complete | Robot parks in corner | Task complete signal |

### Expected Outcome
Desk is tidier and user was entertained watching the process.

---

## Data Contracts

### SortingTask
```typescript
interface SortingTask {
  id: string;
  type: 'lego' | 'custom';
  zones: ColorZone[];
  items_sorted: number;
  items_remaining: number;
  special_finds: string[];  // Notable items
  status: 'active' | 'paused' | 'complete';
}
```

### ColorZone
```typescript
interface ColorZone {
  color: string;
  position: { x: number; y: number };
  count: number;
}
```

### PatrolConfig
```typescript
interface PatrolConfig {
  boundary: { x: number; y: number }[];  // Polygon points
  patrol_speed: number;  // 0-100
  push_distance: number;  // How far to push items (cm)
  personality_modifier: number;  // Affects style
}
```

---

## Stories

### STORY-HELP-001: Color Detection System
**Points:** 5
**Covers:** HELP-001

As a robot, I need to accurately detect LEGO colors so I can sort correctly.

**Tasks:**
- [ ] Calibrate RGB sensor for common LEGO colors
- [ ] Build color lookup table
- [ ] Handle edge cases (black, white, clear)
- [ ] Add "rare" color detection (gold, etc.)
- [ ] Test accuracy with real LEGOs

### STORY-HELP-002: Sorting Algorithm
**Points:** 5
**Covers:** HELP-001, ARCH-HELP-001

As a robot, I need a systematic sorting approach so I cover all pieces.

**Tasks:**
- [ ] Grid-based scanning pattern
- [ ] Piece detection algorithm
- [ ] Path planning to zones
- [ ] Completion detection
- [ ] Handling missed pieces

### STORY-HELP-003: LEGO Personality Behaviors
**Points:** 3
**Covers:** HELP-001

As a robot, I need to show personality while sorting so it's entertaining.

**Tasks:**
- [ ] Excitement for rare pieces
- [ ] Satisfaction progress sounds
- [ ] Frustration if piece won't move
- [ ] Celebration at completion
- [ ] Personality affects all behaviors

### STORY-HELP-004: Desk Patrol System
**Points:** 5
**Covers:** HELP-002

As a robot, I need to patrol a defined area and tidy it.

**Tasks:**
- [ ] Boundary definition system
- [ ] Perimeter patrol algorithm
- [ ] Edge object detection
- [ ] Gentle push mechanism
- [ ] Full coverage tracking

### STORY-HELP-005: Follow Mode
**Points:** 5
**Covers:** HELP-003

As a robot, I need to follow my user like a loyal companion.

**Tasks:**
- [ ] Distance maintenance algorithm
- [ ] Speed matching
- [ ] Loss of target handling
- [ ] Personality-based following style
- [ ] Stop/resume on command

### STORY-HELP-006: LEGO Sort Journey Test
**Points:** 5
**Covers:** J-HELP-LEGO-SORT

As a tester, I need to verify complete LEGO sorting experience.

**Tasks:**
- [ ] Journey test file
- [ ] Accuracy measurement
- [ ] Personality behavior verification
- [ ] Edge case handling
- [ ] Document with video

---

## Dependencies

- **Requires:** EPIC-002 (Personality) for character behaviors
- **Soft dependency:** Companion app for configuration
- **Hardware:** Quad RGB sensor, flat sorting surface

---

## Open Questions

1. Can the robot learn your preferred sorting categories?
2. What's the maximum object weight for pushing?
3. Should patrol record a "mess map" over time?
4. Can multiple robots cooperate on sorting?
