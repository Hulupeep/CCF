/**
 * News Service Module
 * Exports all news-related services and utilities
 */

export { NewsAPIClient, createNewsAPIClient } from './NewsAPIClient';
export { NewsPreferencesManager, getNewsPreferencesManager } from './NewsPreferencesManager';
export { NewsService, getNewsService } from './NewsService';
export { NewsBriefingGenerator, getNewsBriefingGenerator } from './NewsBriefingGenerator';

// Re-export types for convenience
export type {
  NewsPreferences,
  NewsArticle,
  NewsResponse,
  TopHeadlinesParams,
  SearchParams,
  UserFeedback,
  BriefingSection
} from '../../types/voice';
