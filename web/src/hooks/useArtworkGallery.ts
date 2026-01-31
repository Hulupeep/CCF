/**
 * React Hook for Artwork Gallery Management
 * Provides state management and operations for the drawing gallery
 */

import { useState, useEffect, useCallback } from 'react';
import { Drawing, DrawingFilter, GalleryPagination } from '../types/drawing';
import { artworkStorage } from '../services/artworkStorage';

interface UseArtworkGalleryResult {
  drawings: Drawing[];
  loading: boolean;
  error: string | null;
  pagination: GalleryPagination;
  uniqueMoods: string[];
  filter: DrawingFilter;
  setFilter: (filter: DrawingFilter) => void;
  setPage: (page: number) => void;
  deleteDrawing: (id: string) => Promise<void>;
  refreshGallery: () => Promise<void>;
}

export const useArtworkGallery = (itemsPerPage: number = 20): UseArtworkGalleryResult => {
  const [drawings, setDrawings] = useState<Drawing[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [pagination, setPagination] = useState<GalleryPagination>({
    page: 1,
    itemsPerPage,
    totalItems: 0,
    totalPages: 0,
  });
  const [uniqueMoods, setUniqueMoods] = useState<string[]>([]);
  const [filter, setFilter] = useState<DrawingFilter>({});

  /**
   * Load drawings from IndexedDB
   */
  const loadDrawings = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      await artworkStorage.init();

      const { drawings: loadedDrawings, total } = await artworkStorage.getDrawings(
        filter,
        pagination.page,
        itemsPerPage
      );

      setDrawings(loadedDrawings);
      setPagination(prev => ({
        ...prev,
        totalItems: total,
        totalPages: Math.ceil(total / itemsPerPage),
      }));

      // Load unique moods for filter dropdown
      const moods = await artworkStorage.getUniqueMoods();
      setUniqueMoods(moods);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load drawings');
    } finally {
      setLoading(false);
    }
  }, [filter, pagination.page, itemsPerPage]);

  /**
   * Initialize and load drawings on mount
   */
  useEffect(() => {
    loadDrawings();
  }, [loadDrawings]);

  /**
   * Delete a drawing
   */
  const deleteDrawing = useCallback(async (id: string) => {
    try {
      await artworkStorage.deleteDrawing(id);
      await loadDrawings(); // Refresh after deletion
    } catch (err) {
      throw new Error(err instanceof Error ? err.message : 'Failed to delete drawing');
    }
  }, [loadDrawings]);

  /**
   * Set the current page
   */
  const setPage = useCallback((page: number) => {
    setPagination(prev => ({ ...prev, page }));
  }, []);

  /**
   * Refresh the gallery
   */
  const refreshGallery = useCallback(async () => {
    await loadDrawings();
  }, [loadDrawings]);

  return {
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
  };
};
