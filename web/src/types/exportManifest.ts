/**
 * Export Manifest Type Definitions
 * Contract: ARCH-005, LEARN-007
 * Journey: Data Export/Import System
 * Invariant: I-ARCH-009 - Data must be versioned and validated
 */

import { PersonalityConfig } from './personality';
import { Drawing } from './drawing';
import { GameStatistics } from './game';
import { InventoryExport } from './inventory';

export const EXPORT_VERSION = '1.0.0';

export type DataType = 'personality' | 'drawings' | 'stats' | 'inventory';

export interface ExportManifest {
  version: string;
  exportedAt: number;
  dataTypes: DataType[];
  metadata: {
    deviceId?: string;
    userName?: string;
    exportReason?: string;
  };
  data: {
    personalities?: PersonalityConfig[];
    drawings?: Drawing[];
    stats?: GameStatistics;
    inventory?: InventoryExport;
  };
}

export interface ImportResult {
  success: boolean;
  dataTypes: DataType[];
  itemsImported: number;
  errors: string[];
  warnings: string[];
  backupCreated: boolean;
  backupPath?: string;
}

export interface ExportOptions {
  dataTypes: DataType[];
  format: 'json' | 'csv';
  includeMetadata?: boolean;
  deviceId?: string;
  userName?: string;
  exportReason?: string;
}

export interface ImportOptions {
  createBackup: boolean;
  validateSchema: boolean;
  overwriteExisting: boolean;
  skipInvalid: boolean;
}

export interface ValidationError {
  dataType: DataType;
  field: string;
  message: string;
  value?: any;
}

export interface BackupInfo {
  timestamp: number;
  path: string;
  dataTypes: DataType[];
  size: number;
}

/**
 * Validates export manifest structure
 */
export function validateExportManifest(data: any): data is ExportManifest {
  if (!data || typeof data !== 'object') {
    return false;
  }

  // Check required fields
  if (
    typeof data.version !== 'string' ||
    typeof data.exportedAt !== 'number' ||
    !Array.isArray(data.dataTypes) ||
    typeof data.data !== 'object'
  ) {
    return false;
  }

  // Check version compatibility
  const [major] = data.version.split('.');
  const [currentMajor] = EXPORT_VERSION.split('.');
  if (major !== currentMajor) {
    return false;
  }

  // Check data types are valid
  const validTypes: DataType[] = ['personality', 'drawings', 'stats', 'inventory'];
  if (!data.dataTypes.every((type: any) => validTypes.includes(type))) {
    return false;
  }

  // Check data object has keys matching dataTypes
  for (const type of data.dataTypes) {
    const dataKey = type === 'personality' ? 'personalities' : type;
    if (!(dataKey in data.data)) {
      return false;
    }
  }

  return true;
}

/**
 * Validates personality data
 */
export function validatePersonalityData(config: any): ValidationError[] {
  const errors: ValidationError[] = [];

  const requiredKeys = [
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

  for (const key of requiredKeys) {
    if (!(key in config)) {
      errors.push({
        dataType: 'personality',
        field: key,
        message: `Missing required field: ${key}`,
      });
    } else if (typeof config[key] !== 'number') {
      errors.push({
        dataType: 'personality',
        field: key,
        message: `Field must be a number`,
        value: config[key],
      });
    } else if (config[key] < 0 || config[key] > 1) {
      errors.push({
        dataType: 'personality',
        field: key,
        message: `Value must be between 0.0 and 1.0 (I-PERS-001)`,
        value: config[key],
      });
    }
  }

  return errors;
}

/**
 * Validates drawing data
 */
export function validateDrawingData(drawing: any): ValidationError[] {
  const errors: ValidationError[] = [];

  if (!drawing.id || typeof drawing.id !== 'string') {
    errors.push({
      dataType: 'drawings',
      field: 'id',
      message: 'Missing or invalid drawing ID',
    });
  }

  if (!drawing.createdAt || typeof drawing.createdAt !== 'number') {
    errors.push({
      dataType: 'drawings',
      field: 'createdAt',
      message: 'Missing or invalid createdAt timestamp',
    });
  }

  if (!Array.isArray(drawing.strokes)) {
    errors.push({
      dataType: 'drawings',
      field: 'strokes',
      message: 'Strokes must be an array',
    });
  }

  if (!drawing.metadata || typeof drawing.metadata !== 'object') {
    errors.push({
      dataType: 'drawings',
      field: 'metadata',
      message: 'Missing or invalid metadata',
    });
  }

  return errors;
}

/**
 * Validates game statistics data
 */
export function validateGameStatsData(stats: any): ValidationError[] {
  const errors: ValidationError[] = [];

  if (typeof stats.totalGames !== 'number') {
    errors.push({
      dataType: 'stats',
      field: 'totalGames',
      message: 'Missing or invalid totalGames',
    });
  }

  if (!stats.byGame || typeof stats.byGame !== 'object') {
    errors.push({
      dataType: 'stats',
      field: 'byGame',
      message: 'Missing or invalid byGame object',
    });
  }

  if (!Array.isArray(stats.sessions)) {
    errors.push({
      dataType: 'stats',
      field: 'sessions',
      message: 'Sessions must be an array',
    });
  }

  return errors;
}

/**
 * Validates inventory data
 */
export function validateInventoryData(inventory: any): ValidationError[] {
  const errors: ValidationError[] = [];

  if (inventory.version !== '1.0') {
    errors.push({
      dataType: 'inventory',
      field: 'version',
      message: `Unsupported inventory version: ${inventory.version}`,
    });
  }

  if (!Array.isArray(inventory.stations) || inventory.stations.length !== 4) {
    errors.push({
      dataType: 'inventory',
      field: 'stations',
      message: 'Inventory must have exactly 4 stations',
    });
  }

  const requiredStations = ['red', 'green', 'blue', 'yellow'];
  const stationIds = inventory.stations?.map((s: any) => s.id) || [];
  for (const stationId of requiredStations) {
    if (!stationIds.includes(stationId)) {
      errors.push({
        dataType: 'inventory',
        field: 'stations',
        message: `Missing station: ${stationId}`,
      });
    }
  }

  return errors;
}

/**
 * Creates a default export options object
 */
export function createDefaultExportOptions(): ExportOptions {
  return {
    dataTypes: ['personality', 'drawings', 'stats', 'inventory'],
    format: 'json',
    includeMetadata: true,
  };
}

/**
 * Creates a default import options object
 */
export function createDefaultImportOptions(): ImportOptions {
  return {
    createBackup: true,
    validateSchema: true,
    overwriteExisting: false,
    skipInvalid: true,
  };
}

/**
 * Generates a backup filename with timestamp
 */
export function generateBackupFilename(dataTypes: DataType[]): string {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
  const types = dataTypes.join('-');
  return `mbot-backup-${types}-${timestamp}.json`;
}

/**
 * Generates an export filename with timestamp
 */
export function generateExportFilename(dataTypes: DataType[], format: 'json' | 'csv'): string {
  const timestamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, -5);
  const types = dataTypes.join('-');
  return `mbot-export-${types}-${timestamp}.${format}`;
}
