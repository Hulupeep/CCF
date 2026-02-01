/**
 * Voice Recognition and Speaker Identification Types
 *
 * Contract: I-VOICE-001 through I-VOICE-006
 * Implements multi-user voice profiles with ≥85% identification confidence
 */

// Voice Recognition Types
export interface VoiceProfile {
  userId: string;
  name: string;
  voiceSamples: VoiceSample[];
  voiceprint: Float32Array; // Biometric voice signature
  confidence: number; // 0.0 - 1.0
  enrolledAt: number;
  lastUsed: number;
  useCount: number;
  metadata: {
    age?: number;
    isChild: boolean;
    language: string;
  };
}

export interface VoiceSample {
  id: string;
  audioData: ArrayBuffer; // WAV/MP3 audio
  duration: number; // milliseconds
  sampleRate: number;
  recordedAt: number;
  phrase: string; // What was said
}

export interface VoiceIdentification {
  userId: string | null;
  confidence: number; // 0.0 - 1.0
  alternativeMatches: Array<{ userId: string; confidence: number }>;
  isAnonymous: boolean;
  timestamp: number;
}

// Personal Assistant Types
export interface PersonalBriefing {
  userId: string;
  timestamp: number;
  sections: BriefingSection[];
  spokenText: string; // Full text to speak aloud
  duration: number; // Estimated speaking time in seconds
}

export interface BriefingSection {
  type: 'greeting' | 'news' | 'email' | 'calendar' | 'memory' | 'question' | 'weather';
  title: string;
  content: string;
  priority: 'high' | 'medium' | 'low';
  metadata?: Record<string, any>;
  articles?: NewsArticle[]; // For news sections
}

// News Integration
export interface NewsPreferences {
  userId: string;
  topics: string[]; // ['technology', 'sports', 'science', ...]
  sources: string[]; // ['bbc', 'reuters', 'techcrunch', ...]
  excludeTopics: string[];
  maxArticles: number;
  readingLevel?: 'child' | 'teen' | 'adult';
  topicWeights: Record<string, number>; // For learning preferences
  lastUpdated: number;
}

export interface NewsArticle {
  id: string;
  headline: string;
  summary: string;
  content?: string;
  source: string;
  category: string;
  author?: string;
  publishedAt: number;
  url: string;
  imageUrl?: string;
  relevanceScore: number; // 0.0 - 1.0 (personalized)
}

export interface NewsResponse {
  status: string;
  totalResults: number;
  articles: NewsArticle[];
}

export interface TopHeadlinesParams {
  country?: string;
  category?: string;
  sources?: string;
  q?: string;
  pageSize?: number;
  page?: number;
}

export interface SearchParams {
  q: string;
  sources?: string;
  domains?: string;
  excludeDomains?: string;
  from?: string;
  to?: string;
  language?: string;
  sortBy?: 'relevancy' | 'popularity' | 'publishedAt';
  pageSize?: number;
  page?: number;
}

export interface UserFeedback {
  articleId: string;
  action: 'read' | 'skip' | 'like' | 'dislike';
  duration?: number; // Time spent reading
  timestamp: number;
}

// Email Integration
export interface EmailAccount {
  userId: string;
  provider: 'gmail' | 'outlook' | 'yahoo' | 'custom';
  email: string;
  accessToken: string; // OAuth2 token (encrypted)
  refreshToken: string; // OAuth2 refresh token (encrypted)
  expiresAt: number;
  lastSynced: number;
}

export interface EmailSummary {
  totalUnread: number;
  importantCount: number;
  topSenders: string[];
  categories: {
    work: number;
    personal: number;
    promotions: number;
    social: number;
  };
  highlights: EmailHighlight[];
}

export interface EmailHighlight {
  from: string;
  subject: string;
  snippet: string;
  importance: 'high' | 'medium' | 'low';
  timestamp: number;
}

// Conversational Memory
export interface ConversationMemory {
  userId: string;
  conversations: Conversation[];
  activities: DailyActivity[];
  preferences: UserPreferences;
  relationships: Relationship[];
}

export interface Conversation {
  id: string;
  timestamp: number;
  turns: ConversationTurn[];
  topic: string;
  sentiment: 'positive' | 'neutral' | 'negative';
  keyPoints: string[]; // Important facts mentioned
}

export interface ConversationTurn {
  speaker: 'user' | 'mbot';
  text: string;
  timestamp: number;
  intent?: string;
  entities?: Record<string, any>;
}

export interface DailyActivity {
  date: string; // YYYY-MM-DD
  userId: string;
  plannedActivities: string[]; // What user said they'd do
  completedActivities: string[]; // What was confirmed done
  notes: string;
  mood?: string;
}

export interface UserPreferences {
  userId: string;
  newsTopics: string[];
  contentRestrictions: ContentRestriction[];
  privacySettings: PrivacySettings;
  personalDetails: {
    nickname?: string;
    birthday?: string;
    interests: string[];
    occupation?: string;
  };
}

export interface ContentRestriction {
  type: 'age_appropriate' | 'no_politics' | 'no_violence' | 'family_friendly';
  enabled: boolean;
}

export interface PrivacySettings {
  allowVoiceRecording: boolean;
  allowEmailAccess: boolean;
  allowNewsPersonalization: boolean;
  shareDataWithFamily: boolean;
  retentionDays: number; // How long to keep conversation history
}

export interface Relationship {
  userId: string;
  relatedUserId: string;
  relationship: 'parent' | 'child' | 'sibling' | 'other';
  notes?: string;
}

// Follow-up Questions
export interface FollowUpQuestion {
  id: string;
  userId: string;
  question: string;
  context: string; // What triggered this question
  priority: number; // 0-100
  validUntil: number; // Timestamp when question becomes stale
  answered: boolean;
  answer?: string;
}

// Voice Command Types (from contract)
export interface VoiceCommand {
  pattern: RegExp;
  action: string;
  description: string;
  requiresConfirmation: boolean;
  handler: (params: any) => Promise<void>;
}

export interface CommandResult {
  recognized: boolean;
  confidence: number; // 0-1
  command?: string;
  params?: any;
  action?: string;
  error?: string;
}

export interface VoiceSettings {
  enabled: boolean;
  wakeWordEnabled: boolean;
  wakeWord: string; // default "hey robot"
  language: string; // default "en-US"
  continuous: boolean;
  interimResults: boolean;
  audioFeedback: boolean;
  visualFeedback: boolean; // always true
  noiseThreshold: number; // dB
  confidenceThreshold: number; // 0-1
}

export interface VoiceCommandHistory {
  id: string;
  transcript: string;
  recognized: boolean;
  confidence: number;
  action?: string;
  timestamp: number;
  executionTime: number; // ms
}

// Enrollment Types
export interface EnrollmentState {
  step: number;
  name: string;
  samples: VoiceSample[];
  recording: boolean;
  completed: boolean;
}

export interface EnrollmentPhrase {
  text: string;
  recorded: boolean;
  attempt: number;
}

// News API Constants
export const NEWS_TOPICS = [
  'technology',
  'sports',
  'science',
  'politics',
  'entertainment',
  'business',
  'health',
  'world',
  'environment',
  'education'
] as const;

export type NewsTopic = typeof NEWS_TOPICS[number];

export const NEWS_SOURCES = {
  bbc: 'BBC News',
  cnn: 'CNN',
  reuters: 'Reuters',
  'the-verge': 'The Verge',
  techcrunch: 'TechCrunch',
  'national-geographic': 'National Geographic',
  espn: 'ESPN',
  'associated-press': 'Associated Press'
} as const;

// Default factory functions
export function createDefaultNewsPreferences(userId: string): NewsPreferences {
  return {
    userId,
    topics: ['technology', 'science'],
    sources: [],
    excludeTopics: [],
    maxArticles: 5,
    readingLevel: 'adult',
    topicWeights: {},
    lastUpdated: Date.now()
  };
}

// Storage keys
export const VOICE_ASSISTANT_STORAGE_KEYS = {
  NEWS_PREFERENCES: 'mbot_news_preferences',
  BRIEFING_HISTORY: 'mbot_briefing_history',
  USER_PROFILES: 'mbot_user_profiles',
  CONVERSATION_MEMORY: 'mbot_conversation_memory'
} as const;
