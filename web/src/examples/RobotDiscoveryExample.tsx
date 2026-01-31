/**
 * Example App for Multi-Robot Discovery
 * Issue #77 - STORY-ARCH-008
 *
 * Demonstrates:
 * - Robot discovery with mock service
 * - Connection state management
 * - Event handling
 * - Integration with existing components
 */

import React, { useState } from 'react';
import { RobotDiscovery } from '../components/RobotDiscovery';
import { PersonalityMixer } from '../components/PersonalityMixer';
import { MockDiscoveryService } from '../services/robotDiscovery';
import '../components/RobotDiscovery.css';
import '../components/PersonalityMixer.css';

export function RobotDiscoveryExample() {
  const [discoveryService] = useState(() => new MockDiscoveryService());
  const [connectedRobot, setConnectedRobot] = useState<string | null>(null);
  const [eventLog, setEventLog] = useState<string[]>([]);

  const addLogEntry = (message: string) => {
    const timestamp = new Date().toLocaleTimeString();
    setEventLog((prev) => [`[${timestamp}] ${message}`, ...prev.slice(0, 19)]);
  };

  const handleRobotConnect = (robotId: string) => {
    setConnectedRobot(robotId);
    addLogEntry(`🟢 Connected to ${robotId}`);
  };

  const handleRobotDisconnect = (robotId: string) => {
    if (connectedRobot === robotId) {
      setConnectedRobot(null);
    }
    addLogEntry(`⚪ Disconnected from ${robotId}`);
  };

  return (
    <div style={styles.container}>
      {/* Header */}
      <header style={styles.header}>
        <h1 style={styles.title}>🤖 Multi-Robot Discovery Demo</h1>
        <p style={styles.subtitle}>
          Issue #77 - Discover and connect to multiple mBot robots
        </p>
      </header>

      {/* Main Content */}
      <div style={styles.main}>
        {/* Discovery Panel */}
        <div style={styles.section}>
          <RobotDiscovery
            discoveryService={discoveryService}
            onRobotConnect={handleRobotConnect}
            onRobotDisconnect={handleRobotDisconnect}
          />
        </div>

        {/* Connected Robot Info */}
        {connectedRobot && (
          <div style={styles.section}>
            <div style={styles.connectedInfo}>
              <h3 style={styles.connectedTitle}>
                ✅ Connected to: {connectedRobot}
              </h3>
              <p style={styles.connectedSubtitle}>
                Now you can interact with this robot's personality
              </p>

              {/* Example: Personality Mixer for connected robot */}
              <PersonalityMixer
                wsUrl={`ws://localhost:8081`} // Would be robot's actual address
                onConfigChange={(config) => {
                  addLogEntry(`⚙️ Personality updated for ${connectedRobot}`);
                }}
              />
            </div>
          </div>
        )}

        {/* Event Log */}
        <div style={styles.section}>
          <div style={styles.eventLog}>
            <h3 style={styles.eventLogTitle}>📋 Event Log</h3>
            <div style={styles.eventLogContent}>
              {eventLog.length === 0 ? (
                <p style={styles.noEvents}>No events yet. Try connecting to a robot!</p>
              ) : (
                eventLog.map((event, index) => (
                  <div key={index} style={styles.eventEntry}>
                    {event}
                  </div>
                ))
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Footer */}
      <footer style={styles.footer}>
        <div style={styles.footerContent}>
          <div style={styles.footerSection}>
            <h4>🔍 Discovery Status</h4>
            <p>Using Mock Discovery Service</p>
            <p>3 robots available for testing</p>
          </div>
          <div style={styles.footerSection}>
            <h4>📡 Connection</h4>
            <p>
              {connectedRobot
                ? `Connected to ${connectedRobot}`
                : 'Not connected'}
            </p>
          </div>
          <div style={styles.footerSection}>
            <h4>ℹ️ Info</h4>
            <p>Issue #77 - STORY-ARCH-008</p>
            <p>I-DISC-001: mDNS Standard</p>
          </div>
        </div>
      </footer>
    </div>
  );
}

// Inline styles for demo
const styles = {
  container: {
    minHeight: '100vh',
    background: 'linear-gradient(135deg, #0f0f1e 0%, #1a1a2e 100%)',
    color: '#ffffff',
    padding: '20px',
  },
  header: {
    textAlign: 'center' as const,
    marginBottom: '40px',
    paddingBottom: '20px',
    borderBottom: '2px solid rgba(124, 77, 255, 0.3)',
  },
  title: {
    fontSize: '36px',
    margin: '0 0 10px 0',
    background: 'linear-gradient(135deg, #7c4dff 0%, #00bcd4 100%)',
    WebkitBackgroundClip: 'text',
    WebkitTextFillColor: 'transparent',
    backgroundClip: 'text',
  },
  subtitle: {
    fontSize: '16px',
    color: '#8080a0',
    margin: 0,
  },
  main: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '30px',
    maxWidth: '1400px',
    margin: '0 auto',
  },
  section: {
    width: '100%',
  },
  connectedInfo: {
    background: 'rgba(72, 187, 120, 0.1)',
    border: '2px solid rgba(72, 187, 120, 0.3)',
    borderRadius: '12px',
    padding: '24px',
  },
  connectedTitle: {
    margin: '0 0 8px 0',
    fontSize: '24px',
    color: '#48bb78',
  },
  connectedSubtitle: {
    margin: '0 0 24px 0',
    color: '#8080a0',
    fontSize: '14px',
  },
  eventLog: {
    background: 'rgba(255, 255, 255, 0.05)',
    border: '1px solid rgba(255, 255, 255, 0.1)',
    borderRadius: '12px',
    padding: '20px',
  },
  eventLogTitle: {
    margin: '0 0 16px 0',
    fontSize: '20px',
  },
  eventLogContent: {
    maxHeight: '300px',
    overflowY: 'auto' as const,
    background: 'rgba(0, 0, 0, 0.3)',
    borderRadius: '8px',
    padding: '12px',
  },
  noEvents: {
    color: '#8080a0',
    textAlign: 'center' as const,
    margin: 0,
  },
  eventEntry: {
    padding: '8px 12px',
    background: 'rgba(255, 255, 255, 0.05)',
    borderRadius: '6px',
    marginBottom: '8px',
    fontSize: '13px',
    fontFamily: "'Monaco', 'Courier New', monospace",
  },
  footer: {
    marginTop: '60px',
    paddingTop: '30px',
    borderTop: '1px solid rgba(255, 255, 255, 0.1)',
  },
  footerContent: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
    gap: '30px',
    maxWidth: '1200px',
    margin: '0 auto',
  },
  footerSection: {
    color: '#8080a0',
  },
};

export default RobotDiscoveryExample;
