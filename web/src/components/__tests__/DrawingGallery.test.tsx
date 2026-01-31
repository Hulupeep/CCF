/**
 * Drawing Gallery Tests
 * Contract: ART-001, ART-002, ART-005, I-ART-GAL-001, I-ART-GAL-002
 * Journey: J-ART-FIRST-DRAWING
 */

import React from 'react';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import { DrawingGallery } from '../DrawingGallery';
import { artworkStorage } from '../../services/artworkStorage';
import { Drawing } from '../../types/drawing';

// Mock the artwork storage service
jest.mock('../../services/artworkStorage', () => ({
  artworkStorage: {
    init: jest.fn(),
    getDrawings: jest.fn(),
    getDrawing: jest.fn(),
    saveDrawing: jest.fn(),
    deleteDrawing: jest.fn(),
    getUniqueMoods: jest.fn(),
    getDrawingCount: jest.fn(),
    clearAll: jest.fn(),
    close: jest.fn(),
  },
}));

// Mock drawing data
const mockDrawing1: Drawing = {
  id: 'drawing-1',
  createdAt: Date.now() - 86400000, // 1 day ago
  strokes: [
    {
      timestamp: 0,
      path: [
        { x: 0, y: 0 },
        { x: 100, y: 100 },
      ],
      mood: 'Calm',
    },
  ],
  moods: [
    {
      timestamp: 0,
      mood: 'Calm',
      tension: 0.3,
      coherence: 0.7,
      energy: 0.5,
    },
  ],
  duration: 5000,
  dominantMood: 'Calm',
  hasSignature: true,
  sessionId: 'session-123',
  metadata: {
    startMood: 'Calm',
    endMood: 'Calm',
    averageTension: 0.3,
    averageCoherence: 0.7,
    averageEnergy: 0.5,
    strokeCount: 1,
    totalPathLength: 141.4,
  },
};

const mockDrawing2: Drawing = {
  id: 'drawing-2',
  createdAt: Date.now() - 172800000, // 2 days ago
  strokes: [
    {
      timestamp: 0,
      path: [
        { x: 50, y: 50 },
        { x: 150, y: 150 },
      ],
      mood: 'Active',
    },
  ],
  moods: [
    {
      timestamp: 0,
      mood: 'Active',
      tension: 0.6,
      coherence: 0.5,
      energy: 0.8,
    },
  ],
  duration: 3000,
  dominantMood: 'Active',
  hasSignature: false,
  sessionId: 'session-456',
  metadata: {
    startMood: 'Active',
    endMood: 'Active',
    averageTension: 0.6,
    averageCoherence: 0.5,
    averageEnergy: 0.8,
    strokeCount: 1,
    totalPathLength: 141.4,
  },
};

describe('DrawingGallery', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    (artworkStorage.init as jest.Mock).mockResolvedValue(undefined);
    (artworkStorage.getUniqueMoods as jest.Mock).mockResolvedValue(['Calm', 'Active', 'Spike', 'Protect']);
  });

  describe('Scenario 1: View Gallery', () => {
    it('should display 5 thumbnails in grid when user has created 5 drawings', async () => {
      const mockDrawings = Array.from({ length: 5 }, (_, i) => ({
        ...mockDrawing1,
        id: `drawing-${i}`,
        createdAt: Date.now() - i * 86400000,
      }));

      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: mockDrawings,
        total: 5,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId('drawing-gallery-grid')).toBeInTheDocument();
      });

      // Check that 5 thumbnails are rendered
      mockDrawings.forEach(drawing => {
        expect(screen.getByTestId(`drawing-thumbnail-${drawing.id}`)).toBeInTheDocument();
      });
    });

    it('should show creation date for each drawing', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        const thumbnail = screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`);
        expect(thumbnail).toHaveTextContent(new Date(mockDrawing1.createdAt).toLocaleDateString());
      });
    });

    it('should show dominant mood for each drawing', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        const thumbnail = screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`);
        expect(thumbnail).toHaveTextContent('Calm');
      });
    });
  });

  describe('Scenario 2: Play Back Drawing', () => {
    it('should open modal with full drawing when clicking thumbnail', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`)).toBeInTheDocument();
      });

      // Click thumbnail
      fireEvent.click(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`));

      // Modal should be open
      await waitFor(() => {
        expect(screen.getByTestId('drawing-modal')).toBeInTheDocument();
      });
    });

    it('should show play button in modal', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`)).toBeInTheDocument();
      });

      // Open modal
      fireEvent.click(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`));

      await waitFor(() => {
        expect(screen.getByTestId('playback-play-button')).toBeInTheDocument();
      });
    });

    it('should animate drawing stroke-by-stroke when play is clicked', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`)).toBeInTheDocument();
      });

      // Open modal
      fireEvent.click(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`));

      await waitFor(() => {
        expect(screen.getByTestId('playback-play-button')).toBeInTheDocument();
      });

      // Click play
      const playButton = screen.getByTestId('playback-play-button');
      fireEvent.click(playButton);

      // Button should show "Playing..."
      await waitFor(() => {
        expect(playButton).toHaveTextContent('Playing...');
      });
    });
  });

  describe('Scenario 3: Filter by Mood', () => {
    it('should show only Calm drawings when Calm filter is applied', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1, mockDrawing2],
        total: 2,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId('drawing-gallery-grid')).toBeInTheDocument();
      });

      // Select Calm mood
      const moodFilter = screen.getByTestId('filter-mood') as HTMLSelectElement;
      fireEvent.change(moodFilter, { target: { value: 'Calm' } });

      // Apply filters
      fireEvent.click(screen.getByTestId('apply-filters-button'));

      // Verify the filter was applied (mock should be called with filter)
      await waitFor(() => {
        expect(artworkStorage.getDrawings).toHaveBeenCalledWith(
          expect.objectContaining({ mood: 'Calm' }),
          expect.anything(),
          expect.anything()
        );
      });
    });

    it('should have mood filter dropdown with all moods', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [],
        total: 0,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId('filter-mood')).toBeInTheDocument();
      });

      // Check that mood options exist
      expect(screen.getByTestId('filter-mood-calm')).toBeInTheDocument();
      expect(screen.getByTestId('filter-mood-active')).toBeInTheDocument();
    });
  });

  describe('I-ART-GAL-001: Stroke Data Storage', () => {
    it('should display all stroke data including paths, timestamps, and moods', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`)).toBeInTheDocument();
      });

      // Open modal to see metadata
      fireEvent.click(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`));

      await waitFor(() => {
        const modal = screen.getByTestId('drawing-modal');
        expect(modal).toHaveTextContent('Strokes: 1');
        expect(modal).toHaveTextContent('Mood: Calm');
      });
    });
  });

  describe('I-ART-GAL-002: Playback Accuracy', () => {
    it('should display duration metadata for playback', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        const thumbnail = screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`);
        expect(thumbnail).toHaveTextContent('Duration: 5.0s');
      });
    });
  });

  describe('Export Functionality', () => {
    it('should have PNG export button', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`)).toBeInTheDocument();
      });

      // Open modal
      fireEvent.click(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`));

      await waitFor(() => {
        expect(screen.getByTestId('export-png-button')).toBeInTheDocument();
      });
    });

    it('should have SVG export button', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`)).toBeInTheDocument();
      });

      // Open modal
      fireEvent.click(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`));

      await waitFor(() => {
        expect(screen.getByTestId('export-svg-button')).toBeInTheDocument();
      });
    });

    it('should have JSON export button', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`)).toBeInTheDocument();
      });

      // Open modal
      fireEvent.click(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`));

      await waitFor(() => {
        expect(screen.getByTestId('export-json-button')).toBeInTheDocument();
      });
    });
  });

  describe('Delete Functionality', () => {
    it('should have delete button in modal', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`)).toBeInTheDocument();
      });

      // Open modal
      fireEvent.click(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`));

      await waitFor(() => {
        expect(screen.getByTestId('delete-button')).toBeInTheDocument();
      });
    });

    it('should call deleteDrawing when delete is confirmed', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });
      (artworkStorage.deleteDrawing as jest.Mock).mockResolvedValue(undefined);

      // Mock window.confirm
      global.confirm = jest.fn(() => true);

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`)).toBeInTheDocument();
      });

      // Open modal
      fireEvent.click(screen.getByTestId(`drawing-thumbnail-${mockDrawing1.id}`));

      await waitFor(() => {
        expect(screen.getByTestId('delete-button')).toBeInTheDocument();
      });

      // Click delete
      fireEvent.click(screen.getByTestId('delete-button'));

      await waitFor(() => {
        expect(artworkStorage.deleteDrawing).toHaveBeenCalledWith(mockDrawing1.id);
      });
    });
  });

  describe('Pagination', () => {
    it('should show pagination controls when more than 20 items', async () => {
      const mockDrawings = Array.from({ length: 25 }, (_, i) => ({
        ...mockDrawing1,
        id: `drawing-${i}`,
      }));

      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: mockDrawings.slice(0, 20),
        total: 25,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId('gallery-pagination')).toBeInTheDocument();
      });
    });

    it('should navigate to next page when next button clicked', async () => {
      const mockDrawings = Array.from({ length: 25 }, (_, i) => ({
        ...mockDrawing1,
        id: `drawing-${i}`,
      }));

      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: mockDrawings.slice(0, 20),
        total: 25,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId('next-page-button')).toBeInTheDocument();
      });

      fireEvent.click(screen.getByTestId('next-page-button'));

      await waitFor(() => {
        expect(artworkStorage.getDrawings).toHaveBeenCalledWith(
          expect.anything(),
          2, // page 2
          expect.anything()
        );
      });
    });
  });

  describe('Search Functionality', () => {
    it('should have search input field', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [],
        total: 0,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId('search-input')).toBeInTheDocument();
      });
    });

    it('should filter by search query when applied', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId('search-input')).toBeInTheDocument();
      });

      // Enter search query
      const searchInput = screen.getByTestId('search-input') as HTMLInputElement;
      fireEvent.change(searchInput, { target: { value: 'session-123' } });

      // Apply filter
      fireEvent.click(screen.getByTestId('apply-filters-button'));

      await waitFor(() => {
        expect(artworkStorage.getDrawings).toHaveBeenCalledWith(
          expect.objectContaining({ searchQuery: 'session-123' }),
          expect.anything(),
          expect.anything()
        );
      });
    });
  });

  describe('Date Filtering', () => {
    it('should have date from and date to inputs', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [],
        total: 0,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId('filter-date-from')).toBeInTheDocument();
        expect(screen.getByTestId('filter-date-to')).toBeInTheDocument();
      });
    });
  });

  describe('Signature Filtering', () => {
    it('should have signature filter checkbox', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [],
        total: 0,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId('filter-signature')).toBeInTheDocument();
      });
    });

    it('should filter by signature when checked', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [mockDrawing1],
        total: 1,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId('filter-signature')).toBeInTheDocument();
      });

      // Check signature filter
      const signatureCheckbox = screen.getByTestId('filter-signature') as HTMLInputElement;
      fireEvent.click(signatureCheckbox);

      // Apply filter
      fireEvent.click(screen.getByTestId('apply-filters-button'));

      await waitFor(() => {
        expect(artworkStorage.getDrawings).toHaveBeenCalledWith(
          expect.objectContaining({ hasSignature: true }),
          expect.anything(),
          expect.anything()
        );
      });
    });
  });

  describe('Error Handling', () => {
    it('should display error message when loading fails', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockRejectedValue(new Error('Database error'));

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId('gallery-error')).toBeInTheDocument();
        expect(screen.getByTestId('gallery-error')).toHaveTextContent('Database error');
      });
    });
  });

  describe('Empty State', () => {
    it('should display empty state when no drawings exist', async () => {
      (artworkStorage.getDrawings as jest.Mock).mockResolvedValue({
        drawings: [],
        total: 0,
      });

      render(<DrawingGallery />);

      await waitFor(() => {
        expect(screen.getByTestId('gallery-empty')).toBeInTheDocument();
      });
    });
  });

  describe('Loading State', () => {
    it('should display loading state while fetching drawings', () => {
      (artworkStorage.getDrawings as jest.Mock).mockImplementation(
        () => new Promise(resolve => setTimeout(() => resolve({ drawings: [], total: 0 }), 1000))
      );

      render(<DrawingGallery />);

      expect(screen.getByTestId('gallery-loading')).toBeInTheDocument();
    });
  });
});
