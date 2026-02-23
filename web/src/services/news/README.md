# News Service

Personalized news integration with learning capabilities for CCF on RuVector.

## Features

- Fetch news from NewsAPI.org
- User preference management
- Personalized relevance ranking
- Learning from user interactions
- Automated rate limiting and caching
- Voice-optimized briefing generation

## Architecture

```
NewsAPIClient
  ├── Fetches from NewsAPI.org
  ├── 15-minute caching
  └── Rate limiting (100 req/day)

NewsPreferencesManager
  ├── Topic preferences
  ├── Weight adjustments
  └── Learning algorithm

NewsService
  ├── Orchestrates fetching
  ├── Personalization (relevance scoring)
  └── Preference updates

NewsBriefingGenerator
  ├── Formats for speech
  ├── Estimates duration
  └── Creates summaries
```

## Quick Start

```typescript
import { getNewsService } from './news';

const service = getNewsService();
const articles = await service.getBriefing('user-123', 5);
```

See `/docs/guides/news-integration-guide.md` for full documentation.

## Contract Compliance

**I-NEWS-001:** News personalization with relevance ranking
- ✅ Filters news based on user preferences
- ✅ Ranks by relevance (0.0-1.0 scale)
- ✅ Learns from user interactions
- ✅ Adjusts topic weights automatically

## Testing

```bash
npm test -- tests/integration/news-service.test.ts
```

Coverage: >90%

## Components

| Component | Purpose | Lines |
|-----------|---------|-------|
| `NewsAPIClient.ts` | API communication | 200 |
| `NewsPreferencesManager.ts` | Preference management | 250 |
| `NewsService.ts` | Main orchestration | 350 |
| `NewsBriefingGenerator.ts` | Speech formatting | 250 |

Total: ~1,050 lines

## Environment Variables

```bash
VITE_NEWS_API_KEY=your_key_here
```

## UI Components

- `NewsPreferences.tsx` - Preference management UI (300 lines)
- `NewsArticleList.tsx` - Article display with feedback (250 lines)

## API Limits

Free tier (NewsAPI.org):
- 100 requests/day
- 15-minute cache TTL
- Automatic rate limit enforcement

## Next Steps

1. Integrate with voice assistant (#95)
2. Add email integration (#95)
3. Implement push notifications
4. Add calendar integration
