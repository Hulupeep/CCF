/**
 * News Article List Component
 * Displays news articles with interaction tracking
 */

import React, { useState } from 'react';
import type { NewsArticle, UserFeedback } from '../../types/voice';

interface NewsArticleListProps {
  articles: NewsArticle[];
  userId: string;
  onArticleClick?: (article: NewsArticle) => void;
  onFeedback?: (article: NewsArticle, feedback: UserFeedback) => void;
}

export function NewsArticleList({
  articles,
  userId,
  onArticleClick,
  onFeedback
}: NewsArticleListProps) {
  const [expandedArticle, setExpandedArticle] = useState<string | null>(null);
  const [readArticles, setReadArticles] = useState<Set<string>>(new Set());

  const handleArticleClick = (article: NewsArticle) => {
    // Mark as read
    setReadArticles(prev => new Set([...prev, article.id]));

    // Track feedback
    if (onFeedback) {
      const feedback: UserFeedback = {
        articleId: article.id,
        action: 'read',
        timestamp: Date.now()
      };
      onFeedback(article, feedback);
    }

    // Toggle expansion
    setExpandedArticle(expandedArticle === article.id ? null : article.id);

    // Callback
    if (onArticleClick) {
      onArticleClick(article);
    }
  };

  const handleLike = (article: NewsArticle, e: React.MouseEvent) => {
    e.stopPropagation();

    if (onFeedback) {
      const feedback: UserFeedback = {
        articleId: article.id,
        action: 'like',
        timestamp: Date.now()
      };
      onFeedback(article, feedback);
    }
  };

  const handleSkip = (article: NewsArticle, e: React.MouseEvent) => {
    e.stopPropagation();

    if (onFeedback) {
      const feedback: UserFeedback = {
        articleId: article.id,
        action: 'skip',
        timestamp: Date.now()
      };
      onFeedback(article, feedback);
    }
  };

  const formatTimestamp = (timestamp: number): string => {
    const date = new Date(timestamp);
    const now = Date.now();
    const diff = now - timestamp;

    const hours = Math.floor(diff / (1000 * 60 * 60));
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (hours < 1) return 'Just now';
    if (hours < 24) return `${hours}h ago`;
    if (days < 7) return `${days}d ago`;
    return date.toLocaleDateString();
  };

  const getRelevanceColor = (score: number): string => {
    if (score >= 0.7) return '#4CAF50';
    if (score >= 0.4) return '#FFC107';
    return '#FF5722';
  };

  if (articles.length === 0) {
    return (
      <div className="news-article-list empty" data-testid="news-article-list">
        <p>No news articles available</p>
      </div>
    );
  }

  return (
    <div className="news-article-list" data-testid="news-article-list">
      {articles.map(article => {
        const isExpanded = expandedArticle === article.id;
        const isRead = readArticles.has(article.id);

        return (
          <article
            key={article.id}
            className={`news-article ${isRead ? 'read' : ''} ${isExpanded ? 'expanded' : ''}`}
            onClick={() => handleArticleClick(article)}
            data-testid={`news-article-${article.id}`}
          >
            {/* Article Image */}
            {article.imageUrl && (
              <div className="article-image">
                <img src={article.imageUrl} alt={article.headline} />
              </div>
            )}

            {/* Article Content */}
            <div className="article-content">
              {/* Header */}
              <div className="article-header">
                <div className="meta">
                  <span className="source">{article.source}</span>
                  <span className="separator">•</span>
                  <span className="time">{formatTimestamp(article.publishedAt)}</span>
                  {article.category && (
                    <>
                      <span className="separator">•</span>
                      <span className="category">{article.category}</span>
                    </>
                  )}
                </div>

                {/* Relevance Score */}
                <div
                  className="relevance-indicator"
                  style={{ color: getRelevanceColor(article.relevanceScore) }}
                  title={`Relevance: ${(article.relevanceScore * 100).toFixed(0)}%`}
                >
                  ●
                </div>
              </div>

              {/* Headline */}
              <h3 className="headline">{article.headline}</h3>

              {/* Summary */}
              {article.summary && (
                <p className="summary">{article.summary}</p>
              )}

              {/* Expanded Content */}
              {isExpanded && article.content && (
                <div className="full-content">
                  <p>{article.content}</p>

                  {article.author && (
                    <p className="author">By {article.author}</p>
                  )}

                  <a
                    href={article.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="read-more"
                    onClick={e => e.stopPropagation()}
                  >
                    Read full article →
                  </a>
                </div>
              )}

              {/* Actions */}
              <div className="article-actions">
                <button
                  className="action-btn like"
                  onClick={e => handleLike(article, e)}
                  title="Like this article"
                >
                  👍 Like
                </button>

                <button
                  className="action-btn skip"
                  onClick={e => handleSkip(article, e)}
                  title="Not interested"
                >
                  👎 Skip
                </button>

                <a
                  href={article.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="action-btn external"
                  onClick={e => e.stopPropagation()}
                >
                  🔗 Open
                </a>
              </div>
            </div>
          </article>
        );
      })}

      <style jsx>{`
        .news-article-list {
          display: flex;
          flex-direction: column;
          gap: 20px;
        }

        .news-article-list.empty {
          text-align: center;
          padding: 40px;
          color: #999;
        }

        .news-article {
          display: flex;
          gap: 15px;
          padding: 20px;
          background: white;
          border-radius: 12px;
          box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
          cursor: pointer;
          transition: all 0.2s;
        }

        .news-article:hover {
          box-shadow: 0 4px 16px rgba(0, 0, 0, 0.15);
          transform: translateY(-2px);
        }

        .news-article.read {
          opacity: 0.7;
        }

        .article-image {
          flex-shrink: 0;
          width: 200px;
          height: 150px;
          border-radius: 8px;
          overflow: hidden;
          background: #f0f0f0;
        }

        .article-image img {
          width: 100%;
          height: 100%;
          object-fit: cover;
        }

        .article-content {
          flex: 1;
          display: flex;
          flex-direction: column;
          gap: 10px;
        }

        .article-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .meta {
          display: flex;
          align-items: center;
          gap: 8px;
          font-size: 14px;
          color: #666;
        }

        .source {
          font-weight: 600;
          color: #333;
        }

        .separator {
          color: #ddd;
        }

        .category {
          text-transform: capitalize;
          color: #4CAF50;
        }

        .relevance-indicator {
          font-size: 24px;
        }

        .headline {
          margin: 0;
          font-size: 20px;
          line-height: 1.4;
          color: #222;
        }

        .summary {
          margin: 0;
          color: #555;
          line-height: 1.6;
        }

        .full-content {
          padding-top: 10px;
          border-top: 1px solid #e0e0e0;
        }

        .full-content p {
          color: #444;
          line-height: 1.6;
        }

        .author {
          font-size: 14px;
          font-style: italic;
          color: #666;
        }

        .read-more {
          display: inline-block;
          margin-top: 10px;
          color: #4CAF50;
          text-decoration: none;
          font-weight: 600;
        }

        .read-more:hover {
          text-decoration: underline;
        }

        .article-actions {
          display: flex;
          gap: 10px;
          margin-top: 10px;
        }

        .action-btn {
          padding: 8px 16px;
          border: 1px solid #ddd;
          background: white;
          border-radius: 6px;
          font-size: 14px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .action-btn:hover {
          background: #f5f5f5;
        }

        .action-btn.like:hover {
          background: #E8F5E9;
          border-color: #4CAF50;
        }

        .action-btn.skip:hover {
          background: #FFEBEE;
          border-color: #f44336;
        }

        .action-btn.external {
          text-decoration: none;
          color: inherit;
        }

        @media (max-width: 768px) {
          .news-article {
            flex-direction: column;
          }

          .article-image {
            width: 100%;
            height: 200px;
          }
        }
      `}</style>
    </div>
  );
}
