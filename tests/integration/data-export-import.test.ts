/**
 * Data Export/Import Integration Tests
 * Contract: ARCH-005, LEARN-007
 * Journey: Data Export/Import System
 * Tests: Export/Import validation logic
 */

const {
  validateExportManifest,
  validatePersonalityData,
  validateDrawingData,
  validateGameStatsData,
  validateInventoryData,
  EXPORT_VERSION,
} = require('../../web/src/types/exportManifest');

describe('Data Export/Import System - Validation', () => {
  describe('Export Manifest Validation', () => {
    it('should validate correct manifest structure', () => {
      const manifest = {
        version: '1.0.0',
        exportedAt: Date.now(),
        dataTypes: ['personality'],
        metadata: {},
        data: {
          personalities: [],
        },
      };

      expect(validateExportManifest(manifest)).toBe(true);
    });

    it('should reject manifest with missing fields', () => {
      const invalid = {
        version: '1.0.0',
        // Missing exportedAt, dataTypes, data
      };

      expect(validateExportManifest(invalid)).toBe(false);
    });

    it('should reject manifest with incompatible version', () => {
      const manifest = {
        version: '2.0.0', // Different major version
        exportedAt: Date.now(),
        dataTypes: ['personality'],
        metadata: {},
        data: {
          personalities: [],
        },
      };

      expect(validateExportManifest(manifest)).toBe(false);
    });

    it('should reject manifest with invalid data types', () => {
      const manifest = {
        version: '1.0.0',
        exportedAt: Date.now(),
        dataTypes: ['invalid_type'],
        metadata: {},
        data: {},
      };

      expect(validateExportManifest(manifest)).toBe(false);
    });

    it('should reject manifest with mismatched dataTypes and data keys', () => {
      const manifest = {
        version: '1.0.0',
        exportedAt: Date.now(),
        dataTypes: ['personality'],
        metadata: {},
        data: {
          // Missing personalities key
        },
      };

      expect(validateExportManifest(manifest)).toBe(false);
    });

    it('should accept manifest with all data types', () => {
      const manifest = {
        version: '1.0.0',
        exportedAt: Date.now(),
        dataTypes: ['personality', 'drawings', 'stats', 'inventory'],
        metadata: {},
        data: {
          personalities: [],
          drawings: [],
          stats: {},
          inventory: {},
        },
      };

      expect(validateExportManifest(manifest)).toBe(true);
    });
  });

  describe('Personality Data Validation', () => {
    it('should validate correct personality config (I-PERS-001)', () => {
      const validConfig = {
        tension_baseline: 0.5,
        coherence_baseline: 0.5,
        energy_baseline: 0.5,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      const errors = validatePersonalityData(validConfig);
      expect(errors).toHaveLength(0);
    });

    it('should reject personality with out-of-range values (I-PERS-001)', () => {
      const invalidConfig = {
        tension_baseline: 1.5, // Out of range
        coherence_baseline: 0.5,
        energy_baseline: 0.5,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      const errors = validatePersonalityData(invalidConfig);
      expect(errors.length).toBeGreaterThan(0);
      expect(errors.some(e => e.message.includes('I-PERS-001'))).toBe(true);
    });

    it('should reject personality with negative values (I-PERS-001)', () => {
      const invalidConfig = {
        tension_baseline: -0.1, // Negative
        coherence_baseline: 0.5,
        energy_baseline: 0.5,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      const errors = validatePersonalityData(invalidConfig);
      expect(errors.length).toBeGreaterThan(0);
    });

    it('should reject personality with missing required fields', () => {
      const incompleteConfig = {
        tension_baseline: 0.5,
        // Missing other fields
      };

      const errors = validatePersonalityData(incompleteConfig);
      expect(errors.length).toBeGreaterThan(0);
    });

    it('should reject personality with non-number values', () => {
      const invalidConfig = {
        tension_baseline: '0.5', // String instead of number
        coherence_baseline: 0.5,
        energy_baseline: 0.5,
        startle_sensitivity: 0.5,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      const errors = validatePersonalityData(invalidConfig);
      expect(errors.length).toBeGreaterThan(0);
    });

    it('should validate boundary values 0.0 and 1.0', () => {
      const boundaryConfig = {
        tension_baseline: 0.0,
        coherence_baseline: 1.0,
        energy_baseline: 0.0,
        startle_sensitivity: 1.0,
        recovery_speed: 0.5,
        curiosity_drive: 0.5,
        movement_expressiveness: 0.5,
        sound_expressiveness: 0.5,
        light_expressiveness: 0.5,
      };

      const errors = validatePersonalityData(boundaryConfig);
      expect(errors).toHaveLength(0);
    });
  });

  describe('Drawing Data Validation', () => {
    it('should validate correct drawing structure', () => {
      const validDrawing = {
        id: 'test-1',
        createdAt: Date.now(),
        strokes: [],
        moods: [],
        duration: 1000,
        dominantMood: 'Calm',
        hasSignature: false,
        metadata: {
          startMood: 'Calm',
          endMood: 'Calm',
          averageTension: 0.5,
          averageCoherence: 0.5,
          averageEnergy: 0.5,
          strokeCount: 0,
          totalPathLength: 0,
        },
      };

      const errors = validateDrawingData(validDrawing);
      expect(errors).toHaveLength(0);
    });

    it('should reject drawing with missing id', () => {
      const invalidDrawing = {
        createdAt: Date.now(),
        strokes: [],
        metadata: {},
      };

      const errors = validateDrawingData(invalidDrawing);
      expect(errors.length).toBeGreaterThan(0);
      expect(errors.some(e => e.field === 'id')).toBe(true);
    });

    it('should reject drawing with invalid createdAt', () => {
      const invalidDrawing = {
        id: 'test-1',
        createdAt: 'not-a-timestamp',
        strokes: [],
        metadata: {},
      };

      const errors = validateDrawingData(invalidDrawing);
      expect(errors.some(e => e.field === 'createdAt')).toBe(true);
    });

    it('should reject drawing with missing strokes array', () => {
      const invalidDrawing = {
        id: 'test-1',
        createdAt: Date.now(),
        metadata: {},
      };

      const errors = validateDrawingData(invalidDrawing);
      expect(errors.some(e => e.field === 'strokes')).toBe(true);
    });

    it('should reject drawing with missing metadata', () => {
      const invalidDrawing = {
        id: 'test-1',
        createdAt: Date.now(),
        strokes: [],
      };

      const errors = validateDrawingData(invalidDrawing);
      expect(errors.some(e => e.field === 'metadata')).toBe(true);
    });
  });

  describe('Game Statistics Validation', () => {
    it('should validate correct game stats structure', () => {
      const validStats = {
        totalGames: 10,
        byGame: {
          tictactoe: {
            totalGames: 10,
            wins: 5,
            losses: 3,
            draws: 2,
          },
        },
        achievements: [],
        leaderboard: [],
        sessions: [],
      };

      const errors = validateGameStatsData(validStats);
      expect(errors).toHaveLength(0);
    });

    it('should reject stats with missing totalGames', () => {
      const invalidStats = {
        byGame: {},
        sessions: [],
      };

      const errors = validateGameStatsData(invalidStats);
      expect(errors.some(e => e.field === 'totalGames')).toBe(true);
    });

    it('should reject stats with invalid byGame object', () => {
      const invalidStats = {
        totalGames: 10,
        byGame: 'not-an-object',
        sessions: [],
      };

      const errors = validateGameStatsData(invalidStats);
      expect(errors.some(e => e.field === 'byGame')).toBe(true);
    });

    it('should reject stats with missing sessions array', () => {
      const invalidStats = {
        totalGames: 10,
        byGame: {},
      };

      const errors = validateGameStatsData(invalidStats);
      expect(errors.some(e => e.field === 'sessions')).toBe(true);
    });
  });

  describe('Inventory Data Validation', () => {
    it('should validate correct inventory structure', () => {
      const validInventory = {
        version: '1.0',
        exportedAt: Date.now(),
        stations: [
          { id: 'red', color: '#EF4444', count: 25, capacity: 100, lastUpdated: Date.now() },
          { id: 'green', color: '#10B981', count: 30, capacity: 100, lastUpdated: Date.now() },
          { id: 'blue', color: '#3B82F6', count: 40, capacity: 100, lastUpdated: Date.now() },
          { id: 'yellow', color: '#F59E0B', count: 35, capacity: 100, lastUpdated: Date.now() },
        ],
        history: { daily: [], weekly: [] },
        metadata: {
          totalPieces: 130,
          lastSortTime: Date.now(),
          sortCount: 5,
        },
      };

      const errors = validateInventoryData(validInventory);
      expect(errors).toHaveLength(0);
    });

    it('should reject inventory with wrong version', () => {
      const invalidInventory = {
        version: '2.0',
        stations: [],
      };

      const errors = validateInventoryData(invalidInventory);
      expect(errors.some(e => e.field === 'version')).toBe(true);
    });

    it('should reject inventory with wrong number of stations', () => {
      const invalidInventory = {
        version: '1.0',
        stations: [
          { id: 'red', count: 25, capacity: 100 },
          { id: 'green', count: 30, capacity: 100 },
          // Missing blue and yellow
        ],
      };

      const errors = validateInventoryData(invalidInventory);
      expect(errors.some(e => e.field === 'stations')).toBe(true);
    });

    it('should reject inventory with missing stations', () => {
      const invalidInventory = {
        version: '1.0',
        stations: [
          { id: 'red', count: 25, capacity: 100, lastUpdated: Date.now() },
          { id: 'green', count: 30, capacity: 100, lastUpdated: Date.now() },
          { id: 'blue', count: 40, capacity: 100, lastUpdated: Date.now() },
          { id: 'purple', count: 35, capacity: 100, lastUpdated: Date.now() }, // Wrong station
        ],
      };

      const errors = validateInventoryData(invalidInventory);
      expect(errors.some(e => e.message.includes('Missing station: yellow'))).toBe(true);
    });
  });

  describe('Version Compatibility', () => {
    it('should export with correct version constant', () => {
      expect(EXPORT_VERSION).toBe('1.0.0');
    });

    it('should accept same major version', () => {
      const manifest = {
        version: '1.5.0', // Same major version
        exportedAt: Date.now(),
        dataTypes: ['personality'],
        metadata: {},
        data: {
          personalities: [],
        },
      };

      expect(validateExportManifest(manifest)).toBe(true);
    });

    it('should reject different major version', () => {
      const manifest = {
        version: '2.0.0',
        exportedAt: Date.now(),
        dataTypes: ['personality'],
        metadata: {},
        data: {
          personalities: [],
        },
      };

      expect(validateExportManifest(manifest)).toBe(false);
    });
  });

  describe('Data Type Coverage', () => {
    it('should support personality data type', () => {
      const manifest = {
        version: '1.0.0',
        exportedAt: Date.now(),
        dataTypes: ['personality'],
        metadata: {},
        data: { personalities: [] },
      };
      expect(validateExportManifest(manifest)).toBe(true);
    });

    it('should support drawings data type', () => {
      const manifest = {
        version: '1.0.0',
        exportedAt: Date.now(),
        dataTypes: ['drawings'],
        metadata: {},
        data: { drawings: [] },
      };
      expect(validateExportManifest(manifest)).toBe(true);
    });

    it('should support stats data type', () => {
      const manifest = {
        version: '1.0.0',
        exportedAt: Date.now(),
        dataTypes: ['stats'],
        metadata: {},
        data: { stats: {} },
      };
      expect(validateExportManifest(manifest)).toBe(true);
    });

    it('should support inventory data type', () => {
      const manifest = {
        version: '1.0.0',
        exportedAt: Date.now(),
        dataTypes: ['inventory'],
        metadata: {},
        data: { inventory: {} },
      };
      expect(validateExportManifest(manifest)).toBe(true);
    });
  });
});
