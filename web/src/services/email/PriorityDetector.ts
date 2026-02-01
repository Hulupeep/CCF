/**
 * Email Priority Detector - Intelligent email importance and categorization
 *
 * Contract: I-EMAIL-001
 * Issue: #95 Component 3/5
 */

import { Email } from './EmailService';

export class PriorityDetector {
  private vipSenders: Set<string> = new Set();
  private urgentKeywords = [
    'urgent',
    'important',
    'asap',
    'critical',
    'immediate',
    'priority',
    'action required',
    'time-sensitive',
    'deadline',
    'emergency',
  ];

  private workKeywords = [
    'meeting',
    'project',
    'deadline',
    'report',
    'task',
    'proposal',
    'review',
    'approval',
    'budget',
    'client',
    'invoice',
  ];

  private promotionKeywords = [
    'sale',
    'discount',
    'offer',
    'deal',
    'coupon',
    'promo',
    'limited time',
    'save',
    'shop',
    'buy now',
  ];

  private socialKeywords = [
    'liked',
    'commented',
    'shared',
    'tagged',
    'mentioned',
    'followed',
    'friend request',
    'notification',
  ];

  /**
   * Add VIP sender
   */
  addVIPSender(email: string): void {
    this.vipSenders.add(email.toLowerCase());
  }

  /**
   * Remove VIP sender
   */
  removeVIPSender(email: string): void {
    this.vipSenders.delete(email.toLowerCase());
  }

  /**
   * Check if sender is VIP
   */
  isVIPSender(from: string): boolean {
    const emailMatch = from.match(/<(.+?)>/);
    const email = emailMatch ? emailMatch[1] : from;
    return this.vipSenders.has(email.toLowerCase());
  }

  /**
   * Detect email importance
   * Returns 'high', 'medium', or 'low'
   *
   * Scoring algorithm:
   * - VIP sender: +40 points
   * - Urgent keywords: +30 points
   * - In reply to thread: +20 points
   * - Starred/flagged: +10 points
   *
   * Score >= 60: high
   * Score >= 30: medium
   * Score < 30: low
   */
  detectImportance(email: Email): 'high' | 'medium' | 'low' {
    let score = 0;

    // VIP sender check
    if (this.isVIPSender(email.from)) {
      score += 40;
    }

    // Keyword analysis
    if (this.hasUrgentKeywords(email.subject)) {
      score += 30;
    }

    // Thread participation
    if (email.inReplyTo) {
      score += 20;
    }

    // Flagged/starred
    if (email.isStarred) {
      score += 10;
    }

    // Provider-supplied importance
    if (email.importance === 'high') {
      score += 20;
    }

    // Categorization bonus (work emails slightly more important)
    const category = this.categorizeEmail(email);
    if (category === 'work') {
      score += 5;
    }

    if (score >= 60) return 'high';
    if (score >= 30) return 'medium';
    return 'low';
  }

  /**
   * Categorize email into work, personal, promotions, or social
   */
  categorizeEmail(email: Email): 'work' | 'personal' | 'promotions' | 'social' {
    const text = `${email.subject} ${email.snippet}`.toLowerCase();

    // Check for promotions
    const promotionScore = this.countKeywordMatches(text, this.promotionKeywords);
    if (promotionScore >= 2) return 'promotions';

    // Check for social
    const socialScore = this.countKeywordMatches(text, this.socialKeywords);
    if (socialScore >= 2) return 'social';

    // Check for work
    const workScore = this.countKeywordMatches(text, this.workKeywords);
    if (workScore >= 2) return 'work';

    // Check labels (Gmail)
    if (email.labels) {
      if (email.labels.includes('CATEGORY_PROMOTIONS')) return 'promotions';
      if (email.labels.includes('CATEGORY_SOCIAL')) return 'social';
      if (email.labels.includes('CATEGORY_PRIMARY')) {
        // Primary could be work or personal, use keywords
        return workScore > 0 ? 'work' : 'personal';
      }
    }

    // Default to personal
    return 'personal';
  }

  /**
   * Check if email subject contains urgent keywords
   */
  private hasUrgentKeywords(subject: string): boolean {
    const lowerSubject = subject.toLowerCase();
    return this.urgentKeywords.some(keyword => lowerSubject.includes(keyword));
  }

  /**
   * Count keyword matches in text
   */
  private countKeywordMatches(text: string, keywords: string[]): number {
    let count = 0;
    for (const keyword of keywords) {
      if (text.includes(keyword)) count++;
    }
    return count;
  }

  /**
   * Learn from user interactions (future enhancement)
   * Track which emails user opens, replies to, stars, etc.
   */
  async learnFromInteraction(
    email: Email,
    interaction: 'open' | 'reply' | 'star' | 'archive' | 'delete'
  ): Promise<void> {
    // Future: Use machine learning to improve importance detection
    // Store interaction data and retrain model periodically

    console.log(`Learning from interaction: ${interaction} on email ${email.id}`);

    // Example: If user frequently replies to a sender, mark as VIP
    if (interaction === 'reply' || interaction === 'star') {
      // Could automatically add to VIP list after N interactions
    }
  }
}
