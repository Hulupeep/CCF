/**
 * Example App demonstrating PersonalityMixer usage
 */

import React from 'react';
import PersonalityMixer from './components/PersonalityMixer';
import './components/PersonalityMixer.css';
import { PersonalityConfig } from './types/personality';

function App() {
  const handleConfigChange = (config: PersonalityConfig) => {
    console.log('Personality config changed:', config);
  };

  return (
    <div className="app">
      <header style={{ textAlign: 'center', padding: '20px 0' }}>
        <h1 style={{ fontSize: '32px', marginBottom: '10px' }}>🎨 mBot2 Personality Mixer</h1>
        <p style={{ color: '#8080a0', fontSize: '14px' }}>
          Adjust personality parameters and see immediate behavior changes
        </p>
      </header>

      <PersonalityMixer
        wsUrl="ws://localhost:8081"
        onConfigChange={handleConfigChange}
      />
    </div>
  );
}

export default App;
