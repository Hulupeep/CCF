/**
 * Drawing Gallery Type Definitions
 * Contract: ART-001, ART-002, ART-005
 * Journey: J-ART-FIRST-DRAWING
 */

export interface Point {
  x: number;
  y: number;
}

export interface MoodEvent {
  timestamp: number;
  mood: 'Calm' | 'Active' | 'Spike' | 'Protect';
  tension: number;
  coherence: number;
  energy: number;
}

export interface Stroke {
  timestamp: number;
  path: Point[];
  mood: string;
  color?: string;
  width?: number;
}

export interface Drawing {
  id: string;
  createdAt: number;
  strokes: Stroke[];
  moods: MoodEvent[];
  duration: number;
  dominantMood: string;
  hasSignature: boolean;
  sessionId?: string;
  thumbnailData?: string;
  metadata: {
    startMood: string;
    endMood: string;
    averageTension: number;
    averageCoherence: number;
    averageEnergy: number;
    strokeCount: number;
    totalPathLength: number;
  };
}

export interface DrawingFilter {
  mood?: string;
  dateFrom?: number;
  dateTo?: number;
  sessionId?: string;
  searchQuery?: string;
  hasSignature?: boolean;
}

export interface GalleryPagination {
  page: number;
  itemsPerPage: number;
  totalItems: number;
  totalPages: number;
}

export type ExportFormat = 'png' | 'svg' | 'json';

export interface ExportOptions {
  format: ExportFormat;
  width?: number;
  height?: number;
  backgroundColor?: string;
  includeMetadata?: boolean;
}
