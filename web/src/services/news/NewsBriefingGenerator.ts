/**
 * News Briefing Generator
 * Generates formatted news briefings for voice output
 *
 * Contract: I-NEWS-001 - Personalized news delivery
 */

import type {
  BriefingSection,
  NewsArticle
} from '../../types/voice';

export class NewsBriefingGenerator {
  /**
   * Generate a news briefing section from articles
   */
  async generateBriefing(
    userId: string,
    articles: NewsArticle[],
    userName?: string
  ): Promise<BriefingSection> {
    const content = await this.formatForSpeech(articles, userName);

    return {
      type: 'news',
      title: 'News Headlines',
      content,
      priority: 'high',
      articles,
      metadata: {
        articleCount: articles.length,
        sources: [...new Set(articles.map(a => a.source))],
        categories: [...new Set(articles.map(a => a.category).filter(Boolean))]
      }
    };
  }

  /**
   * Summarize a single article
   * Extracts key points and creates a concise summary
   */
  async summarizeArticle(article: NewsArticle): Promise<string> {
    // Use the existing summary if it's good
    if (article.summary && article.summary.length > 50 && article.summary.length < 200) {
      return article.summary;
    }

    // If summary is too long, truncate it
    if (article.summary && article.summary.length >= 200) {
      const truncated = article.summary.substring(0, 197) + '...';
      return truncated;
    }

    // If no summary, use the headline
    if (!article.summary) {
      return article.headline;
    }

    // Fallback
    return article.summary || article.headline;
  }

  /**
   * Format articles for speech output
   * Creates natural-sounding briefing text
   */
  async formatForSpeech(articles: NewsArticle[], userName?: string): Promise<string> {
    if (articles.length === 0) {
      return "I don't have any news updates for you right now.";
    }

    const parts: string[] = [];

    // Greeting
    if (userName) {
      parts.push(`Here are your top news stories, ${userName}:`);
    } else {
      parts.push("Here are today's top news stories:");
    }

    // Format each article
    for (let i = 0; i < Math.min(articles.length, 5); i++) {
      const article = articles[i];
      const number = this.numberToWord(i + 1);
      const summary = await this.summarizeArticle(article);

      parts.push(`${number}: ${article.headline}.`);

      // Add summary if it's different from headline
      if (summary !== article.headline && summary.length > 20) {
        parts.push(summary);
      }

      // Add source
      parts.push(`From ${article.source}.`);

      // Add pause between articles
      if (i < articles.length - 1 && i < 4) {
        parts.push(''); // Empty line for pause
      }
    }

    // Closing
    if (articles.length > 5) {
      parts.push(`There are ${articles.length - 5} more stories available.`);
    }

    return parts.join(' ').trim();
  }

  /**
   * Format articles for display (not speech)
   */
  formatForDisplay(articles: NewsArticle[]): string {
    if (articles.length === 0) {
      return "No news articles available.";
    }

    const lines: string[] = [];

    articles.forEach((article, i) => {
      lines.push(`${i + 1}. ${article.headline}`);
      lines.push(`   Source: ${article.source} | ${this.formatTimestamp(article.publishedAt)}`);

      if (article.summary) {
        lines.push(`   ${article.summary}`);
      }

      if (article.url) {
        lines.push(`   Read more: ${article.url}`);
      }

      lines.push(''); // Empty line between articles
    });

    return lines.join('\n');
  }

  /**
   * Generate a brief headline summary (for notifications)
   */
  generateHeadlineSummary(articles: NewsArticle[], maxCount: number = 3): string {
    if (articles.length === 0) {
      return "No new headlines";
    }

    const headlines = articles
      .slice(0, maxCount)
      .map((article, i) => `${i + 1}. ${article.headline}`)
      .join('\n');

    if (articles.length > maxCount) {
      return `${headlines}\n... and ${articles.length - maxCount} more`;
    }

    return headlines;
  }

  /**
   * Generate a category-based briefing
   */
  async generateCategoryBriefing(
    articles: NewsArticle[],
    category: string
  ): Promise<BriefingSection> {
    const categoryArticles = articles.filter(a =>
      a.category?.toLowerCase() === category.toLowerCase()
    );

    const content = await this.formatForSpeech(categoryArticles);

    return {
      type: 'news',
      title: `${this.capitalize(category)} News`,
      content,
      priority: 'medium',
      articles: categoryArticles,
      metadata: {
        category,
        articleCount: categoryArticles.length
      }
    };
  }

  /**
   * Generate multi-category briefing
   */
  async generateMultiCategoryBriefing(
    articles: NewsArticle[],
    categories: string[]
  ): Promise<BriefingSection[]> {
    const sections: BriefingSection[] = [];

    for (const category of categories) {
      const categoryArticles = articles.filter(a =>
        a.category?.toLowerCase() === category.toLowerCase()
      );

      if (categoryArticles.length > 0) {
        const section = await this.generateCategoryBriefing(articles, category);
        sections.push(section);
      }
    }

    return sections;
  }

  /**
   * Generate a quick news summary (one sentence per article)
   */
  generateQuickSummary(articles: NewsArticle[], maxArticles: number = 3): string {
    if (articles.length === 0) {
      return "No news available.";
    }

    const summaries = articles
      .slice(0, maxArticles)
      .map(article => article.headline)
      .join('. ');

    return summaries + '.';
  }

  /**
   * Convert number to word (1-20)
   */
  private numberToWord(num: number): string {
    const words = [
      'First', 'Second', 'Third', 'Fourth', 'Fifth',
      'Sixth', 'Seventh', 'Eighth', 'Ninth', 'Tenth'
    ];

    if (num >= 1 && num <= words.length) {
      return words[num - 1];
    }

    return `Number ${num}`;
  }

  /**
   * Format timestamp for display
   */
  private formatTimestamp(timestamp: number): string {
    const date = new Date(timestamp);
    const now = Date.now();
    const diff = now - timestamp;

    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (hours < 1) {
      return 'Just now';
    } else if (hours < 24) {
      return `${hours} hour${hours > 1 ? 's' : ''} ago`;
    } else if (days < 7) {
      return `${days} day${days > 1 ? 's' : ''} ago`;
    } else {
      return date.toLocaleDateString();
    }
  }

  /**
   * Capitalize first letter
   */
  private capitalize(str: string): string {
    return str.charAt(0).toUpperCase() + str.slice(1);
  }

  /**
   * Estimate speaking duration in seconds
   */
  estimateSpeakingDuration(text: string): number {
    // Average speaking rate: ~150 words per minute
    const words = text.split(/\s+/).length;
    const minutes = words / 150;
    return Math.ceil(minutes * 60);
  }
}

// Singleton instance
let instance: NewsBriefingGenerator | null = null;

/**
 * Get the singleton briefing generator instance
 */
export function getNewsBriefingGenerator(): NewsBriefingGenerator {
  if (!instance) {
    instance = new NewsBriefingGenerator();
  }
  return instance;
}
