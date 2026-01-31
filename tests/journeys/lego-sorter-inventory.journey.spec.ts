/**
 * Journey Test: LEGO Sorter Inventory Dashboard
 * Contract: SORT-004, SORT-006
 * Journey: J-HELP-LEGO-SORT
 * Issue: #74
 */

import { test, expect } from '@playwright/test';

test.describe('Journey: LEGO Sorter Inventory Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to inventory dashboard
    await page.goto('http://localhost:3000/inventory');

    // Wait for dashboard to load
    await page.waitForSelector('[data-testid="inventory-dashboard"]');
  });

  test('Scenario: View Inventory Dashboard', async ({ page }) => {
    // Given LEGO sorter is running
    // When I navigate to /inventory

    // Then I see 4 station cards
    await expect(page.getByTestId('station-card-red')).toBeVisible();
    await expect(page.getByTestId('station-card-green')).toBeVisible();
    await expect(page.getByTestId('station-card-blue')).toBeVisible();
    await expect(page.getByTestId('station-card-yellow')).toBeVisible();

    // And each shows current count
    await expect(page.getByTestId('station-count-red')).toBeVisible();
    await expect(page.getByTestId('station-count-green')).toBeVisible();
    await expect(page.getByTestId('station-count-blue')).toBeVisible();
    await expect(page.getByTestId('station-count-yellow')).toBeVisible();

    // And each shows capacity %
    await expect(page.getByTestId('capacity-bar-red')).toBeVisible();
    await expect(page.getByTestId('capacity-bar-green')).toBeVisible();
    await expect(page.getByTestId('capacity-bar-blue')).toBeVisible();
    await expect(page.getByTestId('capacity-bar-yellow')).toBeVisible();
  });

  test('Scenario: Real-Time Update', async ({ page }) => {
    // When robot drops piece in Red
    // Simulate WebSocket message from robot
    await page.evaluate(() => {
      const event = new MessageEvent('message', {
        data: JSON.stringify({
          type: 'inventory_update',
          payload: {
            stationId: 'red',
            count: 42,
            timestamp: Date.now(),
          },
        }),
      });
      // Trigger WebSocket message handler
      window.dispatchEvent(event);
    });

    // Then Red count updates within 1s
    await expect(page.getByTestId('station-count-red')).toContainText('42', {
      timeout: 1000,
    });

    // And card flashes
    await expect(page.getByTestId('station-card-red')).toHaveClass(/flashing/, {
      timeout: 500,
    });
  });

  test('Scenario: Manual Adjustment', async ({ page }) => {
    // Given I want to correct inventory
    // When I click Adjust on Blue station
    await page.getByTestId('adjust-button-blue').click();

    // Then adjustment modal appears
    await expect(page.getByTestId('adjust-modal')).toBeVisible();

    // When I enter new count
    await page.getByTestId('adjust-value-input').fill('75');
    await page.getByTestId('adjust-reason-input').fill('Manual inventory count');

    // And I confirm
    await page.getByTestId('adjust-confirm-button').click();

    // Then count updates
    await expect(page.getByTestId('station-count-blue')).toContainText('75');

    // And modal closes
    await expect(page.getByTestId('adjust-modal')).not.toBeVisible();
  });

  test('Scenario: Low Stock Alert', async ({ page }) => {
    // Given threshold is set to 20
    await expect(page.getByTestId('threshold-input')).toHaveValue('20');

    // When station count drops below threshold
    await page.getByTestId('adjust-button-green').click();
    await page.getByTestId('adjust-value-input').fill('15');
    await page.getByTestId('adjust-reason-input').fill('Testing low stock alert');
    await page.getByTestId('adjust-confirm-button').click();

    // Then alert appears
    await expect(page.getByTestId('inventory-alerts')).toBeVisible();
    await expect(page.getByTestId('alert-green')).toBeVisible();
    await expect(page.getByTestId('alert-green')).toContainText('low');
  });

  test('Scenario: Critical Stock Alert', async ({ page }) => {
    // When station is empty
    await page.getByTestId('reset-button-yellow').click();

    // Confirm reset dialog
    page.on('dialog', dialog => dialog.accept());

    // Then critical alert appears
    await expect(page.getByTestId('inventory-alerts')).toBeVisible();
    await expect(page.getByTestId('alert-yellow')).toBeVisible();
    await expect(page.getByTestId('alert-yellow')).toContainText('empty');
  });

  test('Scenario: Export Inventory', async ({ page }) => {
    // When I click Export button
    await page.getByTestId('export-button').click();

    // Then export modal appears
    await expect(page.getByTestId('export-modal')).toBeVisible();

    // When I click Download
    const downloadPromise = page.waitForEvent('download');
    await page.getByText('Download').click();

    // Then JSON file downloads
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(/mbot-inventory-.*\.json/);
  });

  test('Scenario: Import Inventory', async ({ page }) => {
    // Given I have exported inventory data
    const testData = {
      version: '1.0',
      exportedAt: Date.now(),
      stations: [
        { id: 'red', color: '#EF4444', count: 50, capacity: 100, lastUpdated: Date.now() },
        { id: 'green', color: '#10B981', count: 30, capacity: 100, lastUpdated: Date.now() },
        { id: 'blue', color: '#3B82F6', count: 70, capacity: 100, lastUpdated: Date.now() },
        { id: 'yellow', color: '#F59E0B', count: 20, capacity: 100, lastUpdated: Date.now() },
      ],
      history: { daily: [], weekly: [] },
      metadata: { totalPieces: 170, lastSortTime: Date.now(), sortCount: 0 },
    };

    // When I click Import button
    await page.getByTestId('import-button').click();

    // Then import modal appears
    await expect(page.getByTestId('import-modal')).toBeVisible();

    // When I upload JSON file
    const fileInput = page.getByTestId('import-file-input');
    await fileInput.setInputFiles({
      name: 'test-inventory.json',
      mimeType: 'application/json',
      buffer: Buffer.from(JSON.stringify(testData)),
    });

    // Then inventory updates
    await expect(page.getByTestId('station-count-red')).toContainText('50');
    await expect(page.getByTestId('station-count-green')).toContainText('30');
    await expect(page.getByTestId('station-count-blue')).toContainText('70');
    await expect(page.getByTestId('station-count-yellow')).toContainText('20');
  });

  test('Scenario: Reset Single Station', async ({ page }) => {
    // Given station has non-zero count
    await page.getByTestId('adjust-button-red').click();
    await page.getByTestId('adjust-value-input').fill('50');
    await page.getByTestId('adjust-reason-input').fill('Setup test');
    await page.getByTestId('adjust-confirm-button').click();
    await expect(page.getByTestId('station-count-red')).toContainText('50');

    // When I click Reset button
    page.on('dialog', dialog => dialog.accept());
    await page.getByTestId('reset-button-red').click();

    // Then count resets to 0
    await expect(page.getByTestId('station-count-red')).toContainText('0');
  });

  test('Scenario: Reset All Stations', async ({ page }) => {
    // Given multiple stations have counts
    // Set up test data
    await page.evaluate(() => {
      localStorage.setItem('mbot_inventory_stations', JSON.stringify({
        red: { id: 'red', color: '#EF4444', count: 50, capacity: 100, lastUpdated: Date.now() },
        green: { id: 'green', color: '#10B981', count: 30, capacity: 100, lastUpdated: Date.now() },
        blue: { id: 'blue', color: '#3B82F6', count: 70, capacity: 100, lastUpdated: Date.now() },
        yellow: { id: 'yellow', color: '#F59E0B', count: 20, capacity: 100, lastUpdated: Date.now() },
      }));
    });
    await page.reload();

    // When I click Reset All button
    page.on('dialog', dialog => dialog.accept());
    await page.getByTestId('reset-all-button').click();

    // Then all stations reset to 0
    await expect(page.getByTestId('station-count-red')).toContainText('0');
    await expect(page.getByTestId('station-count-green')).toContainText('0');
    await expect(page.getByTestId('station-count-blue')).toContainText('0');
    await expect(page.getByTestId('station-count-yellow')).toContainText('0');
  });

  test('Scenario: Configure Threshold', async ({ page }) => {
    // When I change threshold to 30
    await page.getByTestId('threshold-input').fill('30');

    // And station count is 25
    await page.getByTestId('adjust-button-blue').click();
    await page.getByTestId('adjust-value-input').fill('25');
    await page.getByTestId('adjust-reason-input').fill('Testing threshold');
    await page.getByTestId('adjust-confirm-button').click();

    // Then alert appears (25 < 30)
    await expect(page.getByTestId('inventory-alerts')).toBeVisible();
    await expect(page.getByTestId('alert-blue')).toBeVisible();
  });

  test('Scenario: NFC Status Display', async ({ page }) => {
    // Then I see NFC status indicator
    await expect(page.getByTestId('nfc-status')).toBeVisible();

    // When NFC connects
    await page.evaluate(() => {
      const event = new MessageEvent('message', {
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
      window.dispatchEvent(event);
    });

    // Then last sync time appears
    await expect(page.getByTestId('nfc-last-sync')).toBeVisible();
  });

  test('Scenario: WebSocket Connection Status', async ({ page }) => {
    // Then I see WebSocket status indicator
    await expect(page.getByTestId('websocket-status')).toBeVisible();

    // Connection status should be visible
    const status = page.getByTestId('websocket-status');
    await expect(status).toContainText(/Robot (Connected|Disconnected)/);
  });

  test('Invariant: I-SORT-INV-001 - NFC Sync Interval', async ({ page }) => {
    // NFC must sync every 5 seconds max
    await page.evaluate(() => {
      const { NFC_SYNC_INTERVAL_MS } = require('../src/types/inventory');
      if (NFC_SYNC_INTERVAL_MS > 5000) {
        throw new Error(`NFC sync interval ${NFC_SYNC_INTERVAL_MS}ms exceeds 5000ms maximum`);
      }
    });
  });

  test('Contract: SORT-004 - Inventory Persistence', async ({ page }) => {
    // Given I set inventory counts
    await page.getByTestId('adjust-button-red').click();
    await page.getByTestId('adjust-value-input').fill('88');
    await page.getByTestId('adjust-reason-input').fill('Persistence test');
    await page.getByTestId('adjust-confirm-button').click();

    // When I reload the page
    await page.reload();

    // Then inventory is restored from localStorage
    await expect(page.getByTestId('station-count-red')).toContainText('88');
  });
});
