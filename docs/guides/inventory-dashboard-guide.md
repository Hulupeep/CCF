# Inventory Dashboard Guide

**Feature:** Wave 6 Sprint 1
**Component:** `InventoryDashboard.tsx`
**Difficulty:** Beginner
**Time to Learn:** 10 minutes

## Overview

Real-time LEGO inventory tracking with NFC synchronization, stock alerts, and historical snapshots.

### Key Features

- 4 station cards (Red, Green, Blue, Yellow) with live counts
- Real-time WebSocket integration (<1s latency)
- Stock alert system (low ≤10, critical ≤3)
- NFC sync monitoring (≤5s requirement)
- Manual adjustment with reason logging
- 30-day daily snapshots, 12-week weekly snapshots
- Import/export JSON with validation
- LocalStorage persistence

---

## Quick Start

```typescript
import { InventoryDashboard } from '@/components/InventoryDashboard';

function App() {
  return <InventoryDashboard wsUrl="ws://localhost:4000" />;
}
```

---

## Station Management

### The 4 Stations

```typescript
interface Station {
  color: 'red' | 'green' | 'blue' | 'yellow';
  count: number;
  lowStockThreshold: 10;
  criticalThreshold: 3;
  lastSync: number; // NFC timestamp
}
```

### Stock Alerts

- **Normal**: count > 10 (green)
- **Low Stock**: count ≤ 10 (yellow warning)
- **Critical**: count ≤ 3 (red alert)

---

## Manual Adjustments

```typescript
import { inventoryStorage } from '@/services/inventoryStorage';

// Adjust count with reason
await inventoryStorage.adjustStation('red', 5, 'Found pieces under table');

// View adjustment history
const history = await inventoryStorage.getAdjustmentHistory('red');
```

---

## NFC Sync

Contract requirement: NFC sync must complete within 5 seconds (I-SORT-INV-001)

```typescript
// Monitor sync status
const { lastSync, syncDuration } = useInventorySync();

if (syncDuration > 5000) {
  console.warn('NFC sync exceeded 5s limit!');
}
```

---

## Historical Tracking

- **Daily Snapshots**: Last 30 days
- **Weekly Snapshots**: Last 12 weeks

```typescript
const history = await inventoryStorage.getHistoricalData('red', 'daily');
// Returns array of { date, count } for last 30 days
```

---

## Export/Import

```typescript
// Export
const data = await inventoryStorage.exportInventory();

// Import with validation
await inventoryStorage.importInventory(data, { validate: true });
```

---

## Related Features

- [WebSocket V2](websocket-v2-guide.md)
- [Data Export/Import](data-export-import-guide.md)

---

**Last Updated:** 2026-02-01
**Status:** Production Ready ✅
