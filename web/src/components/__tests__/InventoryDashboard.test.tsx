/**
 * Inventory Dashboard Tests
 * Contract: SORT-004 (Inventory Must Persist)
 * Issue: #74
 */

import { describe, it, expect, beforeEach, afterEach, vi } from '@jest/globals';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { InventoryDashboard } from '../InventoryDashboard';
import * as storage from '../../services/inventoryStorage';

// Mock WebSocket
class MockWebSocket {
  static OPEN = 1;
  readyState = MockWebSocket.OPEN;
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onerror: ((error: any) => void) | null = null;
  onmessage: ((event: { data: string }) => void) | null = null;

  constructor(public url: string) {
    setTimeout(() => {
      if (this.onopen) this.onopen();
    }, 0);
  }

  send(data: string) {
    // Mock send
  }

  close() {
    if (this.onclose) this.onclose();
  }
}

(global as any).WebSocket = MockWebSocket;

// Mock localStorage
const localStorageMock: { [key: string]: string } = {};

beforeEach(() => {
  global.localStorage = {
    getItem: (key: string) => localStorageMock[key] || null,
    setItem: (key: string, value: string) => {
      localStorageMock[key] = value;
    },
    removeItem: (key: string) => {
      delete localStorageMock[key];
    },
    clear: () => {
      Object.keys(localStorageMock).forEach(key => delete localStorageMock[key]);
    },
    length: 0,
    key: (index: number) => null,
  } as Storage;

  // Clear storage before each test
  Object.keys(localStorageMock).forEach(key => delete localStorageMock[key]);
});

afterEach(() => {
  vi.clearAllMocks();
});

describe('InventoryDashboard', () => {
  describe('Scenario: View Inventory Dashboard', () => {
    it('should render 4 station cards', () => {
      render(<InventoryDashboard />);

      expect(screen.getByTestId('station-card-red')).toBeInTheDocument();
      expect(screen.getByTestId('station-card-green')).toBeInTheDocument();
      expect(screen.getByTestId('station-card-blue')).toBeInTheDocument();
      expect(screen.getByTestId('station-card-yellow')).toBeInTheDocument();
    });

    it('should show current count for each station', () => {
      render(<InventoryDashboard />);

      expect(screen.getByTestId('station-count-red')).toHaveTextContent('0');
      expect(screen.getByTestId('station-count-green')).toHaveTextContent('0');
      expect(screen.getByTestId('station-count-blue')).toHaveTextContent('0');
      expect(screen.getByTestId('station-count-yellow')).toHaveTextContent('0');
    });

    it('should show capacity bars for each station', () => {
      render(<InventoryDashboard />);

      expect(screen.getByTestId('capacity-bar-red')).toBeInTheDocument();
      expect(screen.getByTestId('capacity-bar-green')).toBeInTheDocument();
      expect(screen.getByTestId('capacity-bar-blue')).toBeInTheDocument();
      expect(screen.getByTestId('capacity-bar-yellow')).toBeInTheDocument();
    });
  });

  describe('Scenario: Real-Time Update', () => {
    it('should update count when WebSocket message received', async () => {
      const { container } = render(<InventoryDashboard />);

      // Wait for WebSocket to connect
      await waitFor(() => {
        const status = screen.getByTestId('websocket-status');
        expect(status).toHaveTextContent('Robot Connected');
      });

      // Simulate WebSocket message
      const ws = (global as any).WebSocket.instances?.[0];
      if (ws && ws.onmessage) {
        ws.onmessage({
          data: JSON.stringify({
            type: 'inventory_update',
            payload: {
              stationId: 'red',
              count: 42,
              timestamp: Date.now(),
            },
          }),
        });
      }

      await waitFor(() => {
        const count = screen.getByTestId('station-count-red');
        expect(count).toHaveTextContent('42');
      });
    });

    it('should flash card when update received', async () => {
      render(<InventoryDashboard />);

      await waitFor(() => {
        const status = screen.getByTestId('websocket-status');
        expect(status).toHaveTextContent('Robot Connected');
      });

      const ws = (global as any).WebSocket.instances?.[0];
      if (ws && ws.onmessage) {
        ws.onmessage({
          data: JSON.stringify({
            type: 'inventory_update',
            payload: {
              stationId: 'green',
              count: 10,
              timestamp: Date.now(),
            },
          }),
        });
      }

      await waitFor(() => {
        const card = screen.getByTestId('station-card-green');
        expect(card).toHaveClass('flashing');
      });

      // Flash should disappear after 1 second
      await waitFor(() => {
        const card = screen.getByTestId('station-card-green');
        expect(card).not.toHaveClass('flashing');
      }, { timeout: 1500 });
    });
  });

  describe('Scenario: Low Stock Alerts', () => {
    it('should show alert when count is below threshold', async () => {
      render(<InventoryDashboard />);

      // Manually update a station to low count
      const adjustBtn = screen.getByTestId('adjust-button-red');
      fireEvent.click(adjustBtn);

      await waitFor(() => {
        expect(screen.getByTestId('adjust-modal')).toBeInTheDocument();
      });

      const valueInput = screen.getByTestId('adjust-value-input') as HTMLInputElement;
      const reasonInput = screen.getByTestId('adjust-reason-input') as HTMLInputElement;

      fireEvent.change(valueInput, { target: { value: '15' } });
      fireEvent.change(reasonInput, { target: { value: 'Test low stock' } });

      const confirmBtn = screen.getByTestId('adjust-confirm-button');
      fireEvent.click(confirmBtn);

      await waitFor(() => {
        expect(screen.getByTestId('inventory-alerts')).toBeInTheDocument();
        expect(screen.getByTestId('alert-red')).toBeInTheDocument();
      });
    });

    it('should show critical alert when count is zero', async () => {
      render(<InventoryDashboard />);

      // Initial state should have zero counts, triggering critical alerts
      await waitFor(() => {
        const alerts = screen.getByTestId('inventory-alerts');
        expect(alerts).toBeInTheDocument();
        expect(screen.getByTestId('alert-red')).toHaveClass('alert-critical');
      });
    });
  });

  describe('Scenario: Manual Adjustment', () => {
    it('should open adjustment modal when Adjust button clicked', () => {
      render(<InventoryDashboard />);

      const adjustBtn = screen.getByTestId('adjust-button-blue');
      fireEvent.click(adjustBtn);

      expect(screen.getByTestId('adjust-modal')).toBeInTheDocument();
    });

    it('should update count after manual adjustment', async () => {
      render(<InventoryDashboard />);

      const adjustBtn = screen.getByTestId('adjust-button-blue');
      fireEvent.click(adjustBtn);

      const valueInput = screen.getByTestId('adjust-value-input') as HTMLInputElement;
      const reasonInput = screen.getByTestId('adjust-reason-input') as HTMLInputElement;

      fireEvent.change(valueInput, { target: { value: '75' } });
      fireEvent.change(reasonInput, { target: { value: 'Manual inventory count' } });

      const confirmBtn = screen.getByTestId('adjust-confirm-button');
      fireEvent.click(confirmBtn);

      await waitFor(() => {
        const count = screen.getByTestId('station-count-blue');
        expect(count).toHaveTextContent('75');
      });
    });

    it('should require reason for adjustment', async () => {
      const alertSpy = vi.spyOn(window, 'alert').mockImplementation(() => {});

      render(<InventoryDashboard />);

      const adjustBtn = screen.getByTestId('adjust-button-yellow');
      fireEvent.click(adjustBtn);

      const valueInput = screen.getByTestId('adjust-value-input') as HTMLInputElement;
      fireEvent.change(valueInput, { target: { value: '50' } });

      const confirmBtn = screen.getByTestId('adjust-confirm-button');
      fireEvent.click(confirmBtn);

      await waitFor(() => {
        expect(alertSpy).toHaveBeenCalledWith('Please provide a reason for the adjustment');
      });

      alertSpy.mockRestore();
    });
  });

  describe('Scenario: Reset Station', () => {
    it('should reset station to zero when reset button clicked', async () => {
      const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);

      render(<InventoryDashboard />);

      // First set a non-zero value
      const adjustBtn = screen.getByTestId('adjust-button-green');
      fireEvent.click(adjustBtn);

      const valueInput = screen.getByTestId('adjust-value-input') as HTMLInputElement;
      const reasonInput = screen.getByTestId('adjust-reason-input') as HTMLInputElement;

      fireEvent.change(valueInput, { target: { value: '30' } });
      fireEvent.change(reasonInput, { target: { value: 'Test' } });

      const confirmBtn = screen.getByTestId('adjust-confirm-button');
      fireEvent.click(confirmBtn);

      await waitFor(() => {
        const count = screen.getByTestId('station-count-green');
        expect(count).toHaveTextContent('30');
      });

      // Now reset
      const resetBtn = screen.getByTestId('reset-button-green');
      fireEvent.click(resetBtn);

      await waitFor(() => {
        const count = screen.getByTestId('station-count-green');
        expect(count).toHaveTextContent('0');
      });

      confirmSpy.mockRestore();
    });
  });

  describe('Scenario: Export Inventory', () => {
    it('should show export modal when Export button clicked', () => {
      render(<InventoryDashboard />);

      const exportBtn = screen.getByTestId('export-button');
      fireEvent.click(exportBtn);

      expect(screen.getByTestId('export-modal')).toBeInTheDocument();
    });

    it('should download JSON file on export', async () => {
      const createElementSpy = vi.spyOn(document, 'createElement');

      render(<InventoryDashboard />);

      const exportBtn = screen.getByTestId('export-button');
      fireEvent.click(exportBtn);

      const downloadBtn = screen.getByText('Download');
      fireEvent.click(downloadBtn);

      await waitFor(() => {
        expect(createElementSpy).toHaveBeenCalledWith('a');
      });

      createElementSpy.mockRestore();
    });
  });

  describe('Scenario: Import Inventory', () => {
    it('should show import modal when Import button clicked', () => {
      render(<InventoryDashboard />);

      const importBtn = screen.getByTestId('import-button');
      fireEvent.click(importBtn);

      expect(screen.getByTestId('import-modal')).toBeInTheDocument();
    });
  });

  describe('Scenario: NFC Status Display', () => {
    it('should show NFC status indicator', () => {
      render(<InventoryDashboard />);

      expect(screen.getByTestId('nfc-status')).toBeInTheDocument();
    });

    it('should show last sync time when NFC is connected', async () => {
      render(<InventoryDashboard />);

      // Simulate NFC status update
      await waitFor(() => {
        const ws = (global as any).WebSocket.instances?.[0];
        if (ws && ws.onmessage) {
          ws.onmessage({
            data: JSON.stringify({
              type: 'nfc_status',
              payload: {
                isConnected: true,
                lastSync: Date.now(),
                syncInterval: 5000,
                failedSyncs: 0,
              },
            }),
          });
        }
      });

      await waitFor(() => {
        expect(screen.getByTestId('nfc-last-sync')).toBeInTheDocument();
      });
    });
  });

  describe('Invariant: I-SORT-INV-001 (NFC Sync)', () => {
    it('should respect 5 second max sync interval', () => {
      const { NFC_SYNC_INTERVAL_MS } = require('../../types/inventory');
      expect(NFC_SYNC_INTERVAL_MS).toBeLessThanOrEqual(5000);
    });
  });

  describe('Contract: SORT-004 (Inventory Must Persist)', () => {
    it('should persist inventory to localStorage', async () => {
      render(<InventoryDashboard />);

      const adjustBtn = screen.getByTestId('adjust-button-red');
      fireEvent.click(adjustBtn);

      const valueInput = screen.getByTestId('adjust-value-input') as HTMLInputElement;
      const reasonInput = screen.getByTestId('adjust-reason-input') as HTMLInputElement;

      fireEvent.change(valueInput, { target: { value: '88' } });
      fireEvent.change(reasonInput, { target: { value: 'Persistence test' } });

      const confirmBtn = screen.getByTestId('adjust-confirm-button');
      fireEvent.click(confirmBtn);

      await waitFor(() => {
        const stored = localStorage.getItem('mbot_inventory_stations');
        expect(stored).toBeTruthy();

        const parsed = JSON.parse(stored!);
        expect(parsed.red.count).toBe(88);
      });
    });

    it('should load inventory from localStorage on mount', () => {
      const testData = {
        red: { id: 'red', color: '#EF4444', count: 50, capacity: 100, lastUpdated: Date.now() },
        green: { id: 'green', color: '#10B981', count: 30, capacity: 100, lastUpdated: Date.now() },
        blue: { id: 'blue', color: '#3B82F6', count: 70, capacity: 100, lastUpdated: Date.now() },
        yellow: { id: 'yellow', color: '#F59E0B', count: 20, capacity: 100, lastUpdated: Date.now() },
      };

      localStorage.setItem('mbot_inventory_stations', JSON.stringify(testData));

      render(<InventoryDashboard />);

      expect(screen.getByTestId('station-count-red')).toHaveTextContent('50');
      expect(screen.getByTestId('station-count-green')).toHaveTextContent('30');
      expect(screen.getByTestId('station-count-blue')).toHaveTextContent('70');
      expect(screen.getByTestId('station-count-yellow')).toHaveTextContent('20');
    });
  });

  describe('Threshold Configuration', () => {
    it('should allow configuring low stock threshold', () => {
      render(<InventoryDashboard />);

      const thresholdInput = screen.getByTestId('threshold-input') as HTMLInputElement;
      expect(thresholdInput.value).toBe('20'); // Default

      fireEvent.change(thresholdInput, { target: { value: '30' } });

      expect(thresholdInput.value).toBe('30');
    });
  });

  describe('Reset All Functionality', () => {
    it('should reset all stations when Reset All button clicked', async () => {
      const confirmSpy = vi.spyOn(window, 'confirm').mockReturnValue(true);

      render(<InventoryDashboard />);

      // Set some values first
      const stations = storage.loadStations();
      stations.red.count = 50;
      stations.green.count = 30;
      storage.saveStations(stations);

      const resetAllBtn = screen.getByTestId('reset-all-button');
      fireEvent.click(resetAllBtn);

      await waitFor(() => {
        expect(screen.getByTestId('station-count-red')).toHaveTextContent('0');
        expect(screen.getByTestId('station-count-green')).toHaveTextContent('0');
        expect(screen.getByTestId('station-count-blue')).toHaveTextContent('0');
        expect(screen.getByTestId('station-count-yellow')).toHaveTextContent('0');
      });

      confirmSpy.mockRestore();
    });
  });
});
