/**
 * Marketplace Validation Tests
 * Issue #85 - I-CLOUD-004: Validation before publication
 */

import { validatePersonalityForMarketplace, validateRating, validateReport } from '../validation';
import { createDefaultConfig } from '../../../types/personality';

describe('Marketplace Validation', () => {
  describe('validatePersonalityForMarketplace', () => {
    const validConfig = createDefaultConfig();
    const validName = 'Test Personality';
    const validDescription = 'This is a test personality for unit testing purposes.';
    const validTags = ['test', 'energetic'];

    test('accepts valid personality', () => {
      const result = validatePersonalityForMarketplace(
        validName,
        validDescription,
        validTags,
        validConfig
      );

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    test('rejects name too short', () => {
      const result = validatePersonalityForMarketplace('AB', validDescription, validTags, validConfig);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Name must be at least 3 characters');
    });

    test('rejects name too long', () => {
      const longName = 'A'.repeat(51);
      const result = validatePersonalityForMarketplace(
        longName,
        validDescription,
        validTags,
        validConfig
      );

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Name must not exceed 50 characters');
    });

    test('rejects description too short', () => {
      const result = validatePersonalityForMarketplace(validName, 'Short', validTags, validConfig);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Description must be at least 10 characters');
    });

    test('rejects description too long', () => {
      const longDesc = 'A'.repeat(501);
      const result = validatePersonalityForMarketplace(validName, longDesc, validTags, validConfig);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Description must not exceed 500 characters');
    });

    test('rejects empty tags', () => {
      const result = validatePersonalityForMarketplace(validName, validDescription, [], validConfig);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('At least one tag is required');
    });

    test('rejects too many tags', () => {
      const manyTags = Array.from({ length: 11 }, (_, i) => `tag${i}`);
      const result = validatePersonalityForMarketplace(
        validName,
        validDescription,
        manyTags,
        validConfig
      );

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Maximum 10 tags allowed');
    });

    test('rejects invalid tag characters', () => {
      const result = validatePersonalityForMarketplace(
        validName,
        validDescription,
        ['valid-tag', 'Invalid_Tag'],
        validConfig
      );

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.includes('Invalid_Tag'))).toBe(true);
    });

    test('rejects invalid personality config (out of bounds)', () => {
      const invalidConfig = { ...validConfig, tension_baseline: 1.5 };
      const result = validatePersonalityForMarketplace(
        validName,
        validDescription,
        validTags,
        invalidConfig
      );

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Personality configuration is invalid (parameters must be 0.0-1.0)');
    });

    test('rejects inappropriate content', () => {
      const result = validatePersonalityForMarketplace(
        'Spam Personality',
        validDescription,
        validTags,
        validConfig
      );

      expect(result.valid).toBe(false);
      expect(result.errors.some((e) => e.includes('spam'))).toBe(true);
    });
  });

  describe('validateRating', () => {
    test('accepts valid ratings 1-5', () => {
      expect(validateRating(1)).toBe(true);
      expect(validateRating(2)).toBe(true);
      expect(validateRating(3)).toBe(true);
      expect(validateRating(4)).toBe(true);
      expect(validateRating(5)).toBe(true);
    });

    test('rejects ratings out of bounds', () => {
      expect(validateRating(0)).toBe(false);
      expect(validateRating(6)).toBe(false);
      expect(validateRating(-1)).toBe(false);
    });

    test('rejects non-integer ratings', () => {
      expect(validateRating(3.5)).toBe(false);
      expect(validateRating(4.7)).toBe(false);
    });
  });

  describe('validateReport', () => {
    test('accepts valid report reason', () => {
      const result = validateReport('This personality contains inappropriate content.');

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    test('rejects reason too short', () => {
      const result = validateReport('Bad');

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Report reason must be at least 10 characters');
    });

    test('rejects reason too long', () => {
      const longReason = 'A'.repeat(501);
      const result = validateReport(longReason);

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Report reason must not exceed 500 characters');
    });
  });
});
