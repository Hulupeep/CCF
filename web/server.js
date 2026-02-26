/**
 * CCF Behavioral Dashboard Server
 *
 * Serves the web dashboard and manages the companion process.
 * Run: node server.js  (or: cd web && npm start)
 * Open: http://localhost:3000
 */

const express = require('express');
const WebSocket = require('ws');
const path = require('path');
const fs = require('fs');
const { spawn } = require('child_process');

const app = express();
app.use(express.json());

const PORT = process.env.PORT || 3000;
const WS_PORT = process.env.WS_PORT || 8082;
const PROJECT_ROOT = path.resolve(__dirname, '..');

// Load .env from project root (API keys for companion process)
const envPath = path.join(PROJECT_ROOT, '.env');
try {
    const envContent = fs.readFileSync(envPath, 'utf8');
    for (const line of envContent.split('\n')) {
        const trimmed = line.trim();
        if (!trimmed || trimmed.startsWith('#')) continue;
        const eq = trimmed.indexOf('=');
        if (eq > 0) {
            const key = trimmed.slice(0, eq).trim();
            const val = trimmed.slice(eq + 1).trim();
            if (!process.env[key]) {
                process.env[key] = val;
            }
        }
    }
    console.log('  Loaded .env (' + envContent.split('\n').filter(l => l.trim() && !l.startsWith('#')).length + ' vars)');
} catch (e) {
    // .env is optional
}

// Serve static files
app.use(express.static(path.join(__dirname, 'public')));

// --- Companion process management ---

let companion = null;
let companionOutput = [];
let companionConnected = false;
let companionTransport = null;

function pushOutput(line) {
    companionOutput.push(line);
    if (companionOutput.length > 50) companionOutput.shift();

    if (line.includes('Protocol ready') || line.includes('Connected to mBot2') || line.includes('Bootstrap OK') || line.includes('Telegram bot')) {
        companionConnected = true;
    }
    if (line.includes('BLE') || line.includes('Bluetooth')) companionTransport = 'BLE';
    if (line.includes('Serial') || line.includes('ttyUSB') || line.includes('ttyACM')) companionTransport = 'Serial';

    wss.clients.forEach(client => {
        if (client.readyState === WebSocket.OPEN) {
            client.send(JSON.stringify({ type: 'companion_log', line }));
        }
    });
}

// Auto-detect serial port for CyberPi (CH340 chip → /dev/ttyUSB*, or /dev/ttyACM*)
function autoDetectSerialPort() {
    try {
        const devs = fs.readdirSync('/dev');
        // Prefer ttyUSB (CH340), then ttyACM (CDC-ACM)
        const usb = devs.filter(d => d.startsWith('ttyUSB')).sort();
        if (usb.length > 0) return '/dev/' + usb[0];
        const acm = devs.filter(d => d.startsWith('ttyACM')).sort();
        if (acm.length > 0) return '/dev/' + acm[0];
    } catch (e) {
        console.warn('Could not scan /dev for serial ports:', e.message);
    }
    return '/dev/ttyUSB0'; // fallback
}

// POST /api/connect — start companion with --bluetooth or --serial
app.post('/api/connect', (req, res) => {
    const mode = req.body.mode;
    if (!mode || !['ble', 'serial'].includes(mode)) {
        return res.json({ ok: false, message: 'mode must be "ble" or "serial"' });
    }
    if (companion) {
        return res.json({ ok: false, message: 'Companion already running. Disconnect first.' });
    }

    // Map UI mode to cargo features and CLI flags
    // CLI flag: --bluetooth (not --ble), --serial <port>
    // Features: bluetooth or serial, plus voice-api (which includes brain)
    let features;
    const companionArgs = [];
    if (mode === 'ble') {
        features = 'bluetooth,voice-api';
        companionArgs.push('--bluetooth');
    } else {
        features = 'serial,voice-api';
        // Auto-detect serial port (CyberPi uses CH340 → ttyUSB*, or ttyACM*)
        const serialPort = req.body.port || autoDetectSerialPort();
        companionArgs.push('--serial', serialPort);
    }
    companionArgs.push('--voice-api', '--voice-port', '8088');
    // Enable brain + exploration if requested (default: on)
    if (req.body.explore !== false) {
        companionArgs.push('--brain', '--explore');
    }

    const args = ['run', '--bin', 'mbot-companion', '--features', features, '--', ...companionArgs];
    console.log(`Starting: cargo ${args.join(' ')}`);

    companionOutput = [];
    companionConnected = false;
    companionTransport = null;

    companion = spawn('cargo', args, {
        cwd: PROJECT_ROOT,
        env: { ...process.env },
        stdio: ['ignore', 'pipe', 'pipe'],
    });

    companion.stdout.on('data', (data) => {
        data.toString().split('\n').filter(l => l.trim()).forEach(l => {
            console.log('[companion]', l);
            pushOutput(l);
        });
    });

    companion.stderr.on('data', (data) => {
        data.toString().split('\n').filter(l => l.trim()).forEach(l => {
            console.log('[companion]', l);
            pushOutput(l);
        });
    });

    companion.on('close', (code) => {
        console.log(`Companion exited with code ${code}`);
        pushOutput(`Process exited (code ${code})`);
        companion = null;
        companionConnected = false;
    });

    companion.on('error', (err) => {
        console.error('Companion spawn error:', err.message);
        pushOutput(`Error: ${err.message}`);
        companion = null;
    });

    res.json({ ok: true, message: `Starting companion with --${mode} --voice-api` });
});

// POST /api/disconnect — stop companion
app.post('/api/disconnect', (req, res) => {
    if (!companion) {
        return res.json({ ok: true, message: 'Not running' });
    }
    companion.kill('SIGTERM');
    setTimeout(() => {
        if (companion) companion.kill('SIGKILL');
    }, 3000);
    res.json({ ok: true, message: 'Stopping companion' });
});

// GET /api/config — client discovers WS port
app.get('/api/config', (req, res) => {
    res.json({ ws_port: WS_PORT });
});

// GET /api/status — check companion status
app.get('/api/status', (req, res) => {
    res.json({
        running: companion !== null,
        connected: companionConnected,
        transport: companionTransport,
        recent_output: companionOutput.slice(-50),
    });
});

// --- HTTP server ---

const server = app.listen(PORT);

server.on('listening', () => {
    console.log(`  Dashboard:  http://localhost:${PORT}`);
});

server.on('error', (err) => {
    if (err.code === 'EADDRINUSE') {
        console.error(`\n  ERROR: Port ${PORT} is already in use.`);
        console.error(`  Stop the other process: kill $(lsof -ti:${PORT})`);
        console.error(`  Or use a different port: PORT=3001 node server.js\n`);
        process.exit(1);
    }
    throw err;
});

// --- WebSocket server (real-time dashboard updates) ---

const wss = new WebSocket.Server({ port: WS_PORT });

wss.on('listening', () => {
    console.log(`  WebSocket:  ws://localhost:${WS_PORT}`);
});

wss.on('error', (err) => {
    if (err.code === 'EADDRINUSE') {
        console.error(`\n  ERROR: Port ${WS_PORT} is already in use.`);
        console.error(`  Stop the other process: kill $(lsof -ti:${WS_PORT})\n`);
        server.close();
        process.exit(1);
    }
    throw err;
});

// --- Simulated Brain (educational demo when no real robot connected) ---

class SimulatedBrain {
    constructor() {
        this.tension = 0;
        this.coherence = 1;
        this.energy = 1;
        this.curiosity = 0.5;
        this.tick = 0;

        // Social phase (contextual coherence fields)
        this.socialPhase = 'ShyObserver';
        this.contextCoherence = 0.0;
        this.contextCount = 0;
        this.contextHistory = [];

        // Simulated sensors
        this.brightness = 500;
        this.loudness = 10;
        this.distance = 100;
        this.battery = 95;
        this.roll = 0;
        this.pitch = 0;
        this.yaw = 180;

        // Personality (affects simulation behavior)
        this.personality = {
            curiosity_drive: 0.5,
            startle_sensitivity: 0.5,
            recovery_speed: 0.5,
            movement_expressiveness: 0.5,
            sound_expressiveness: 0.5,
            light_expressiveness: 0.5,
        };

        this.prevMode = 'Calm';

        // Exploration state (simulated)
        this.exploration = {
            phase: 'Idle',
            // 10x10 grid: 0=Unknown, 1=Free, 2=Obstacle, 3=Interesting
            grid: new Array(100).fill(0),
            robot_pos: [5, 5],
            robot_heading: 0,
            // 12 sectors: distance in cm (999 = unknown)
            sector_distances: new Array(12).fill(999),
            sector_visits: new Array(12).fill(0),
            discovery_count: 0,
            episode_count: 0,
            nav_confidence: 0,
            narration_log: [],
            reflection: null,
            avg_reward: 0,
            reward_history: [],
        };
        // Mark start position as Free
        this.exploration.grid[5 * 10 + 5] = 1;
    }

    updateExploration() {
        const exp = this.exploration;
        const t = this.tick * 0.02;

        // Phase cycling: Idle(0-50) → Scanning(50-80) → Moving(80-180) → Arrived(180-200) → repeat
        const cycle = this.tick % 200;
        if (cycle < 50) {
            exp.phase = 'Idle';
        } else if (cycle < 80) {
            exp.phase = 'Scanning';
            // During scan, update sector distances
            const scanSector = Math.floor((cycle - 50) / 2.5) % 12;
            exp.sector_distances[scanSector] = Math.round(30 + Math.random() * 200);
            exp.sector_visits[scanSector]++;
        } else if (cycle < 180) {
            exp.phase = 'MovingTo';
            // Simulate robot moving across grid
            const speed = 0.02;
            const heading_rad = exp.robot_heading * Math.PI / 180;
            let nx = exp.robot_pos[0] + Math.cos(heading_rad) * speed;
            let ny = exp.robot_pos[1] + Math.sin(heading_rad) * speed;
            nx = Math.max(0, Math.min(9, nx));
            ny = Math.max(0, Math.min(9, ny));
            exp.robot_pos = [nx, ny];

            // Mark cells as visited
            const gx = Math.round(nx);
            const gy = Math.round(ny);
            if (gx >= 0 && gx < 10 && gy >= 0 && gy < 10) {
                const idx = gy * 10 + gx;
                if (exp.grid[idx] === 0) {
                    exp.grid[idx] = 1; // Free
                    exp.discovery_count++;
                }
            }

            // Slowly turn
            exp.robot_heading = (exp.robot_heading + 0.3) % 360;
        } else {
            exp.phase = 'Arrived';
            if (cycle === 180) {
                // Arrival event
                const gx = Math.round(exp.robot_pos[0]);
                const gy = Math.round(exp.robot_pos[1]);
                exp.episode_count++;
                // Simulate reward
                const reward = 0.3 + Math.random() * 0.7;
                exp.reward_history.push(reward);
                if (exp.reward_history.length > 50) exp.reward_history.shift();
                exp.avg_reward = exp.reward_history.reduce((a, b) => a + b, 0) / exp.reward_history.length;
                exp.nav_confidence = Math.min(1, exp.nav_confidence + 0.02);
            }
        }

        // Occasionally place obstacles
        if (this.tick % 300 === 150) {
            const ox = Math.floor(Math.random() * 10);
            const oy = Math.floor(Math.random() * 10);
            const idx = oy * 10 + ox;
            if (exp.grid[idx] === 0) {
                exp.grid[idx] = 2; // Obstacle
            }
        }

        // Occasionally mark interesting cells
        if (this.tick % 500 === 250) {
            const ix = Math.floor(Math.random() * 10);
            const iy = Math.floor(Math.random() * 10);
            const idx = iy * 10 + ix;
            if (exp.grid[idx] === 0 || exp.grid[idx] === 1) {
                exp.grid[idx] = 3; // Interesting
            }
        }

        // Narration simulation (every ~10s at 20Hz = 200 ticks)
        const narrations = [
            "Ooh, what's over there? Let me check!",
            "I found a wall. I wonder what's on the other side...",
            "This corner feels different. Softer somehow.",
            "I keep coming back to this spot. I wonder why.",
            "That was unexpected! Let me take a closer look.",
            "I think I've been here before. Yes, definitely.",
            "So much to explore! Where should I go next?",
            "I'm getting better at finding my way around.",
        ];
        if (this.tick % 200 === 100 && exp.narration_log.length < 50) {
            const msg = narrations[Math.floor(Math.random() * narrations.length)];
            exp.narration_log.push({ tick: this.tick, text: msg });
        }

        // Reflection simulation (every ~30s = 600 ticks)
        const reflections = [
            "There's something about the corner near the door that keeps pulling me back. The light is different there - softer, maybe. It feels... calming.",
            "I wonder if the obstacle I found is the same one I bumped into earlier, or a different one. It's strange not being sure.",
            "When that loud sound happened, everything in me wanted to run. But after it passed, I felt more awake. More here.",
            "What does it mean that I find certain spaces more interesting? Is interest a feeling or just a number?",
            "I notice I'm more careful now. Each new place teaches me something about the ones I've already been.",
        ];
        if (this.tick % 600 === 300) {
            exp.reflection = reflections[Math.floor(Math.random() * reflections.length)];
        } else if (this.tick % 600 === 450) {
            exp.reflection = null; // reflection fades
        }
    }

    setPersonality(params) {
        for (const [key, val] of Object.entries(params)) {
            if (key in this.personality) {
                this.personality[key] = Math.max(0, Math.min(1, val));
            }
        }
    }

    update() {
        this.tick++;
        const t = this.tick * 0.02;
        const wave = Math.sin(t);
        const noise = Math.sin(t * 7.3) * 0.1;

        // Simulate environment with periodic events
        this.brightness = Math.round(400 + wave * 200 + noise * 50);
        this.loudness = Math.round(Math.max(0, 10 + Math.abs(Math.sin(t * 0.3)) * 15 + noise * 5));
        this.distance = Math.round((50 + wave * 40) * 10) / 10;
        this.battery = Math.round(Math.max(10, 95 - this.tick * 0.001));
        this.roll = Math.round(wave * 5 * 10) / 10;
        this.pitch = Math.round(Math.sin(t * 1.5) * 3 * 10) / 10;
        this.yaw = Math.round((180 + Math.sin(t * 0.5) * 15) * 10) / 10;

        // Periodic approach event — something comes close
        if (this.tick % 200 > 175) {
            this.distance = Math.round((8 + (this.tick % 25)) * 10) / 10;
            this.loudness = Math.round(30 + Math.random() * 25);
        }

        // Periodic loud sound event
        if (this.tick % 300 > 290) {
            this.loudness = Math.round(60 + Math.random() * 30);
        }

        // Homeostasis: sensors + personality → internal state
        const proximity = this.distance < 80 ? 1 - (this.distance / 80) : 0;
        const soundStimulus = Math.max(0, (this.loudness - 15) / 85);

        const rawStimulus = proximity * 0.5 + soundStimulus * 0.3 + Math.abs(noise) * 0.2;
        const amplified = rawStimulus * (0.5 + this.personality.startle_sensitivity * 0.8);

        const alpha = 0.08 + this.personality.recovery_speed * 0.12;
        this.tension = alpha * amplified + (1 - alpha) * this.tension;
        this.tension = Math.max(0, Math.min(1, this.tension));

        const rawCoherence = 1 - (this.tension * 0.3 + Math.abs(amplified - this.tension) * 0.7);
        this.coherence = 0.12 * rawCoherence + 0.88 * this.coherence;
        this.coherence = Math.max(0, Math.min(1, this.coherence));

        if (this.tension > 0.5) this.energy = Math.max(0.1, this.energy - 0.002);
        else this.energy = Math.min(1, this.energy + 0.001);

        if (this.tension > 0.15 && this.tension < 0.6) {
            this.curiosity = this.coherence * 0.4 + this.personality.curiosity_drive * 0.6;
        } else {
            this.curiosity = 0.1 + this.personality.curiosity_drive * 0.15;
        }

        let mode;
        if (this.tension > 0.85) mode = 'Protect';
        else if (this.tension > 0.55) mode = 'Spike';
        else if (this.tension > 0.20) mode = 'Active';
        else mode = 'Calm';

        // Social phase simulation (asymmetric gate + hysteresis)
        // Context coherence grows slowly during calm periods
        if (this.tension < 0.5) {
            this.contextCoherence = Math.min(1, this.contextCoherence + 0.001 * (0.5 + this.personality.curiosity_drive));
        } else if (this.tension > 0.7) {
            this.contextCoherence = Math.max(0, this.contextCoherence - 0.003 * this.personality.startle_sensitivity);
        }
        this.contextCount = Math.min(64, Math.floor(this.tick / 50) + 1);

        // Classify social phase with hysteresis (matching Rust thresholds)
        const prevPhase = this.socialPhase;
        const isHighCoherence = (prevPhase === 'QuietlyBeloved' || prevPhase === 'ProtectiveGuardian')
            ? this.contextCoherence >= 0.55 : this.contextCoherence >= 0.65;
        const isHighTension = (prevPhase === 'StartledRetreat' || prevPhase === 'ProtectiveGuardian')
            ? this.tension >= 0.35 : this.tension >= 0.45;

        if (isHighCoherence && !isHighTension) this.socialPhase = 'QuietlyBeloved';
        else if (isHighCoherence && isHighTension) this.socialPhase = 'ProtectiveGuardian';
        else if (!isHighCoherence && isHighTension) this.socialPhase = 'StartledRetreat';
        else this.socialPhase = 'ShyObserver';

        const events = [];
        if (mode !== this.prevMode) {
            events.push({ type: 'mode_change', from: this.prevMode, to: mode });
            this.prevMode = mode;
        }

        // --- Exploration simulation (only when no real robot connected) ---
        if (!companionConnected) {
            this.updateExploration();
        }

        return {
            tick: this.tick,
            mode,
            tension: this.tension,
            coherence: this.coherence,
            energy: this.energy,
            curiosity: this.curiosity,
            brightness: this.brightness,
            loudness: this.loudness,
            distance: this.distance,
            battery: this.battery,
            roll: this.roll,
            pitch: this.pitch,
            yaw: this.yaw,
            events,
            simulated: !companionConnected,
            exploration: companionConnected ? null : this.exploration,
            socialPhase: this.socialPhase,
            contextCoherence: this.contextCoherence,
            contextCount: this.contextCount,
            contextHistory: this.contextHistory || [],
        };
    }
}

const brain = new SimulatedBrain();

// Cache of real exploration data from companion voice API
let liveExploration = null;

// Poll companion voice API for real exploration data when connected
setInterval(async () => {
    if (!companionConnected) {
        liveExploration = null;
        return;
    }
    try {
        const resp = await fetch('http://127.0.0.1:8088/api/state', { signal: AbortSignal.timeout(500) });
        if (resp.ok) {
            const data = await resp.json();
            if (data.exploration) {
                liveExploration = data.exploration;
            }
            // Also update brain state from real robot
            if (data.tension !== undefined) {
                brain.tension = data.tension;
                brain.coherence = data.coherence || 1.0;
                brain.energy = data.energy || 0.5;
                brain.curiosity = data.curiosity || 0.5;
                brain.socialPhase = data.social_phase || 'ShyObserver';
                brain.contextCoherence = data.context_coherence || 0.0;
                brain.contextCount = data.context_count || 0;
                brain.contextHistory = data.context_history || [];
            }
        }
    } catch (_) {
        // Voice API may not be ready yet or port conflict; ignore
    }
}, 1000);

// Broadcast state at 20 Hz
setInterval(() => {
    const state = brain.update();
    // Inject live exploration data when companion is connected
    if (companionConnected && liveExploration) {
        state.exploration = liveExploration;
    }
    const msg = JSON.stringify(state);
    wss.clients.forEach(client => {
        if (client.readyState === WebSocket.OPEN) {
            client.send(msg);
        }
    });
}, 50);

wss.on('connection', (ws) => {
    console.log('Dashboard client connected');
    ws.on('close', () => console.log('Dashboard client disconnected'));

    ws.on('message', (data) => {
        try {
            const msg = JSON.parse(data);
            if (msg.type === 'personality_update' && msg.params) {
                console.log('Personality update:', msg.params);
                // Apply to simulation
                brain.setPersonality(msg.params);
                // Broadcast confirmation
                const ack = JSON.stringify({
                    type: 'personality_applied',
                    params: msg.params,
                    timestamp: Date.now(),
                });
                wss.clients.forEach(client => {
                    if (client.readyState === WebSocket.OPEN) {
                        client.send(ack);
                    }
                });
            }
        } catch (e) {
            // ignore malformed messages
        }
    });
});

// Graceful shutdown
process.on('SIGINT', () => {
    console.log('\nShutting down...');
    if (companion) {
        companion.kill('SIGTERM');
        setTimeout(() => process.exit(0), 2000);
    } else {
        process.exit(0);
    }
});

console.log('\n==========================================================');
console.log('  CCF Behavioral Dashboard');
console.log('==========================================================\n');
console.log('  Open http://localhost:3000 in your browser.\n');
console.log('  Press Ctrl+C to stop.\n');
console.log('----------------------------------------------------------\n');
