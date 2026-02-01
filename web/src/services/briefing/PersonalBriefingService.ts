/**
 * Personal Briefing Service - Component 5/5 (#95)
 *
 * Orchestrates voice recognition, news, email, and memory into personalized daily briefings
 * Contract: I-VOICE-003 (Personalized Content)
 */

import { PersonalBriefing, BriefingSection, NewsArticle, EmailSummary, DailyActivity, FollowUpQuestion, UserPreferences } from '../../types/voice';
import { VoiceProfileService } from '../voice/VoiceProfileService';
import { NewsService } from '../news/NewsService';
import { EmailService } from '../email/EmailService';
import { ConversationMemoryService } from '../memory/ConversationMemoryService';
import { SectionBuilder } from './SectionBuilder';
import { TTSService } from './TTSService';

export class PersonalBriefingService {
  private voiceService: VoiceProfileService;
  private newsService: NewsService;
  private emailService: EmailService;
  private memoryService: ConversationMemoryService;
  private sectionBuilder: SectionBuilder;
  private ttsService: TTSService;

  constructor() {
    this.voiceService = new VoiceProfileService();
    this.newsService = new NewsService();
    this.emailService = new EmailService();
    this.memoryService = new ConversationMemoryService();
    this.sectionBuilder = new SectionBuilder();
    this.ttsService = new TTSService();
  }

  /**
   * Generate complete personalized briefing
   * Contract: I-VOICE-003 (Personalized Content)
   */
  async generateBriefing(userId: string): Promise<PersonalBriefing> {
    const preferences = await this.memoryService.getUserPreferences(userId);
    const timeOfDay = this.getTimeOfDay();

    const sections: BriefingSection[] = [];

    // Build greeting section
    const greetingSection = await this.buildGreetingSection(userId, timeOfDay);
    sections.push(greetingSection);

    // Build news section if allowed
    if (preferences.privacySettings.allowNewsPersonalization) {
      const newsSection = await this.buildNewsSection(userId);
      if (newsSection.content) {
        sections.push(newsSection);
      }
    }

    // Build email section if allowed
    if (preferences.privacySettings.allowEmailAccess) {
      const emailSection = await this.buildEmailSection(userId);
      if (emailSection.content) {
        sections.push(emailSection);
      }
    }

    // Build memory section (yesterday's activities)
    const memorySection = await this.buildMemorySection(userId);
    if (memorySection.content) {
      sections.push(memorySection);
    }

    // Build question section
    const questionSection = await this.buildQuestionSection(userId);
    if (questionSection.content) {
      sections.push(questionSection);
    }

    // Order sections by priority
    const orderedSections = this.sectionBuilder.orderSections(sections);

    // Generate full spoken text
    const spokenText = orderedSections.map(s => s.content).join(' ');

    // Estimate duration (average speaking rate: 150 words per minute)
    const wordCount = spokenText.split(/\s+/).length;
    const duration = Math.ceil((wordCount / 150) * 60); // seconds

    const briefing: PersonalBriefing = {
      userId,
      timestamp: Date.now(),
      sections: orderedSections,
      spokenText,
      duration
    };

    // Store briefing in memory
    await this.storeBriefingHistory(userId, briefing);

    return briefing;
  }

  /**
   * Build greeting section
   */
  async buildGreetingSection(userId: string, timeOfDay: string): Promise<BriefingSection> {
    const profile = await this.voiceService.getProfile(userId);
    const preferences = await this.memoryService.getUserPreferences(userId);

    const name = preferences.personalDetails.nickname || profile?.name || 'friend';
    const content = this.sectionBuilder.formatGreeting(name, timeOfDay, {
      isChild: profile?.metadata.isChild || false
    });

    return {
      type: 'greeting',
      title: 'Greeting',
      content,
      priority: 'high'
    };
  }

  /**
   * Build news section
   * Contract: I-NEWS-001 (News Personalization)
   */
  async buildNewsSection(userId: string): Promise<BriefingSection> {
    try {
      const newsPreferences = await this.newsService.getPreferences(userId);
      const articles = await this.newsService.getPersonalizedNews(userId);

      if (articles.length === 0) {
        return {
          type: 'news',
          title: 'News',
          content: '',
          priority: 'medium'
        };
      }

      const topArticles = articles.slice(0, newsPreferences.maxArticles);
      const content = this.sectionBuilder.formatNewsHeadlines(topArticles, newsPreferences.maxArticles);

      return {
        type: 'news',
        title: 'News Headlines',
        content,
        priority: 'medium',
        metadata: { articleCount: topArticles.length },
        articles: topArticles
      };
    } catch (error) {
      console.error('Failed to fetch news:', error);
      return {
        type: 'news',
        title: 'News',
        content: '',
        priority: 'medium'
      };
    }
  }

  /**
   * Build email section
   * Contract: I-EMAIL-001 (Email Privacy)
   */
  async buildEmailSection(userId: string): Promise<BriefingSection> {
    try {
      const accounts = await this.emailService.getAccounts(userId);

      if (accounts.length === 0) {
        return {
          type: 'email',
          title: 'Email',
          content: '',
          priority: 'low'
        };
      }

      const summary = await this.emailService.getSummary(userId);
      const content = this.sectionBuilder.formatEmailSummary(summary);

      return {
        type: 'email',
        title: 'Email Summary',
        content,
        priority: summary.importantCount > 0 ? 'high' : 'medium',
        metadata: { summary }
      };
    } catch (error) {
      console.error('Failed to fetch email:', error);
      return {
        type: 'email',
        title: 'Email',
        content: '',
        priority: 'low'
      };
    }
  }

  /**
   * Build memory section (yesterday's activities)
   * Contract: I-MEMORY-001 (Activity Tracking)
   */
  async buildMemorySection(userId: string): Promise<BriefingSection> {
    try {
      const yesterday = this.getYesterdayDate();
      const activity = await this.memoryService.getDailyActivity(userId, yesterday);

      if (!activity || activity.plannedActivities.length === 0) {
        return {
          type: 'memory',
          title: 'Yesterday',
          content: '',
          priority: 'low'
        };
      }

      const content = this.sectionBuilder.formatMemoryRecall([activity]);

      return {
        type: 'memory',
        title: 'Yesterday',
        content,
        priority: 'medium',
        metadata: { activity }
      };
    } catch (error) {
      console.error('Failed to fetch memory:', error);
      return {
        type: 'memory',
        title: 'Yesterday',
        content: '',
        priority: 'low'
      };
    }
  }

  /**
   * Build follow-up question section
   */
  async buildQuestionSection(userId: string): Promise<BriefingSection> {
    try {
      const questions = await this.memoryService.getFollowUpQuestions(userId);

      if (questions.length === 0) {
        return {
          type: 'question',
          title: 'Question',
          content: '',
          priority: 'low'
        };
      }

      // Get highest priority unanswered question
      const topQuestion = questions
        .filter(q => !q.answered && q.validUntil > Date.now())
        .sort((a, b) => b.priority - a.priority)[0];

      if (!topQuestion) {
        return {
          type: 'question',
          title: 'Question',
          content: '',
          priority: 'low'
        };
      }

      const content = this.sectionBuilder.formatFollowUpQuestion(topQuestion);

      return {
        type: 'question',
        title: 'Question',
        content,
        priority: 'high',
        metadata: { questionId: topQuestion.id }
      };
    } catch (error) {
      console.error('Failed to fetch questions:', error);
      return {
        type: 'question',
        title: 'Question',
        content: '',
        priority: 'low'
      };
    }
  }

  /**
   * Deliver briefing (speak and display)
   */
  async deliverBriefing(userId: string, briefing: PersonalBriefing): Promise<void> {
    // Speak the briefing aloud
    await this.speakBriefing(briefing.spokenText);

    // Store delivery event
    await this.memoryService.storeConversationTurn({
      speaker: 'mbot',
      text: briefing.spokenText,
      timestamp: Date.now(),
      intent: 'briefing_delivery'
    }, userId);
  }

  /**
   * Speak briefing using TTS
   */
  async speakBriefing(text: string): Promise<void> {
    await this.ttsService.synthesizeSpeech(text);
  }

  /**
   * Store briefing in history
   */
  private async storeBriefingHistory(userId: string, briefing: PersonalBriefing): Promise<void> {
    const storageKey = `briefing_history_${userId}`;
    const history = JSON.parse(localStorage.getItem(storageKey) || '[]');

    history.unshift(briefing);

    // Keep last 30 briefings
    if (history.length > 30) {
      history.splice(30);
    }

    localStorage.setItem(storageKey, JSON.stringify(history));
  }

  /**
   * Get time of day
   */
  private getTimeOfDay(): string {
    const hour = new Date().getHours();
    if (hour < 12) return 'morning';
    if (hour < 17) return 'afternoon';
    return 'evening';
  }

  /**
   * Get yesterday's date in YYYY-MM-DD format
   */
  private getYesterdayDate(): string {
    const yesterday = new Date();
    yesterday.setDate(yesterday.getDate() - 1);
    return yesterday.toISOString().split('T')[0];
  }

  /**
   * Get briefing history
   */
  async getBriefingHistory(userId: string, limit: number = 10): Promise<PersonalBriefing[]> {
    const storageKey = `briefing_history_${userId}`;
    const history = JSON.parse(localStorage.getItem(storageKey) || '[]');
    return history.slice(0, limit);
  }
}
