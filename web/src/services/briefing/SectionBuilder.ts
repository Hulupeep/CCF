/**
 * Section Builder - Formats briefing sections for speech
 *
 * Handles formatting of different section types for natural speech delivery
 */

import { BriefingSection, NewsArticle, EmailSummary, DailyActivity, FollowUpQuestion } from '../../types/voice';

export class SectionBuilder {
  /**
   * Format greeting for speech
   */
  formatGreeting(name: string, timeOfDay: string, personality: { isChild: boolean }): string {
    const greeting = timeOfDay === 'morning' ? 'Good morning' :
                     timeOfDay === 'afternoon' ? 'Good afternoon' :
                     'Good evening';

    if (personality.isChild) {
      return `${greeting}, ${name}! Ready for an awesome day?`;
    }

    return `${greeting}, ${name}! Here's your daily update.`;
  }

  /**
   * Format news headlines for speech
   */
  formatNewsHeadlines(articles: NewsArticle[], maxCount: number): string {
    const headlines = articles.slice(0, maxCount);

    if (headlines.length === 0) {
      return '';
    }

    const intro = `Here are your top ${headlines.length} news ${headlines.length === 1 ? 'story' : 'stories'}: `;

    const formattedHeadlines = headlines.map((article, index) => {
      const prefix = headlines.length > 1 ? `${index + 1}. ` : '';
      return `${prefix}${article.headline} from ${article.source}.`;
    }).join(' ');

    return intro + formattedHeadlines;
  }

  /**
   * Format email summary for speech
   */
  formatEmailSummary(summary: EmailSummary): string {
    if (summary.totalUnread === 0) {
      return 'You have no unread emails.';
    }

    let text = `You have ${summary.totalUnread} unread email${summary.totalUnread === 1 ? '' : 's'}`;

    if (summary.importantCount > 0) {
      text += `, ${summary.importantCount} marked important`;
    }

    text += '.';

    // Add highlights
    if (summary.highlights.length > 0) {
      const topHighlight = summary.highlights[0];
      text += ` Top message: ${topHighlight.subject} from ${topHighlight.from}.`;
    }

    return text;
  }

  /**
   * Format memory recall for speech
   */
  formatMemoryRecall(activities: DailyActivity[]): string {
    if (activities.length === 0) {
      return '';
    }

    const activity = activities[0]; // Most recent

    if (activity.plannedActivities.length === 0) {
      return '';
    }

    const planned = activity.plannedActivities[0];
    let text = `Yesterday you mentioned you wanted to ${planned}.`;

    if (activity.completedActivities.length > 0) {
      text += ` Did you get to it?`;
    } else {
      text += ` How did it go?`;
    }

    return text;
  }

  /**
   * Format follow-up question for speech
   */
  formatFollowUpQuestion(question: FollowUpQuestion): string {
    return question.question;
  }

  /**
   * Order sections by priority
   */
  orderSections(sections: BriefingSection[]): BriefingSection[] {
    const priorityOrder = { high: 0, medium: 1, low: 2 };

    return sections
      .filter(s => s.content && s.content.trim().length > 0)
      .sort((a, b) => {
        const priorityDiff = priorityOrder[a.priority] - priorityOrder[b.priority];
        if (priorityDiff !== 0) return priorityDiff;

        // Secondary sort by type order
        const typeOrder = {
          greeting: 0,
          memory: 1,
          email: 2,
          news: 3,
          calendar: 4,
          weather: 5,
          question: 6
        };

        return typeOrder[a.type] - typeOrder[b.type];
      });
  }
}
