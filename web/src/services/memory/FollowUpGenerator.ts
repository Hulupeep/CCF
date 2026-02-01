/**
 * Follow-Up Question Generator - Generate intelligent follow-up questions
 *
 * Contract: I-VOICE-004
 * Generates contextual follow-up questions based on conversation history and activities
 */

import { FollowUpQuestion, DailyActivity, Conversation } from '../../types/voice';
import { conversationMemoryService } from './ConversationMemoryService';
import { memoryStore } from './MemoryStore';
import { v4 as uuidv4 } from 'uuid';

interface QuestionTemplate {
  pattern: string;
  priority: number;
  validityHours: number;
  contextCheck: (context: any) => boolean;
}

export class FollowUpGenerator {
  private questionTemplates: QuestionTemplate[] = [
    // Yesterday's activity follow-ups
    {
      pattern: "Yesterday you mentioned {activity}. Did you get a chance to {verb} it?",
      priority: 80,
      validityHours: 48,
      contextCheck: (ctx) => ctx.hasYesterdayActivity && !ctx.hasCompletionConfirmation,
    },
    {
      pattern: "How did {activity} go yesterday?",
      priority: 75,
      validityHours: 36,
      contextCheck: (ctx) => ctx.hasYesterdayActivity,
    },

    // Ongoing project follow-ups
    {
      pattern: "How's your {project} coming along?",
      priority: 70,
      validityHours: 72,
      contextCheck: (ctx) => ctx.hasOngoingProject,
    },
    {
      pattern: "Have you made progress on {project}?",
      priority: 65,
      validityHours: 96,
      contextCheck: (ctx) => ctx.hasOngoingProject && ctx.daysSinceLastMention > 2,
    },

    // Learning follow-ups
    {
      pattern: "Still practicing {skill}?",
      priority: 60,
      validityHours: 120,
      contextCheck: (ctx) => ctx.hasLearningGoal,
    },
    {
      pattern: "How's learning {skill} going?",
      priority: 55,
      validityHours: 96,
      contextCheck: (ctx) => ctx.hasLearningGoal,
    },

    // Event follow-ups
    {
      pattern: "Your {event} was today, right? How did it go?",
      priority: 90,
      validityHours: 12,
      contextCheck: (ctx) => ctx.hasUpcomingEvent && ctx.isEventDay,
    },
    {
      pattern: "Getting ready for {event}?",
      priority: 70,
      validityHours: 48,
      contextCheck: (ctx) => ctx.hasUpcomingEvent && ctx.daysUntilEvent <= 2,
    },

    // General interest follow-ups
    {
      pattern: "Still interested in {topic}?",
      priority: 50,
      validityHours: 168,
      contextCheck: (ctx) => ctx.hasRecentInterest,
    },
  ];

  /**
   * Generate follow-up questions for a user
   */
  async generateFollowUps(userId: string): Promise<FollowUpQuestion[]> {
    const questions: FollowUpQuestion[] = [];

    // Check yesterday's activities
    const yesterdayQuestions = await this.checkYesterdayActivity(userId);
    if (yesterdayQuestions) {
      questions.push(yesterdayQuestions);
    }

    // Check ongoing projects
    const projectQuestions = await this.checkOngoingProjects(userId);
    questions.push(...projectQuestions);

    // Check previous goals
    const goalQuestions = await this.recallPreviousGoals(userId);
    questions.push(...goalQuestions);

    // Prioritize and return
    return this.prioritizeQuestions(questions);
  }

  /**
   * Check yesterday's activity and generate follow-up
   */
  async checkYesterdayActivity(userId: string): Promise<FollowUpQuestion | null> {
    const yesterday = await conversationMemoryService.getYesterdayActivity(userId);
    if (!yesterday || yesterday.plannedActivities.length === 0) {
      return null;
    }

    // Check if already completed
    const hasUncompletedActivities = yesterday.plannedActivities.some(
      (activity) => !yesterday.completedActivities.includes(activity)
    );

    if (!hasUncompletedActivities) {
      return null;
    }

    // Get first uncompleted activity
    const activity = yesterday.plannedActivities.find(
      (a) => !yesterday.completedActivities.includes(a)
    )!;

    const verb = this.extractVerb(activity);
    const question = verb
      ? `Yesterday you mentioned ${activity}. Did you get a chance to ${verb} it?`
      : `Yesterday you mentioned ${activity}. How did it go?`;

    return {
      id: uuidv4(),
      userId,
      question,
      context: `yesterday_activity:${activity}`,
      priority: 80,
      validUntil: Date.now() + 48 * 60 * 60 * 1000, // 48 hours
      answered: false,
    };
  }

  /**
   * Check ongoing projects from conversation history
   */
  async checkOngoingProjects(userId: string): Promise<FollowUpQuestion[]> {
    const questions: FollowUpQuestion[] = [];
    const conversations = await conversationMemoryService.getConversations(userId, 7);

    // Extract project mentions
    const projectKeywords = ['building', 'making', 'creating', 'working on', 'learning'];
    const projects = new Map<string, { lastMention: number; mentions: number }>();

    conversations.forEach((conv) => {
      conv.keyPoints.forEach((point) => {
        const lowerPoint = point.toLowerCase();
        projectKeywords.forEach((keyword) => {
          if (lowerPoint.includes(keyword)) {
            const project = point;
            const existing = projects.get(project);
            if (existing) {
              existing.mentions++;
              existing.lastMention = Math.max(existing.lastMention, conv.timestamp);
            } else {
              projects.set(project, { lastMention: conv.timestamp, mentions: 1 });
            }
          }
        });
      });
    });

    // Generate follow-ups for projects mentioned multiple times
    projects.forEach((info, project) => {
      if (info.mentions >= 2) {
        const daysSinceLastMention =
          (Date.now() - info.lastMention) / (24 * 60 * 60 * 1000);

        if (daysSinceLastMention >= 1 && daysSinceLastMention <= 7) {
          questions.push({
            id: uuidv4(),
            userId,
            question: `How's your ${project} coming along?`,
            context: `ongoing_project:${project}`,
            priority: 70 - Math.floor(daysSinceLastMention * 5),
            validUntil: Date.now() + 72 * 60 * 60 * 1000, // 72 hours
            answered: false,
          });
        }
      }
    });

    return questions;
  }

  /**
   * Recall previous goals and generate follow-ups
   */
  async recallPreviousGoals(userId: string): Promise<FollowUpQuestion[]> {
    const questions: FollowUpQuestion[] = [];
    const conversations = await conversationMemoryService.getConversations(userId, 14);

    // Extract goal-related statements
    const goalKeywords = ['want to', 'going to', 'plan to', 'hope to', 'trying to'];
    const goals = new Map<string, number>();

    conversations.forEach((conv) => {
      conv.turns.forEach((turn) => {
        if (turn.speaker === 'user') {
          const lowerText = turn.text.toLowerCase();
          goalKeywords.forEach((keyword) => {
            if (lowerText.includes(keyword)) {
              // Extract the goal phrase
              const startIndex = lowerText.indexOf(keyword) + keyword.length;
              const goalPhrase = turn.text
                .substring(startIndex)
                .split(/[.!?,]/)[0]
                .trim();

              if (goalPhrase.length > 5 && goalPhrase.length < 80) {
                goals.set(goalPhrase, conv.timestamp);
              }
            }
          });
        }
      });
    });

    // Generate follow-ups for goals mentioned 3-14 days ago
    goals.forEach((timestamp, goal) => {
      const daysAgo = (Date.now() - timestamp) / (24 * 60 * 60 * 1000);

      if (daysAgo >= 3 && daysAgo <= 14) {
        questions.push({
          id: uuidv4(),
          userId,
          question: `Remember you wanted to ${goal}. How's that going?`,
          context: `previous_goal:${goal}`,
          priority: 60 - Math.floor(daysAgo * 2),
          validUntil: Date.now() + 120 * 60 * 60 * 1000, // 5 days
          answered: false,
        });
      }
    });

    return questions;
  }

  /**
   * Prioritize questions
   */
  async prioritizeQuestions(
    questions: FollowUpQuestion[]
  ): Promise<FollowUpQuestion[]> {
    // Remove duplicates
    const seen = new Set<string>();
    const unique = questions.filter((q) => {
      const key = q.question.toLowerCase();
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    });

    // Sort by priority (highest first)
    unique.sort((a, b) => b.priority - a.priority);

    // Save to store
    for (const question of unique) {
      await memoryStore.saveFollowUpQuestion(question);
    }

    return unique;
  }

  /**
   * Mark question as answered
   */
  async markQuestionAnswered(questionId: string, answer: string): Promise<void> {
    await memoryStore.markQuestionAnswered(questionId, answer);

    // Extract information from answer and update activities if needed
    const question = await this.getQuestionById(questionId);
    if (!question) return;

    // If it was about yesterday's activity, mark as completed if confirmed
    if (question.context.startsWith('yesterday_activity:')) {
      const activity = question.context.replace('yesterday_activity:', '');
      const answerLower = answer.toLowerCase();

      if (
        answerLower.includes('yes') ||
        answerLower.includes('finished') ||
        answerLower.includes('completed') ||
        answerLower.includes('done')
      ) {
        const yesterday = this.getDateString(
          new Date(Date.now() - 24 * 60 * 60 * 1000)
        );
        await conversationMemoryService.markActivityCompleted(
          question.userId,
          yesterday,
          activity
        );
      }
    }
  }

  /**
   * Get active questions for user
   */
  async getActiveQuestions(userId: string): Promise<FollowUpQuestion[]> {
    return memoryStore.getActiveQuestions(userId);
  }

  /**
   * Clean up expired questions
   */
  async cleanupExpiredQuestions(): Promise<number> {
    return memoryStore.deleteExpiredQuestions();
  }

  // ==================== Private Helpers ====================

  /**
   * Get question by ID
   */
  private async getQuestionById(id: string): Promise<FollowUpQuestion | null> {
    // Would need to add this method to MemoryStore
    // For now, return null
    return null;
  }

  /**
   * Extract verb from activity phrase
   */
  private extractVerb(activity: string): string | null {
    const words = activity.split(/\s+/);
    const verbs = [
      'build',
      'make',
      'create',
      'draw',
      'paint',
      'write',
      'read',
      'play',
      'practice',
      'learn',
      'finish',
      'complete',
      'do',
    ];

    for (const word of words) {
      if (verbs.includes(word.toLowerCase())) {
        return word.toLowerCase();
      }
    }

    return null;
  }

  /**
   * Get date string in YYYY-MM-DD format
   */
  private getDateString(date: Date): string {
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  }
}

// Export singleton instance
export const followUpGenerator = new FollowUpGenerator();
