/**
 * Marketplace Validation Service
 * Issue #85 - I-CLOUD-004: Validation before publication
 */

import { PersonalityConfig, validatePersonalityConfig } from '../../types/personality';
import { ValidationResult } from '../../types/marketplace';

const MIN_NAME_LENGTH = 3;
const MAX_NAME_LENGTH = 50;
const MIN_DESCRIPTION_LENGTH = 10;
const MAX_DESCRIPTION_LENGTH = 500;
const MAX_TAGS = 10;
const ALLOWED_TAG_CHARS = /^[a-z0-9-]+$/;

export function validatePersonalityForMarketplace(
  name: string,
  description: string,
  tags: string[],
  config: PersonalityConfig
): ValidationResult {
  const errors: string[] = [];

  // Validate name
  if (!name || name.trim().length < MIN_NAME_LENGTH) {
    errors.push(`Name must be at least ${MIN_NAME_LENGTH} characters`);
  }
  if (name.length > MAX_NAME_LENGTH) {
    errors.push(`Name must not exceed ${MAX_NAME_LENGTH} characters`);
  }

  // Validate description
  if (!description || description.trim().length < MIN_DESCRIPTION_LENGTH) {
    errors.push(`Description must be at least ${MIN_DESCRIPTION_LENGTH} characters`);
  }
  if (description.length > MAX_DESCRIPTION_LENGTH) {
    errors.push(`Description must not exceed ${MAX_DESCRIPTION_LENGTH} characters`);
  }

  // Validate tags
  if (tags.length === 0) {
    errors.push('At least one tag is required');
  }
  if (tags.length > MAX_TAGS) {
    errors.push(`Maximum ${MAX_TAGS} tags allowed`);
  }
  tags.forEach((tag) => {
    if (!ALLOWED_TAG_CHARS.test(tag)) {
      errors.push(`Tag "${tag}" contains invalid characters (use lowercase, numbers, and hyphens only)`);
    }
  });

  // Validate personality config (ARCH-004 contract compliance)
  if (!validatePersonalityConfig(config)) {
    errors.push('Personality configuration is invalid (parameters must be 0.0-1.0)');
  }

  // Check for inappropriate content (basic filter)
  const inappropriateWords = ['spam', 'test123', 'asdf']; // Extend as needed
  const textToCheck = `${name} ${description} ${tags.join(' ')}`.toLowerCase();
  inappropriateWords.forEach((word) => {
    if (textToCheck.includes(word)) {
      errors.push(`Content contains inappropriate term: "${word}"`);
    }
  });

  return {
    valid: errors.length === 0,
    errors,
  };
}

export function validateRating(rating: number): boolean {
  return Number.isInteger(rating) && rating >= 1 && rating <= 5;
}

export function validateReport(reason: string): ValidationResult {
  const errors: string[] = [];

  if (!reason || reason.trim().length < 10) {
    errors.push('Report reason must be at least 10 characters');
  }
  if (reason.length > 500) {
    errors.push('Report reason must not exceed 500 characters');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}
