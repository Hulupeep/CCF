/**
 * Marketplace Browser Component
 * Issue #85 - Cloud Personality Marketplace (FUTURE - DOD)
 *
 * Implements:
 * - I-CLOUD-004: Marketplace personalities must pass validation before publication
 * - I-CLOUD-005: Rating system prevents abuse (1 rating per user per personality)
 * - I-CLOUD-006: Search results must return within 500ms for up to 10,000 personalities
 *
 * Features:
 * - Browse, search, filter, and sort personalities
 * - Download personalities to local presets
 * - Rate personalities (1-5 stars, once per user)
 * - Report inappropriate content
 * - Preview personality parameters before download
 * - Publish custom personalities
 */

import React, { useState, useCallback, useEffect } from 'react';
import { useMarketplace } from '../../hooks/marketplace/useMarketplace';
import { useLocalStorage } from '../../hooks/useLocalStorage';
import { PersonalityConfig, CustomPersonality } from '../../types/personality';
import { MarketplaceListing, SearchQuery } from '../../types/marketplace';

interface MarketplaceBrowserProps {
  onPersonalityLoad?: (config: PersonalityConfig) => void;
  currentConfig?: PersonalityConfig;
  className?: string;
}

export const MarketplaceBrowser: React.FC<MarketplaceBrowserProps> = ({
  onPersonalityLoad,
  currentConfig,
  className = '',
}) => {
  const marketplace = useMarketplace();
  const [customPersonalities, setCustomPersonalities] = useLocalStorage<CustomPersonality[]>(
    'mbot-custom-personalities',
    []
  );

  // UI state
  const [activeTab, setActiveTab] = useState<'browse' | 'my-published'>('browse');
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedTags, setSelectedTags] = useState<string[]>([]);
  const [minRating, setMinRating] = useState<number>(0);
  const [sortBy, setSortBy] = useState<'popular' | 'rating' | 'newest' | 'downloads'>('popular');
  const [currentPage, setCurrentPage] = useState(1);

  // Modals
  const [previewPersonality, setPreviewPersonality] = useState<MarketplaceListing | null>(null);
  const [publishDialogOpen, setPublishDialogOpen] = useState(false);
  const [reportDialogOpen, setReportDialogOpen] = useState(false);
  const [reportPersonalityId, setReportPersonalityId] = useState<string | null>(null);
  const [reportReason, setReportReason] = useState('');

  // Publish form
  const [publishDescription, setPublishDescription] = useState('');
  const [publishTags, setPublishTags] = useState('');

  // Load user published on tab switch
  useEffect(() => {
    if (activeTab === 'my-published') {
      marketplace.loadUserPublished();
    }
  }, [activeTab, marketplace]);

  // Search
  const handleSearch = useCallback(() => {
    const query: SearchQuery = {
      query: searchQuery || undefined,
      tags: selectedTags.length > 0 ? selectedTags : undefined,
      minRating: minRating > 0 ? minRating : undefined,
      sortBy,
      page: currentPage,
      limit: 20,
    };
    marketplace.search(query);
  }, [searchQuery, selectedTags, minRating, sortBy, currentPage, marketplace]);

  // Auto-search on filter change
  useEffect(() => {
    handleSearch();
  }, [handleSearch]);

  // Download personality
  const handleDownload = useCallback(
    async (listing: MarketplaceListing) => {
      try {
        const config = await marketplace.download(listing.id);

        // Add to local custom personalities
        const custom: CustomPersonality = {
          name: `${listing.name} (Marketplace)`,
          config,
          created_at: Date.now(),
        };
        setCustomPersonalities((prev) => [...prev, custom]);

        // Load into mixer
        onPersonalityLoad?.(config);

        alert(`Downloaded "${listing.name}" successfully!`);
      } catch (error) {
        alert(`Failed to download: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    },
    [marketplace, setCustomPersonalities, onPersonalityLoad]
  );

  // Rate personality
  const handleRate = useCallback(
    async (personalityId: string, rating: number) => {
      try {
        await marketplace.rate(personalityId, rating);
        alert('Rating submitted successfully!');
      } catch (error) {
        alert(`Failed to rate: ${error instanceof Error ? error.message : 'Already rated'}`);
      }
    },
    [marketplace]
  );

  // Open report dialog
  const handleOpenReport = useCallback((personalityId: string) => {
    setReportPersonalityId(personalityId);
    setReportDialogOpen(true);
  }, []);

  // Submit report
  const handleSubmitReport = useCallback(async () => {
    if (!reportPersonalityId || !reportReason.trim()) return;

    try {
      await marketplace.report(reportPersonalityId, reportReason);
      alert('Report submitted successfully. Thank you!');
      setReportDialogOpen(false);
      setReportReason('');
      setReportPersonalityId(null);
    } catch (error) {
      alert(`Failed to report: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }, [reportPersonalityId, reportReason, marketplace]);

  // Publish personality
  const handlePublish = useCallback(async () => {
    if (!currentConfig) {
      alert('No personality configuration available');
      return;
    }

    if (!publishDescription.trim() || !publishTags.trim()) {
      alert('Please fill in all fields');
      return;
    }

    try {
      const tags = publishTags
        .split(',')
        .map((t) => t.trim().toLowerCase())
        .filter((t) => t.length > 0);

      await marketplace.publish(currentConfig, {
        description: publishDescription,
        tags,
      });

      alert('Personality published successfully!');
      setPublishDialogOpen(false);
      setPublishDescription('');
      setPublishTags('');
    } catch (error) {
      alert(`Failed to publish: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }, [currentConfig, publishDescription, publishTags, marketplace]);

  // Unpublish personality
  const handleUnpublish = useCallback(
    async (personalityId: string) => {
      if (!confirm('Are you sure you want to unpublish this personality?')) return;

      try {
        await marketplace.unpublish(personalityId);
        alert('Personality unpublished successfully');
      } catch (error) {
        alert(`Failed to unpublish: ${error instanceof Error ? error.message : 'Unknown error'}`);
      }
    },
    [marketplace]
  );

  // Popular tags (mock - would come from backend)
  const popularTags = ['energetic', 'calm', 'curious', 'playful', 'creative', 'focused'];

  return (
    <div className={`marketplace-browser ${className}`} data-testid="marketplace-browser">
      {/* Tabs */}
      <div className="marketplace-tabs">
        <button
          onClick={() => setActiveTab('browse')}
          className={activeTab === 'browse' ? 'active' : ''}
          data-testid="marketplace-tab"
        >
          Browse Marketplace
        </button>
        <button
          onClick={() => setActiveTab('my-published')}
          className={activeTab === 'my-published' ? 'active' : ''}
          data-testid="my-published-tab"
        >
          My Published
        </button>
        <button
          onClick={() => setPublishDialogOpen(true)}
          className="publish-btn"
          data-testid="publish-personality-btn"
        >
          Publish Current
        </button>
      </div>

      {/* Browse Tab */}
      {activeTab === 'browse' && (
        <div className="browse-tab">
          {/* Search and Filters */}
          <div className="search-filters">
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search personalities..."
              data-testid="marketplace-search"
              className="search-input"
            />

            {/* Tag Filter */}
            <div className="tag-filter" data-testid="marketplace-tag-filter">
              <label>Tags:</label>
              <div className="tag-buttons">
                {popularTags.map((tag) => (
                  <button
                    key={tag}
                    onClick={() =>
                      setSelectedTags((prev) =>
                        prev.includes(tag) ? prev.filter((t) => t !== tag) : [...prev, tag]
                      )
                    }
                    className={selectedTags.includes(tag) ? 'active' : ''}
                    data-testid={`tag-${tag}`}
                  >
                    {tag}
                  </button>
                ))}
              </div>
            </div>

            {/* Rating Filter */}
            <div className="rating-filter" data-testid="marketplace-rating-filter">
              <label>Min Rating:</label>
              <select value={minRating} onChange={(e) => setMinRating(Number(e.target.value))}>
                <option value={0}>Any</option>
                <option value={3}>3+ stars</option>
                <option value={4}>4+ stars</option>
                <option value={4.5}>4.5+ stars</option>
              </select>
            </div>

            {/* Sort */}
            <div className="sort-select" data-testid="marketplace-sort">
              <label>Sort by:</label>
              <select value={sortBy} onChange={(e) => setSortBy(e.target.value as any)}>
                <option value="popular">Most Popular</option>
                <option value="rating">Highest Rated</option>
                <option value="newest">Newest</option>
                <option value="downloads">Most Downloaded</option>
              </select>
            </div>
          </div>

          {/* Trending */}
          {!marketplace.searchLoading && !marketplace.searchResults && (
            <div className="trending-section">
              <h2>Trending Personalities</h2>
              <div className="personality-grid">
                {marketplace.trending.map((listing) => (
                  <PersonalityCard
                    key={listing.id}
                    listing={listing}
                    onDownload={handleDownload}
                    onRate={handleRate}
                    onReport={handleOpenReport}
                    onPreview={setPreviewPersonality}
                  />
                ))}
              </div>
            </div>
          )}

          {/* Search Results */}
          {marketplace.searchLoading && <div className="loading">Searching...</div>}
          {marketplace.searchError && (
            <div className="error" data-testid="search-error">
              {marketplace.searchError}
            </div>
          )}
          {marketplace.searchResults && (
            <div className="search-results">
              <h2>
                Found {marketplace.searchResults.total} personalities
              </h2>
              <div className="personality-grid">
                {marketplace.searchResults.listings.map((listing) => (
                  <PersonalityCard
                    key={listing.id}
                    listing={listing}
                    onDownload={handleDownload}
                    onRate={handleRate}
                    onReport={handleOpenReport}
                    onPreview={setPreviewPersonality}
                  />
                ))}
              </div>

              {/* Pagination */}
              {marketplace.searchResults.hasMore && (
                <button
                  onClick={() => setCurrentPage((p) => p + 1)}
                  className="load-more"
                  data-testid="load-more"
                >
                  Load More
                </button>
              )}
            </div>
          )}
        </div>
      )}

      {/* My Published Tab */}
      {activeTab === 'my-published' && (
        <div className="my-published-tab">
          <h2>My Published Personalities</h2>
          {marketplace.userPublishedLoading && <div className="loading">Loading...</div>}
          {marketplace.userPublished.length === 0 && !marketplace.userPublishedLoading && (
            <div className="empty-state">
              <p>You haven't published any personalities yet.</p>
              <button onClick={() => setPublishDialogOpen(true)}>Publish Your First</button>
            </div>
          )}
          <div className="personality-grid">
            {marketplace.userPublished.map((listing) => (
              <div key={listing.id} className="published-card" data-testid={`published-${listing.id}`}>
                <PersonalityCard
                  listing={listing}
                  onDownload={handleDownload}
                  onRate={handleRate}
                  onReport={handleOpenReport}
                  onPreview={setPreviewPersonality}
                  showUnpublish
                  onUnpublish={handleUnpublish}
                />
                {!listing.validated && (
                  <div className="validation-errors">
                    <strong>Validation Failed:</strong>
                    <ul>
                      {listing.validationErrors?.map((error, i) => (
                        <li key={i}>{error}</li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Preview Modal */}
      {previewPersonality && (
        <PreviewModal
          listing={previewPersonality}
          onClose={() => setPreviewPersonality(null)}
          onDownload={handleDownload}
        />
      )}

      {/* Publish Dialog */}
      {publishDialogOpen && (
        <PublishDialog
          description={publishDescription}
          tags={publishTags}
          onDescriptionChange={setPublishDescription}
          onTagsChange={setPublishTags}
          onPublish={handlePublish}
          onCancel={() => setPublishDialogOpen(false)}
        />
      )}

      {/* Report Dialog */}
      {reportDialogOpen && (
        <ReportDialog
          reason={reportReason}
          onReasonChange={setReportReason}
          onSubmit={handleSubmitReport}
          onCancel={() => {
            setReportDialogOpen(false);
            setReportReason('');
            setReportPersonalityId(null);
          }}
        />
      )}
    </div>
  );
};

// Personality Card Component
interface PersonalityCardProps {
  listing: MarketplaceListing;
  onDownload: (listing: MarketplaceListing) => void;
  onRate: (personalityId: string, rating: number) => void;
  onReport: (personalityId: string) => void;
  onPreview: (listing: MarketplaceListing) => void;
  showUnpublish?: boolean;
  onUnpublish?: (personalityId: string) => void;
}

const PersonalityCard: React.FC<PersonalityCardProps> = ({
  listing,
  onDownload,
  onRate,
  onReport,
  onPreview,
  showUnpublish,
  onUnpublish,
}) => {
  const [userRating, setUserRating] = useState<number>(0);

  const handleRateClick = (rating: number) => {
    setUserRating(rating);
    onRate(listing.id, rating);
  };

  return (
    <div className="personality-card" data-testid={`personality-card-${listing.id}`}>
      {listing.thumbnailUrl && (
        <img src={listing.thumbnailUrl} alt={listing.name} className="thumbnail" />
      )}
      <h3>{listing.name}</h3>
      <p className="author">by {listing.authorName}</p>
      <p className="description">{listing.description}</p>

      {/* Tags */}
      <div className="tags">
        {listing.tags.map((tag) => (
          <span key={tag} className="tag">
            {tag}
          </span>
        ))}
      </div>

      {/* Metrics */}
      <div className="metrics">
        <span className="rating" data-testid={`star-rating-${listing.id}`}>
          ⭐ {listing.rating.toFixed(1)} ({listing.ratingCount})
        </span>
        <span className="downloads">📥 {listing.downloadCount}</span>
      </div>

      {/* Actions */}
      <div className="actions">
        <button
          onClick={() => onPreview(listing)}
          data-testid={`preview-personality-${listing.id}`}
          className="btn-secondary"
        >
          Preview
        </button>
        <button
          onClick={() => onDownload(listing)}
          data-testid={`download-personality-${listing.id}`}
          className="btn-primary"
        >
          Download
        </button>
      </div>

      {/* Rating */}
      <div className="rate-section" data-testid={`rate-personality-${listing.id}`}>
        <label>Rate this:</label>
        <div className="star-buttons">
          {[1, 2, 3, 4, 5].map((star) => (
            <button
              key={star}
              onClick={() => handleRateClick(star)}
              className={star <= userRating ? 'active' : ''}
              data-testid={`rate-star-${listing.id}-${star}`}
            >
              ⭐
            </button>
          ))}
        </div>
      </div>

      {/* Report */}
      <button
        onClick={() => onReport(listing.id)}
        data-testid={`report-personality-${listing.id}`}
        className="report-btn"
      >
        🚩 Report
      </button>

      {/* Unpublish (owner only) */}
      {showUnpublish && onUnpublish && (
        <button
          onClick={() => onUnpublish(listing.id)}
          data-testid={`unpublish-personality-${listing.id}`}
          className="unpublish-btn"
        >
          Unpublish
        </button>
      )}
    </div>
  );
};

// Preview Modal Component
interface PreviewModalProps {
  listing: MarketplaceListing;
  onClose: () => void;
  onDownload: (listing: MarketplaceListing) => void;
}

const PreviewModal: React.FC<PreviewModalProps> = ({ listing, onClose, onDownload }) => {
  return (
    <div className="modal-overlay" data-testid={`preview-modal-${listing.id}`}>
      <div className="modal preview-modal">
        <h2>{listing.name}</h2>
        <p className="author">by {listing.authorName}</p>
        <p className="description">{listing.description}</p>

        <h3>Personality Parameters</h3>
        <div className="parameter-preview">
          {Object.entries(listing.config).map(([key, value]) => (
            <div key={key} className="param-row">
              <span className="param-name">{key.replace(/_/g, ' ')}:</span>
              <span className="param-value">{(value as number).toFixed(2)}</span>
              <div className="param-bar" style={{ width: `${(value as number) * 100}%` }} />
            </div>
          ))}
        </div>

        <div className="modal-actions">
          <button onClick={() => onDownload(listing)} className="btn-primary">
            Download
          </button>
          <button onClick={onClose} className="btn-secondary">
            Close
          </button>
        </div>
      </div>
    </div>
  );
};

// Publish Dialog Component
interface PublishDialogProps {
  description: string;
  tags: string;
  onDescriptionChange: (value: string) => void;
  onTagsChange: (value: string) => void;
  onPublish: () => void;
  onCancel: () => void;
}

const PublishDialog: React.FC<PublishDialogProps> = ({
  description,
  tags,
  onDescriptionChange,
  onTagsChange,
  onPublish,
  onCancel,
}) => {
  return (
    <div className="modal-overlay" data-testid="publish-dialog">
      <div className="modal publish-dialog">
        <h2>Publish Personality</h2>
        <p>Share your custom personality with the community!</p>

        <div className="form-group">
          <label>Description (first line will be the title):</label>
          <textarea
            value={description}
            onChange={(e) => onDescriptionChange(e.target.value)}
            placeholder="My Amazing Personality\n\nThis personality is great for..."
            rows={6}
            data-testid="publish-description"
          />
        </div>

        <div className="form-group">
          <label>Tags (comma-separated):</label>
          <input
            type="text"
            value={tags}
            onChange={(e) => onTagsChange(e.target.value)}
            placeholder="energetic, playful, creative"
            data-testid="publish-tags"
          />
          <small>Use lowercase, numbers, and hyphens only</small>
        </div>

        <div className="modal-actions">
          <button onClick={onPublish} className="btn-primary" data-testid="publish-confirm">
            Publish
          </button>
          <button onClick={onCancel} className="btn-secondary" data-testid="publish-cancel">
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
};

// Report Dialog Component
interface ReportDialogProps {
  reason: string;
  onReasonChange: (value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
}

const ReportDialog: React.FC<ReportDialogProps> = ({
  reason,
  onReasonChange,
  onSubmit,
  onCancel,
}) => {
  return (
    <div className="modal-overlay" data-testid="report-dialog">
      <div className="modal report-dialog">
        <h2>Report Personality</h2>
        <p>Please describe why this content is inappropriate:</p>

        <textarea
          value={reason}
          onChange={(e) => onReasonChange(e.target.value)}
          placeholder="This personality contains..."
          rows={4}
          data-testid="report-reason"
        />

        <div className="modal-actions">
          <button onClick={onSubmit} className="btn-primary" data-testid="report-submit">
            Submit Report
          </button>
          <button onClick={onCancel} className="btn-secondary" data-testid="report-cancel">
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
};

export default MarketplaceBrowser;
