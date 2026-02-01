# mBot Mobile App - Testing Checklist

**Issue:** #88 (STORY-MOBILE-001)
**Contract:** J-MOBILE-CONTROL

Use this checklist to verify all features and invariants.

## Unit Tests

```bash
cd mobile
npm test
```

### Service Tests

- [ ] `WebSocketClient.test.ts`
  - [ ] Connect within 3 seconds (I-MOBILE-001)
  - [ ] Auto-reconnect on disconnect
  - [ ] Max reconnect delay 5000ms
  - [ ] Handle messages correctly
  - [ ] Unsubscribe cleanup works

- [ ] `CacheService.test.ts`
  - [ ] Enforce minimum 24 hour cache (I-MOBILE-003)
  - [ ] Save and load app state
  - [ ] Save and load drawings
  - [ ] Cache expiry validation
  - [ ] Clear cache works

## E2E Tests (Detox)

```bash
detox build --configuration ios.sim.debug
detox test --configuration ios.sim.debug
```

### Journey Test: J-MOBILE-CONTROL

- [ ] **Scenario: Connect from Phone**
  - [ ] Discovery list appears
  - [ ] Scan finds robots within 10 seconds
  - [ ] Connect completes within 3 seconds
  - [ ] Connection status shows "Connected"
  - [ ] Navigates to Personality Mixer

- [ ] **Scenario: Adjust Personality**
  - [ ] All 9 sliders visible
  - [ ] Slider values match robot state
  - [ ] Adjust tension slider
  - [ ] Robot responds within 200ms
  - [ ] Neural visualizer updates

- [ ] **Scenario: View Gallery**
  - [ ] Gallery thumbnails load within 2 seconds
  - [ ] Tap drawing opens full view
  - [ ] Playback button visible
  - [ ] Can play animation

- [ ] **Scenario: Offline Mode**
  - [ ] Disconnect shows "Offline"
  - [ ] Cached drawings still accessible
  - [ ] Reconnect within 5 seconds
  - [ ] Shows "Connected" after reconnect

## Manual Testing - iOS

### Device Preparation
- [ ] iPhone/iPad running iOS 14+
- [ ] Connected to same WiFi as robot
- [ ] Xcode installed (for development)

### Discovery Screen
- [ ] App launches to Discovery screen
- [ ] "Scan Again" button visible and tappable
- [ ] Loading indicator shows during scan
- [ ] Robot list populates with discovered robots
- [ ] Each robot shows: name, IP, port, status
- [ ] Status badge shows "online" in green
- [ ] Tap robot connects within 3 seconds
- [ ] Error messages display clearly

**data-testid:**
- [ ] `discovery-list` renders
- [ ] `scan-btn` tappable
- [ ] `connect-btn-robot-001` tappable

### Personality Mixer Screen
- [ ] Navigates after successful connection
- [ ] Connection indicator shows "Connected"
- [ ] All 9 sliders visible and scrollable:
  - [ ] Energy
  - [ ] Tension
  - [ ] Curiosity
  - [ ] Playfulness
  - [ ] Confidence
  - [ ] Focus
  - [ ] Empathy
  - [ ] Creativity
  - [ ] Persistence
- [ ] Each slider shows label, description, value
- [ ] Slider interaction is smooth (60 FPS)
- [ ] Value updates immediately
- [ ] Robot responds within 200ms
- [ ] "Reset to Default" button works
- [ ] "Save Preset" button visible

**data-testid:**
- [ ] `personality-mixer` renders
- [ ] `slider-energy` through `slider-persistence` work
- [ ] `connection-status` shows status
- [ ] `reset-btn` tappable
- [ ] `save-preset-btn` tappable

### Neural Visualizer Screen
- [ ] Navigate via bottom tab
- [ ] Connection indicator shows status
- [ ] Neural graph renders (2D/3D)
- [ ] Nodes visible with colors:
  - [ ] Sensory (green)
  - [ ] Motor (blue)
  - [ ] Cognitive (orange)
- [ ] Connections between nodes visible
- [ ] Animation plays by default
- [ ] Play/Pause button toggles animation
- [ ] Zoom controls work (+ and -)
- [ ] Stats display: Active Nodes, Connections, Activity
- [ ] Updates in real-time when connected

**data-testid:**
- [ ] `neural-visualizer` renders
- [ ] `viz-play-pause` toggles
- [ ] `connection-status` shows status

### Gallery Screen
- [ ] Navigate via bottom tab
- [ ] Connection indicator shows status
- [ ] Thumbnails load within 2 seconds
- [ ] Grid layout (2 columns)
- [ ] Each thumbnail shows: image, name, date
- [ ] Tap thumbnail opens full view
- [ ] Full view shows drawing
- [ ] Close button returns to grid
- [ ] Playback button visible in full view
- [ ] Refresh button works
- [ ] Loading indicator during refresh
- [ ] Empty state shows when no drawings

**data-testid:**
- [ ] `gallery-grid` renders
- [ ] `drawing-thumb-{id}` tappable
- [ ] `drawing-full-{id}` displays
- [ ] `playback-btn-{id}` tappable

### Offline Mode (iOS)
- [ ] Connect to robot successfully
- [ ] Turn off WiFi on device
- [ ] Connection indicator changes to "Offline"
- [ ] Last known state remains visible
- [ ] Can navigate between tabs
- [ ] Gallery shows cached drawings
- [ ] Cannot adjust personality sliders (disabled)
- [ ] Turn WiFi back on
- [ ] Auto-reconnects within 5 seconds
- [ ] Connection indicator shows "Connected"
- [ ] Personality sliders re-enabled

### Responsive Design (iOS)
Test on multiple screen sizes:

- [ ] **iPhone SE (375x667)** - Small screen
  - [ ] All UI elements visible
  - [ ] No horizontal scrolling
  - [ ] Sliders fit on screen
  - [ ] Bottom tabs accessible

- [ ] **iPhone 14 (390x844)** - Standard
  - [ ] Proper spacing
  - [ ] Readable text
  - [ ] Touch targets adequate

- [ ] **iPhone 14 Pro Max (430x932)** - Large
  - [ ] Layout scales nicely
  - [ ] No excessive whitespace

- [ ] **iPad Mini (768x1024)** - Tablet
  - [ ] Two-column layout where applicable
  - [ ] Larger touch targets
  - [ ] Better use of space

## Manual Testing - Android

### Device Preparation
- [ ] Android device/emulator running API 30+
- [ ] Connected to same WiFi as robot
- [ ] Android Studio installed (for development)

### All Screens (Android)
Repeat all iOS tests on Android:

- [ ] Discovery Screen tests
- [ ] Personality Mixer Screen tests
- [ ] Neural Visualizer Screen tests
- [ ] Gallery Screen tests
- [ ] Offline Mode tests
- [ ] Responsive Design tests

### Android-Specific Tests
- [ ] Back button navigation works
- [ ] Material Design components render correctly
- [ ] Roboto font displays properly
- [ ] Permission dialogs appear correctly
- [ ] App survives process kill and restore

## Performance Testing

### Connection Performance
- [ ] Robot discovery completes within 10 seconds
- [ ] WebSocket connect completes within 3 seconds
- [ ] Auto-reconnect completes within 5 seconds
- [ ] Personality update responds within 200ms

### UI Performance
- [ ] Smooth scrolling (60 FPS)
- [ ] Smooth slider interaction
- [ ] No frame drops during animation
- [ ] Neural visualizer runs at 60 FPS

### Memory Performance
- [ ] No memory leaks after 10 screen transitions
- [ ] App remains responsive after 1 hour
- [ ] Cache size remains reasonable (< 100 MB)

## Invariant Validation

### I-MOBILE-001: Auto-reconnect within 5 seconds
- [ ] Test 1: Disconnect WiFi → Reconnect within 5s
- [ ] Test 2: Robot restart → Reconnect within 5s
- [ ] Test 3: Network switch → Reconnect within 5s
- [ ] Test 4: Max delay capped at 5000ms
- [ ] Test 5: Exponential backoff works correctly

### I-MOBILE-002: Responsive UI (320px-768px)
- [ ] Test 1: iPhone SE (375px) - All elements visible
- [ ] Test 2: Small Android (360px) - No clipping
- [ ] Test 3: Large phone (430px) - Scales properly
- [ ] Test 4: iPad Mini (768px) - Uses space well
- [ ] Test 5: No horizontal scrolling at any width

### I-MOBILE-003: Cache for 24 hours
- [ ] Test 1: Cache persists after app close
- [ ] Test 2: Cache valid after 23 hours
- [ ] Test 3: Cache expired after 25 hours
- [ ] Test 4: Cannot set cache expiry < 24 hours
- [ ] Test 5: Cache cleared on expiry

## Integration Testing

### With Robot Backend
- [ ] Discover real robot via mDNS
- [ ] Connect to robot WebSocket server
- [ ] Receive real neural state updates
- [ ] Adjust personality → Robot responds
- [ ] Fetch real drawing gallery
- [ ] View real drawing with playback
- [ ] Handle robot disconnect gracefully
- [ ] Handle robot reconnect automatically

## Edge Cases

### Network Issues
- [ ] Handle no WiFi on launch
- [ ] Handle intermittent connectivity
- [ ] Handle slow network (3G simulation)
- [ ] Handle WiFi to cellular switch
- [ ] Handle VPN connection

### Robot Issues
- [ ] Handle robot not responding
- [ ] Handle invalid robot messages
- [ ] Handle malformed JSON
- [ ] Handle robot sending errors
- [ ] Handle robot firmware mismatch

### App State
- [ ] Handle app backgrounding
- [ ] Handle app foregrounding
- [ ] Handle low memory warning
- [ ] Handle device rotation
- [ ] Handle system interruptions (call, notification)

## Accessibility

### VoiceOver (iOS) / TalkBack (Android)
- [ ] All buttons have labels
- [ ] All images have alt text
- [ ] Sliders have value announcements
- [ ] Navigation is logical
- [ ] Focus order makes sense

### Other
- [ ] Text scales with system font size
- [ ] Color contrast meets WCAG AA
- [ ] Touch targets at least 44x44 pt

## Sign-off

### Unit Tests
- [ ] All Jest tests passing
- [ ] Code coverage > 70%

### E2E Tests
- [ ] All Detox tests passing
- [ ] J-MOBILE-CONTROL journey complete

### iOS Testing
- [ ] Tested on iPhone SE
- [ ] Tested on iPhone 14
- [ ] Tested on iPad

### Android Testing
- [ ] Tested on small phone (360px)
- [ ] Tested on large phone (430px)
- [ ] Tested on tablet (768px)

### Invariants
- [ ] I-MOBILE-001 validated
- [ ] I-MOBILE-002 validated
- [ ] I-MOBILE-003 validated

### Integration
- [ ] Connected to real robot
- [ ] All features work end-to-end

### Documentation
- [ ] README.md reviewed
- [ ] QUICKSTART.md tested
- [ ] SETUP.md accurate
- [ ] ARCHITECTURE.md complete

---

**Tested by:** _________________
**Date:** _________________
**Result:** ☐ PASS  ☐ FAIL

**Notes:**
