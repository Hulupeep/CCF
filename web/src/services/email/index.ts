/**
 * Email Service Exports
 *
 * Contract: I-EMAIL-001
 * Issue: #95 Component 3/5
 */

export { EmailService } from './EmailService';
export type { Email, EmailClientInterface } from './EmailService';

export { GmailClient } from './GmailClient';
export type { GmailTokens, GmailMessage } from './GmailClient';

export { OutlookClient } from './OutlookClient';
export type { OutlookTokens, OutlookMessage } from './OutlookClient';

export { PriorityDetector } from './PriorityDetector';
