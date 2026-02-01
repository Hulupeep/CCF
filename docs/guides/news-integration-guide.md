# News Integration Guide

This guide explains how to use the news integration and personalization system in mBot RuVector.

## Overview

The news system provides:
- Fetching news from NewsAPI.org or similar services
- User preference management
- Personalized news ranking based on interests
- Learning from user interactions
- Formatted news briefings for voice output

**Contract:** I-NEWS-001 - News personalization with relevance ranking

## Quick Start

### 1. Setup API Key

Sign up for a free API key at [NewsAPI.org](https://newsapi.org/register).

Add to your `.env` file:
```bash
VITE_NEWS_API_KEY=your_api_key_here
```

### 2. Basic Usage

```typescript
import { getNewsService, getNewsPreferencesManager } from '../services/news';

const userId = 'user-123';
const newsService = getNewsService();
const prefsManager = getNewsPreferencesManager();

// Get user preferences
const preferences = await prefsManager.getPreferences(userId);

// Fetch personalized news
const articles = await newsService.fetchNews(preferences);

// Display articles
console.log(`Found ${articles.length} articles`);
articles.forEach(article => {
  console.log(`${article.headline} (relevance: ${article.relevanceScore})`);
});
```

## Components

### NewsAPIClient

Handles communication with the news API, including caching and rate limiting.

```typescript
import { createNewsAPIClient } from '../services/news';

const client = createNewsAPIClient();

// Get top headlines
const headlines = await client.getTopHeadlines({
  country: 'us',
  category: 'technology',
  pageSize: 10
});

// Search for articles
const search = await client.searchEverything({
  q: 'artificial intelligence',
  language: 'en',
  sortBy: 'relevancy'
});
```

**Features:**
- 15-minute response caching
- Automatic rate limiting (100 req/day for free tier)
- Error handling with retry logic
- Request tracking

### NewsPreferencesManager

Manages user preferences and learns from interactions.

```typescript
import { getNewsPreferencesManager } from '../services/news';

const manager = getNewsPreferencesManager();
const userId = 'user-123';

// Add topics
await manager.addTopic(userId, 'technology');
await manager.addTopic(userId, 'science');

// Exclude topics
await manager.excludeTopic(userId, 'politics');

// Adjust preferences based on feedback
await manager.adjustWeights(userId, 'technology', 0.2); // Increase interest

// Get topic weights
const weights = await manager.getTopicWeights(userId);
console.log(weights); // { technology: 1.2, science: 1.0, ... }
```

**Learning Algorithm:**
- User reads article → +0.2 weight
- User skips article → -0.2 weight
- User likes article → +0.4 weight
- User dislikes article → -0.4 weight
- Long reading time → additional +0.2 weight (max)

Weights are clamped between 0.1 (MIN_WEIGHT) and 2.0 (MAX_WEIGHT).

### NewsService

Main service that orchestrates fetching and personalization.

```typescript
import { getNewsService } from '../services/news';

const service = getNewsService();
const userId = 'user-123';

// Get morning briefing
const briefing = await service.getBriefing(userId, 5);

// Search with personalization
const articles = await service.searchNews('machine learning');
const personalized = await service.personalizeNews(userId, articles);

// Update preferences based on interaction
await service.updatePreferences(userId, articles[0], 'read');
```

**Relevance Scoring:**

Each article gets a score from 0.0 to 1.0 based on:
1. **Topic match** (40%): Does the article match user's interests?
2. **Source preference** (20%): Has user engaged with this source before?
3. **Recency** (20%): How recent is the article? (decays over 24 hours)
4. **Content quality** (20%): Has image, author, substantial content?

### NewsBriefingGenerator

Formats news for voice output and display.

```typescript
import { getNewsBriefingGenerator } from '../services/news';

const generator = getNewsBriefingGenerator();

// Generate briefing section
const briefing = await generator.generateBriefing(userId, articles, 'Alice');

console.log(briefing.content);
// "Here are your top news stories, Alice: First: Breaking Tech News..."

// Estimate speaking time
const duration = generator.estimateSpeakingDuration(briefing.content);
console.log(`Will take ${duration} seconds to speak`);

// Quick summary for notifications
const summary = generator.generateQuickSummary(articles, 3);
```

## UI Components

### NewsPreferences

React component for managing preferences.

```tsx
import { NewsPreferences } from '../components/news';

function App() {
  const userId = 'user-123';

  return (
    <NewsPreferences
      userId={userId}
      onSave={(prefs) => console.log('Saved:', prefs)}
    />
  );
}
```

**Features:**
- Topic selection grid
- Excluded topics management
- Briefing length slider
- Reading level selector
- Learned preferences visualization

### NewsArticleList

Display articles with interaction tracking.

```tsx
import { NewsArticleList } from '../components/news';

function NewsFeed() {
  const [articles, setArticles] = useState([]);

  const handleFeedback = (article, feedback) => {
    // Learn from user interaction
    newsService.learnFromFeedback(userId, feedback);
  };

  return (
    <NewsArticleList
      articles={articles}
      userId={userId}
      onFeedback={handleFeedback}
    />
  );
}
```

**Interaction Tracking:**
- Tracks which articles are read
- Records like/skip feedback
- Measures reading time
- Updates preferences automatically

## Example Workflows

### Morning Briefing

```typescript
async function generateMorningBriefing(userId: string, userName: string) {
  const service = getNewsService();
  const generator = getNewsBriefingGenerator();

  // Fetch personalized news
  const articles = await service.getBriefing(userId, 5);

  // Generate speech-friendly briefing
  const briefing = await generator.generateBriefing(userId, articles, userName);

  return {
    text: briefing.content,
    duration: generator.estimateSpeakingDuration(briefing.content),
    articles: briefing.articles
  };
}

// Usage
const briefing = await generateMorningBriefing('user-123', 'Alice');
console.log(briefing.text);
// "Here are your top news stories, Alice: ..."
```

### Learning from Interactions

```typescript
async function trackArticleRead(userId: string, article: NewsArticle, readTime: number) {
  const service = getNewsService();

  // Record the read
  await service.updatePreferences(userId, article, 'read');

  // If user spent significant time, it's a strong signal
  if (readTime > 30000) { // 30 seconds
    const prefsManager = getNewsPreferencesManager();
    await prefsManager.learnFromFeedback(userId, article, {
      articleId: article.id,
      action: 'read',
      duration: readTime,
      timestamp: Date.now()
    });
  }
}
```

### Category-Based Briefing

```typescript
async function getCategoryBriefing(userId: string, category: string) {
  const service = getNewsService();
  const generator = getNewsBriefingGenerator();

  // Fetch topic-specific news
  const articles = await service.getTopicNews(userId, category, 10);

  // Generate category briefing
  const briefing = await generator.generateCategoryBriefing(articles, category);

  return briefing;
}

// Usage
const techBriefing = await getCategoryBriefing('user-123', 'technology');
```

## Rate Limits

### Free Tier (NewsAPI.org)
- 100 requests per day
- 15-minute cache reduces API calls
- Check current usage: `client.getRequestCount()`

### Caching Strategy

```typescript
const client = createNewsAPIClient();

// First call hits API
const articles1 = await client.getTopHeadlines({ category: 'tech' });

// Second call uses cache (within 15 minutes)
const articles2 = await client.getTopHeadlines({ category: 'tech' });

// Check if data is cached
const hasCached = client.hasCachedData('headlines');

// Clear cache manually
client.clearCache();
```

## Best Practices

### 1. Batch Requests

Instead of fetching each topic separately:

```typescript
// ❌ Bad: Multiple API calls
for (const topic of topics) {
  await fetchTopicNews(topic);
}

// ✅ Good: Single call with multiple categories
const allNews = await service.fetchNews(preferences);
```

### 2. Respect Rate Limits

```typescript
try {
  const articles = await client.getTopHeadlines(params);
} catch (error) {
  if (error.message.includes('Rate limit')) {
    // Show cached data or wait
    showCachedNews();
  }
}
```

### 3. Progressive Enhancement

```typescript
// Start with user preferences
let articles = await service.getBriefing(userId);

// If empty, fall back to top headlines
if (articles.length === 0) {
  const response = await client.getTopHeadlines({
    country: 'us',
    pageSize: 5
  });
  articles = response.articles;
}
```

### 4. Optimize for Voice

```typescript
const generator = getNewsBriefingGenerator();

// Generate concise speech output
const speech = await generator.formatForSpeech(articles);

// Limit to 3 articles for voice briefing
const briefArticles = articles.slice(0, 3);
```

## Troubleshooting

### "News API key not configured"

Set the environment variable:
```bash
VITE_NEWS_API_KEY=your_key_here
```

### "Rate limit exceeded"

You've hit the 100 requests/day limit. Solutions:
1. Wait until tomorrow
2. Use cached data
3. Upgrade to paid tier
4. Increase `CACHE_TTL` to reduce API calls

### "No articles returned"

Possible causes:
1. Invalid API key
2. No articles match filters
3. Network error

Debug:
```typescript
const client = createNewsAPIClient();
console.log('Request count:', client.getRequestCount());
console.log('Has cached data:', client.hasCachedData('headlines'));
```

### Personalization not working

Check if preferences are saved:
```typescript
const manager = getNewsPreferencesManager();
const prefs = await manager.getPreferences(userId);
console.log('Topics:', prefs.topics);
console.log('Weights:', prefs.topicWeights);
```

## API Reference

### Available Topics

```typescript
const topics = [
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
];
```

### Available Sources

```typescript
const sources = {
  bbc: 'BBC News',
  cnn: 'CNN',
  reuters: 'Reuters',
  'the-verge': 'The Verge',
  techcrunch: 'TechCrunch',
  'national-geographic': 'National Geographic',
  espn: 'ESPN',
  'associated-press': 'Associated Press'
};
```

### Reading Levels

- `'child'`: Simplified content, appropriate for ages 6-12
- `'teen'`: Moderate complexity, ages 13-17
- `'adult'`: Full content, no simplification

## Testing

### Unit Tests

```bash
npm test -- tests/integration/news-service.test.ts
```

### Manual Testing

```typescript
// Test preferences
const manager = getNewsPreferencesManager();
await manager.addTopic('test-user', 'technology');
const prefs = await manager.getPreferences('test-user');
console.assert(prefs.topics.includes('technology'));

// Test fetching
const service = getNewsService();
const articles = await service.fetchNews(prefs);
console.assert(articles.length > 0);

// Test learning
await service.updatePreferences('test-user', articles[0], 'read');
const weights = await manager.getTopicWeights('test-user');
console.log('Updated weights:', weights);
```

## Next Steps

1. Integrate with voice assistant for morning briefings
2. Add email integration for complete briefing
3. Implement push notifications for breaking news
4. Add calendar integration for contextual news

## Support

- Documentation: `/docs/guides/news-integration-guide.md`
- Contract: `docs/contracts/feature_voice.yml`
- Tests: `tests/integration/news-service.test.ts`
- Issue: [#95](https://github.com/Hulupeep/mbot_ruvector/issues/95)
