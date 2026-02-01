/**
 * Personality Slider Component
 * Issue: #88 (STORY-MOBILE-001)
 * I-MOBILE-002: Responsive 320px-768px
 */

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import Slider from '@react-native-community/slider';

interface PersonalitySliderProps {
  label: string;
  description: string;
  value: number;
  onValueChange: (value: number) => void;
  disabled?: boolean;
  testID?: string;
}

export function PersonalitySlider({
  label,
  description,
  value,
  onValueChange,
  disabled = false,
  testID,
}: PersonalitySliderProps) {
  return (
    <View style={styles.container} testID={testID}>
      <View style={styles.header}>
        <Text style={styles.label}>{label}</Text>
        <Text style={styles.value}>{(value * 100).toFixed(0)}%</Text>
      </View>

      <Text style={styles.description}>{description}</Text>

      <Slider
        style={styles.slider}
        minimumValue={0}
        maximumValue={1}
        value={value}
        onValueChange={onValueChange}
        disabled={disabled}
        minimumTrackTintColor="#007AFF"
        maximumTrackTintColor="#e0e0e0"
        thumbTintColor="#007AFF"
      />
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: '#fff',
    padding: 16,
    marginHorizontal: 16,
    marginVertical: 8,
    borderRadius: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 1 },
    shadowOpacity: 0.1,
    shadowRadius: 2,
    elevation: 2,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 4,
  },
  label: {
    fontSize: 16,
    fontWeight: '600',
  },
  value: {
    fontSize: 16,
    fontWeight: 'bold',
    color: '#007AFF',
  },
  description: {
    fontSize: 13,
    color: '#666',
    marginBottom: 8,
  },
  slider: {
    width: '100%',
    height: 40,
  },
});
