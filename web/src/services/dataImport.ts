/**
 * Data Import Service
 * Contract: ARCH-005, LEARN-007
 * Journey: Data Export/Import System
 * Handles importing and validating personalities, drawings, stats, and inventory
 */

import type {
  ExportManifest,
  ImportOptions,
  ImportResult,
  DataType,
  ValidationError,
  BackupInfo,
} from '../types/exportManifest';
import type { PersonalityConfig } from '../types/personality';
import type { Drawing } from '../types/drawing';
import type { GameStatistics } from '../types/game';
import type { InventoryExport } from '../types/inventory';
import {
  validateExportManifest,
  validatePersonalityData,
  validateDrawingData,
  validateGameStatsData,
  validateInventoryData,
  generateBackupFilename,
} from '../types/exportManifest';

/**
 * Imports data from JSON file
 */
export async function importFromJSON(
  file: File,
  options: ImportOptions
): Promise<ImportResult> {
  const result: ImportResult = {
    success: false,
    dataTypes: [],
    itemsImported: 0,
    errors: [],
    warnings: [],
    backupCreated: false,
  };

  try {
    // Read file content
    const content = await file.text();
    const data = JSON.parse(content);

    // Validate manifest structure
    if (!validateExportManifest(data)) {
      result.errors.push('Invalid export manifest structure');
      return result;
    }

    const manifest = data as ExportManifest;

    // Create backup if requested
    if (options.createBackup) {
      const backup = await createBackup(manifest.dataTypes);
      result.backupCreated = backup.success;
      result.backupPath = backup.path;
      if (!backup.success) {
        result.warnings.push('Failed to create backup');
      }
    }

    // Validate and import each data type
    for (const dataType of manifest.dataTypes) {
      try {
        const importCount = await importDataType(
          dataType,
          manifest.data,
          options
        );
        result.dataTypes.push(dataType);
        result.itemsImported += importCount;
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        result.errors.push(`Failed to import ${dataType}: ${message}`);
        if (!options.skipInvalid) {
          return result;
        }
      }
    }

    result.success = result.errors.length === 0 || options.skipInvalid;
    return result;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    result.errors.push(`Import failed: ${message}`);
    return result;
  }
}

/**
 * Creates a backup of current data
 */
async function createBackup(dataTypes: DataType[]): Promise<{
  success: boolean;
  path?: string;
}> {
  try {
    const backup: Record<string, any> = {
      timestamp: Date.now(),
      dataTypes,
    };

    // Collect current data
    for (const dataType of dataTypes) {
      switch (dataType) {
        case 'personality':
          backup.personalities = localStorage.getItem('mbot_custom_personalities');
          break;
        case 'drawings':
          backup.drawings = localStorage.getItem('mbot_drawings');
          break;
        case 'stats':
          backup.stats = localStorage.getItem('mbot_game_statistics');
          break;
        case 'inventory':
          backup.inventory = localStorage.getItem('mbot_inventory');
          break;
      }
    }

    // Store backup in localStorage
    const filename = generateBackupFilename(dataTypes);
    localStorage.setItem(`mbot_backup_${backup.timestamp}`, JSON.stringify(backup));

    // Store backup metadata
    const backups = getBackupList();
    backups.push({
      timestamp: backup.timestamp,
      path: filename,
      dataTypes,
      size: JSON.stringify(backup).length,
    });
    localStorage.setItem('mbot_backups', JSON.stringify(backups));

    return { success: true, path: filename };
  } catch (error) {
    console.error('Failed to create backup:', error);
    return { success: false };
  }
}

/**
 * Gets list of available backups
 */
export function getBackupList(): BackupInfo[] {
  try {
    const stored = localStorage.getItem('mbot_backups');
    if (!stored) return [];
    return JSON.parse(stored);
  } catch {
    return [];
  }
}

/**
 * Restores data from a backup
 */
export async function restoreFromBackup(timestamp: number): Promise<ImportResult> {
  const result: ImportResult = {
    success: false,
    dataTypes: [],
    itemsImported: 0,
    errors: [],
    warnings: [],
    backupCreated: false,
  };

  try {
    const backupKey = `mbot_backup_${timestamp}`;
    const stored = localStorage.getItem(backupKey);
    if (!stored) {
      result.errors.push('Backup not found');
      return result;
    }

    const backup = JSON.parse(stored);

    // Restore each data type
    for (const dataType of backup.dataTypes) {
      try {
        switch (dataType) {
          case 'personality':
            if (backup.personalities) {
              localStorage.setItem('mbot_custom_personalities', backup.personalities);
              result.itemsImported++;
            }
            break;
          case 'drawings':
            if (backup.drawings) {
              localStorage.setItem('mbot_drawings', backup.drawings);
              result.itemsImported++;
            }
            break;
          case 'stats':
            if (backup.stats) {
              localStorage.setItem('mbot_game_statistics', backup.stats);
              result.itemsImported++;
            }
            break;
          case 'inventory':
            if (backup.inventory) {
              localStorage.setItem('mbot_inventory', backup.inventory);
              result.itemsImported++;
            }
            break;
        }
        result.dataTypes.push(dataType);
      } catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        result.errors.push(`Failed to restore ${dataType}: ${message}`);
      }
    }

    result.success = result.errors.length === 0;
    return result;
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    result.errors.push(`Restore failed: ${message}`);
    return result;
  }
}

/**
 * Imports a specific data type
 */
async function importDataType(
  dataType: DataType,
  data: ExportManifest['data'],
  options: ImportOptions
): Promise<number> {
  switch (dataType) {
    case 'personality':
      return importPersonalities(data.personalities || [], options);
    case 'drawings':
      return importDrawings(data.drawings || [], options);
    case 'stats':
      return importGameStats(data.stats, options);
    case 'inventory':
      return importInventory(data.inventory, options);
    default:
      throw new Error(`Unknown data type: ${dataType}`);
  }
}

/**
 * Imports personalities
 */
function importPersonalities(
  personalities: PersonalityConfig[],
  options: ImportOptions
): number {
  if (options.validateSchema) {
    const errors: ValidationError[] = [];
    for (const config of personalities) {
      const configErrors = validatePersonalityData(config);
      errors.push(...configErrors);
    }
    if (errors.length > 0 && !options.skipInvalid) {
      throw new Error(
        `Personality validation failed: ${errors.map(e => e.message).join(', ')}`
      );
    }
  }

  // Load existing personalities
  const stored = localStorage.getItem('mbot_custom_personalities');
  let existing = stored ? JSON.parse(stored) : [];

  if (options.overwriteExisting) {
    existing = [];
  }

  // Add new personalities
  const timestamp = Date.now();
  for (const config of personalities) {
    existing.push({
      name: 'Imported Personality',
      config,
      created_at: timestamp,
    });
  }

  localStorage.setItem('mbot_custom_personalities', JSON.stringify(existing));
  return personalities.length;
}

/**
 * Imports drawings
 */
function importDrawings(drawings: Drawing[], options: ImportOptions): number {
  if (options.validateSchema) {
    const errors: ValidationError[] = [];
    for (const drawing of drawings) {
      const drawingErrors = validateDrawingData(drawing);
      errors.push(...drawingErrors);
    }
    if (errors.length > 0 && !options.skipInvalid) {
      throw new Error(
        `Drawing validation failed: ${errors.map(e => e.message).join(', ')}`
      );
    }
  }

  // Load existing drawings
  const stored = localStorage.getItem('mbot_drawings');
  let existing = stored ? JSON.parse(stored) : [];

  if (options.overwriteExisting) {
    existing = [];
  }

  // Add new drawings (avoid duplicates by ID)
  for (const drawing of drawings) {
    const existingIndex = existing.findIndex((d: Drawing) => d.id === drawing.id);
    if (existingIndex >= 0) {
      if (options.overwriteExisting) {
        existing[existingIndex] = drawing;
      }
    } else {
      existing.push(drawing);
    }
  }

  localStorage.setItem('mbot_drawings', JSON.stringify(existing));
  return drawings.length;
}

/**
 * Imports game statistics
 */
function importGameStats(
  stats: GameStatistics | null | undefined,
  options: ImportOptions
): number {
  if (!stats) return 0;

  if (options.validateSchema) {
    const errors = validateGameStatsData(stats);
    if (errors.length > 0 && !options.skipInvalid) {
      throw new Error(
        `Game stats validation failed: ${errors.map(e => e.message).join(', ')}`
      );
    }
  }

  // Load existing stats
  const stored = localStorage.getItem('mbot_game_statistics');
  let existing = stored ? JSON.parse(stored) : null;

  if (options.overwriteExisting || !existing) {
    // Replace entirely
    localStorage.setItem('mbot_game_statistics', JSON.stringify(stats));
  } else {
    // Merge stats
    const merged = mergeGameStats(existing, stats);
    localStorage.setItem('mbot_game_statistics', JSON.stringify(merged));
  }

  return 1;
}

/**
 * Merges game statistics
 */
function mergeGameStats(
  existing: GameStatistics,
  imported: GameStatistics
): GameStatistics {
  return {
    totalGames: existing.totalGames + imported.totalGames,
    byGame: {
      ...existing.byGame,
      ...imported.byGame,
    },
    achievements: [...existing.achievements, ...imported.achievements],
    leaderboard: [...existing.leaderboard, ...imported.leaderboard],
    sessions: [...existing.sessions, ...imported.sessions],
  };
}

/**
 * Imports inventory
 */
function importInventory(
  inventory: InventoryExport | null | undefined,
  options: ImportOptions
): number {
  if (!inventory) return 0;

  if (options.validateSchema) {
    const errors = validateInventoryData(inventory);
    if (errors.length > 0 && !options.skipInvalid) {
      throw new Error(
        `Inventory validation failed: ${errors.map(e => e.message).join(', ')}`
      );
    }
  }

  localStorage.setItem('mbot_inventory', JSON.stringify(inventory));
  return 1;
}

/**
 * Deletes a backup
 */
export function deleteBackup(timestamp: number): boolean {
  try {
    const backupKey = `mbot_backup_${timestamp}`;
    localStorage.removeItem(backupKey);

    // Update backup list
    const backups = getBackupList();
    const filtered = backups.filter(b => b.timestamp !== timestamp);
    localStorage.setItem('mbot_backups', JSON.stringify(filtered));

    return true;
  } catch {
    return false;
  }
}

/**
 * Validates import file before processing
 */
export async function validateImportFile(file: File): Promise<{
  valid: boolean;
  errors: string[];
  warnings: string[];
  manifest?: ExportManifest;
}> {
  const result = {
    valid: false,
    errors: [] as string[],
    warnings: [] as string[],
  };

  try {
    // Check file type
    if (!file.name.endsWith('.json')) {
      result.errors.push('Only JSON files are supported for import');
      return result;
    }

    // Parse content
    const content = await file.text();
    const data = JSON.parse(content);

    // Validate manifest
    if (!validateExportManifest(data)) {
      result.errors.push('Invalid export manifest structure');
      return result;
    }

    const manifest = data as ExportManifest;

    // Check version compatibility
    const [major] = manifest.version.split('.');
    const [currentMajor] = '1.0.0'.split('.');
    if (major !== currentMajor) {
      result.warnings.push(
        `Version mismatch: file is v${manifest.version}, current is v1.0.0`
      );
    }

    // Validate each data type
    for (const dataType of manifest.dataTypes) {
      let errors: ValidationError[] = [];
      switch (dataType) {
        case 'personality':
          if (manifest.data.personalities) {
            for (const config of manifest.data.personalities) {
              errors.push(...validatePersonalityData(config));
            }
          }
          break;
        case 'drawings':
          if (manifest.data.drawings) {
            for (const drawing of manifest.data.drawings) {
              errors.push(...validateDrawingData(drawing));
            }
          }
          break;
        case 'stats':
          if (manifest.data.stats) {
            errors.push(...validateGameStatsData(manifest.data.stats));
          }
          break;
        case 'inventory':
          if (manifest.data.inventory) {
            errors.push(...validateInventoryData(manifest.data.inventory));
          }
          break;
      }

      if (errors.length > 0) {
        result.warnings.push(
          `${dataType}: ${errors.length} validation issue(s)`
        );
      }
    }

    result.valid = result.errors.length === 0;
    return { ...result, manifest };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    result.errors.push(`Failed to validate file: ${message}`);
    return result;
  }
}
