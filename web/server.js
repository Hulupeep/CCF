/**
 * mBot2 RuVector Dashboard Server
 *
 * Real-time visualization of the robot's AI state.
 * Run: npm start
 * Open: http://localhost:3000
 */

const express = require('express');
const WebSocket = require('ws');
const path = require('path');

const app = express();
const PORT = process.env.PORT || 3000;
const WS_PORT = process.env.WS_PORT || 8081;

// Serve static files
app.use(express.static(path.join(__dirname, 'public')));

// Start HTTP server
const server = app.listen(PORT, () => {
    console.log(`🤖 mBot2 Dashboard: http://localhost:${PORT}`);
});

// Start WebSocket server
const wss = new WebSocket.Server({ port: WS_PORT });
console.log(`📡 WebSocket server on port ${WS_PORT}`);

// Simulated robot brain (replace with real connection)
class SimulatedBrain {
    constructor() {
        this.tension = 0;
        this.coherence = 1;
        this.energy = 1;
        this.curiosity = 0.5;
        this.tick = 0;
        this.distance = 100;
    }

    update() {
        this.tick++;

        // Simulate varying distance
        const wave = Math.sin(this.tick * 0.02);
        this.distance = 50 + wave * 40;

        // Occasional "event" (something approaches)
        if (this.tick % 200 > 180) {
            this.distance = 10 + (this.tick % 20);
        }

        // Calculate tension from distance
        const proximity = this.distance < 100 ? 1 - (this.distance / 100) : 0;
        const rawTension = proximity * 0.7 + Math.abs(wave) * 0.3;

        // EMA smoothing
        const alpha = 0.15;
        this.tension = alpha * rawTension + (1 - alpha) * this.tension;
        this.tension = Math.max(0, Math.min(1, this.tension));

        // Coherence drops with high tension
        const rawCoherence = 1 - (this.tension * 0.4 + Math.abs(rawTension - this.tension) * 0.6);
        this.coherence = alpha * rawCoherence + (1 - alpha) * this.coherence;
        this.coherence = Math.max(0, Math.min(1, this.coherence));

        // Energy management
        if (this.tension > 0.5) {
            this.energy = Math.max(0.1, this.energy - 0.001);
        } else {
            this.energy = Math.min(1, this.energy + 0.0005);
        }

        // Curiosity
        if (this.tension > 0.2 && this.tension < 0.6) {
            this.curiosity = Math.min(1, this.coherence * 0.7 + Math.abs(wave) * 0.3);
        } else {
            this.curiosity = 0.2;
        }

        // Determine reflex mode
        let mode;
        if (this.tension > 0.85) mode = 'Protect';
        else if (this.tension > 0.55) mode = 'Spike';
        else if (this.tension > 0.20) mode = 'Active';
        else mode = 'Calm';

        return {
            tick: this.tick,
            mode,
            tension: this.tension,
            coherence: this.coherence,
            energy: this.energy,
            curiosity: this.curiosity,
            distance: this.distance,
            gyro: wave * 15,
            sound: 0.1 + Math.abs(wave) * 0.2,
            light: 0.5 + wave * 0.2,
            encoderLeft: this.tick * 5,
            encoderRight: this.tick * 5,
        };
    }
}

const brain = new SimulatedBrain();

// Broadcast state to all clients
setInterval(() => {
    const state = brain.update();

    wss.clients.forEach(client => {
        if (client.readyState === WebSocket.OPEN) {
            client.send(JSON.stringify(state));
        }
    });
}, 50); // 20Hz update rate

// Handle new connections
wss.on('connection', (ws) => {
    console.log('📱 Dashboard connected');

    ws.on('close', () => {
        console.log('📱 Dashboard disconnected');
    });

    ws.on('message', (data) => {
        try {
            const msg = JSON.parse(data);
            console.log('Received:', msg);

            // Handle personality parameter updates from mixer
            if (msg.type === 'personality_update') {
                console.log('🎨 Personality update:', msg.params);
                // In real implementation, send to CyberPi via serial
                // For now, broadcast confirmation to all clients
                wss.clients.forEach(client => {
                    if (client.readyState === WebSocket.OPEN) {
                        client.send(JSON.stringify({
                            type: 'personality_applied',
                            params: msg.params,
                            timestamp: Date.now()
                        }));
                    }
                });
            }
        } catch (e) {
            console.error('Invalid message:', data);
        }
    });
});

console.log('\n🧠 RuVector mBot2 Dashboard Ready!');
console.log('   Simulating robot brain state...');
console.log('   Connect real mBot2 via --serial or --bluetooth\n');
