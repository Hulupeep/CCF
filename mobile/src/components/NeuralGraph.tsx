/**
 * Neural Graph Component - 2D/3D neural state visualization
 * Issue: #88 (STORY-MOBILE-001)
 */

import React, { useEffect } from 'react';
import { View, StyleSheet, Dimensions } from 'react-native';
import Svg, { Circle, Line, G } from 'react-native-svg';
import { NeuralState } from '../types';

interface NeuralGraphProps {
  neuralState: NeuralState;
  animationActive: boolean;
  zoom: number;
}

const { width, height } = Dimensions.get('window');

export function NeuralGraph({ neuralState, animationActive, zoom }: NeuralGraphProps) {
  // Simple 2D projection of 3D positions
  const project = (x: number, y: number, z: number) => {
    const centerX = width / 2;
    const centerY = height / 2;
    const scale = 100 * zoom;

    return {
      x: centerX + x * scale,
      y: centerY + y * scale,
    };
  };

  const getNodeColor = (type: string) => {
    switch (type) {
      case 'sensory':
        return '#4caf50'; // Green
      case 'motor':
        return '#2196f3'; // Blue
      case 'cognitive':
        return '#ff9800'; // Orange
      default:
        return '#999';
    }
  };

  return (
    <View style={styles.container}>
      <Svg width={width} height={height - 200}>
        <G>
          {/* Draw connections */}
          {neuralState.connections.map((conn, idx) => {
            const fromNode = neuralState.activeNodes.find((n) => n.id === conn.from);
            const toNode = neuralState.activeNodes.find((n) => n.id === conn.to);

            if (!fromNode || !toNode) return null;

            const from = project(fromNode.position.x, fromNode.position.y, fromNode.position.z);
            const to = project(toNode.position.x, toNode.position.y, toNode.position.z);

            return (
              <Line
                key={`conn-${idx}`}
                x1={from.x}
                y1={from.y}
                x2={to.x}
                y2={to.y}
                stroke={conn.active ? '#fff' : '#333'}
                strokeWidth={conn.weight * 2}
                opacity={conn.active ? 0.8 : 0.3}
              />
            );
          })}

          {/* Draw nodes */}
          {neuralState.activeNodes.map((node, idx) => {
            const pos = project(node.position.x, node.position.y, node.position.z);
            const radius = 4 + node.activation * 8;

            return (
              <Circle
                key={`node-${idx}`}
                cx={pos.x}
                cy={pos.y}
                r={radius}
                fill={getNodeColor(node.type)}
                opacity={0.5 + node.activation * 0.5}
              />
            );
          })}
        </G>
      </Svg>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#000',
  },
});
