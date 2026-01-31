# J-SORT-RESET Journey Flow

## User Journey Flow

```mermaid
graph TD
    Start[User dumps mixed LEGO in tray] --> Check{Tray empty?}
    Check -->|Yes| Error[Display: Add pieces to begin]
    Check -->|No| Init[Press Start Sorting]

    Init --> PreCheck{Preconditions OK?}
    PreCheck -->|Calibrated?| PreCheck2{Carousel ready?}
    PreCheck -->|Not calibrated| ErrorCal[Show calibration error]
    PreCheck2 -->|Vision active?| BeginSort[Begin sorting loop]
    PreCheck2 -->|Not ready| ErrorSetup[Show setup error]

    BeginSort --> SortLoop{Pieces remaining?}

    SortLoop -->|Yes| PickPiece[Pick up piece]
    PickPiece --> Detect[Detect color]
    Detect --> Success{Pick successful?}

    Success -->|Yes| Place[Place in correct bin]
    Success -->|No| Retry{Retries < 3?}

    Retry -->|Yes| PickPiece
    Retry -->|No| SkipPiece[Skip piece, increment skip count]

    Place --> UpdateProgress[Update progress display]
    SkipPiece --> UpdateProgress

    UpdateProgress --> CheckJam{3 consecutive failures?}
    CheckJam -->|Yes| Jam[Pause, show jam message]
    CheckJam -->|No| CheckPause{Pause pressed?}

    CheckPause -->|Yes| Paused[Pause at home position]
    CheckPause -->|No| CheckEStop{E-Stop pressed?}

    CheckEStop -->|Yes| EmergencyStop[Halt immediately, save state]
    CheckEStop -->|No| SortLoop

    Jam --> ClearJam[User clears jam]
    ClearJam --> Resume1[Press Resume]

    Paused --> Resume2[Press Resume]
    Resume1 --> SortLoop
    Resume2 --> SortLoop

    EmergencyStop --> Resume3[Press Resume]
    Resume3 --> SortLoop

    SortLoop -->|No pieces| Complete[Display: Sorting Complete]

    Complete --> ShowSummary[Show completion summary]
    ShowSummary --> SummaryData{User taps details?}

    SummaryData -->|Yes| DetailView[Show bin-by-bin breakdown]
    SummaryData -->|No| Done[Journey complete]
    DetailView --> Done

    style Start fill:#90EE90
    style Complete fill:#90EE90
    style Done fill:#4169E1
    style Error fill:#FFB6C1
    style ErrorCal fill:#FFB6C1
    style ErrorSetup fill:#FFB6C1
    style Jam fill:#FFD700
    style EmergencyStop fill:#FF6347
```

## State Machine

```mermaid
stateDiagram-v2
    [*] --> Idle

    Idle --> CheckingPreconditions: Start pressed
    CheckingPreconditions --> Error: Preconditions fail
    CheckingPreconditions --> Active: Preconditions OK
    Error --> Idle: User fixes issue

    Active --> Paused: Pause pressed
    Active --> Stopped: E-Stop pressed
    Active --> JamDetected: 3 consecutive failures
    Active --> Complete: All pieces sorted

    Paused --> Active: Resume pressed
    JamDetected --> Active: Jam cleared + Resume
    Stopped --> Active: Resume pressed

    Complete --> [*]: User acknowledges

    note right of Active
        Sorting loop running
        - Pick piece
        - Detect color
        - Place in bin
        - Update progress
    end note

    note right of JamDetected
        Robot paused
        Waiting for manual
        jam clearance
    end note

    note right of Stopped
        Emergency stop
        State saved
        Safe to resume
    end note
```

## Test Scenario Coverage Map

```mermaid
mindmap
  root((J-SORT-RESET<br/>Tests))
    Critical Scenarios
      Happy Path
        20 pieces → sorted
        Progress tracking
        Summary display
        Inventory updates
      Partial Completion
        13 sorted, 2 skipped
        Retry failures
        Skip tracking
      Pause/Resume
        Mid-sort pause
        State preservation
        Resume continuation
      Jam Recovery
        3 failures → jam
        Manual clear
        Resume sorting
    Standard Scenarios
      Empty Tray
        Detection
        Error message
      Summary Details
        Bin breakdown
        Before/after/delta
      Emergency Stop
        Immediate halt
        State saved
        Resume capability
    Contract Validation
      SORT-001
        Calibration drift ≤2°
        Pre/post check
      SORT-004
        Inventory persistence
        Reload verification
    Performance
      Sorting Rate
        ≤6s per piece
        ≤120s for 20 pieces
        ≥80% success rate
```

## Data Flow

```mermaid
flowchart LR
    subgraph Input
        Tray[Tray with<br/>mixed pieces]
        User[User actions]
    end

    subgraph Processing
        Vision[Vision system<br/>detects color]
        Gripper[Gripper picks<br/>piece]
        PathPlan[Path planning]
        Place[Place in bin]
    end

    subgraph State
        Progress[Progress tracking]
        Inventory[Inventory counts]
        Metrics[Journey metrics]
    end

    subgraph Output
        Display[Progress display]
        Summary[Completion summary]
        Deltas[Bin deltas]
    end

    Tray --> Vision
    User --> Control{Control signals}

    Control -->|Start| Vision
    Control -->|Pause| Paused[Paused state]
    Control -->|Resume| Vision
    Control -->|E-Stop| Stopped[Stopped state]

    Vision --> Gripper
    Gripper --> PathPlan
    PathPlan --> Place

    Place --> Progress
    Place --> Inventory
    Place --> Metrics

    Progress --> Display
    Metrics --> Summary
    Inventory --> Deltas

    Paused --> Progress
    Stopped --> Progress
```

## Test ID Mapping

```mermaid
graph TD
    subgraph Preconditions
        TC1[calibration-status]
        TC2[carousel-status]
        TC3[vision-status]
        TC4[tray-status]
    end

    subgraph Journey Control
        TJ1[journey-start]
        TJ2[journey-pause]
        TJ3[journey-resume]
        TJ4[emergency-stop]
    end

    subgraph Progress Tracking
        TP1[journey-progress]
        TP2[sorting-status]
        TP3[pieces-remaining]
        TP4[robot-position]
    end

    subgraph Summary Display
        TS1[journey-complete]
        TS2[journey-summary]
        TS3[summary-sorted]
        TS4[summary-skipped]
        TS5[summary-duration]
        TS6[summary-bins]
        TS7[summary-details]
    end

    subgraph Inventory
        TI1[bin-count-binid]
        TI2[delta-breakdown]
        TI3[delta-binid]
    end

    subgraph Error Handling
        TE1[error-message]
        TE2[clear-jam]
    end

    subgraph Test Setup
        TT1[test-setup]
        TT2[piece-count]
        TT3[colors]
        TT4[apply-test-setup]
    end
```

## Performance Targets

| Metric | Target | Test Validation |
|--------|--------|----------------|
| **Pieces per minute** | ≥10 | Measured in performance test |
| **Time per piece** | ≤6 seconds | Avg calculated from total time |
| **Success rate** | ≥80% | Sorted / (Sorted + Skipped) |
| **Total time (20 pieces)** | ≤120 seconds | Timeout enforced |
| **Calibration drift** | ≤2° | Pre/post comparison |
| **Jam detection** | 3 consecutive failures | Simulated in test |
| **Pause latency** | Current op completes | ≤10 seconds |
| **E-Stop latency** | Immediate halt | ≤2 seconds |

## Contract Validation Points

```mermaid
sequenceDiagram
    participant Test
    participant System
    participant Hardware
    participant Storage

    Note over Test,Storage: Contract SORT-001: Calibration Persistence

    Test->>System: Check calibration before
    System->>Hardware: Read servo angles
    Hardware-->>System: Angles: [90°, 45°, 180°]
    System-->>Test: Calibrated: ±1°

    Test->>System: Start sorting journey
    System->>Hardware: Sort 20 pieces
    Hardware-->>System: Complete

    Test->>System: Check calibration after
    System->>Hardware: Read servo angles
    Hardware-->>System: Angles: [91°, 46°, 181°]
    System-->>Test: Calibrated: ±1° (drift: 1°)

    Test->>Test: Assert drift ≤2° ✓

    Note over Test,Storage: Contract SORT-004: Inventory Persistence

    Test->>System: Complete sorting
    System->>Storage: Save inventory counts
    Storage-->>System: Saved

    Test->>System: Reload page
    System->>Storage: Load inventory counts
    Storage-->>System: Red: 18, Blue: 12, etc.

    Test->>Test: Assert counts match ✓
```

## Error Recovery Flow

```mermaid
flowchart TD
    Active[Active Sorting]

    Active --> E1{Error Type?}

    E1 -->|Pick failure| Retry[Retry pick]
    E1 -->|Detection failure| Retry
    E1 -->|Place failure| Retry

    Retry --> Count{Retry count?}
    Count -->|< 3| Active
    Count -->|= 3| Check{Consecutive?}

    Check -->|Yes, 3 consecutive| Jam[Jam detected]
    Check -->|No| Skip[Skip piece]

    Skip --> Active

    Jam --> Pause1[Pause at home]
    Pause1 --> Display1[Display: Needs help]
    Display1 --> Wait1[Wait for user]
    Wait1 --> Clear[User clears jam]
    Clear --> Resume1[Resume pressed]
    Resume1 --> Active

    E1 -->|User pause| Pause2[Pause safely]
    Pause2 --> Wait2[Wait for resume]
    Wait2 --> Resume2[Resume pressed]
    Resume2 --> Active

    E1 -->|E-Stop| Stop[Emergency stop]
    Stop --> Save[Save state]
    Save --> Halt[Halt immediately]
    Halt --> Wait3[Wait for resume]
    Wait3 --> Resume3[Resume pressed]
    Resume3 --> Active

    Active --> Done{Complete?}
    Done -->|Yes| Summary[Show summary]
    Done -->|No| Active
```

## Key Learnings

1. **Pause must be safe**: Current operation completes before pausing
2. **E-Stop is immediate**: No graceful completion, halt instantly
3. **Jam detection**: 3 consecutive failures trigger jam state
4. **State preservation**: All counters and progress saved on pause/stop
5. **Inventory persistence**: Changes written to storage, survive reload
6. **Progress visibility**: User always sees current piece count
7. **Performance target**: 6 seconds per piece max, 80% success min
8. **Contract enforcement**: Automated validation of SORT-001, SORT-004
