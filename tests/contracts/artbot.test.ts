/**
 * Contract Test: ArtBot Shape Drawing (ART-003)
 *
 * This test enforces the contract in docs/contracts/feature_artbot.yml
 * Validates shape drawing invariants:
 * - I-ART-003: Shape closure tolerance (5mm)
 * - I-ART-003a: Geometric accuracy (10% variance)
 * - ARCH-ART-002: Command queuing
 * - ARCH-ART-003: Paper bounds
 * - ARCH-001: no_std compatibility
 * - ARCH-002: Deterministic rendering
 *
 * Location: tests/contracts/artbot.test.ts
 */

import { describe, it, expect } from '@jest/globals'
import * as fs from 'fs'
import * as path from 'path'
import * as glob from 'glob'

// Helper: Find all files matching scope patterns
function getFilesInScope(patterns: string[], basePath: string): string[] {
  const includes: string[] = []
  const excludes: string[] = []

  for (const pattern of patterns) {
    if (pattern.startsWith('!')) {
      excludes.push(pattern.slice(1))
    } else {
      includes.push(pattern)
    }
  }

  const files: string[] = []
  for (const pattern of includes) {
    const matches = glob.sync(pattern, {
      cwd: basePath,
      absolute: true,
      ignore: excludes,
    })
    files.push(...matches)
  }

  return [...new Set(files)]
}

// Helper: Find pattern matches with line numbers
function findPatternViolations(
  content: string,
  pattern: RegExp,
  filePath: string
): Array<{ file: string; line: number; match: string }> {
  const violations: Array<{ file: string; line: number; match: string }> = []
  const lines = content.split('\n')

  lines.forEach((line, index) => {
    if (pattern.test(line)) {
      violations.push({
        file: filePath,
        line: index + 1,
        match: line.trim(),
      })
    }
  })

  return violations
}

// Helper: Check if file contains pattern
function fileContainsPattern(filePath: string, pattern: RegExp): boolean {
  if (!fs.existsSync(filePath)) return false
  const content = fs.readFileSync(filePath, 'utf-8')
  return pattern.test(content)
}

describe('Contract: feature_artbot (ART-003)', () => {
  const basePath = path.resolve(__dirname, '../..')
  const artbotScope = ['crates/mbot-core/src/artbot/**/*.rs']

  describe('I-ART-003: Shape Closure Tolerance (5mm)', () => {
    it('Shape code defines CLOSURE_TOLERANCE constant', () => {
      const files = getFilesInScope(artbotScope, basePath)

      if (files.length === 0) {
        console.warn('No artbot files found yet - contract will be enforced when implemented')
        return
      }

      let hasClosureTolerance = false

      for (const file of files) {
        if (fileContainsPattern(file, /CLOSURE_TOLERANCE.*5\.0|closure_gap.*<.*5\.0/)) {
          hasClosureTolerance = true
          break
        }
      }

      if (!hasClosureTolerance) {
        throw new Error(
          `CONTRACT VIOLATION: I-ART-003\n` +
            `Shape code must define CLOSURE_TOLERANCE_MM = 5.0\n` +
            `and validate that closed shapes close within 5mm.\n\n` +
            `Fix: Add const CLOSURE_TOLERANCE_MM: f32 = 5.0;\n` +
            `Contract: docs/contracts/feature_artbot.yml`
        )
      }
    })

    it('ShapeResult struct includes closure_gap field', () => {
      const files = getFilesInScope(artbotScope, basePath)

      if (files.length === 0) return

      let hasClosureGap = false

      for (const file of files) {
        if (fileContainsPattern(file, /closure_gap\s*:/)) {
          hasClosureGap = true
          break
        }
      }

      if (!hasClosureGap) {
        throw new Error(
          `CONTRACT VIOLATION: I-ART-003\n` +
            `ShapeResult must include closure_gap field.\n\n` +
            `Fix: Add closure_gap: f32 to ShapeResult struct\n` +
            `Contract: docs/contracts/feature_artbot.yml`
        )
      }
    })
  })

  describe('I-ART-003a: Geometric Accuracy (10% variance)', () => {
    it('Shape code defines VARIANCE_TOLERANCE constant', () => {
      const files = getFilesInScope(artbotScope, basePath)

      if (files.length === 0) return

      let hasVarianceTolerance = false

      for (const file of files) {
        if (fileContainsPattern(file, /VARIANCE_TOLERANCE.*0\.1|path_variance.*<.*0\.1/)) {
          hasVarianceTolerance = true
          break
        }
      }

      if (!hasVarianceTolerance) {
        throw new Error(
          `CONTRACT VIOLATION: I-ART-003a\n` +
            `Shape code must define VARIANCE_TOLERANCE = 0.10 (10%)\n` +
            `and validate that shape paths stay within tolerance.\n\n` +
            `Fix: Add const VARIANCE_TOLERANCE: f32 = 0.10;\n` +
            `Contract: docs/contracts/feature_artbot.yml`
        )
      }
    })

    it('ShapeResult struct includes path_variance field', () => {
      const files = getFilesInScope(artbotScope, basePath)

      if (files.length === 0) return

      let hasPathVariance = false

      for (const file of files) {
        if (fileContainsPattern(file, /path_variance\s*:/)) {
          hasPathVariance = true
          break
        }
      }

      if (!hasPathVariance) {
        throw new Error(
          `CONTRACT VIOLATION: I-ART-003a\n` +
            `ShapeResult must include path_variance field.\n\n` +
            `Fix: Add path_variance: f32 to ShapeResult struct\n` +
            `Contract: docs/contracts/feature_artbot.yml`
        )
      }
    })
  })

  describe('ARCH-ART-002: Command Queuing', () => {
    it('No immediate execution patterns', () => {
      const files = getFilesInScope(artbotScope, basePath)
      const violations: Array<{ file: string; line: number; match: string }> = []

      const forbiddenPatterns = [
        { pattern: /execute_now/, reason: 'immediate execution' },
        { pattern: /immediate_draw/, reason: 'immediate draw' },
        { pattern: /direct_motor/, reason: 'direct motor control' },
      ]

      for (const file of files) {
        if (!fs.existsSync(file)) continue
        const content = fs.readFileSync(file, 'utf-8')

        for (const { pattern } of forbiddenPatterns) {
          const matches = findPatternViolations(content, pattern, file)
          violations.push(...matches)
        }
      }

      if (violations.length > 0) {
        const message =
          `CONTRACT VIOLATION: ARCH-ART-002\n` +
          `Drawing commands must be queued, not executed immediately:\n\n` +
          violations
            .map(
              (v) =>
                `  ${v.file.replace(basePath, '')}:${v.line}\n` + `    ${v.match}\n`
            )
            .join('\n') +
          `\nFix: Return Vec<DrawCommand> instead of executing directly.\n` +
          `Contract: docs/contracts/feature_artbot.yml`

        throw new Error(message)
      }
    })

    it('Uses DrawCommand pattern', () => {
      const files = getFilesInScope(artbotScope, basePath)

      if (files.length === 0) return

      let usesCommandPattern = false

      for (const file of files) {
        if (fileContainsPattern(file, /DrawCommand|commands_generated|CommandQueue/)) {
          usesCommandPattern = true
          break
        }
      }

      if (!usesCommandPattern) {
        throw new Error(
          `CONTRACT VIOLATION: ARCH-ART-002\n` +
            `Shape drawing must use command queue pattern.\n\n` +
            `Fix: Define DrawCommand enum and return Vec<DrawCommand>\n` +
            `Contract: docs/contracts/feature_artbot.yml`
        )
      }
    })
  })

  describe('ARCH-ART-003: Paper Bounds', () => {
    it('Shape code includes bounds checking', () => {
      const files = getFilesInScope(artbotScope, basePath)

      if (files.length === 0) return

      let hasBoundsCheck = false

      for (const file of files) {
        if (
          fileContainsPattern(
            file,
            /PaperBounds|bounds_check|constrain_to_bounds|clamp.*bounds|boundary_violations/
          )
        ) {
          hasBoundsCheck = true
          break
        }
      }

      if (!hasBoundsCheck) {
        throw new Error(
          `CONTRACT VIOLATION: ARCH-ART-003\n` +
            `All drawing movements must respect physical paper bounds.\n\n` +
            `Fix: Add PaperBounds struct and bounds checking to shape functions\n` +
            `Contract: docs/contracts/feature_artbot.yml`
        )
      }
    })
  })

  describe('ARCH-001: no_std Compatibility', () => {
    it('No std:: imports in artbot module', () => {
      const files = getFilesInScope(artbotScope, basePath)
      const violations: Array<{ file: string; line: number; match: string }> = []

      for (const file of files) {
        if (!fs.existsSync(file)) continue
        const content = fs.readFileSync(file, 'utf-8')

        // Skip lines that are cfg-gated for std
        const lines = content.split('\n')
        let inStdBlock = false

        lines.forEach((line, index) => {
          if (line.includes('#[cfg(not(feature = "no_std"))]')) {
            inStdBlock = true
            return
          }
          if (inStdBlock && line.trim().startsWith('}')) {
            inStdBlock = false
            return
          }

          if (!inStdBlock && /use std::/.test(line) && !line.includes('cfg')) {
            violations.push({
              file,
              line: index + 1,
              match: line.trim(),
            })
          }
        })
      }

      if (violations.length > 0) {
        const message =
          `CONTRACT VIOLATION: ARCH-001\n` +
          `artbot module must be no_std compatible:\n\n` +
          violations
            .map(
              (v) =>
                `  ${v.file.replace(basePath, '')}:${v.line}\n` + `    ${v.match}\n`
            )
            .join('\n') +
          `\nFix: Use core:: or alloc:: instead of std::\n` +
          `Contract: docs/contracts/feature_artbot.yml`

        throw new Error(message)
      }
    })

    it('No println! in artbot module', () => {
      const files = getFilesInScope(artbotScope, basePath)
      const violations: Array<{ file: string; line: number; match: string }> = []

      for (const file of files) {
        if (!fs.existsSync(file)) continue
        const content = fs.readFileSync(file, 'utf-8')

        const matches = findPatternViolations(content, /println!/, file)
        // Filter out test code
        const nonTestMatches = matches.filter(
          (m) => !m.match.includes('#[test]') && !m.match.includes('mod tests')
        )
        violations.push(...nonTestMatches)
      }

      if (violations.length > 0) {
        const message =
          `CONTRACT VIOLATION: ARCH-001\n` +
          `println! is not available in no_std:\n\n` +
          violations
            .map(
              (v) =>
                `  ${v.file.replace(basePath, '')}:${v.line}\n` + `    ${v.match}\n`
            )
            .join('\n') +
          `\nFix: Remove println! or use conditional compilation\n` +
          `Contract: docs/contracts/feature_artbot.yml`

        throw new Error(message)
      }
    })
  })

  describe('ARCH-002: Deterministic Rendering', () => {
    it('No non-deterministic sources', () => {
      const files = getFilesInScope(artbotScope, basePath)
      const violations: Array<{ file: string; line: number; match: string }> = []

      const forbiddenPatterns = [
        { pattern: /rand::thread_rng/, reason: 'non-deterministic RNG' },
        { pattern: /SystemTime::now/, reason: 'non-deterministic time' },
        { pattern: /Instant::now/, reason: 'non-deterministic time (use passed timestamp)' },
      ]

      for (const file of files) {
        if (!fs.existsSync(file)) continue
        const content = fs.readFileSync(file, 'utf-8')

        for (const { pattern } of forbiddenPatterns) {
          const matches = findPatternViolations(content, pattern, file)
          violations.push(...matches)
        }
      }

      if (violations.length > 0) {
        const message =
          `CONTRACT VIOLATION: ARCH-002\n` +
          `Shape rendering must be deterministic:\n\n` +
          violations
            .map(
              (v) =>
                `  ${v.file.replace(basePath, '')}:${v.line}\n` + `    ${v.match}\n`
            )
            .join('\n') +
          `\nFix: Use seeded RNG and passed timestamps for determinism\n` +
          `Contract: docs/contracts/feature_artbot.yml`

        throw new Error(message)
      }
    })

    it('Scribble requires seed parameter', () => {
      const files = getFilesInScope(artbotScope, basePath)

      for (const file of files) {
        if (!fs.existsSync(file)) continue
        const content = fs.readFileSync(file, 'utf-8')

        // If scribble is implemented, it must take a seed
        if (/fn\s+.*scribble/i.test(content)) {
          if (!/seed\s*:\s*u64|seed\s*:\s*u32/.test(content)) {
            throw new Error(
              `CONTRACT VIOLATION: ARCH-002\n` +
                `Scribble function must take a seed parameter for determinism.\n\n` +
                `Fix: Add seed: u64 parameter to scribble function\n` +
                `Contract: docs/contracts/feature_artbot.yml`
            )
          }
        }
      }
    })
  })

  describe('Shape Enum Definition', () => {
    it('Defines required shape types', () => {
      const files = getFilesInScope(artbotScope, basePath)

      if (files.length === 0) return

      const requiredShapes = ['Circle', 'Spiral', 'Line', 'Arc', 'Scribble']
      let foundShapes: string[] = []

      for (const file of files) {
        if (!fs.existsSync(file)) continue
        const content = fs.readFileSync(file, 'utf-8')

        for (const shape of requiredShapes) {
          if (new RegExp(`${shape}\\s*[{,]|${shape}\\s*$`, 'm').test(content)) {
            foundShapes.push(shape)
          }
        }
      }

      foundShapes = [...new Set(foundShapes)]
      const missingShapes = requiredShapes.filter((s) => !foundShapes.includes(s))

      if (missingShapes.length > 0 && foundShapes.length > 0) {
        // Only fail if some shapes exist but others are missing
        throw new Error(
          `CONTRACT VIOLATION: ART-003\n` +
            `Missing required shape types: ${missingShapes.join(', ')}\n` +
            `Found: ${foundShapes.join(', ')}\n\n` +
            `Fix: Add missing shape variants to ShapeType enum\n` +
            `Contract: docs/contracts/feature_artbot.yml`
        )
      }
    })
  })

  describe('Compliance Checklist', () => {
    it('Documents compliance questions', () => {
      const checklist = [
        {
          question: 'Adding a closed shape (circle, spiral)?',
          action: 'Validate closure gap < 5mm (I-ART-003)',
        },
        {
          question: 'Adding any shape drawing?',
          action: 'Ensure path variance < 10% (I-ART-003a)',
        },
        {
          question: 'Adding movement commands?',
          action: 'Queue commands, check bounds (ARCH-ART-002, ARCH-ART-003)',
        },
        {
          question: 'Using math functions?',
          action: 'Use libm/core versions for no_std (ARCH-001)',
        },
      ]

      // Verify checklist is complete
      expect(checklist.length).toBeGreaterThan(0)
      checklist.forEach((item) => {
        expect(item.question).toBeDefined()
        expect(item.action).toBeDefined()
      })
    })
  })
})
