/**
 * Neural Visualizer Screen - Real-time neural state visualization
 * Issue: #88 (STORY-MOBILE-001)
 */

import React, { useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Platform,
  Dimensions,
} from 'react-native';
import { useNeuralState, useAppService } from '../hooks/useAppService';
import { NeuralGraph } from '../components/NeuralGraph';
import { ConnectionIndicator } from '../components/ConnectionIndicator';

const { width, height } = Dimensions.get('window');

export function NeuralVisualizerScreen() {
  const neuralState = useNeuralState();
  const { connected } = useAppService();
  const [animationActive, setAnimationActive] = useState(true);
  const [zoom, setZoom] = useState(1.0);

  const toggleAnimation = () => {
    setAnimationActive(!animationActive);
  };

  const handleZoom = (factor: number) => {
    setZoom((prev) => Math.max(0.5, Math.min(2.0, prev + factor)));
  };

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Neural State</Text>
        <ConnectionIndicator connected={connected} testID="connection-status" />
      </View>

      <View style={styles.visualizerContainer} testID="neural-visualizer">
        {neuralState ? (
          <NeuralGraph
            neuralState={neuralState}
            animationActive={animationActive}
            zoom={zoom}
          />
        ) : (
          <View style={styles.placeholderContainer}>
            <Text style={styles.placeholderText}>
              {connected ? 'Waiting for neural data...' : 'Not connected'}
            </Text>
          </View>
        )}
      </View>

      <View style={styles.controls}>
        <TouchableOpacity
          style={styles.controlButton}
          onPress={toggleAnimation}
          testID="viz-play-pause"
        >
          <Text style={styles.controlButtonText}>
            {animationActive ? 'Pause' : 'Play'}
          </Text>
        </TouchableOpacity>

        <View style={styles.zoomControls}>
          <TouchableOpacity
            style={styles.zoomButton}
            onPress={() => handleZoom(-0.2)}
          >
            <Text style={styles.zoomButtonText}>-</Text>
          </TouchableOpacity>

          <Text style={styles.zoomText}>{(zoom * 100).toFixed(0)}%</Text>

          <TouchableOpacity
            style={styles.zoomButton}
            onPress={() => handleZoom(0.2)}
          >
            <Text style={styles.zoomButtonText}>+</Text>
          </TouchableOpacity>
        </View>
      </View>

      {neuralState && (
        <View style={styles.stats}>
          <View style={styles.statItem}>
            <Text style={styles.statLabel}>Active Nodes</Text>
            <Text style={styles.statValue}>{neuralState.activeNodes.length}</Text>
          </View>
          <View style={styles.statItem}>
            <Text style={styles.statLabel}>Connections</Text>
            <Text style={styles.statValue}>{neuralState.connections.length}</Text>
          </View>
          <View style={styles.statItem}>
            <Text style={styles.statLabel}>Activity</Text>
            <Text style={styles.statValue}>
              {(neuralState.activity * 100).toFixed(0)}%
            </Text>
          </View>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#000',
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    paddingTop: Platform.OS === 'ios' ? 60 : 20,
    backgroundColor: '#1a1a1a',
  },
  title: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#fff',
    ...Platform.select({
      ios: { fontFamily: 'System' },
      android: { fontFamily: 'Roboto' },
    }),
  },
  visualizerContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  placeholderContainer: {
    justifyContent: 'center',
    alignItems: 'center',
  },
  placeholderText: {
    color: '#666',
    fontSize: 16,
  },
  controls: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    backgroundColor: '#1a1a1a',
  },
  controlButton: {
    backgroundColor: '#007AFF',
    paddingHorizontal: 24,
    paddingVertical: 12,
    borderRadius: 8,
  },
  controlButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  zoomControls: {
    flexDirection: 'row',
    alignItems: 'center',
    gap: 12,
  },
  zoomButton: {
    backgroundColor: '#333',
    width: 40,
    height: 40,
    borderRadius: 20,
    justifyContent: 'center',
    alignItems: 'center',
  },
  zoomButtonText: {
    color: '#fff',
    fontSize: 24,
    fontWeight: 'bold',
  },
  zoomText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
    minWidth: 50,
    textAlign: 'center',
  },
  stats: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    padding: 16,
    backgroundColor: '#1a1a1a',
    borderTopWidth: 1,
    borderTopColor: '#333',
  },
  statItem: {
    alignItems: 'center',
  },
  statLabel: {
    color: '#999',
    fontSize: 12,
    marginBottom: 4,
  },
  statValue: {
    color: '#fff',
    fontSize: 20,
    fontWeight: 'bold',
  },
});
