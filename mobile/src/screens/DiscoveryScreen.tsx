/**
 * Discovery Screen - Find and connect to robots
 * Issue: #88 (STORY-MOBILE-001)
 */

import React, { useEffect } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  ActivityIndicator,
  Platform,
} from 'react-native';
import { useRobotDiscovery, useAppService } from '../hooks/useAppService';
import { Robot } from '../types';

export function DiscoveryScreen({ navigation }: any) {
  const { robots, scanning, error, startScan, connect } = useRobotDiscovery();
  const { connected } = useAppService();

  useEffect(() => {
    // Auto-scan on mount
    startScan();
  }, []);

  useEffect(() => {
    // Navigate to mixer when connected
    if (connected) {
      navigation.navigate('Mixer');
    }
  }, [connected, navigation]);

  const handleConnect = async (robotId: string) => {
    try {
      await connect(robotId);
    } catch (err) {
      console.error('Connection failed:', err);
    }
  };

  const renderRobot = ({ item }: { item: Robot }) => (
    <TouchableOpacity
      style={styles.robotCard}
      onPress={() => handleConnect(item.id)}
      testID={`connect-btn-${item.id}`}
    >
      <View style={styles.robotInfo}>
        <Text style={styles.robotName}>{item.name}</Text>
        <Text style={styles.robotAddress}>
          {item.ipAddress}:{item.port}
        </Text>
        <View style={[styles.statusBadge, item.status === 'online' && styles.statusOnline]}>
          <Text style={styles.statusText}>{item.status}</Text>
        </View>
      </View>
    </TouchableOpacity>
  );

  return (
    <View style={styles.container}>
      <Text style={styles.title}>Discover Robots</Text>

      {error && (
        <View style={styles.errorContainer}>
          <Text style={styles.errorText}>{error}</Text>
        </View>
      )}

      {scanning ? (
        <View style={styles.scanningContainer}>
          <ActivityIndicator size="large" color="#007AFF" />
          <Text style={styles.scanningText}>Scanning for robots...</Text>
        </View>
      ) : (
        <FlatList
          data={robots}
          renderItem={renderRobot}
          keyExtractor={(item) => item.id}
          testID="discovery-list"
          style={styles.list}
          ListEmptyComponent={
            <View style={styles.emptyContainer}>
              <Text style={styles.emptyText}>No robots found</Text>
              <Text style={styles.emptyHint}>
                Make sure your robot is powered on and connected to the same WiFi network
              </Text>
            </View>
          }
        />
      )}

      <TouchableOpacity
        style={[styles.scanButton, scanning && styles.scanButtonDisabled]}
        onPress={startScan}
        disabled={scanning}
        testID="scan-btn"
      >
        <Text style={styles.scanButtonText}>
          {scanning ? 'Scanning...' : 'Scan Again'}
        </Text>
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#f5f5f5',
    padding: 16,
  },
  title: {
    fontSize: 28,
    fontWeight: 'bold',
    marginBottom: 20,
    marginTop: Platform.OS === 'ios' ? 40 : 20,
    ...Platform.select({
      ios: { fontFamily: 'System' },
      android: { fontFamily: 'Roboto' },
    }),
  },
  errorContainer: {
    backgroundColor: '#ff3b30',
    padding: 12,
    borderRadius: 8,
    marginBottom: 16,
  },
  errorText: {
    color: '#fff',
    fontSize: 14,
  },
  scanningContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  scanningText: {
    marginTop: 16,
    fontSize: 16,
    color: '#666',
  },
  list: {
    flex: 1,
  },
  robotCard: {
    backgroundColor: '#fff',
    padding: 16,
    borderRadius: 12,
    marginBottom: 12,
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  robotInfo: {
    flex: 1,
  },
  robotName: {
    fontSize: 18,
    fontWeight: '600',
    marginBottom: 4,
  },
  robotAddress: {
    fontSize: 14,
    color: '#666',
    marginBottom: 8,
  },
  statusBadge: {
    alignSelf: 'flex-start',
    paddingHorizontal: 8,
    paddingVertical: 4,
    borderRadius: 4,
    backgroundColor: '#ccc',
  },
  statusOnline: {
    backgroundColor: '#34c759',
  },
  statusText: {
    color: '#fff',
    fontSize: 12,
    fontWeight: '600',
  },
  emptyContainer: {
    padding: 32,
    alignItems: 'center',
  },
  emptyText: {
    fontSize: 18,
    fontWeight: '600',
    marginBottom: 8,
    color: '#666',
  },
  emptyHint: {
    fontSize: 14,
    color: '#999',
    textAlign: 'center',
  },
  scanButton: {
    backgroundColor: '#007AFF',
    padding: 16,
    borderRadius: 12,
    alignItems: 'center',
    marginTop: 16,
  },
  scanButtonDisabled: {
    backgroundColor: '#ccc',
  },
  scanButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
