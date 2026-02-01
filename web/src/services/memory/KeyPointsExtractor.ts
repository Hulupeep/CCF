/**
 * Key Points Extractor - Extract important facts from conversations using NLP
 *
 * Contract: I-MEMORY-001
 * Uses natural language processing to identify key points, entities, and topics
 */

import { Conversation, ConversationTurn } from '../../types/voice';

interface Entity {
  text: string;
  type: 'person' | 'place' | 'organization' | 'date' | 'time' | 'event' | 'other';
  confidence: number;
}

interface Intent {
  name: string;
  confidence: number;
  parameters?: Record<string, any>;
}

export class KeyPointsExtractor {
  private stopWords = new Set([
    'the',
    'a',
    'an',
    'and',
    'or',
    'but',
    'in',
    'on',
    'at',
    'to',
    'for',
    'of',
    'with',
    'by',
    'from',
    'up',
    'about',
    'into',
    'through',
    'during',
    'before',
    'after',
    'above',
    'below',
    'between',
    'under',
    'is',
    'are',
    'was',
    'were',
    'be',
    'been',
    'being',
    'have',
    'has',
    'had',
    'do',
    'does',
    'did',
    'will',
    'would',
    'could',
    'should',
    'may',
    'might',
    'must',
    'can',
    'i',
    'you',
    'he',
    'she',
    'it',
    'we',
    'they',
    'my',
    'your',
    'his',
    'her',
    'its',
    'our',
    'their',
  ]);

  private actionVerbs = new Set([
    'build',
    'make',
    'create',
    'draw',
    'paint',
    'play',
    'learn',
    'practice',
    'study',
    'read',
    'write',
    'code',
    'design',
    'plan',
    'finish',
    'complete',
    'start',
    'begin',
    'try',
    'want',
    'need',
    'hope',
    'wish',
    'like',
    'love',
    'enjoy',
  ]);

  /**
   * Extract key points from a conversation
   */
  async extractKeyPoints(conversation: Conversation): Promise<string[]> {
    const keyPoints: string[] = [];
    const userTurns = conversation.turns.filter((turn) => turn.speaker === 'user');

    for (const turn of userTurns) {
      // Extract entities
      const entities = await this.extractEntities(turn.text);
      entities.forEach((entity) => {
        if (entity.confidence > 0.7 && entity.type !== 'other') {
          keyPoints.push(`${entity.type}: ${entity.text}`);
        }
      });

      // Extract action phrases
      const actions = this.extractActionPhrases(turn.text);
      keyPoints.push(...actions);

      // Extract important statements
      const statements = this.extractImportantStatements(turn.text);
      keyPoints.push(...statements);
    }

    // Deduplicate and return
    return Array.from(new Set(keyPoints));
  }

  /**
   * Extract named entities from text
   */
  async extractEntities(text: string): Promise<Entity[]> {
    const entities: Entity[] = [];

    // Person names (capitalized words not at sentence start)
    const personPattern = /\b([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)\b/g;
    const sentences = text.split(/[.!?]+/);

    sentences.forEach((sentence) => {
      const trimmed = sentence.trim();
      if (trimmed.length === 0) return;

      const matches = Array.from(trimmed.matchAll(personPattern));
      matches.forEach((match, index) => {
        // Skip first match if it's at sentence start
        if (index === 0 && match.index === 0) return;

        const name = match[1];
        if (!this.isCommonWord(name.toLowerCase())) {
          entities.push({
            text: name,
            type: 'person',
            confidence: 0.8,
          });
        }
      });
    });

    // Dates
    const datePattern = /\b(today|tomorrow|yesterday|monday|tuesday|wednesday|thursday|friday|saturday|sunday|\d{1,2}\/\d{1,2}\/\d{2,4})\b/gi;
    const dateMatches = text.match(datePattern);
    if (dateMatches) {
      dateMatches.forEach((date) => {
        entities.push({
          text: date,
          type: 'date',
          confidence: 0.9,
        });
      });
    }

    // Time
    const timePattern = /\b(\d{1,2}:\d{2}\s*(?:am|pm)?|\d{1,2}\s*(?:am|pm))\b/gi;
    const timeMatches = text.match(timePattern);
    if (timeMatches) {
      timeMatches.forEach((time) => {
        entities.push({
          text: time,
          type: 'time',
          confidence: 0.9,
        });
      });
    }

    // Places (simple pattern - "in/at/to [Place]")
    const placePattern = /\b(?:in|at|to)\s+([A-Z][a-z]+(?:\s+[A-Z][a-z]+)*)\b/g;
    const placeMatches = Array.from(text.matchAll(placePattern));
    placeMatches.forEach((match) => {
      const place = match[1];
      if (!this.isCommonWord(place.toLowerCase())) {
        entities.push({
          text: place,
          type: 'place',
          confidence: 0.7,
        });
      }
    });

    // Events (building, learning, playing, etc.)
    const eventPattern = /\b(building|learning|playing|practicing|studying|working on)\s+(.+?)(?:\.|!|\?|,|$)/gi;
    const eventMatches = Array.from(text.matchAll(eventPattern));
    eventMatches.forEach((match) => {
      entities.push({
        text: `${match[1]} ${match[2]}`.trim(),
        type: 'event',
        confidence: 0.8,
      });
    });

    return entities;
  }

  /**
   * Detect intents from conversation turns
   */
  async detectIntents(turns: ConversationTurn[]): Promise<string[]> {
    const intents: string[] = [];

    for (const turn of turns) {
      if (turn.speaker !== 'user') continue;

      const text = turn.text.toLowerCase();

      // Question intents
      if (text.includes('?')) {
        if (text.includes('what') || text.includes('how')) {
          intents.push('asking_question');
        }
        if (text.includes('can you') || text.includes('will you')) {
          intents.push('requesting_help');
        }
      }

      // Planning intents
      if (
        text.includes('want to') ||
        text.includes('going to') ||
        text.includes('plan to') ||
        text.includes('will')
      ) {
        intents.push('planning_activity');
      }

      // Completion intents
      if (
        text.includes('finished') ||
        text.includes('completed') ||
        text.includes('done')
      ) {
        intents.push('reporting_completion');
      }

      // Learning intents
      if (
        text.includes('learning') ||
        text.includes('studying') ||
        text.includes('practicing')
      ) {
        intents.push('learning_activity');
      }

      // Creating intents
      if (
        text.includes('building') ||
        text.includes('making') ||
        text.includes('creating')
      ) {
        intents.push('creating_something');
      }
    }

    return Array.from(new Set(intents));
  }

  /**
   * Identify topics from conversation
   */
  async identifyTopics(conversation: Conversation): Promise<string[]> {
    const topics: string[] = [];
    const allText = conversation.turns
      .filter((turn) => turn.speaker === 'user')
      .map((turn) => turn.text)
      .join(' ')
      .toLowerCase();

    // Extract significant phrases (2-3 word combinations)
    const words = allText.split(/\s+/).filter((word) => !this.stopWords.has(word));

    // Count word frequency
    const wordFreq = new Map<string, number>();
    words.forEach((word) => {
      wordFreq.set(word, (wordFreq.get(word) || 0) + 1);
    });

    // Get top words
    const topWords = Array.from(wordFreq.entries())
      .filter(([word, count]) => count >= 2 && word.length > 3)
      .sort((a, b) => b[1] - a[1])
      .slice(0, 5)
      .map(([word]) => word);

    topics.push(...topWords);

    // Extract topic keywords
    const topicKeywords = [
      'lego',
      'building',
      'drawing',
      'painting',
      'coding',
      'programming',
      'robot',
      'game',
      'music',
      'guitar',
      'piano',
      'math',
      'science',
      'history',
      'reading',
      'writing',
      'sports',
      'soccer',
      'basketball',
    ];

    topicKeywords.forEach((keyword) => {
      if (allText.includes(keyword)) {
        topics.push(keyword);
      }
    });

    return Array.from(new Set(topics));
  }

  /**
   * Extract action phrases (what user plans to do)
   */
  private extractActionPhrases(text: string): string[] {
    const phrases: string[] = [];
    const lowerText = text.toLowerCase();

    // Pattern: "want to [action]", "going to [action]", "will [action]"
    const patterns = [
      /(?:want to|going to|will|plan to)\s+([^.!?,]+)/gi,
      /(?:i'm|i am)\s+([a-z]+ing)\s+([^.!?,]+)/gi,
    ];

    patterns.forEach((pattern) => {
      const matches = Array.from(text.matchAll(pattern));
      matches.forEach((match) => {
        const phrase = match[1].trim();
        if (phrase.length > 3 && phrase.length < 50) {
          phrases.push(phrase);
        }
      });
    });

    return phrases;
  }

  /**
   * Extract important statements
   */
  private extractImportantStatements(text: string): string[] {
    const statements: string[] = [];

    // Statements with action verbs
    this.actionVerbs.forEach((verb) => {
      const pattern = new RegExp(`\\b${verb}\\s+([^.!?,]{5,40})`, 'gi');
      const matches = Array.from(text.matchAll(pattern));
      matches.forEach((match) => {
        statements.push(`${verb} ${match[1]}`.trim());
      });
    });

    // Statements expressing feelings or opinions
    const emotionPatterns = [
      /\b(?:love|like|enjoy|hate|dislike)\s+([^.!?,]+)/gi,
      /\b(?:excited about|interested in|curious about)\s+([^.!?,]+)/gi,
    ];

    emotionPatterns.forEach((pattern) => {
      const matches = Array.from(text.matchAll(pattern));
      matches.forEach((match) => {
        statements.push(match[0].trim());
      });
    });

    return statements;
  }

  /**
   * Check if word is a common word (not a name)
   */
  private isCommonWord(word: string): boolean {
    const commonWords = new Set([
      'monday',
      'tuesday',
      'wednesday',
      'thursday',
      'friday',
      'saturday',
      'sunday',
      'january',
      'february',
      'march',
      'april',
      'may',
      'june',
      'july',
      'august',
      'september',
      'october',
      'november',
      'december',
    ]);
    return this.stopWords.has(word) || commonWords.has(word);
  }
}

// Export singleton instance
export const keyPointsExtractor = new KeyPointsExtractor();
