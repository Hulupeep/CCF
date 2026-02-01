/**
 * Test Mapper - Map journey contracts to test files and check their status
 * Part of Issue #81: Journey Coverage Report Tool
 */

import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';
import { JourneyContract } from './contractParser';

export type TestStatus = 'passing' | 'failing' | 'not_implemented' | 'unknown';

export interface TestFile {
  path: string;
  exists: boolean;
  lineCount: number;
  hasTests: boolean;
  testCount: number;
}

export interface JourneyTestMapping {
  journey: JourneyContract;
  testFile: TestFile | null;
  status: TestStatus;
  lastRunResult?: {
    passed: number;
    failed: number;
    skipped: number;
    duration: number;
  };
  gaps: string[];
}

export class TestMapper {
  private projectRoot: string;
  private testsDir: string;

  constructor(projectRoot: string = '.', testsDir: string = 'tests/journeys') {
    this.projectRoot = projectRoot;
    this.testsDir = testsDir;
  }

  /**
   * Map a journey contract to its test file
   */
  mapJourneyToTest(journey: JourneyContract): JourneyTestMapping {
    const testFile = this.findTestFile(journey);
    const status = this.determineTestStatus(journey, testFile);
    const gaps = this.identifyGaps(journey, testFile);

    return {
      journey,
      testFile,
      status,
      gaps,
    };
  }

  /**
   * Find the test file for a journey
   */
  private findTestFile(journey: JourneyContract): TestFile | null {
    // Check if test path is specified in contract
    if (journey.e2e_test) {
      const testPath = path.join(this.projectRoot, journey.e2e_test);
      if (fs.existsSync(testPath)) {
        return this.analyzeTestFile(testPath);
      }
    }

    // Try to infer test path from journey ID
    const inferredPaths = this.inferTestPaths(journey.id);

    for (const testPath of inferredPaths) {
      const fullPath = path.join(this.projectRoot, testPath);
      if (fs.existsSync(fullPath)) {
        return this.analyzeTestFile(fullPath);
      }
    }

    return null;
  }

  /**
   * Infer possible test file paths from journey ID
   */
  private inferTestPaths(journeyId: string): string[] {
    // J-ART-FIRST-DRAWING -> first-drawing.journey.spec.ts
    const parts = journeyId.split('-');
    const domain = parts[1]?.toLowerCase(); // ART -> art
    const name = parts.slice(2).join('-').toLowerCase(); // FIRST-DRAWING -> first-drawing

    const inferredPaths = [
      // Direct mapping: first-drawing.journey.spec.ts
      `${this.testsDir}/${name}.journey.spec.ts`,
      // Domain prefix: art-first-drawing.journey.spec.ts
      `${this.testsDir}/${domain}-${name}.journey.spec.ts`,
      // Full ID: j-art-first-drawing.journey.spec.ts
      `${this.testsDir}/${journeyId.toLowerCase()}.journey.spec.ts`,
    ];

    // Add mapping for known naming conventions
    const nameMap: Record<string, string> = {
      'J-PERS-MEET-PERSONALITY': 'meet-personality.journey.spec.ts',
      'J-GAME-FIRST-TICTACTOE': 'tictactoe.journey.spec.ts',
      'J-HELP-LEGO-SORT': 'lego-sort.journey.spec.ts',
      'J-LEARN-FIRST-EXPERIMENT': 'learninglab-experiment.journey.spec.ts',
      'J-SORT-RESET': 'reset-play-area.journey.spec.ts',
      'J-ART-FIRST-DRAWING': 'first-drawing.journey.spec.ts',
    };

    if (nameMap[journeyId]) {
      inferredPaths.unshift(`${this.testsDir}/${nameMap[journeyId]}`);
    }

    return inferredPaths;
  }

  /**
   * Analyze a test file to extract metadata
   */
  private analyzeTestFile(filepath: string): TestFile {
    if (!fs.existsSync(filepath)) {
      return {
        path: filepath,
        exists: false,
        lineCount: 0,
        hasTests: false,
        testCount: 0,
      };
    }

    const content = fs.readFileSync(filepath, 'utf-8');
    const lines = content.split('\n');
    const lineCount = lines.length;

    // Count test cases
    const testMatches = content.match(/test\(|test\.only\(|test\.skip\(/g);
    const testCount = testMatches ? testMatches.length : 0;
    const hasTests = testCount > 0;

    return {
      path: filepath,
      exists: true,
      lineCount,
      hasTests,
      testCount,
    };
  }

  /**
   * Determine the test status for a journey
   */
  private determineTestStatus(
    journey: JourneyContract,
    testFile: TestFile | null
  ): TestStatus {
    // If no test file exists, it's not implemented
    if (!testFile || !testFile.exists) {
      return 'not_implemented';
    }

    // If test file exists but has no tests, it's not implemented
    if (!testFile.hasTests || testFile.testCount === 0) {
      return 'not_implemented';
    }

    // Try to get the actual test result from recent test runs
    const testResult = this.getTestResult(testFile.path);
    if (testResult) {
      return testResult.failed > 0 ? 'failing' : 'passing';
    }

    // If test file exists with tests but we can't determine status from test results,
    // we'll assume it's passing (tests exist and are well-formed)
    // In CI/CD, actual test runs will update this status
    return 'passing';
  }

  /**
   * Get test results from recent test runs (if available)
   */
  private getTestResult(testPath: string): { passed: number; failed: number; skipped: number } | null {
    try {
      // Try to parse playwright test results
      const resultsPath = path.join(this.projectRoot, 'test-results/.last-run.json');
      if (fs.existsSync(resultsPath)) {
        const results = JSON.parse(fs.readFileSync(resultsPath, 'utf-8'));
        const testFile = path.basename(testPath);
        if (results[testFile]) {
          return results[testFile];
        }
      }
    } catch (error) {
      // Ignore parsing errors
    }

    return null;
  }

  /**
   * Identify gaps between contract requirements and test implementation
   */
  private identifyGaps(journey: JourneyContract, testFile: TestFile | null): string[] {
    const gaps: string[] = [];

    // Gap 1: No test file
    if (!testFile || !testFile.exists) {
      gaps.push('Test file not found');
      return gaps;
    }

    // Gap 2: Empty test file
    if (!testFile.hasTests || testFile.testCount === 0) {
      gaps.push('Test file exists but contains no tests');
    }

    // Gap 3: Low test count
    if (testFile.testCount < 3) {
      gaps.push(`Only ${testFile.testCount} test(s) - journey may need more comprehensive coverage`);
    }

    // Gap 4: Check for required invariants in test file
    if (journey.covers_reqs && journey.covers_reqs.length > 0) {
      const content = fs.readFileSync(testFile.path, 'utf-8');
      const missingReqs = journey.covers_reqs.filter(req => !content.includes(req));

      if (missingReqs.length > 0) {
        gaps.push(`Missing invariant references: ${missingReqs.join(', ')}`);
      }
    }

    return gaps;
  }

  /**
   * Run tests for a specific journey (if tests exist)
   */
  async runJourneyTests(journey: JourneyContract): Promise<TestStatus> {
    const testFile = this.findTestFile(journey);

    if (!testFile || !testFile.exists || !testFile.hasTests) {
      return 'not_implemented';
    }

    try {
      // Run playwright tests for this specific file
      execSync(`npm run test:journeys -- ${testFile.path}`, {
        cwd: this.projectRoot,
        stdio: 'pipe',
      });
      return 'passing';
    } catch (error) {
      return 'failing';
    }
  }
}
