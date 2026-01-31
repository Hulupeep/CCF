/**
 * Drawing Gallery Component
 * Contract: ART-001, ART-002, ART-005, I-ART-GAL-001, I-ART-GAL-002
 * Journey: J-ART-FIRST-DRAWING
 * Issue: #60 (STORY-ART-006)
 */

import React, { useState, useEffect, useMemo } from 'react';
import { Drawing, DrawingFilter, ExportFormat } from '../types/drawing';
import { useArtworkGallery } from '../hooks/useArtworkGallery';
import {
  generateThumbnail,
  exportToPNG,
  exportToSVG,
  exportToJSON,
  downloadFile,
  downloadPNG,
} from '../services/drawingExport';

interface DrawingGalleryProps {
  itemsPerPage?: number;
  onPlaybackRequest?: (drawing: Drawing) => void;
}

export const DrawingGallery: React.FC<DrawingGalleryProps> = ({
  itemsPerPage = 20,
  onPlaybackRequest,
}) => {
  const {
    drawings,
    loading,
    error,
    pagination,
    uniqueMoods,
    filter,
    setFilter,
    setPage,
    deleteDrawing,
    refreshGallery,
  } = useArtworkGallery(itemsPerPage);

  const [selectedDrawing, setSelectedDrawing] = useState<Drawing | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedMood, setSelectedMood] = useState<string>('');
  const [dateFrom, setDateFrom] = useState<string>('');
  const [dateTo, setDateTo] = useState<string>('');
  const [showSignatureOnly, setShowSignatureOnly] = useState(false);
  const [thumbnailCache, setThumbnailCache] = useState<Map<string, string>>(new Map());
  const [isPlayingBack, setIsPlayingBack] = useState(false);
  const [playbackProgress, setPlaybackProgress] = useState(0);

  /**
   * Generate thumbnails for drawings
   */
  useEffect(() => {
    const newCache = new Map(thumbnailCache);
    drawings.forEach(drawing => {
      if (!newCache.has(drawing.id)) {
        try {
          const thumbnail = generateThumbnail(drawing);
          newCache.set(drawing.id, thumbnail);
        } catch (err) {
          console.error(`Failed to generate thumbnail for ${drawing.id}:`, err);
        }
      }
    });
    setThumbnailCache(newCache);
  }, [drawings]);

  /**
   * Apply filters
   */
  const handleFilterChange = () => {
    const newFilter: DrawingFilter = {
      searchQuery: searchQuery || undefined,
      mood: selectedMood || undefined,
      dateFrom: dateFrom ? new Date(dateFrom).getTime() : undefined,
      dateTo: dateTo ? new Date(dateTo).getTime() : undefined,
      hasSignature: showSignatureOnly ? true : undefined,
    };
    setFilter(newFilter);
    setPage(1); // Reset to first page
  };

  /**
   * Clear all filters
   */
  const handleClearFilters = () => {
    setSearchQuery('');
    setSelectedMood('');
    setDateFrom('');
    setDateTo('');
    setShowSignatureOnly(false);
    setFilter({});
    setPage(1);
  };

  /**
   * Handle drawing deletion with confirmation
   */
  const handleDelete = async (id: string) => {
    if (window.confirm('Are you sure you want to delete this drawing? This action cannot be undone.')) {
      try {
        await deleteDrawing(id);
        if (selectedDrawing?.id === id) {
          setSelectedDrawing(null);
        }
      } catch (err) {
        alert(`Failed to delete drawing: ${err instanceof Error ? err.message : 'Unknown error'}`);
      }
    }
  };

  /**
   * Export drawing in selected format
   */
  const handleExport = (drawing: Drawing, format: ExportFormat) => {
    try {
      const timestamp = new Date(drawing.createdAt).toISOString().replace(/[:.]/g, '-');
      const filename = `mbot-drawing-${timestamp}`;

      switch (format) {
        case 'png': {
          const dataUrl = exportToPNG(drawing, {
            format: 'png',
            width: 1200,
            height: 900,
            backgroundColor: '#ffffff',
            includeMetadata: true,
          });
          downloadPNG(dataUrl, `${filename}.png`);
          break;
        }
        case 'svg': {
          const svgContent = exportToSVG(drawing, {
            format: 'svg',
            width: 1200,
            height: 900,
            backgroundColor: '#ffffff',
            includeMetadata: true,
          });
          downloadFile(svgContent, `${filename}.svg`, 'image/svg+xml');
          break;
        }
        case 'json': {
          const jsonContent = exportToJSON(drawing);
          downloadFile(jsonContent, `${filename}.json`, 'application/json');
          break;
        }
      }
    } catch (err) {
      alert(`Failed to export: ${err instanceof Error ? err.message : 'Unknown error'}`);
    }
  };

  /**
   * Play back drawing animation
   * Contract: I-ART-GAL-002 - Playback matches original speed
   */
  const handlePlayback = async (drawing: Drawing) => {
    if (onPlaybackRequest) {
      onPlaybackRequest(drawing);
      return;
    }

    setIsPlayingBack(true);
    setPlaybackProgress(0);

    const startTime = drawing.strokes[0]?.timestamp || 0;
    const endTime = drawing.strokes[drawing.strokes.length - 1]?.timestamp || 0;
    const totalDuration = endTime - startTime;

    // Simulate playback progress
    const interval = setInterval(() => {
      setPlaybackProgress(prev => {
        const next = prev + (100 / (totalDuration / 50)); // Update every 50ms
        if (next >= 100) {
          clearInterval(interval);
          setIsPlayingBack(false);
          return 100;
        }
        return next;
      });
    }, 50);
  };

  /**
   * Format duration in human-readable format
   */
  const formatDuration = (ms: number): string => {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    const minutes = Math.floor(ms / 60000);
    const seconds = Math.floor((ms % 60000) / 1000);
    return `${minutes}m ${seconds}s`;
  };

  /**
   * Pagination controls
   */
  const renderPagination = () => {
    const pages = [];
    const maxPagesToShow = 5;
    const startPage = Math.max(1, pagination.page - Math.floor(maxPagesToShow / 2));
    const endPage = Math.min(pagination.totalPages, startPage + maxPagesToShow - 1);

    for (let i = startPage; i <= endPage; i++) {
      pages.push(
        <button
          key={i}
          onClick={() => setPage(i)}
          disabled={i === pagination.page}
          data-testid={`page-button-${i}`}
          style={{
            padding: '8px 12px',
            margin: '0 4px',
            border: '1px solid #ddd',
            backgroundColor: i === pagination.page ? '#4A90E2' : '#fff',
            color: i === pagination.page ? '#fff' : '#333',
            cursor: i === pagination.page ? 'default' : 'pointer',
            borderRadius: '4px',
          }}
        >
          {i}
        </button>
      );
    }

    return (
      <div data-testid="gallery-pagination" style={{ marginTop: '24px', textAlign: 'center' }}>
        <button
          onClick={() => setPage(1)}
          disabled={pagination.page === 1}
          data-testid="first-page-button"
          style={{ marginRight: '8px', padding: '8px 12px' }}
        >
          First
        </button>
        <button
          onClick={() => setPage(pagination.page - 1)}
          disabled={pagination.page === 1}
          data-testid="prev-page-button"
          style={{ marginRight: '8px', padding: '8px 12px' }}
        >
          Previous
        </button>
        {pages}
        <button
          onClick={() => setPage(pagination.page + 1)}
          disabled={pagination.page === pagination.totalPages}
          data-testid="next-page-button"
          style={{ marginLeft: '8px', padding: '8px 12px' }}
        >
          Next
        </button>
        <button
          onClick={() => setPage(pagination.totalPages)}
          disabled={pagination.page === pagination.totalPages}
          data-testid="last-page-button"
          style={{ marginLeft: '8px', padding: '8px 12px' }}
        >
          Last
        </button>
        <div style={{ marginTop: '12px', color: '#666' }}>
          Page {pagination.page} of {pagination.totalPages} ({pagination.totalItems} drawings)
        </div>
      </div>
    );
  };

  if (error) {
    return (
      <div data-testid="gallery-error" style={{ padding: '24px', color: '#E24A4A' }}>
        Error loading gallery: {error}
      </div>
    );
  }

  return (
    <div data-testid="drawing-gallery" style={{ padding: '24px', maxWidth: '1400px', margin: '0 auto' }}>
      <h1>Drawing Gallery</h1>

      {/* Filter Controls */}
      <div
        data-testid="gallery-filters"
        style={{
          marginBottom: '24px',
          padding: '16px',
          backgroundColor: '#f5f5f5',
          borderRadius: '8px',
        }}
      >
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '16px' }}>
          <div>
            <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>
              Search
            </label>
            <input
              type="text"
              value={searchQuery}
              onChange={e => setSearchQuery(e.target.value)}
              placeholder="Session ID or date..."
              data-testid="search-input"
              style={{ width: '100%', padding: '8px', borderRadius: '4px', border: '1px solid #ddd' }}
            />
          </div>

          <div>
            <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>
              Mood
            </label>
            <select
              value={selectedMood}
              onChange={e => setSelectedMood(e.target.value)}
              data-testid="filter-mood"
              style={{ width: '100%', padding: '8px', borderRadius: '4px', border: '1px solid #ddd' }}
            >
              <option value="">All Moods</option>
              {uniqueMoods.map(mood => (
                <option key={mood} value={mood} data-testid={`filter-mood-${mood.toLowerCase()}`}>
                  {mood}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>
              From Date
            </label>
            <input
              type="date"
              value={dateFrom}
              onChange={e => setDateFrom(e.target.value)}
              data-testid="filter-date-from"
              style={{ width: '100%', padding: '8px', borderRadius: '4px', border: '1px solid #ddd' }}
            />
          </div>

          <div>
            <label style={{ display: 'block', marginBottom: '4px', fontWeight: 'bold' }}>
              To Date
            </label>
            <input
              type="date"
              value={dateTo}
              onChange={e => setDateTo(e.target.value)}
              data-testid="filter-date-to"
              style={{ width: '100%', padding: '8px', borderRadius: '4px', border: '1px solid #ddd' }}
            />
          </div>
        </div>

        <div style={{ marginTop: '16px' }}>
          <label style={{ display: 'flex', alignItems: 'center', cursor: 'pointer' }}>
            <input
              type="checkbox"
              checked={showSignatureOnly}
              onChange={e => setShowSignatureOnly(e.target.checked)}
              data-testid="filter-signature"
              style={{ marginRight: '8px' }}
            />
            Show only drawings with signature
          </label>
        </div>

        <div style={{ marginTop: '16px', display: 'flex', gap: '12px' }}>
          <button
            onClick={handleFilterChange}
            data-testid="apply-filters-button"
            style={{
              padding: '10px 20px',
              backgroundColor: '#4A90E2',
              color: '#fff',
              border: 'none',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            Apply Filters
          </button>
          <button
            onClick={handleClearFilters}
            data-testid="clear-filters-button"
            style={{
              padding: '10px 20px',
              backgroundColor: '#fff',
              color: '#333',
              border: '1px solid #ddd',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            Clear Filters
          </button>
          <button
            onClick={refreshGallery}
            data-testid="refresh-button"
            style={{
              padding: '10px 20px',
              backgroundColor: '#fff',
              color: '#333',
              border: '1px solid #ddd',
              borderRadius: '4px',
              cursor: 'pointer',
            }}
          >
            Refresh
          </button>
        </div>
      </div>

      {/* Loading State */}
      {loading && (
        <div data-testid="gallery-loading" style={{ textAlign: 'center', padding: '48px' }}>
          Loading drawings...
        </div>
      )}

      {/* Empty State */}
      {!loading && drawings.length === 0 && (
        <div data-testid="gallery-empty" style={{ textAlign: 'center', padding: '48px', color: '#666' }}>
          No drawings found. Try adjusting your filters or create some artwork!
        </div>
      )}

      {/* Gallery Grid */}
      {!loading && drawings.length > 0 && (
        <>
          <div
            data-testid="drawing-gallery-grid"
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fill, minmax(250px, 1fr))',
              gap: '24px',
            }}
          >
            {drawings.map(drawing => (
              <div
                key={drawing.id}
                data-testid={`drawing-thumbnail-${drawing.id}`}
                style={{
                  border: '1px solid #ddd',
                  borderRadius: '8px',
                  overflow: 'hidden',
                  backgroundColor: '#fff',
                  boxShadow: '0 2px 4px rgba(0,0,0,0.1)',
                  transition: 'transform 0.2s',
                  cursor: 'pointer',
                }}
                onMouseEnter={e => {
                  (e.currentTarget as HTMLElement).style.transform = 'scale(1.02)';
                }}
                onMouseLeave={e => {
                  (e.currentTarget as HTMLElement).style.transform = 'scale(1)';
                }}
                onClick={() => setSelectedDrawing(drawing)}
              >
                {/* Thumbnail */}
                <div
                  style={{
                    width: '100%',
                    height: '200px',
                    backgroundColor: '#f9f9f9',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}
                >
                  {thumbnailCache.has(drawing.id) ? (
                    <img
                      src={thumbnailCache.get(drawing.id)}
                      alt={`Drawing ${drawing.id}`}
                      style={{ maxWidth: '100%', maxHeight: '100%' }}
                    />
                  ) : (
                    <div>Generating thumbnail...</div>
                  )}
                </div>

                {/* Metadata */}
                <div style={{ padding: '12px' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px' }}>
                    <span
                      style={{
                        padding: '4px 8px',
                        backgroundColor: getMoodColorForBadge(drawing.dominantMood),
                        color: '#fff',
                        borderRadius: '4px',
                        fontSize: '12px',
                        fontWeight: 'bold',
                      }}
                    >
                      {drawing.dominantMood}
                    </span>
                    {drawing.hasSignature && (
                      <span
                        style={{
                          padding: '4px 8px',
                          backgroundColor: '#7B68EE',
                          color: '#fff',
                          borderRadius: '4px',
                          fontSize: '12px',
                        }}
                      >
                        ✓ Signed
                      </span>
                    )}
                  </div>

                  <div style={{ fontSize: '14px', color: '#666' }}>
                    <div>{new Date(drawing.createdAt).toLocaleDateString()}</div>
                    <div>{new Date(drawing.createdAt).toLocaleTimeString()}</div>
                    <div style={{ marginTop: '4px' }}>Duration: {formatDuration(drawing.duration)}</div>
                    <div>Strokes: {drawing.strokes.length}</div>
                    {drawing.sessionId && (
                      <div style={{ fontSize: '12px', marginTop: '4px', wordBreak: 'break-all' }}>
                        Session: {drawing.sessionId.substring(0, 8)}...
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>

          {/* Pagination */}
          {pagination.totalPages > 1 && renderPagination()}
        </>
      )}

      {/* Modal for Full-Size Viewing */}
      {selectedDrawing && (
        <div
          data-testid="drawing-modal"
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: 'rgba(0, 0, 0, 0.8)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
          }}
          onClick={() => setSelectedDrawing(null)}
        >
          <div
            style={{
              backgroundColor: '#fff',
              borderRadius: '8px',
              maxWidth: '90%',
              maxHeight: '90%',
              overflow: 'auto',
              position: 'relative',
            }}
            onClick={e => e.stopPropagation()}
          >
            {/* Close Button */}
            <button
              onClick={() => setSelectedDrawing(null)}
              data-testid="modal-close-button"
              style={{
                position: 'absolute',
                top: '12px',
                right: '12px',
                padding: '8px 16px',
                backgroundColor: '#E24A4A',
                color: '#fff',
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                zIndex: 1001,
              }}
            >
              ✕ Close
            </button>

            {/* Drawing Display */}
            <div style={{ padding: '24px' }}>
              <div
                style={{
                  backgroundColor: '#f9f9f9',
                  padding: '24px',
                  borderRadius: '8px',
                  marginBottom: '16px',
                  minHeight: '400px',
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                }}
              >
                {thumbnailCache.has(selectedDrawing.id) && (
                  <img
                    src={thumbnailCache.get(selectedDrawing.id)}
                    alt={`Drawing ${selectedDrawing.id}`}
                    style={{ maxWidth: '100%', maxHeight: '600px' }}
                    data-testid="modal-drawing-image"
                  />
                )}
              </div>

              {/* Playback Controls */}
              <div style={{ marginBottom: '16px' }}>
                <button
                  onClick={() => handlePlayback(selectedDrawing)}
                  disabled={isPlayingBack}
                  data-testid="playback-play-button"
                  style={{
                    padding: '12px 24px',
                    backgroundColor: isPlayingBack ? '#ddd' : '#4A90E2',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: isPlayingBack ? 'not-allowed' : 'pointer',
                    marginRight: '12px',
                  }}
                >
                  {isPlayingBack ? '▶ Playing...' : '▶ Play'}
                </button>
                {isPlayingBack && (
                  <div
                    style={{
                      marginTop: '12px',
                      height: '8px',
                      backgroundColor: '#ddd',
                      borderRadius: '4px',
                      overflow: 'hidden',
                    }}
                  >
                    <div
                      style={{
                        height: '100%',
                        width: `${playbackProgress}%`,
                        backgroundColor: '#4A90E2',
                        transition: 'width 0.05s linear',
                      }}
                    />
                  </div>
                )}
              </div>

              {/* Metadata Display */}
              <div style={{ marginBottom: '16px', padding: '16px', backgroundColor: '#f5f5f5', borderRadius: '8px' }}>
                <h3>Metadata</h3>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px', fontSize: '14px' }}>
                  <div>
                    <strong>Created:</strong> {new Date(selectedDrawing.createdAt).toLocaleString()}
                  </div>
                  <div>
                    <strong>Mood:</strong> {selectedDrawing.dominantMood}
                  </div>
                  <div>
                    <strong>Duration:</strong> {formatDuration(selectedDrawing.duration)}
                  </div>
                  <div>
                    <strong>Strokes:</strong> {selectedDrawing.strokes.length}
                  </div>
                  <div>
                    <strong>Has Signature:</strong> {selectedDrawing.hasSignature ? 'Yes' : 'No'}
                  </div>
                  {selectedDrawing.sessionId && (
                    <div>
                      <strong>Session:</strong> {selectedDrawing.sessionId}
                    </div>
                  )}
                  <div>
                    <strong>Avg Tension:</strong> {selectedDrawing.metadata.averageTension.toFixed(2)}
                  </div>
                  <div>
                    <strong>Avg Coherence:</strong> {selectedDrawing.metadata.averageCoherence.toFixed(2)}
                  </div>
                  <div>
                    <strong>Avg Energy:</strong> {selectedDrawing.metadata.averageEnergy.toFixed(2)}
                  </div>
                  <div>
                    <strong>Path Length:</strong> {selectedDrawing.metadata.totalPathLength.toFixed(1)}mm
                  </div>
                </div>
              </div>

              {/* Action Buttons */}
              <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap' }}>
                <button
                  onClick={() => handleExport(selectedDrawing, 'png')}
                  data-testid="export-png-button"
                  style={{
                    padding: '10px 20px',
                    backgroundColor: '#4A90E2',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                  }}
                >
                  Export PNG
                </button>
                <button
                  onClick={() => handleExport(selectedDrawing, 'svg')}
                  data-testid="export-svg-button"
                  style={{
                    padding: '10px 20px',
                    backgroundColor: '#4A90E2',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                  }}
                >
                  Export SVG
                </button>
                <button
                  onClick={() => handleExport(selectedDrawing, 'json')}
                  data-testid="export-json-button"
                  style={{
                    padding: '10px 20px',
                    backgroundColor: '#4A90E2',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                  }}
                >
                  Export JSON
                </button>
                <button
                  onClick={() => handleDelete(selectedDrawing.id)}
                  data-testid="delete-button"
                  style={{
                    padding: '10px 20px',
                    backgroundColor: '#E24A4A',
                    color: '#fff',
                    border: 'none',
                    borderRadius: '4px',
                    cursor: 'pointer',
                    marginLeft: 'auto',
                  }}
                >
                  Delete
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

/**
 * Get mood color for badge display
 */
function getMoodColorForBadge(mood: string): string {
  const moodColors: Record<string, string> = {
    Calm: '#4A90E2',
    Active: '#F5A623',
    Spike: '#E24A4A',
    Protect: '#7B68EE',
  };
  return moodColors[mood] || '#666';
}
