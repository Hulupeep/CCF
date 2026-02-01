/**
 * Coverage Calculator - Calculate journey test coverage metrics
 * Part of Issue #81: Journey Coverage Report Tool
 */

import { JourneyContract } from './contractParser';
import { JourneyTestMapping, TestStatus } from './testMapper';

export interface CoverageMetrics {
  total_journeys: number;
  implemented: number;
  not_implemented: number;
  passing: number;
  failing: number;
  unknown: number;
  coverage_percentage: number;
  critical_coverage: CriticalityCoverage;
  important_coverage: CriticalityCoverage;
  future_coverage: CriticalityCoverage;
  release_ready: boolean;
  release_blockers: string[];
}

export interface CriticalityCoverage {
  total: number;
  implemented: number;
  passing: number;
  failing: number;
  not_implemented: number;
  coverage_percentage: number;
}

export class CoverageCalculator {
  /**
   * Calculate overall coverage metrics from journey test mappings
   */
  calculateCoverage(mappings: JourneyTestMapping[]): CoverageMetrics {
    const total = mappings.length;
    const implemented = mappings.filter(m => m.status !== 'not_implemented').length;
    const not_implemented = mappings.filter(m => m.status === 'not_implemented').length;
    const passing = mappings.filter(m => m.status === 'passing').length;
    const failing = mappings.filter(m => m.status === 'failing').length;
    const unknown = mappings.filter(m => m.status === 'unknown').length;

    const coverage_percentage = total > 0 ? (implemented / total) * 100 : 0;

    // Calculate criticality-based coverage
    const critical_coverage = this.calculateCriticalityCoverage(
      mappings.filter(m => m.journey.dod_criticality === 'critical')
    );
    const important_coverage = this.calculateCriticalityCoverage(
      mappings.filter(m => m.journey.dod_criticality === 'important')
    );
    const future_coverage = this.calculateCriticalityCoverage(
      mappings.filter(m => m.journey.dod_criticality === 'future')
    );

    // Determine release readiness
    const release_ready = critical_coverage.passing === critical_coverage.total;
    const release_blockers = this.identifyReleaseBlockers(mappings);

    return {
      total_journeys: total,
      implemented,
      not_implemented,
      passing,
      failing,
      unknown,
      coverage_percentage,
      critical_coverage,
      important_coverage,
      future_coverage,
      release_ready,
      release_blockers,
    };
  }

  /**
   * Calculate coverage metrics for a specific criticality level
   */
  private calculateCriticalityCoverage(mappings: JourneyTestMapping[]): CriticalityCoverage {
    const total = mappings.length;
    const implemented = mappings.filter(m => m.status !== 'not_implemented').length;
    const passing = mappings.filter(m => m.status === 'passing').length;
    const failing = mappings.filter(m => m.status === 'failing').length;
    const not_implemented = mappings.filter(m => m.status === 'not_implemented').length;
    const coverage_percentage = total > 0 ? (implemented / total) * 100 : 0;

    return {
      total,
      implemented,
      passing,
      failing,
      not_implemented,
      coverage_percentage,
    };
  }

  /**
   * Identify critical journeys that are blocking release
   */
  private identifyReleaseBlockers(mappings: JourneyTestMapping[]): string[] {
    const blockers: string[] = [];

    const criticalJourneys = mappings.filter(
      m => m.journey.dod_criticality === 'critical'
    );

    for (const mapping of criticalJourneys) {
      if (mapping.status === 'not_implemented') {
        blockers.push(`${mapping.journey.id}: Test not implemented`);
      } else if (mapping.status === 'failing') {
        blockers.push(`${mapping.journey.id}: Test failing`);
      } else if (mapping.status === 'unknown') {
        blockers.push(`${mapping.journey.id}: Test status unknown`);
      }
    }

    return blockers;
  }

  /**
   * Calculate coverage trend (if historical data available)
   */
  calculateTrend(
    current: CoverageMetrics,
    previous?: CoverageMetrics
  ): {
    delta_percentage: number;
    delta_implemented: number;
    direction: 'improving' | 'declining' | 'stable';
  } {
    if (!previous) {
      return {
        delta_percentage: 0,
        delta_implemented: 0,
        direction: 'stable',
      };
    }

    const delta_percentage = current.coverage_percentage - previous.coverage_percentage;
    const delta_implemented = current.implemented - previous.implemented;

    let direction: 'improving' | 'declining' | 'stable' = 'stable';
    if (delta_percentage > 1) direction = 'improving';
    else if (delta_percentage < -1) direction = 'declining';

    return {
      delta_percentage,
      delta_implemented,
      direction,
    };
  }

  /**
   * Generate a simple text summary of coverage
   */
  generateSummary(metrics: CoverageMetrics): string {
    const lines: string[] = [];

    lines.push('Journey Test Coverage Summary');
    lines.push('='.repeat(50));
    lines.push(`Total Journeys: ${metrics.total_journeys}`);
    lines.push(`Implemented: ${metrics.implemented} (${metrics.coverage_percentage.toFixed(1)}%)`);
    lines.push(`Passing: ${metrics.passing}`);
    lines.push(`Failing: ${metrics.failing}`);
    lines.push(`Not Implemented: ${metrics.not_implemented}`);
    lines.push('');
    lines.push('By Criticality:');
    lines.push(`  Critical: ${metrics.critical_coverage.passing}/${metrics.critical_coverage.total} passing (${metrics.critical_coverage.coverage_percentage.toFixed(1)}%)`);
    lines.push(`  Important: ${metrics.important_coverage.passing}/${metrics.important_coverage.total} passing (${metrics.important_coverage.coverage_percentage.toFixed(1)}%)`);
    lines.push(`  Future: ${metrics.future_coverage.passing}/${metrics.future_coverage.total} passing (${metrics.future_coverage.coverage_percentage.toFixed(1)}%)`);
    lines.push('');
    lines.push(`Release Ready: ${metrics.release_ready ? 'YES ✓' : 'NO ✗'}`);

    if (metrics.release_blockers.length > 0) {
      lines.push('');
      lines.push('Release Blockers:');
      metrics.release_blockers.forEach(blocker => {
        lines.push(`  - ${blocker}`);
      });
    }

    return lines.join('\n');
  }
}
