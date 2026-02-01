# Data Export/Import Guide

**Feature:** Wave 6 Sprint 2
**Difficulty:** Beginner
**Time:** 10 minutes

## Overview
Export and import all app data (personalities, drawings, stats, inventory) in JSON/CSV formats with validation.

## Quick Start
\`\`\`typescript
import { dataExport, dataImport } from '@/services';

// Export all data
const manifest = await dataExport.exportAllData();
const json = JSON.stringify(manifest, null, 2);

// Import with validation
await dataImport.importFromManifest(manifest, {
  strategy: 'merge', // or 'overwrite'
  skipInvalid: true
});
\`\`\`

## Formats
- **JSON**: All data types, includes metadata
- **CSV**: Single data type, spreadsheet compatible

## Backup Management
\`\`\`typescript
// Create backup
await dataExport.createBackup('before-update');

// List backups
const backups = await dataExport.listBackups();

// Restore backup
await dataImport.restoreBackup('before-update');
\`\`\`

## API
See: [Data Export/Import API](../api/WAVE_6_APIs.md#data-export-import)

**Last Updated:** 2026-02-01
