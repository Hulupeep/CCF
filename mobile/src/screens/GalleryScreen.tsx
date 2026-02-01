/**
 * Gallery Screen - View saved drawings
 * Issue: #88 (STORY-MOBILE-001)
 */

import React, { useEffect, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  FlatList,
  TouchableOpacity,
  Image,
  Modal,
  Platform,
  ActivityIndicator,
} from 'react-native';
import { useGallery, useAppService } from '../hooks/useAppService';
import { Drawing } from '../types';
import { ConnectionIndicator } from '../components/ConnectionIndicator';

export function GalleryScreen() {
  const { drawings, loading, fetchDrawings } = useGallery();
  const { connected } = useAppService();
  const [selectedDrawing, setSelectedDrawing] = useState<Drawing | null>(null);

  useEffect(() => {
    fetchDrawings();
  }, []);

  const renderDrawing = ({ item }: { item: Drawing }) => (
    <TouchableOpacity
      style={styles.thumbnail}
      onPress={() => setSelectedDrawing(item)}
      testID={`drawing-thumb-${item.id}`}
    >
      {item.thumbnailUrl ? (
        <Image source={{ uri: item.thumbnailUrl }} style={styles.thumbnailImage} />
      ) : (
        <View style={styles.placeholderThumbnail}>
          <Text style={styles.placeholderText}>No Preview</Text>
        </View>
      )}
      <Text style={styles.thumbnailTitle} numberOfLines={1}>
        {item.name}
      </Text>
      <Text style={styles.thumbnailDate}>
        {new Date(item.timestamp).toLocaleDateString()}
      </Text>
    </TouchableOpacity>
  );

  return (
    <View style={styles.container}>
      <View style={styles.header}>
        <Text style={styles.title}>Gallery</Text>
        <ConnectionIndicator connected={connected} testID="connection-status" />
      </View>

      {loading ? (
        <View style={styles.loadingContainer}>
          <ActivityIndicator size="large" color="#007AFF" />
          <Text style={styles.loadingText}>Loading drawings...</Text>
        </View>
      ) : (
        <FlatList
          data={drawings}
          renderItem={renderDrawing}
          keyExtractor={(item) => item.id}
          numColumns={2}
          testID="gallery-grid"
          style={styles.grid}
          contentContainerStyle={styles.gridContent}
          ListEmptyComponent={
            <View style={styles.emptyContainer}>
              <Text style={styles.emptyText}>No drawings yet</Text>
              <Text style={styles.emptyHint}>
                Create drawings with your robot to see them here
              </Text>
            </View>
          }
        />
      )}

      <TouchableOpacity
        style={[styles.refreshButton, loading && styles.buttonDisabled]}
        onPress={fetchDrawings}
        disabled={loading}
      >
        <Text style={styles.refreshButtonText}>
          {loading ? 'Loading...' : 'Refresh'}
        </Text>
      </TouchableOpacity>

      {/* Full Drawing Modal */}
      <Modal
        visible={selectedDrawing !== null}
        animationType="slide"
        onRequestClose={() => setSelectedDrawing(null)}
      >
        {selectedDrawing && (
          <View style={styles.modalContainer} testID={`drawing-full-${selectedDrawing.id}`}>
            <View style={styles.modalHeader}>
              <Text style={styles.modalTitle}>{selectedDrawing.name}</Text>
              <TouchableOpacity
                style={styles.closeButton}
                onPress={() => setSelectedDrawing(null)}
              >
                <Text style={styles.closeButtonText}>Close</Text>
              </TouchableOpacity>
            </View>

            <View style={styles.drawingContainer}>
              {selectedDrawing.thumbnailUrl ? (
                <Image
                  source={{ uri: selectedDrawing.thumbnailUrl }}
                  style={styles.fullDrawing}
                  resizeMode="contain"
                />
              ) : (
                <View style={styles.placeholderFull}>
                  <Text style={styles.placeholderText}>Drawing Preview</Text>
                </View>
              )}
            </View>

            <View style={styles.modalFooter}>
              <TouchableOpacity
                style={styles.playbackButton}
                testID={`playback-btn-${selectedDrawing.id}`}
              >
                <Text style={styles.playbackButtonText}>Play Animation</Text>
              </TouchableOpacity>
            </View>
          </View>
        )}
      </Modal>
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
  loadingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  loadingText: {
    marginTop: 16,
    fontSize: 16,
    color: '#666',
  },
  grid: {
    flex: 1,
  },
  gridContent: {
    padding: 8,
  },
  thumbnail: {
    flex: 1,
    margin: 8,
    backgroundColor: '#fff',
    borderRadius: 12,
    overflow: 'hidden',
    shadowColor: '#000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: 0.1,
    shadowRadius: 4,
    elevation: 3,
  },
  thumbnailImage: {
    width: '100%',
    aspectRatio: 1,
  },
  placeholderThumbnail: {
    width: '100%',
    aspectRatio: 1,
    backgroundColor: '#e0e0e0',
    justifyContent: 'center',
    alignItems: 'center',
  },
  placeholderText: {
    color: '#999',
    fontSize: 14,
  },
  thumbnailTitle: {
    padding: 8,
    fontSize: 14,
    fontWeight: '600',
  },
  thumbnailDate: {
    paddingHorizontal: 8,
    paddingBottom: 8,
    fontSize: 12,
    color: '#666',
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
  refreshButton: {
    backgroundColor: '#007AFF',
    padding: 16,
    margin: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  buttonDisabled: {
    backgroundColor: '#ccc',
  },
  refreshButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
  modalContainer: {
    flex: 1,
    backgroundColor: '#fff',
  },
  modalHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    padding: 16,
    paddingTop: Platform.OS === 'ios' ? 60 : 20,
    borderBottomWidth: 1,
    borderBottomColor: '#e0e0e0',
  },
  modalTitle: {
    fontSize: 20,
    fontWeight: 'bold',
  },
  closeButton: {
    padding: 8,
  },
  closeButtonText: {
    color: '#007AFF',
    fontSize: 16,
  },
  drawingContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
  },
  fullDrawing: {
    width: '100%',
    height: '100%',
  },
  placeholderFull: {
    width: '100%',
    height: '100%',
    backgroundColor: '#e0e0e0',
    justifyContent: 'center',
    alignItems: 'center',
  },
  modalFooter: {
    padding: 16,
    borderTopWidth: 1,
    borderTopColor: '#e0e0e0',
  },
  playbackButton: {
    backgroundColor: '#007AFF',
    padding: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  playbackButtonText: {
    color: '#fff',
    fontSize: 16,
    fontWeight: '600',
  },
});
