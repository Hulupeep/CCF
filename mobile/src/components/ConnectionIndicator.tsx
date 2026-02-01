/**
 * Connection Indicator Component
 * Shows online/offline status
 * Issue: #88 (STORY-MOBILE-001)
 * I-MOBILE-003: Shows offline mode
 */

import React from 'react';
import { View, Text, StyleSheet } from 'react-native';

interface ConnectionIndicatorProps {
  connected: boolean;
  testID?: string;
}

export function ConnectionIndicator({ connected, testID }: ConnectionIndicatorProps) {
  return (
    <View
      style={[styles.container, connected ? styles.connected : styles.disconnected]}
      testID={testID}
    >
      <View style={[styles.indicator, connected && styles.indicatorOnline]} />
      <Text style={styles.text}>{connected ? 'Connected' : 'Offline'}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 12,
    paddingVertical: 6,
    borderRadius: 16,
  },
  connected: {
    backgroundColor: '#e8f5e9',
  },
  disconnected: {
    backgroundColor: '#ffebee',
  },
  indicator: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: '#f44336',
    marginRight: 6,
  },
  indicatorOnline: {
    backgroundColor: '#4caf50',
  },
  text: {
    fontSize: 12,
    fontWeight: '600',
    color: '#333',
  },
});
