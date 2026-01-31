# Inventory Dashboard

Real-time LEGO Sorter inventory tracking dashboard with NFC integration.

## Issue
- **GitHub Issue**: #74 - STORY-SORT-010: Inventory Dashboard with NFC

## Contracts
- **SORT-004**: Inventory Must Persist (localStorage)
- **SORT-006**: Inventory tracking system
- **I-SORT-INV-001**: NFC sync every 5 seconds max

## Journey
- **J-HELP-LEGO-SORT**: User manages LEGO inventory

## Features

### 1. Real-Time Station Monitoring
- 4 station cards (Red, Green, Blue, Yellow)
- Live count updates via WebSocket
- Capacity percentage visualization
- Visual flash effect on updates

### 2. Stock Alerts
- Configurable low stock threshold (default: 20)
- Low stock warnings (yellow alert)
- Critical stock alerts (red alert, count = 0)
- Per-station alert badges

### 3. Manual Adjustments
- Adjust individual station counts
- Mandatory reason for adjustments
- Adjustment history tracking
- Validation for non-negative values

### 4. NFC Integration
- Real-time NFC status display
- Last sync timestamp
- Sync interval monitoring (≤5s per I-SORT-INV-001)
- Failed sync counter

### 5. Historical Trends
- Daily snapshots (last 30 days)
- Weekly snapshots (last 12 weeks)
- Chart visualization placeholder
- Automatic pruning of old data

### 6. Data Import/Export
- Export inventory as JSON
- Import previously exported data
- Data validation on import
- Includes history and metadata

### 7. Reset Functions
- Reset individual stations
- Reset all stations at once
- Confirmation dialogs for safety

## File Structure

```
web/src/
├── components/
│   ├── InventoryDashboard.tsx       # Main component
│   ├── InventoryDashboard.css       # Styles
│   └── __tests__/
│       └── InventoryDashboard.test.tsx  # Unit tests
├── services/
│   └── inventoryStorage.ts          # LocalStorage service
└── types/
    └── inventory.ts                 # Type definitions

tests/journeys/
└── lego-sorter-inventory.journey.spec.ts  # E2E journey tests
```

## Usage

### Basic Usage

```tsx
import { InventoryDashboard } from './components/InventoryDashboard';

function App() {
  return (
    <InventoryDashboard websocketUrl="ws://localhost:8081" />
  );
}
```

### WebSocket Messages

#### Inventory Update
```json
{
  "type": "inventory_update",
  "payload": {
    "stationId": "red",
    "count": 42,
    "timestamp": 1643723400000
  }
}
```

#### NFC Status
```json
{
  "type": "nfc_status",
  "payload": {
    "isConnected": true,
    "lastSync": 1643723400000,
    "syncInterval": 5000,
    "failedSyncs": 0
  }
}
```

#### Bulk Update
```json
{
  "type": "bulk_update",
  "payload": {
    "stations": {
      "red": 50,
      "green": 30,
      "blue": 70,
      "yellow": 20
    }
  }
}
```

## Data Contracts

### Station
```typescript
interface Station {
  id: 'red' | 'green' | 'blue' | 'yellow';
  color: string;
  count: number;
  capacity: number;
  lastUpdated: number;
  nfcTagId?: string;
  nfcLastSync?: number;
}
```

### Export Format
```typescript
interface InventoryExport {
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
```

## data-testid Attributes

All elements have `data-testid` attributes for testing:

| Element | data-testid | Purpose |
|---------|-------------|---------|
| Dashboard root | `inventory-dashboard` | Main container |
| Station cards | `station-card-{color}` | Individual station cards |
| Station counts | `station-count-{color}` | Count display |
| Capacity bars | `capacity-bar-{color}` | Visual capacity indicator |
| Reset buttons | `reset-button-{color}` | Reset station |
| Adjust buttons | `adjust-button-{color}` | Open adjustment modal |
| WebSocket status | `websocket-status` | Connection indicator |
| NFC status | `nfc-status` | NFC connection indicator |
| NFC last sync | `nfc-last-sync` | Last sync timestamp |
| Alerts container | `inventory-alerts` | Alert messages |
| Individual alerts | `alert-{color}` | Per-station alerts |
| Export button | `export-button` | Export JSON |
| Import button | `import-button` | Import JSON |
| Reset all button | `reset-all-button` | Reset all stations |
| Threshold input | `threshold-input` | Low stock threshold |
| Adjust modal | `adjust-modal` | Adjustment modal |
| Adjust value input | `adjust-value-input` | New count input |
| Adjust reason input | `adjust-reason-input` | Reason input |
| Adjust confirm | `adjust-confirm-button` | Confirm adjustment |
| Export modal | `export-modal` | Export modal |
| Import modal | `import-modal` | Import modal |
| Import file input | `import-file-input` | File upload |

## LocalStorage Keys

- `mbot_inventory_stations` - Current station data
- `mbot_inventory_history` - Historical snapshots
- `mbot_inventory_adjustments` - Manual adjustment log

## Testing

### Run Unit Tests
```bash
npm test -- InventoryDashboard.test.tsx
```

### Run Journey Tests
```bash
npm run test:journeys -- lego-sorter-inventory.journey.spec.ts
```

## Acceptance Criteria (Gherkin)

### ✅ Scenario: View Inventory Dashboard
```gherkin
Given LEGO sorter is running
When I navigate to /inventory
Then I see 4 station cards
And each shows current count
And each shows capacity %
```

### ✅ Scenario: Real-Time Update
```gherkin
When robot drops piece in Red
Then Red count updates within 1s
And card flashes
```

## Performance

- Updates: 20Hz WebSocket (50ms interval)
- Flash animation: 1 second duration
- History retention: 30 days daily, 12 weeks weekly
- Adjustment log: Last 100 entries

## Accessibility

- Semantic HTML structure
- Color contrast ratios meet WCAG AA
- Keyboard navigation support
- Screen reader friendly labels

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Mobile browsers (responsive design)

## Dependencies

- React 19.2+
- WebSocket API (native)
- localStorage API (native)

## Future Enhancements

- Historical trend visualization with charts
- CSV export format
- Multi-language support
- Barcode scanner integration
- Advanced analytics dashboard
