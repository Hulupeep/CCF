/**
 * Voice Dashboard - Main UI for voice assistant
 *
 * Displays voice profiles, briefing panel, conversation history,
 * memory timeline, news preferences, email accounts, and privacy settings
 * data-testid: voice-assistant-dashboard
 */

import React, { useState } from 'react';
import { BriefingPanel } from '../briefing/BriefingPanel';
import { VoiceProfileList } from './VoiceProfileList';
import { ConversationHistory } from './ConversationHistory';
import { MemoryTimeline } from './MemoryTimeline';
import { NewsPreferences } from './NewsPreferences';
import { EmailAccounts } from './EmailAccounts';
import { PrivacySettings } from './PrivacySettings';

export function VoiceDashboard() {
  const [currentUser, setCurrentUser] = useState<string>('user-1'); // Mock user ID
  const [activeTab, setActiveTab] = useState<string>('briefing');

  const tabs = [
    { id: 'briefing', label: 'Daily Briefing', icon: '🌅' },
    { id: 'profiles', label: 'Voice Profiles', icon: '👤' },
    { id: 'conversations', label: 'Conversations', icon: '💬' },
    { id: 'memory', label: 'Memory', icon: '💭' },
    { id: 'news', label: 'News Preferences', icon: '📰' },
    { id: 'email', label: 'Email Accounts', icon: '📧' },
    { id: 'privacy', label: 'Privacy', icon: '🔒' }
  ];

  return (
    <div data-testid="voice-assistant-dashboard" className="voice-dashboard">
      <header className="dashboard-header">
        <h1>Voice Assistant</h1>
        <div className="user-indicator">
          Current User: <strong>{currentUser}</strong>
        </div>
      </header>

      <nav className="dashboard-tabs">
        {tabs.map(tab => (
          <button
            key={tab.id}
            className={`tab ${activeTab === tab.id ? 'active' : ''}`}
            onClick={() => setActiveTab(tab.id)}
            data-testid={`tab-${tab.id}`}
          >
            <span className="tab-icon">{tab.icon}</span>
            <span className="tab-label">{tab.label}</span>
          </button>
        ))}
      </nav>

      <main className="dashboard-content">
        {activeTab === 'briefing' && <BriefingPanel userId={currentUser} />}
        {activeTab === 'profiles' && <VoiceProfileList />}
        {activeTab === 'conversations' && <ConversationHistory userId={currentUser} />}
        {activeTab === 'memory' && <MemoryTimeline userId={currentUser} />}
        {activeTab === 'news' && <NewsPreferences userId={currentUser} />}
        {activeTab === 'email' && <EmailAccounts userId={currentUser} />}
        {activeTab === 'privacy' && <PrivacySettings userId={currentUser} />}
      </main>
    </div>
  );
}
