/**
 * Inventory Storage Service
 * Contract: SORT-004 (Inventory Must Persist)
 * Implements offline-first local storage with history tracking
 */

import {
  Station,
  StationId,
  InventorySnapshot,
  HistoricalData,
  InventoryExport,
  ManualAdjustment,
  createDefaultStation,
  validateInventoryData,
} from '../types/inventory';

const STORAGE_KEY_STATIONS = 'mbot_inventory_stations';
const STORAGE_KEY_HISTORY = 'mbot_inventory_history';
const STORAGE_KEY_ADJUSTMENTS = 'mbot_inventory_adjustments';
const MAX_HISTORY_DAYS = 30;
const MAX_HISTORY_WEEKS = 12;

/**
 * Loads stations from localStorage or creates defaults
 */
export function loadStations(): Record<StationId, Station> {
  try {
    const data = localStorage.getItem(STORAGE_KEY_STATIONS);
    if (data) {
      const parsed = JSON.parse(data);
      return {
        red: parsed.red || createDefaultStation('red'),
        green: parsed.green || createDefaultStation('green'),
        blue: parsed.blue || createDefaultStation('blue'),
        yellow: parsed.yellow || createDefaultStation('yellow'),
      };
    }
  } catch (err) {
    console.error('Failed to load stations from localStorage:', err);
  }

  // Return defaults
  return {
    red: createDefaultStation('red'),
    green: createDefaultStation('green'),
    blue: createDefaultStation('blue'),
    yellow: createDefaultStation('yellow'),
  };
}

/**
 * Saves stations to localStorage
 */
export function saveStations(stations: Record<StationId, Station>): void {
  try {
    localStorage.setItem(STORAGE_KEY_STATIONS, JSON.stringify(stations));
  } catch (err) {
    console.error('Failed to save stations to localStorage:', err);
  }
}

/**
 * Updates a single station's count
 */
export function updateStationCount(
  stations: Record<StationId, Station>,
  stationId: StationId,
  count: number,
  source: 'robot' | 'nfc' | 'manual' = 'robot'
): Record<StationId, Station> {
  const updated = {
    ...stations,
    [stationId]: {
      ...stations[stationId],
      count: Math.max(0, count), // Ensure non-negative
      lastUpdated: Date.now(),
    },
  };

  saveStations(updated);
  addHistorySnapshot(updated);

  return updated;
}

/**
 * Resets a station to zero
 */
export function resetStation(
  stations: Record<StationId, Station>,
  stationId: StationId
): Record<StationId, Station> {
  return updateStationCount(stations, stationId, 0, 'manual');
}

/**
 * Resets all stations to zero
 */
export function resetAllStations(stations: Record<StationId, Station>): Record<StationId, Station> {
  const updated = Object.keys(stations).reduce((acc, id) => {
    const stationId = id as StationId;
    acc[stationId] = {
      ...stations[stationId],
      count: 0,
      lastUpdated: Date.now(),
    };
    return acc;
  }, {} as Record<StationId, Station>);

  saveStations(updated);
  addHistorySnapshot(updated);

  return updated;
}

/**
 * Loads historical data from localStorage
 */
export function loadHistory(): HistoricalData {
  try {
    const data = localStorage.getItem(STORAGE_KEY_HISTORY);
    if (data) {
      return JSON.parse(data);
    }
  } catch (err) {
    console.error('Failed to load history from localStorage:', err);
  }

  return {
    daily: [],
    weekly: [],
  };
}

/**
 * Saves historical data to localStorage
 */
function saveHistory(history: HistoricalData): void {
  try {
    localStorage.setItem(STORAGE_KEY_HISTORY, JSON.stringify(history));
  } catch (err) {
    console.error('Failed to save history to localStorage:', err);
  }
}

/**
 * Adds a snapshot to history
 */
export function addHistorySnapshot(stations: Record<StationId, Station>): void {
  const history = loadHistory();
  const now = Date.now();

  const snapshot: InventorySnapshot = {
    timestamp: now,
    stations: {
      red: stations.red.count,
      green: stations.green.count,
      blue: stations.blue.count,
      yellow: stations.yellow.count,
    },
  };

  // Add to daily history
  history.daily.push(snapshot);

  // Prune old daily entries (keep last 30 days)
  const dayAgo = now - MAX_HISTORY_DAYS * 24 * 60 * 60 * 1000;
  history.daily = history.daily.filter(s => s.timestamp > dayAgo);

  // Add to weekly history (aggregate weekly snapshots)
  const weekAgo = now - 7 * 24 * 60 * 60 * 1000;
  const lastWeeklySnapshot = history.weekly[history.weekly.length - 1];

  if (!lastWeeklySnapshot || lastWeeklySnapshot.timestamp < weekAgo) {
    history.weekly.push(snapshot);

    // Prune old weekly entries (keep last 12 weeks)
    const weeksAgo = now - MAX_HISTORY_WEEKS * 7 * 24 * 60 * 60 * 1000;
    history.weekly = history.weekly.filter(s => s.timestamp > weeksAgo);
  }

  saveHistory(history);
}

/**
 * Records a manual adjustment
 */
export function recordAdjustment(adjustment: ManualAdjustment): void {
  try {
    const data = localStorage.getItem(STORAGE_KEY_ADJUSTMENTS);
    const adjustments: ManualAdjustment[] = data ? JSON.parse(data) : [];

    adjustments.push(adjustment);

    // Keep last 100 adjustments
    if (adjustments.length > 100) {
      adjustments.shift();
    }

    localStorage.setItem(STORAGE_KEY_ADJUSTMENTS, JSON.stringify(adjustments));
  } catch (err) {
    console.error('Failed to record adjustment:', err);
  }
}

/**
 * Loads adjustment history
 */
export function loadAdjustments(): ManualAdjustment[] {
  try {
    const data = localStorage.getItem(STORAGE_KEY_ADJUSTMENTS);
    return data ? JSON.parse(data) : [];
  } catch (err) {
    console.error('Failed to load adjustments:', err);
    return [];
  }
}

/**
 * Exports inventory data as JSON
 */
export function exportInventory(stations: Record<StationId, Station>): InventoryExport {
  const history = loadHistory();
  const totalPieces = Object.values(stations).reduce((sum, s) => sum + s.count, 0);

  return {
    version: '1.0',
    exportedAt: Date.now(),
    stations: Object.values(stations),
    history,
    metadata: {
      totalPieces,
      lastSortTime: Math.max(...Object.values(stations).map(s => s.lastUpdated)),
      sortCount: history.daily.length,
    },
  };
}

/**
 * Imports inventory data from JSON
 */
export function importInventory(data: any): Record<StationId, Station> | null {
  if (!validateInventoryData(data)) {
    console.error('Invalid inventory data format');
    return null;
  }

  const stations: Record<StationId, Station> = {
    red: createDefaultStation('red'),
    green: createDefaultStation('green'),
    blue: createDefaultStation('blue'),
    yellow: createDefaultStation('yellow'),
  };

  // Restore station data
  data.stations.forEach((station: Station) => {
    if (stations[station.id]) {
      stations[station.id] = {
        ...station,
        lastUpdated: Date.now(), // Update timestamp to now
      };
    }
  });

  // Save imported data
  saveStations(stations);

  // Restore history if available
  if (data.history) {
    saveHistory(data.history);
  }

  return stations;
}

/**
 * Clears all inventory data
 */
export function clearAllData(): void {
  try {
    localStorage.removeItem(STORAGE_KEY_STATIONS);
    localStorage.removeItem(STORAGE_KEY_HISTORY);
    localStorage.removeItem(STORAGE_KEY_ADJUSTMENTS);
  } catch (err) {
    console.error('Failed to clear inventory data:', err);
  }
}
