/**
 * Tests for Journey Coverage Report Tool
 * Issue #81: STORY-TEST-003
 */

import { describe, test, expect } from '@jest/globals';
import * as fs from 'fs';
import * as path from 'path';
import { ContractParser } from '../scripts/lib/contractParser';
import { TestMapper } from '../scripts/lib/testMapper';
import { CoverageCalculator } from '../scripts/lib/coverageCalculator';

describe('Journey Coverage Report Tool', () => {
  describe('ContractParser', () => {
    test('should parse CONTRACT_INDEX.yml', () => {
      const parser = new ContractParser('docs/contracts');
      const index = parser.parseContractIndex();

      expect(index).toBeDefined();
      expect(index.metadata).toBeDefined();
      expect(index.metadata.project).toBe('mbot-ruvector');
      expect(index.contracts).toBeDefined();
      expect(Array.isArray(index.contracts)).toBe(true);
    });

    test('should extract journey contracts', () => {
      const parser = new ContractParser('docs/contracts');
      const journeys = parser.extractJourneyContracts();

      expect(Array.isArray(journeys)).toBe(true);
      expect(journeys.length).toBeGreaterThan(0);

      // Verify journey structure
      journeys.forEach(journey => {
        expect(journey.id).toMatch(/^J-/);
        expect(journey.type).toBe('e2e');
        expect(['critical', 'important', 'future']).toContain(journey.dod_criticality);
      });
    });

    test('should group journeys by criticality', () => {
      const parser = new ContractParser('docs/contracts');
      const grouped = parser.getJourneysByCriticality();

      expect(grouped.critical).toBeDefined();
      expect(grouped.important).toBeDefined();
      expect(grouped.future).toBeDefined();

      expect(Array.isArray(grouped.critical)).toBe(true);
      expect(Array.isArray(grouped.important)).toBe(true);
      expect(Array.isArray(grouped.future)).toBe(true);
    });

    test('should get DOD requirements', () => {
      const parser = new ContractParser('docs/contracts');
      const dod = parser.getDODRequirements();

      expect(dod).toBeDefined();
      expect(dod.critical_journeys).toBeDefined();
      expect(Array.isArray(dod.critical_journeys)).toBe(true);
      expect(dod.critical_journeys.length).toBeGreaterThan(0);
    });
  });

  describe('TestMapper', () => {
    test('should map journey to test file', () => {
      const parser = new ContractParser('docs/contracts');
      const journeys = parser.extractJourneyContracts();
      const mapper = new TestMapper('.', 'tests/journeys');

      const firstJourney = journeys[0];
      const mapping = mapper.mapJourneyToTest(firstJourney);

      expect(mapping).toBeDefined();
      expect(mapping.journey).toBe(firstJourney);
      expect(['passing', 'failing', 'not_implemented', 'unknown']).toContain(mapping.status);
    });

    test('should find existing test files', () => {
      const parser = new ContractParser('docs/contracts');
      const journeys = parser.extractJourneyContracts();
      const mapper = new TestMapper('.', 'tests/journeys');

      const mappings = journeys.map(j => mapper.mapJourneyToTest(j));
      const foundTests = mappings.filter(m => m.testFile && m.testFile.exists);

      // We know at least some test files exist
      expect(foundTests.length).toBeGreaterThan(0);
    });

    test('should identify gaps in coverage', () => {
      const parser = new ContractParser('docs/contracts');
      const journeys = parser.extractJourneyContracts();
      const mapper = new TestMapper('.', 'tests/journeys');

      const mappings = journeys.map(j => mapper.mapJourneyToTest(j));

      mappings.forEach(mapping => {
        expect(Array.isArray(mapping.gaps)).toBe(true);

        if (mapping.status === 'not_implemented') {
          expect(mapping.gaps.length).toBeGreaterThan(0);
          expect(mapping.gaps[0]).toBe('Test file not found');
        }
      });
    });
  });

  describe('CoverageCalculator', () => {
    test('should calculate coverage metrics', () => {
      const parser = new ContractParser('docs/contracts');
      const journeys = parser.extractJourneyContracts();
      const mapper = new TestMapper('.', 'tests/journeys');
      const mappings = journeys.map(j => mapper.mapJourneyToTest(j));

      const calculator = new CoverageCalculator();
      const metrics = calculator.calculateCoverage(mappings);

      expect(metrics).toBeDefined();
      expect(metrics.total_journeys).toBe(journeys.length);
      expect(metrics.implemented).toBeGreaterThanOrEqual(0);
      expect(metrics.not_implemented).toBeGreaterThanOrEqual(0);
      expect(metrics.passing).toBeGreaterThanOrEqual(0);
      expect(metrics.failing).toBeGreaterThanOrEqual(0);
      expect(metrics.coverage_percentage).toBeGreaterThanOrEqual(0);
      expect(metrics.coverage_percentage).toBeLessThanOrEqual(100);
    });

    test('should calculate criticality-specific metrics', () => {
      const parser = new ContractParser('docs/contracts');
      const journeys = parser.extractJourneyContracts();
      const mapper = new TestMapper('.', 'tests/journeys');
      const mappings = journeys.map(j => mapper.mapJourneyToTest(j));

      const calculator = new CoverageCalculator();
      const metrics = calculator.calculateCoverage(mappings);

      expect(metrics.critical_coverage).toBeDefined();
      expect(metrics.important_coverage).toBeDefined();
      expect(metrics.future_coverage).toBeDefined();

      expect(metrics.critical_coverage.total).toBeGreaterThan(0);
      expect(metrics.critical_coverage.coverage_percentage).toBeGreaterThanOrEqual(0);
    });

    test('should determine release readiness', () => {
      const parser = new ContractParser('docs/contracts');
      const journeys = parser.extractJourneyContracts();
      const mapper = new TestMapper('.', 'tests/journeys');
      const mappings = journeys.map(j => mapper.mapJourneyToTest(j));

      const calculator = new CoverageCalculator();
      const metrics = calculator.calculateCoverage(mappings);

      expect(typeof metrics.release_ready).toBe('boolean');
      expect(Array.isArray(metrics.release_blockers)).toBe(true);

      // If not release ready, should have blockers
      if (!metrics.release_ready) {
        expect(metrics.release_blockers.length).toBeGreaterThan(0);
      }
    });

    test('should generate text summary', () => {
      const parser = new ContractParser('docs/contracts');
      const journeys = parser.extractJourneyContracts();
      const mapper = new TestMapper('.', 'tests/journeys');
      const mappings = journeys.map(j => mapper.mapJourneyToTest(j));

      const calculator = new CoverageCalculator();
      const metrics = calculator.calculateCoverage(mappings);
      const summary = calculator.generateSummary(metrics);

      expect(typeof summary).toBe('string');
      expect(summary).toContain('Journey Test Coverage Summary');
      expect(summary).toContain('Total Journeys');
      expect(summary).toContain('Release Ready');
    });
  });

  describe('Report Generation', () => {
    test('should generate all report files', () => {
      // Check that report files exist after running the tool
      const htmlPath = path.join('docs', 'journey-coverage-report.html');
      const mdPath = path.join('docs', 'JOURNEY_COVERAGE.md');
      const jsonPath = path.join('docs', 'journey-coverage.json');

      expect(fs.existsSync(htmlPath)).toBe(true);
      expect(fs.existsSync(mdPath)).toBe(true);
      expect(fs.existsSync(jsonPath)).toBe(true);
    });

    test('should generate valid HTML report', () => {
      const htmlPath = path.join('docs', 'journey-coverage-report.html');
      const html = fs.readFileSync(htmlPath, 'utf-8');

      expect(html).toContain('<!DOCTYPE html>');
      expect(html).toContain('Journey Test Coverage Report');
      expect(html).toContain('Release Ready');
      expect(html).toContain('Coverage');
    });

    test('should generate valid Markdown report', () => {
      const mdPath = path.join('docs', 'JOURNEY_COVERAGE.md');
      const md = fs.readFileSync(mdPath, 'utf-8');

      expect(md).toContain('# 🧪 Journey Test Coverage Report');
      expect(md).toContain('## 📊 Summary');
      expect(md).toContain('Release Status');
      expect(md).toContain('| Journey ID |');
    });

    test('should generate valid JSON report', () => {
      const jsonPath = path.join('docs', 'journey-coverage.json');
      const json = fs.readFileSync(jsonPath, 'utf-8');
      const data = JSON.parse(json);

      expect(data.generated_at).toBeDefined();
      expect(data.metrics).toBeDefined();
      expect(data.journeys).toBeDefined();
      expect(Array.isArray(data.journeys)).toBe(true);
    });
  });

  describe('Integration', () => {
    test('should complete full workflow without errors', () => {
      const parser = new ContractParser('docs/contracts');
      const journeys = parser.extractJourneyContracts();
      const mapper = new TestMapper('.', 'tests/journeys');
      const mappings = journeys.map(j => mapper.mapJourneyToTest(j));
      const calculator = new CoverageCalculator();
      const metrics = calculator.calculateCoverage(mappings);

      expect(journeys.length).toBeGreaterThan(0);
      expect(mappings.length).toBe(journeys.length);
      expect(metrics.total_journeys).toBe(journeys.length);
    });

    test('should match total counts', () => {
      const parser = new ContractParser('docs/contracts');
      const journeys = parser.extractJourneyContracts();
      const mapper = new TestMapper('.', 'tests/journeys');
      const mappings = journeys.map(j => mapper.mapJourneyToTest(j));
      const calculator = new CoverageCalculator();
      const metrics = calculator.calculateCoverage(mappings);

      // Verify count consistency
      expect(metrics.implemented + metrics.not_implemented).toBe(metrics.total_journeys);
      expect(metrics.passing + metrics.failing + metrics.not_implemented).toBe(metrics.total_journeys);
    });
  });
});
