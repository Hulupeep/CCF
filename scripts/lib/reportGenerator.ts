/**
 * Report Generator - Generate HTML, Markdown, and JSON coverage reports
 * Part of Issue #81: Journey Coverage Report Tool
 */

import * as fs from 'fs';
import * as path from 'path';
import { JourneyTestMapping } from './testMapper';
import { CoverageMetrics } from './coverageCalculator';

export class ReportGenerator {
  private outputDir: string;

  constructor(outputDir: string = 'docs') {
    this.outputDir = outputDir;
  }

  /**
   * Generate all report formats
   */
  generateAllReports(
    mappings: JourneyTestMapping[],
    metrics: CoverageMetrics
  ): {
    html: string;
    markdown: string;
    json: string;
  } {
    const html = this.generateHTMLReport(mappings, metrics);
    const markdown = this.generateMarkdownReport(mappings, metrics);
    const json = this.generateJSONReport(mappings, metrics);

    // Write reports to files
    fs.writeFileSync(path.join(this.outputDir, 'journey-coverage-report.html'), html);
    fs.writeFileSync(path.join(this.outputDir, 'JOURNEY_COVERAGE.md'), markdown);
    fs.writeFileSync(path.join(this.outputDir, 'journey-coverage.json'), json);

    return { html, markdown, json };
  }

  /**
   * Generate HTML report
   */
  generateHTMLReport(mappings: JourneyTestMapping[], metrics: CoverageMetrics): string {
    const timestamp = new Date().toISOString();

    const statusColor = (status: string) => {
      switch (status) {
        case 'passing': return '#22c55e';
        case 'failing': return '#ef4444';
        case 'not_implemented': return '#94a3b8';
        case 'unknown': return '#f59e0b';
        default: return '#64748b';
      }
    };

    const statusBadge = (status: string) => {
      const color = statusColor(status);
      return `<span style="display: inline-block; padding: 2px 8px; border-radius: 4px; background-color: ${color}20; color: ${color}; font-size: 12px; font-weight: 600;">${status.toUpperCase()}</span>`;
    };

    const progressBar = (value: number, total: number) => {
      const percentage = total > 0 ? (value / total) * 100 : 0;
      return `
        <div style="width: 100%; height: 24px; background-color: #e2e8f0; border-radius: 4px; overflow: hidden;">
          <div style="width: ${percentage}%; height: 100%; background-color: #3b82f6; display: flex; align-items: center; justify-content: center; color: white; font-size: 12px; font-weight: 600;">
            ${percentage.toFixed(1)}%
          </div>
        </div>
      `;
    };

    const criticalityBadge = (criticality: string) => {
      const colors = {
        critical: '#dc2626',
        important: '#f59e0b',
        future: '#64748b',
      };
      return `<span style="display: inline-block; padding: 2px 8px; border-radius: 4px; background-color: ${colors[criticality as keyof typeof colors]}20; color: ${colors[criticality as keyof typeof colors]}; font-size: 11px; font-weight: 600; text-transform: uppercase;">${criticality}</span>`;
    };

    const journeyRows = mappings.map(m => `
      <tr>
        <td style="padding: 12px; border-bottom: 1px solid #e2e8f0;">
          <div style="font-weight: 600; margin-bottom: 4px;">${m.journey.id}</div>
          <div style="font-size: 12px; color: #64748b;">${m.journey.summary}</div>
        </td>
        <td style="padding: 12px; border-bottom: 1px solid #e2e8f0; text-align: center;">
          ${criticalityBadge(m.journey.dod_criticality)}
        </td>
        <td style="padding: 12px; border-bottom: 1px solid #e2e8f0; text-align: center;">
          ${statusBadge(m.status)}
        </td>
        <td style="padding: 12px; border-bottom: 1px solid #e2e8f0;">
          ${m.testFile && m.testFile.exists
            ? `<code style="font-size: 11px; background-color: #f1f5f9; padding: 2px 6px; border-radius: 3px;">${path.basename(m.testFile.path)}</code>`
            : '<span style="color: #94a3b8;">No test file</span>'
          }
        </td>
        <td style="padding: 12px; border-bottom: 1px solid #e2e8f0;">
          ${m.gaps.length > 0
            ? `<ul style="margin: 0; padding-left: 20px; font-size: 12px; color: #ef4444;">${m.gaps.map(g => `<li>${g}</li>`).join('')}</ul>`
            : '<span style="color: #22c55e;">✓ Complete</span>'
          }
        </td>
      </tr>
    `).join('');

    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Journey Test Coverage Report - CCF on RuVector</title>
  <style>
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; line-height: 1.6; color: #1e293b; background-color: #f8fafc; padding: 20px; }
    .container { max-width: 1400px; margin: 0 auto; background-color: white; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); padding: 40px; }
    h1 { font-size: 32px; margin-bottom: 8px; color: #0f172a; }
    .subtitle { color: #64748b; margin-bottom: 32px; }
    .metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 40px; }
    .metric-card { background-color: #f8fafc; border-radius: 8px; padding: 20px; border-left: 4px solid #3b82f6; }
    .metric-value { font-size: 36px; font-weight: 700; color: #0f172a; margin-bottom: 4px; }
    .metric-label { font-size: 14px; color: #64748b; text-transform: uppercase; letter-spacing: 0.5px; }
    .section { margin-bottom: 40px; }
    .section-title { font-size: 20px; font-weight: 600; margin-bottom: 16px; color: #0f172a; }
    table { width: 100%; border-collapse: collapse; margin-bottom: 20px; }
    th { background-color: #f1f5f9; padding: 12px; text-align: left; font-weight: 600; font-size: 13px; text-transform: uppercase; letter-spacing: 0.5px; color: #475569; border-bottom: 2px solid #cbd5e1; }
    .release-status { padding: 20px; border-radius: 8px; margin-bottom: 40px; }
    .release-ready { background-color: #dcfce7; border-left: 4px solid #22c55e; }
    .release-blocked { background-color: #fee2e2; border-left: 4px solid #ef4444; }
    .footer { margin-top: 40px; padding-top: 20px; border-top: 1px solid #e2e8f0; font-size: 12px; color: #94a3b8; }
  </style>
</head>
<body>
  <div class="container">
    <h1>🧪 Journey Test Coverage Report</h1>
    <div class="subtitle">Generated: ${timestamp}</div>

    <div class="release-status ${metrics.release_ready ? 'release-ready' : 'release-blocked'}">
      <div style="font-size: 20px; font-weight: 600; margin-bottom: 8px;">
        ${metrics.release_ready ? '✅ Release Ready' : '⚠️ Release Blocked'}
      </div>
      <div style="font-size: 14px;">
        ${metrics.release_ready
          ? 'All critical journey tests are passing. Ready to release!'
          : `${metrics.release_blockers.length} critical journey test(s) blocking release.`
        }
      </div>
      ${!metrics.release_ready ? `
        <ul style="margin-top: 12px; padding-left: 20px; font-size: 13px;">
          ${metrics.release_blockers.map(b => `<li>${b}</li>`).join('')}
        </ul>
      ` : ''}
    </div>

    <div class="metrics">
      <div class="metric-card">
        <div class="metric-value">${metrics.total_journeys}</div>
        <div class="metric-label">Total Journeys</div>
      </div>
      <div class="metric-card" style="border-left-color: #22c55e;">
        <div class="metric-value" style="color: #22c55e;">${metrics.passing}</div>
        <div class="metric-label">Passing</div>
      </div>
      <div class="metric-card" style="border-left-color: #ef4444;">
        <div class="metric-value" style="color: #ef4444;">${metrics.failing}</div>
        <div class="metric-label">Failing</div>
      </div>
      <div class="metric-card" style="border-left-color: #94a3b8;">
        <div class="metric-value" style="color: #94a3b8;">${metrics.not_implemented}</div>
        <div class="metric-label">Not Implemented</div>
      </div>
    </div>

    <div class="section">
      <div class="section-title">Overall Coverage</div>
      ${progressBar(metrics.implemented, metrics.total_journeys)}
      <div style="margin-top: 8px; font-size: 14px; color: #64748b;">
        ${metrics.implemented} of ${metrics.total_journeys} journeys have tests (${metrics.coverage_percentage.toFixed(1)}%)
      </div>
    </div>

    <div class="section">
      <div class="section-title">Coverage by Criticality</div>
      <div style="display: grid; gap: 20px;">
        <div>
          <div style="display: flex; justify-content: space-between; margin-bottom: 8px;">
            <span style="font-weight: 600;">Critical (Must Pass for Release)</span>
            <span>${metrics.critical_coverage.passing}/${metrics.critical_coverage.total}</span>
          </div>
          ${progressBar(metrics.critical_coverage.passing, metrics.critical_coverage.total)}
        </div>
        <div>
          <div style="display: flex; justify-content: space-between; margin-bottom: 8px;">
            <span style="font-weight: 600;">Important (Should Pass)</span>
            <span>${metrics.important_coverage.passing}/${metrics.important_coverage.total}</span>
          </div>
          ${progressBar(metrics.important_coverage.passing, metrics.important_coverage.total)}
        </div>
        <div>
          <div style="display: flex; justify-content: space-between; margin-bottom: 8px;">
            <span style="font-weight: 600;">Future (Nice to Have)</span>
            <span>${metrics.future_coverage.passing}/${metrics.future_coverage.total}</span>
          </div>
          ${progressBar(metrics.future_coverage.passing, metrics.future_coverage.total)}
        </div>
      </div>
    </div>

    <div class="section">
      <div class="section-title">Journey Test Details</div>
      <table>
        <thead>
          <tr>
            <th>Journey</th>
            <th style="text-align: center;">Criticality</th>
            <th style="text-align: center;">Status</th>
            <th>Test File</th>
            <th>Gaps</th>
          </tr>
        </thead>
        <tbody>
          ${journeyRows}
        </tbody>
      </table>
    </div>

    <div class="footer">
      <div>CCF on RuVector Journey Test Coverage Report</div>
      <div>Generated by Issue #81: Journey Coverage Report Tool</div>
    </div>
  </div>
</body>
</html>`;
  }

  /**
   * Generate Markdown report
   */
  generateMarkdownReport(mappings: JourneyTestMapping[], metrics: CoverageMetrics): string {
    const timestamp = new Date().toISOString();

    const statusEmoji = (status: string) => {
      switch (status) {
        case 'passing': return '✅';
        case 'failing': return '❌';
        case 'not_implemented': return '⚪';
        case 'unknown': return '⚠️';
        default: return '❓';
      }
    };

    const criticalityEmoji = (criticality: string) => {
      switch (criticality) {
        case 'critical': return '🔴';
        case 'important': return '🟡';
        case 'future': return '⚪';
        default: return '⚪';
      }
    };

    const journeyTable = mappings.map(m => {
      const testFile = m.testFile && m.testFile.exists
        ? `\`${path.basename(m.testFile.path)}\``
        : '_No test file_';
      const gaps = m.gaps.length > 0
        ? m.gaps.map(g => `• ${g}`).join('<br>')
        : '✓ Complete';

      return `| ${m.journey.id} | ${criticalityEmoji(m.journey.dod_criticality)} ${m.journey.dod_criticality} | ${statusEmoji(m.status)} ${m.status} | ${testFile} | ${gaps} |`;
    }).join('\n');

    return `# 🧪 Journey Test Coverage Report

**Generated:** ${timestamp}
**Project:** CCF on RuVector

---

## 📊 Summary

${metrics.release_ready ? '✅' : '⚠️'} **Release Status:** ${metrics.release_ready ? 'READY' : 'BLOCKED'}

${!metrics.release_ready ? `
### 🚫 Release Blockers

${metrics.release_blockers.map(b => `- ${b}`).join('\n')}

` : ''}

### Overall Metrics

- **Total Journeys:** ${metrics.total_journeys}
- **Implemented:** ${metrics.implemented} (${metrics.coverage_percentage.toFixed(1)}%)
- **Passing:** ${metrics.passing}
- **Failing:** ${metrics.failing}
- **Not Implemented:** ${metrics.not_implemented}

### Coverage by Criticality

| Criticality | Total | Passing | Coverage |
|-------------|-------|---------|----------|
| 🔴 Critical | ${metrics.critical_coverage.total} | ${metrics.critical_coverage.passing} | ${metrics.critical_coverage.coverage_percentage.toFixed(1)}% |
| 🟡 Important | ${metrics.important_coverage.total} | ${metrics.important_coverage.passing} | ${metrics.important_coverage.coverage_percentage.toFixed(1)}% |
| ⚪ Future | ${metrics.future_coverage.total} | ${metrics.future_coverage.passing} | ${metrics.future_coverage.coverage_percentage.toFixed(1)}% |

---

## 📋 Journey Test Details

| Journey ID | Criticality | Status | Test File | Gaps |
|------------|-------------|--------|-----------|------|
${journeyTable}

---

## 📖 Legend

### Status
- ✅ **passing** - Test exists and passes
- ❌ **failing** - Test exists but fails
- ⚪ **not_implemented** - No test exists
- ⚠️ **unknown** - Test status unclear

### Criticality
- 🔴 **critical** - Must pass for release
- 🟡 **important** - Should pass before release
- ⚪ **future** - Nice to have

---

## 🎯 Definition of Done

All **critical** journey tests must pass before release.

### Critical Journeys
${mappings.filter(m => m.journey.dod_criticality === 'critical').map(m =>
  `- ${statusEmoji(m.status)} ${m.journey.id}: ${m.journey.summary}`
).join('\n')}

### Important Journeys
${mappings.filter(m => m.journey.dod_criticality === 'important').map(m =>
  `- ${statusEmoji(m.status)} ${m.journey.id}: ${m.journey.summary}`
).join('\n')}

---

_Generated by Issue #81: Journey Coverage Report Tool_
`;
  }

  /**
   * Generate JSON report
   */
  generateJSONReport(mappings: JourneyTestMapping[], metrics: CoverageMetrics): string {
    const report = {
      generated_at: new Date().toISOString(),
      metrics,
      journeys: mappings.map(m => ({
        journey_id: m.journey.id,
        summary: m.journey.summary,
        criticality: m.journey.dod_criticality,
        status: m.status,
        test_file: m.testFile?.path || null,
        test_exists: m.testFile?.exists || false,
        test_count: m.testFile?.testCount || 0,
        gaps: m.gaps,
        covers_requirements: m.journey.covers_reqs,
      })),
    };

    return JSON.stringify(report, null, 2);
  }
}
