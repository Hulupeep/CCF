/**
 * Personality Mixer Screen - Adjust robot personality parameters
 * Issue: #88 (STORY-MOBILE-001)
 * I-MOBILE-001: Robot must respond within 200ms
 */

import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Platform,
} from 'react-native';
import { usePersonality, useAppService } from '../hooks/useAppService';
import { PersonalitySlider } from '../components/PersonalitySlider';
import { ConnectionIndicator } from '../components/ConnectionIndicator';

const PERSONALITY_PARAMS = [
  { key: 'energy', label: 'Energy', description: 'Movement speed and vigor' },
  { key: 'tension', label: 'Tension', description: 'Precision vs. relaxation' },
  { key: 'curiosity', label: 'Curiosity', description: 'Exploration behavior' },
  { key: 'playfulness', label: 'Playfulness', description: 'Spontaneous actions' },
  { key: 'confidence', label: 'Confidence', description: 'Decision-making boldness' },
  { key: 'focus', label: 'Focus', description: 'Task concentration' },
  { key: 'empathy', label: 'Empathy', description: 'Response to user emotions' },
  { key: 'creativity', label: 'Creativity', description: 'Novel behaviors' },
  { key: 'persistence', label: 'Persistence', description: 'Goal pursuit' },
];

export function PersonalityMixerScreen() {
  const { config, updateSlider, reset } = usePersonality();
  const { connected } = useAppService();

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Personality Mixer</Text>
        <ConnectionIndicator connected={connected} testID="connection-status" />
      </View>

      <ScrollView style={styles.scrollView} testID="personality-mixer">
        {PERSONALITY_PARAMS.map((param) => (
          <PersonalitySlider
            key={param.key}
            label={param.label}
            description={param.description}
            value={config[param.key as keyof typeof config]}
            onValueChange={(value) =>
              updateSlider(param.key as keyof typeof config, value)
            }
            disabled={!connected}
            testID={`slider-${param.key}`}
          />
        ))}
      </ScrollView>

      <View style={styles.footer}>
        <TouchableOpacity
          style={[styles.resetButton, !connected && styles.buttonDisabled]}
          onPress={reset}
          disabled={!connected}
          testID="reset-btn"
        >
          <Text style={styles.resetButtonText}>Reset to Default</Text>
        </TouchableOpacity>

        <TouchableOpacity
          style={[styles.saveButton, !connected && styles.buttonDisabled]}
          disabled={!connected}
          testID="save-preset-btn"
        >
          <Text style={styles.saveButtonText}>Save Preset</Text>
        </TouchableOpacity>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    paddingTop: Platform.OS === 'ios' ? 60 : 20,
    backgroundColor: '#fff',
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    ...Platform.select({
      ios: { fontFamily: 'System' },
      android: { fontFamily: 'Roboto' },
    }),
  },
  scrollView: {
    flex: 1,
  },
  footer: {
    flexDirection: 'row',
    padding: 16,
    backgroundColor: '#fff',
    borderTopWidth: 1,
    borderTopColor: '#e0e0e0',
    gap: 12,
  },
  resetButton: {
    flex: 1,
    backgroundColor: '#ff3b30',
    padding: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  saveButton: {
    flex: 1,
    backgroundColor: '#007AFF',
    padding: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  buttonDisabled: {
    backgroundColor: '#ccc',
  },
  resetButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  saveButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
