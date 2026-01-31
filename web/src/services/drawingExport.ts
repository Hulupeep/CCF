/**
 * Drawing Export Utilities
 * Handles PNG, SVG, and JSON export functionality
 */

import { Drawing, ExportOptions } from '../types/drawing';

/**
 * Generate thumbnail from drawing strokes
 * Contract: I-ART-GAL-001 - Uses stroke data
 */
export function generateThumbnail(
  drawing: Drawing,
  width: number = 200,
  height: number = 200
): string {
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext('2d');

  if (!ctx) {
    throw new Error('Failed to get canvas context');
  }

  // White background
  ctx.fillStyle = '#ffffff';
  ctx.fillRect(0, 0, width, height);

  // Find bounds of drawing
  const bounds = calculateBounds(drawing);
  if (!bounds) {
    return canvas.toDataURL('image/png');
  }

  // Calculate scale to fit thumbnail
  const scaleX = (width * 0.9) / bounds.width;
  const scaleY = (height * 0.9) / bounds.height;
  const scale = Math.min(scaleX, scaleY);

  // Center the drawing
  const offsetX = (width - bounds.width * scale) / 2 - bounds.minX * scale;
  const offsetY = (height - bounds.height * scale) / 2 - bounds.minY * scale;

  // Draw strokes
  drawing.strokes.forEach(stroke => {
    if (stroke.path.length < 2) return;

    ctx.beginPath();
    ctx.strokeStyle = getMoodColor(stroke.mood);
    ctx.lineWidth = (stroke.width || 2) * scale;
    ctx.lineCap = 'round';
    ctx.lineJoin = 'round';

    const firstPoint = stroke.path[0];
    ctx.moveTo(firstPoint.x * scale + offsetX, firstPoint.y * scale + offsetY);

    for (let i = 1; i < stroke.path.length; i++) {
      const point = stroke.path[i];
      ctx.lineTo(point.x * scale + offsetX, point.y * scale + offsetY);
    }

    ctx.stroke();
  });

  return canvas.toDataURL('image/png');
}

/**
 * Export drawing to PNG
 * Contract: I-ART-GAL-002 - Maintains drawing accuracy
 */
export function exportToPNG(drawing: Drawing, options: ExportOptions): string {
  const width = options.width || 800;
  const height = options.height || 600;
  const canvas = document.createElement('canvas');
  canvas.width = width;
  canvas.height = height;
  const ctx = canvas.getContext('2d');

  if (!ctx) {
    throw new Error('Failed to get canvas context');
  }

  // Background
  ctx.fillStyle = options.backgroundColor || '#ffffff';
  ctx.fillRect(0, 0, width, height);

  // Draw strokes
  const bounds = calculateBounds(drawing);
  if (bounds) {
    const scaleX = (width * 0.9) / bounds.width;
    const scaleY = (height * 0.9) / bounds.height;
    const scale = Math.min(scaleX, scaleY);
    const offsetX = (width - bounds.width * scale) / 2 - bounds.minX * scale;
    const offsetY = (height - bounds.height * scale) / 2 - bounds.minY * scale;

    drawing.strokes.forEach(stroke => {
      if (stroke.path.length < 2) return;

      ctx.beginPath();
      ctx.strokeStyle = stroke.color || getMoodColor(stroke.mood);
      ctx.lineWidth = (stroke.width || 2) * scale;
      ctx.lineCap = 'round';
      ctx.lineJoin = 'round';

      const firstPoint = stroke.path[0];
      ctx.moveTo(firstPoint.x * scale + offsetX, firstPoint.y * scale + offsetY);

      for (let i = 1; i < stroke.path.length; i++) {
        const point = stroke.path[i];
        ctx.lineTo(point.x * scale + offsetX, point.y * scale + offsetY);
      }

      ctx.stroke();
    });
  }

  // Add metadata if requested
  if (options.includeMetadata) {
    ctx.fillStyle = '#000000';
    ctx.font = '12px monospace';
    ctx.fillText(`Created: ${new Date(drawing.createdAt).toLocaleString()}`, 10, height - 30);
    ctx.fillText(`Mood: ${drawing.dominantMood} | Duration: ${drawing.duration}ms`, 10, height - 15);
  }

  return canvas.toDataURL('image/png');
}

/**
 * Export drawing to SVG
 * Contract: I-ART-GAL-002 - Vector format maintains accuracy
 */
export function exportToSVG(drawing: Drawing, options: ExportOptions): string {
  const width = options.width || 800;
  const height = options.height || 600;
  const bounds = calculateBounds(drawing);

  let svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">`;

  // Background
  svg += `<rect width="${width}" height="${height}" fill="${options.backgroundColor || '#ffffff'}"/>`;

  if (bounds) {
    const scaleX = (width * 0.9) / bounds.width;
    const scaleY = (height * 0.9) / bounds.height;
    const scale = Math.min(scaleX, scaleY);
    const offsetX = (width - bounds.width * scale) / 2 - bounds.minX * scale;
    const offsetY = (height - bounds.height * scale) / 2 - bounds.minY * scale;

    // Draw strokes as paths
    drawing.strokes.forEach(stroke => {
      if (stroke.path.length < 2) return;

      const pathData = stroke.path
        .map((point, index) => {
          const x = point.x * scale + offsetX;
          const y = point.y * scale + offsetY;
          return index === 0 ? `M ${x} ${y}` : `L ${x} ${y}`;
        })
        .join(' ');

      const color = stroke.color || getMoodColor(stroke.mood);
      const width = (stroke.width || 2) * scale;

      svg += `<path d="${pathData}" stroke="${color}" stroke-width="${width}" fill="none" stroke-linecap="round" stroke-linejoin="round"/>`;
    });
  }

  // Add metadata if requested
  if (options.includeMetadata) {
    svg += `<text x="10" y="${height - 30}" font-family="monospace" font-size="12" fill="#000000">Created: ${new Date(drawing.createdAt).toLocaleString()}</text>`;
    svg += `<text x="10" y="${height - 15}" font-family="monospace" font-size="12" fill="#000000">Mood: ${drawing.dominantMood} | Duration: ${drawing.duration}ms</text>`;
  }

  svg += '</svg>';
  return svg;
}

/**
 * Export drawing to JSON
 */
export function exportToJSON(drawing: Drawing): string {
  return JSON.stringify(drawing, null, 2);
}

/**
 * Calculate bounding box of all strokes
 */
function calculateBounds(drawing: Drawing): {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
  width: number;
  height: number;
} | null {
  if (drawing.strokes.length === 0) return null;

  let minX = Infinity;
  let minY = Infinity;
  let maxX = -Infinity;
  let maxY = -Infinity;

  drawing.strokes.forEach(stroke => {
    stroke.path.forEach(point => {
      minX = Math.min(minX, point.x);
      minY = Math.min(minY, point.y);
      maxX = Math.max(maxX, point.x);
      maxY = Math.max(maxY, point.y);
    });
  });

  return {
    minX,
    minY,
    maxX,
    maxY,
    width: maxX - minX,
    height: maxY - minY,
  };
}

/**
 * Get color for mood
 */
function getMoodColor(mood: string): string {
  const moodColors: Record<string, string> = {
    Calm: '#4A90E2',
    Active: '#F5A623',
    Spike: '#E24A4A',
    Protect: '#7B68EE',
  };

  return moodColors[mood] || '#000000';
}

/**
 * Download file helper
 */
export function downloadFile(content: string, filename: string, mimeType: string): void {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}

/**
 * Download PNG image
 */
export function downloadPNG(dataUrl: string, filename: string): void {
  const link = document.createElement('a');
  link.href = dataUrl;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}
