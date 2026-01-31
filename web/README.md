# mBot2 RuVector Web Dashboard

Real-time visualization and control dashboard for mBot2 with RuVector AI.

## Features

### 🧠 Neural Visualizer (`index.html`)
- Real-time WebSocket streaming of robot brain state
- Reflex mode display (Calm → Active → Spike → Protect)
- Homeostasis meters (Tension, Coherence, Energy, Curiosity)
- Sensor readings (Distance, Gyro, Sound, Light)
- Robot visualization with distance ring
- **Implements:** STORY-LEARN-001 (Real-Time Visualizer)

### 🎨 Personality Mixer (`personality-mixer.html`)
- 9 adjustable personality parameters with sliders
- Parameter categories: Baselines, Reactivity, Expression
- 6 preset personalities (Mellow, Curious, Zen, Excitable, Timid, Adventurous)
- Real-time parameter transmission (<500ms latency)
- Randomize with safety constraints
- Educational mode (limits startle sensitivity to 80%)
- **Implements:** STORY-LEARN-002 (Personality Mixer UI)

### 🚀 Projects Navigator (`projects.html`)
- Overview of 6 major projects (ArtBot, Personality, GameBot, SORT, LearningLab, HelperBot)
- Implementation status tracking
- Story completion indicators
- Quick project selection for deployment

### 📥 Download Manager (`download.html`)
- Build instructions for Rust → CyberPi pipeline
- CyberPi connection detection
- Flashing guide with troubleshooting
- Prerequisites and toolchain setup
- Serial port configuration help

## Quick Start

```bash
# Install dependencies
cd web
npm install

# Start the server
npm start

# Open in browser
open http://localhost:3000
```

**New to the web dashboard?** See the [Master Setup Guide](../docs/MASTER_GUIDE.md#web-dashboard) for complete instructions on setup, usage, and troubleshooting.

## Architecture

```
┌─────────────┐     WebSocket      ┌──────────────┐     Serial/BT     ┌──────────┐
│   Browser   │ ←─────8081────────→ │  server.js   │ ←────────────────→ │ CyberPi  │
│  Dashboard  │                     │   (Node.js)  │                     │  mBot2   │
└─────────────┘                     └──────────────┘                     └──────────┘
     ↓                                      ↓
 Personality                          Simulated Brain
 Parameters                          (50ms update rate)
```

### Real-Time Data Flow

1. **Robot → Server:** CyberPi sends telemetry via serial/Bluetooth
2. **Server → Browser:** WebSocket broadcasts state at 20Hz (50ms intervals)
3. **Browser → Server:** User sends personality updates/commands
4. **Server → Robot:** Parameters forwarded to CyberPi

## API

### WebSocket Messages (Port 8081)

**From Server → Client:**
```javascript
{
  tick: 12345,
  mode: 'Active',           // Calm | Active | Spike | Protect
  tension: 0.45,            // 0.0 - 1.0
  coherence: 0.78,
  energy: 0.92,
  curiosity: 0.65,
  distance: 23.5,           // cm
  gyro: -12.3,              // °/s
  sound: 0.42,              // 0.0 - 1.0
  light: 0.68,
  encoderLeft: 5230,
  encoderRight: 5245
}
```

**From Client → Server:**
```javascript
{
  type: 'personality_update',
  params: {
    tension_baseline: 0.3,
    coherence_baseline: 0.7,
    energy_baseline: 0.5,
    startle_sensitivity: 0.4,
    recovery_speed: 0.6,
    curiosity_drive: 0.8,
    movement_expressiveness: 0.7,
    sound_expressiveness: 0.5,
    light_expressiveness: 0.6
  }
}
```

## Contracts Satisfied

### LEARN-001: Real-Time Visualizer
- ✅ WebSocket connection established within 2 seconds
- ✅ Reflex mode icon updates within 100ms
- ✅ Accurate gauge displays for tension/coherence/energy
- ✅ Sensor values displayed (ultrasonic, light, sound)
- ✅ Motor outputs visualized
- ✅ Graceful connection loss handling
- ✅ Auto-reconnect on disconnect

### LEARN-002: Personality Mixer UI
- ✅ All personality parameters as sliders
- ✅ Parameters grouped by category
- ✅ Numeric value display for each slider
- ✅ Parameter changes transmitted within 500ms
- ✅ 6 preset personalities available
- ✅ Randomize with safety constraints
- ✅ Educational mode safety bounds (startle ≤ 0.8)
- ✅ Reset to defaults button
- ✅ Clear parameter descriptions

## Development

### Adding New Pages

1. Create HTML file in `public/`
2. Add navigation link to header
3. Update server.js if WebSocket handling needed
4. Test WebSocket connection
5. Document in this README

### Testing Personality Updates

```bash
# Start server
npm start

# Open mixer in browser
open http://localhost:3000/personality-mixer.html

# Watch server console for:
# 🎨 Personality update: { tension_baseline: 0.3, ... }
```

### Connecting Real Hardware

Replace simulated brain in `server.js` with SerialPort connection:

```javascript
const { SerialPort } = require('serialport');
const port = new SerialPort({
  path: '/dev/ttyUSB0',
  baudRate: 115200
});

port.on('data', (data) => {
  const state = JSON.parse(data.toString());
  broadcast(state);
});
```

## Browser Compatibility

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- WebSocket support required
- No IE support

## File Structure

```
web/
├── package.json              # Dependencies
├── server.js                 # WebSocket server + HTTP server
├── public/
│   ├── index.html           # Neural visualizer
│   ├── personality-mixer.html  # Parameter adjustment UI
│   ├── projects.html        # Project navigator
│   └── download.html        # Build & flash guide
└── README.md                # This file
```

## Troubleshooting

### WebSocket Connection Fails
- Check port 8081 is not in use: `lsof -i :8081`
- Verify firewall allows WebSocket connections
- Try different port in server.js and HTML files

### Personality Updates Not Working
- Open browser console (F12) to see errors
- Check server console for received messages
- Verify WebSocket connection is "Connected"

### Page Not Loading
- Ensure server is running: `npm start`
- Check server console for startup messages
- Verify port 3000 is available

## Related Documentation

- [STORY-LEARN-001 Issue #15](https://github.com/Hulupeep/mbot_ruvector/issues/15) - Real-Time Visualizer requirements
- [STORY-LEARN-002 Issue #19](https://github.com/Hulupeep/mbot_ruvector/issues/19) - Personality Mixer requirements
- [mBot2 Official Docs](https://www.makeblock.com/pages/mbot2-support) - Hardware specs
- [RuVector Architecture](../docs/contracts/feature_architecture.yml) - System contracts

## Future Enhancements

- [ ] Serial port auto-detection
- [ ] Personality preset saving/loading
- [ ] Multi-robot support
- [ ] Data recording and replay
- [ ] Mobile-responsive design
- [ ] Dark/light theme toggle
- [ ] Offline mode with local storage
- [ ] Journey test integration
- [ ] Lesson plan framework UI

## License

Same as parent project (see root LICENSE file)
