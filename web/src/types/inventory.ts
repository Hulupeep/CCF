/**
 * Inventory Type Definitions
 * Contract: SORT-004 (Inventory Must Persist)
 * Journey: J-HELP-LEGO-SORT
 */

export type StationId = 'red' | 'green' | 'blue' | 'yellow';

export interface Station {
  id: StationId;
  color: string;
  count: number;
  capacity: number;
  lastUpdated: number;
  nfcTagId?: string;
  nfcLastSync?: number;
}

export interface InventorySnapshot {
  timestamp: number;
  stations: Record<StationId, number>;
}

export interface HistoricalData {
  daily: InventorySnapshot[];
  weekly: InventorySnapshot[];
}

export interface StockAlert {
  stationId: StationId;
  threshold: number;
  currentCount: number;
  severity: 'low' | 'critical';
}

export interface InventoryExport {
  version: '1.0';
  exportedAt: number;
  stations: Station[];
  history: HistoricalData;
  metadata: {
    totalPieces: number;
    lastSortTime: number;
    sortCount: number;
  };
}

export interface NFCStatus {
  isConnected: boolean;
  lastSync: number;
  syncInterval: number;
  failedSyncs: number;
}

export interface ManualAdjustment {
  stationId: StationId;
  previousCount: number;
  newCount: number;
  reason: string;
  timestamp: number;
  userId?: string;
}

export interface WebSocketInventoryUpdate {
  type: 'inventory_update';
  stationId: StationId;
  count: number;
  timestamp: number;
  source: 'robot' | 'nfc' | 'manual';
}

export interface WebSocketMessage {
  type: 'inventory_update' | 'nfc_status' | 'station_reset' | 'bulk_update';
  payload: any;
}

export const DEFAULT_CAPACITY = 100;
export const DEFAULT_LOW_STOCK_THRESHOLD = 20;
export const NFC_SYNC_INTERVAL_MS = 5000; // 5 seconds per I-SORT-INV-001

export const STATION_COLORS: Record<StationId, string> = {
  red: '#EF4444',
  green: '#10B981',
  blue: '#3B82F6',
  yellow: '#F59E0B',
};

/**
 * Creates a default station configuration
 */
export function createDefaultStation(id: StationId): Station {
  return {
    id,
    color: STATION_COLORS[id],
    count: 0,
    capacity: DEFAULT_CAPACITY,
    lastUpdated: Date.now(),
  };
}

/**
 * Validates inventory data before persistence
 */
export function validateInventoryData(data: any): data is InventoryExport {
  return (
    data &&
    data.version === '1.0' &&
    typeof data.exportedAt === 'number' &&
    Array.isArray(data.stations) &&
    data.stations.length === 4 &&
    data.stations.every((s: any) =>
      ['red', 'green', 'blue', 'yellow'].includes(s.id) &&
      typeof s.count === 'number' &&
      typeof s.capacity === 'number'
    )
  );
}

/**
 * Calculates capacity percentage
 */
export function calculateCapacityPercent(count: number, capacity: number): number {
  if (capacity === 0) return 0;
  return Math.min(100, Math.round((count / capacity) * 100));
}

/**
 * Checks if a station needs restocking
 */
export function needsRestock(station: Station, threshold: number = DEFAULT_LOW_STOCK_THRESHOLD): boolean {
  return station.count <= threshold;
}
