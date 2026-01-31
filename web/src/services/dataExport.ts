/**
 * Data Export Service
 * Contract: ARCH-005, LEARN-007
 * Journey: Data Export/Import System
 * Handles exporting personalities, drawings, stats, and inventory
 */

import type {
  ExportManifest,
  ExportOptions,
  DataType,
} from '../types/exportManifest';
import type { PersonalityConfig } from '../types/personality';
import type { Drawing } from '../types/drawing';
import type { GameStatistics } from '../types/game';
import type { InventoryExport } from '../types/inventory';
import {
  EXPORT_VERSION,
  generateExportFilename,
} from '../types/exportManifest';

/**
 * Exports data to JSON format
 */
export async function exportToJSON(options: ExportOptions): Promise<Blob> {
  const manifest = await buildExportManifest(options);
  const json = JSON.stringify(manifest, null, 2);
  return new Blob([json], { type: 'application/json' });
}

/**
 * Exports data to CSV format (for single data types)
 */
export async function exportToCSV(options: ExportOptions): Promise<Blob> {
  if (options.dataTypes.length !== 1) {
    throw new Error('CSV export only supports single data type at a time');
  }

  const dataType = options.dataTypes[0];
  const data = await loadDataForType(dataType);

  let csv: string;
  switch (dataType) {
    case 'personality':
      csv = convertPersonalitiesToCSV(data as PersonalityConfig[]);
      break;
    case 'drawings':
      csv = convertDrawingsToCSV(data as Drawing[]);
      break;
    case 'stats':
      csv = convertStatsToCSV(data as GameStatistics);
      break;
    case 'inventory':
      csv = convertInventoryToCSV(data as InventoryExport);
      break;
    default:
      throw new Error(`Unsupported data type for CSV: ${dataType}`);
  }

  return new Blob([csv], { type: 'text/csv' });
}

/**
 * Builds export manifest from selected data types
 */
async function buildExportManifest(options: ExportOptions): Promise<ExportManifest> {
  const manifest: ExportManifest = {
    version: EXPORT_VERSION,
    exportedAt: Date.now(),
    dataTypes: options.dataTypes,
    metadata: {
      deviceId: options.deviceId,
      userName: options.userName,
      exportReason: options.exportReason,
    },
    data: {},
  };

  // Load data for each selected type
  for (const dataType of options.dataTypes) {
    const data = await loadDataForType(dataType);
    switch (dataType) {
      case 'personality':
        manifest.data.personalities = data as PersonalityConfig[];
        break;
      case 'drawings':
        manifest.data.drawings = data as Drawing[];
        break;
      case 'stats':
        manifest.data.stats = data as GameStatistics;
        break;
      case 'inventory':
        manifest.data.inventory = data as InventoryExport;
        break;
    }
  }

  return manifest;
}

/**
 * Loads data for a specific type from localStorage
 */
async function loadDataForType(dataType: DataType): Promise<any> {
  switch (dataType) {
    case 'personality':
      return loadPersonalities();
    case 'drawings':
      return loadDrawings();
    case 'stats':
      return loadGameStats();
    case 'inventory':
      return loadInventory();
    default:
      throw new Error(`Unknown data type: ${dataType}`);
  }
}

/**
 * Loads personalities from localStorage
 */
function loadPersonalities(): PersonalityConfig[] {
  const stored = localStorage.getItem('mbot_custom_personalities');
  if (!stored) return [];
  try {
    const personalities = JSON.parse(stored);
    return Array.isArray(personalities) ? personalities.map((p: any) => p.config) : [];
  } catch {
    return [];
  }
}

/**
 * Loads drawings from localStorage
 */
function loadDrawings(): Drawing[] {
  const stored = localStorage.getItem('mbot_drawings');
  if (!stored) return [];
  try {
    const drawings = JSON.parse(stored);
    return Array.isArray(drawings) ? drawings : [];
  } catch {
    return [];
  }
}

/**
 * Loads game statistics from localStorage
 */
function loadGameStats(): GameStatistics | null {
  const stored = localStorage.getItem('mbot_game_statistics');
  if (!stored) return null;
  try {
    return JSON.parse(stored);
  } catch {
    return null;
  }
}

/**
 * Loads inventory from localStorage
 */
function loadInventory(): InventoryExport | null {
  const stored = localStorage.getItem('mbot_inventory');
  if (!stored) return null;
  try {
    return JSON.parse(stored);
  } catch {
    return null;
  }
}

/**
 * Converts personalities to CSV format
 */
function convertPersonalitiesToCSV(personalities: PersonalityConfig[]): string {
  const headers = [
    'tension_baseline',
    'coherence_baseline',
    'energy_baseline',
    'startle_sensitivity',
    'recovery_speed',
    'curiosity_drive',
    'movement_expressiveness',
    'sound_expressiveness',
    'light_expressiveness',
  ];

  const rows = personalities.map(p =>
    headers.map(key => p[key as keyof PersonalityConfig]).join(',')
  );

  return [headers.join(','), ...rows].join('\n');
}

/**
 * Converts drawings to CSV format (metadata only)
 */
function convertDrawingsToCSV(drawings: Drawing[]): string {
  const headers = [
    'id',
    'createdAt',
    'duration',
    'dominantMood',
    'hasSignature',
    'strokeCount',
    'totalPathLength',
    'averageTension',
    'averageCoherence',
    'averageEnergy',
  ];

  const rows = drawings.map(d => [
    d.id,
    d.createdAt,
    d.duration,
    d.dominantMood,
    d.hasSignature,
    d.metadata.strokeCount,
    d.metadata.totalPathLength,
    d.metadata.averageTension,
    d.metadata.averageCoherence,
    d.metadata.averageEnergy,
  ].join(','));

  return [headers.join(','), ...rows].join('\n');
}

/**
 * Converts game stats to CSV format
 */
function convertStatsToCSV(stats: GameStatistics): string {
  const headers = [
    'gameType',
    'totalGames',
    'wins',
    'losses',
    'draws',
    'highScore',
    'averageScore',
  ];

  const rows = Object.entries(stats.byGame).map(([gameType, gameStats]) => [
    gameType,
    gameStats.totalGames,
    gameStats.wins,
    gameStats.losses,
    gameStats.draws,
    gameStats.highScore,
    gameStats.averageScore.toFixed(2),
  ].join(','));

  return [headers.join(','), ...rows].join('\n');
}

/**
 * Converts inventory to CSV format
 */
function convertInventoryToCSV(inventory: InventoryExport): string {
  const headers = ['stationId', 'color', 'count', 'capacity', 'lastUpdated'];

  const rows = inventory.stations.map(s => [
    s.id,
    s.color,
    s.count,
    s.capacity,
    s.lastUpdated,
  ].join(','));

  return [headers.join(','), ...rows].join('\n');
}

/**
 * Downloads a blob as a file
 */
export function downloadBlob(blob: Blob, filename: string): void {
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
 * Exports data and triggers download
 */
export async function exportData(options: ExportOptions): Promise<void> {
  const blob = options.format === 'json'
    ? await exportToJSON(options)
    : await exportToCSV(options);

  const filename = generateExportFilename(options.dataTypes, options.format);
  downloadBlob(blob, filename);
}

/**
 * Exports all data types
 */
export async function exportAllData(
  format: 'json' | 'csv' = 'json',
  metadata?: { deviceId?: string; userName?: string; exportReason?: string }
): Promise<void> {
  const options: ExportOptions = {
    dataTypes: ['personality', 'drawings', 'stats', 'inventory'],
    format,
    includeMetadata: true,
    ...metadata,
  };

  await exportData(options);
}

/**
 * Exports a single data type
 */
export async function exportSingleDataType(
  dataType: DataType,
  format: 'json' | 'csv' = 'json'
): Promise<void> {
  const options: ExportOptions = {
    dataTypes: [dataType],
    format,
    includeMetadata: true,
  };

  await exportData(options);
}
