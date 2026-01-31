/**
 * Inventory Dashboard Component
 * Contract: SORT-004 (Inventory Must Persist), SORT-006
 * Journey: J-HELP-LEGO-SORT
 * Issue: #74
 */

import React, { useState, useEffect, useCallback, useRef } from 'react';
import './InventoryDashboard.css';
import {
  Station,
  StationId,
  NFCStatus,
  StockAlert,
  HistoricalData,
  WebSocketMessage,
  STATION_COLORS,
  DEFAULT_LOW_STOCK_THRESHOLD,
  NFC_SYNC_INTERVAL_MS,
  calculateCapacityPercent,
  needsRestock,
} from '../types/inventory';
import {
  loadStations,
  updateStationCount,
  resetStation,
  resetAllStations,
  loadHistory,
  exportInventory,
  importInventory,
  recordAdjustment,
} from '../services/inventoryStorage';

interface InventoryDashboardProps {
  websocketUrl?: string;
}

export const InventoryDashboard: React.FC<InventoryDashboardProps> = ({
  websocketUrl = 'ws://localhost:8081',
}) => {
  const [stations, setStations] = useState<Record<StationId, Station>>(loadStations);
  const [history, setHistory] = useState<HistoricalData>(loadHistory);
  const [alerts, setAlerts] = useState<StockAlert[]>([]);
  const [nfcStatus, setNfcStatus] = useState<NFCStatus>({
    isConnected: false,
    lastSync: 0,
    syncInterval: NFC_SYNC_INTERVAL_MS,
    failedSyncs: 0,
  });
  const [wsConnected, setWsConnected] = useState(false);
  const [flashingStation, setFlashingStation] = useState<StationId | null>(null);
  const [showExportModal, setShowExportModal] = useState(false);
  const [showImportModal, setShowImportModal] = useState(false);
  const [showAdjustModal, setShowAdjustModal] = useState(false);
  const [selectedStation, setSelectedStation] = useState<StationId | null>(null);
  const [adjustmentReason, setAdjustmentReason] = useState('');
  const [adjustmentValue, setAdjustmentValue] = useState('0');
  const [lowStockThreshold, setLowStockThreshold] = useState(DEFAULT_LOW_STOCK_THRESHOLD);

  const wsRef = useRef<WebSocket | null>(null);
  const fileInputRef = useRef<HTMLInputElement | null>(null);

  // Check for low stock alerts
  useEffect(() => {
    const newAlerts: StockAlert[] = [];

    Object.values(stations).forEach(station => {
      if (needsRestock(station, lowStockThreshold)) {
        newAlerts.push({
          stationId: station.id,
          threshold: lowStockThreshold,
          currentCount: station.count,
          severity: station.count === 0 ? 'critical' : 'low',
        });
      }
    });

    setAlerts(newAlerts);
  }, [stations, lowStockThreshold]);

  // WebSocket connection
  useEffect(() => {
    const ws = new WebSocket(websocketUrl);
    wsRef.current = ws;

    ws.onopen = () => {
      console.log('📡 WebSocket connected to robot');
      setWsConnected(true);
    };

    ws.onclose = () => {
      console.log('📡 WebSocket disconnected');
      setWsConnected(false);
    };

    ws.onerror = (error) => {
      console.error('📡 WebSocket error:', error);
      setWsConnected(false);
    };

    ws.onmessage = (event) => {
      try {
        const message: WebSocketMessage = JSON.parse(event.data);

        if (message.type === 'inventory_update') {
          const { stationId, count } = message.payload;
          handleInventoryUpdate(stationId, count, 'robot');
        } else if (message.type === 'nfc_status') {
          setNfcStatus(message.payload);
        } else if (message.type === 'bulk_update') {
          Object.keys(message.payload.stations).forEach((id) => {
            const stationId = id as StationId;
            handleInventoryUpdate(stationId, message.payload.stations[id], 'nfc');
          });
        }
      } catch (err) {
        console.error('Failed to parse WebSocket message:', err);
      }
    };

    return () => {
      ws.close();
    };
  }, [websocketUrl]);

  // Handle inventory updates
  const handleInventoryUpdate = useCallback(
    (stationId: StationId, count: number, source: 'robot' | 'nfc' | 'manual') => {
      setStations(prev => {
        const updated = updateStationCount(prev, stationId, count, source);

        // Flash effect for real-time updates
        if (source === 'robot') {
          setFlashingStation(stationId);
          setTimeout(() => setFlashingStation(null), 1000);
        }

        return updated;
      });
    },
    []
  );

  // Manual adjustment
  const handleManualAdjustment = () => {
    if (!selectedStation) return;

    const newCount = parseInt(adjustmentValue, 10);
    if (isNaN(newCount) || newCount < 0) {
      alert('Please enter a valid non-negative number');
      return;
    }

    if (!adjustmentReason.trim()) {
      alert('Please provide a reason for the adjustment');
      return;
    }

    const previousCount = stations[selectedStation].count;

    recordAdjustment({
      stationId: selectedStation,
      previousCount,
      newCount,
      reason: adjustmentReason,
      timestamp: Date.now(),
    });

    handleInventoryUpdate(selectedStation, newCount, 'manual');

    setShowAdjustModal(false);
    setAdjustmentReason('');
    setAdjustmentValue('0');
    setSelectedStation(null);
  };

  // Export inventory
  const handleExport = () => {
    const data = exportInventory(stations);
    const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `mbot-inventory-${new Date().toISOString().split('T')[0]}.json`;
    a.click();
    URL.revokeObjectURL(url);
    setShowExportModal(false);
  };

  // Import inventory
  const handleImport = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const data = JSON.parse(e.target?.result as string);
        const imported = importInventory(data);

        if (imported) {
          setStations(imported);
          setHistory(loadHistory());
          alert('Inventory imported successfully!');
          setShowImportModal(false);
        } else {
          alert('Failed to import: Invalid data format');
        }
      } catch (err) {
        alert('Failed to import: ' + (err as Error).message);
      }
    };
    reader.readAsText(file);
  };

  // Reset station
  const handleResetStation = (stationId: StationId) => {
    if (confirm(`Reset ${stationId} station to 0?`)) {
      setStations(prev => resetStation(prev, stationId));
    }
  };

  // Reset all stations
  const handleResetAll = () => {
    if (confirm('Reset ALL stations to 0? This cannot be undone.')) {
      setStations(prev => resetAllStations(prev));
    }
  };

  return (
    <div className="inventory-dashboard" data-testid="inventory-dashboard">
      {/* Header */}
      <header className="dashboard-header">
        <h1>LEGO Sorter Inventory</h1>
        <div className="header-status">
          <div
            className={`status-indicator ${wsConnected ? 'connected' : 'disconnected'}`}
            data-testid="websocket-status"
          >
            {wsConnected ? '🟢 Robot Connected' : '🔴 Robot Disconnected'}
          </div>
          <div
            className={`status-indicator ${nfcStatus.isConnected ? 'connected' : 'disconnected'}`}
            data-testid="nfc-status"
          >
            {nfcStatus.isConnected ? '📡 NFC Active' : '📡 NFC Inactive'}
          </div>
          {nfcStatus.isConnected && (
            <div className="nfc-last-sync" data-testid="nfc-last-sync">
              Last sync: {new Date(nfcStatus.lastSync).toLocaleTimeString()}
            </div>
          )}
        </div>
      </header>

      {/* Alerts */}
      {alerts.length > 0 && (
        <div className="alerts-container" data-testid="inventory-alerts">
          {alerts.map(alert => (
            <div
              key={alert.stationId}
              className={`alert alert-${alert.severity}`}
              data-testid={`alert-${alert.stationId}`}
            >
              {alert.severity === 'critical' ? '⚠️' : '⚡'}{' '}
              {alert.stationId.toUpperCase()} bin is {alert.severity === 'critical' ? 'empty' : 'low'} (
              {alert.currentCount}/{alert.threshold})
            </div>
          ))}
        </div>
      )}

      {/* Station Cards */}
      <div className="stations-grid" data-testid="stations-grid">
        {(['red', 'green', 'blue', 'yellow'] as StationId[]).map(stationId => {
          const station = stations[stationId];
          const percent = calculateCapacityPercent(station.count, station.capacity);
          const isFlashing = flashingStation === stationId;
          const isLow = needsRestock(station, lowStockThreshold);

          return (
            <div
              key={stationId}
              className={`station-card ${isFlashing ? 'flashing' : ''} ${isLow ? 'low-stock' : ''}`}
              data-testid={`station-card-${stationId}`}
              style={{ borderColor: station.color }}
            >
              <div className="station-header">
                <h2 style={{ color: station.color }}>{stationId.toUpperCase()}</h2>
                <button
                  className="reset-btn"
                  onClick={() => handleResetStation(stationId)}
                  data-testid={`reset-button-${stationId}`}
                  title="Reset to 0"
                >
                  ↻
                </button>
              </div>

              <div className="station-count" data-testid={`station-count-${stationId}`}>
                {station.count}
              </div>

              <div className="capacity-bar-container">
                <div
                  className="capacity-bar"
                  data-testid={`capacity-bar-${stationId}`}
                  style={{
                    width: `${percent}%`,
                    backgroundColor: station.color,
                  }}
                />
              </div>

              <div className="capacity-text">
                {percent}% ({station.count}/{station.capacity})
              </div>

              <div className="station-footer">
                <div className="last-updated">
                  Updated: {new Date(station.lastUpdated).toLocaleTimeString()}
                </div>
                <button
                  className="adjust-btn"
                  onClick={() => {
                    setSelectedStation(stationId);
                    setAdjustmentValue(station.count.toString());
                    setShowAdjustModal(true);
                  }}
                  data-testid={`adjust-button-${stationId}`}
                >
                  Adjust
                </button>
              </div>
            </div>
          );
        })}
      </div>

      {/* History Chart Placeholder */}
      <div className="history-section" data-testid="history-chart">
        <h2>Historical Trends</h2>
        <div className="chart-container">
          <p>Daily/Weekly trends chart (visualization to be implemented)</p>
          <div className="chart-summary">
            <div>Daily snapshots: {history.daily.length}</div>
            <div>Weekly snapshots: {history.weekly.length}</div>
          </div>
        </div>
      </div>

      {/* Actions */}
      <div className="actions-bar">
        <button
          className="action-btn"
          onClick={() => setShowExportModal(true)}
          data-testid="export-button"
        >
          📤 Export JSON
        </button>
        <button
          className="action-btn"
          onClick={() => setShowImportModal(true)}
          data-testid="import-button"
        >
          📥 Import JSON
        </button>
        <button
          className="action-btn danger"
          onClick={handleResetAll}
          data-testid="reset-all-button"
        >
          ⚠️ Reset All
        </button>
        <div className="threshold-control">
          <label htmlFor="threshold">Low Stock Threshold:</label>
          <input
            id="threshold"
            type="number"
            min="0"
            max="100"
            value={lowStockThreshold}
            onChange={(e) => setLowStockThreshold(parseInt(e.target.value, 10))}
            data-testid="threshold-input"
          />
        </div>
      </div>

      {/* Export Modal */}
      {showExportModal && (
        <div className="modal-overlay" data-testid="export-modal">
          <div className="modal">
            <h2>Export Inventory</h2>
            <p>Download current inventory data as JSON file.</p>
            <div className="modal-actions">
              <button onClick={handleExport}>Download</button>
              <button onClick={() => setShowExportModal(false)}>Cancel</button>
            </div>
          </div>
        </div>
      )}

      {/* Import Modal */}
      {showImportModal && (
        <div className="modal-overlay" data-testid="import-modal">
          <div className="modal">
            <h2>Import Inventory</h2>
            <p>Upload a previously exported JSON file to restore inventory data.</p>
            <input
              ref={fileInputRef}
              type="file"
              accept=".json"
              onChange={handleImport}
              data-testid="import-file-input"
            />
            <div className="modal-actions">
              <button onClick={() => setShowImportModal(false)}>Cancel</button>
            </div>
          </div>
        </div>
      )}

      {/* Adjust Modal */}
      {showAdjustModal && selectedStation && (
        <div className="modal-overlay" data-testid="adjust-modal">
          <div className="modal">
            <h2>Adjust {selectedStation.toUpperCase()} Station</h2>
            <div className="form-group">
              <label htmlFor="adjust-value">New Count:</label>
              <input
                id="adjust-value"
                type="number"
                min="0"
                value={adjustmentValue}
                onChange={(e) => setAdjustmentValue(e.target.value)}
                data-testid="adjust-value-input"
              />
            </div>
            <div className="form-group">
              <label htmlFor="adjust-reason">Reason:</label>
              <input
                id="adjust-reason"
                type="text"
                placeholder="e.g., Manual count, correction, etc."
                value={adjustmentReason}
                onChange={(e) => setAdjustmentReason(e.target.value)}
                data-testid="adjust-reason-input"
              />
            </div>
            <div className="modal-actions">
              <button onClick={handleManualAdjustment} data-testid="adjust-confirm-button">
                Confirm
              </button>
              <button onClick={() => setShowAdjustModal(false)}>Cancel</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
