/**
 * Cloud Sync Panel Component
 * Contract: I-CLOUD-001, I-CLOUD-002, I-CLOUD-003
 * Journey: J-CLOUD-FIRST-SYNC
 *
 * Provides UI for cloud sync authentication and status
 */

import React, { useState } from 'react';
import { useCloudSync } from '../hooks/useCloudSync';
import type { SyncStatus } from '../services/cloudSync';

interface CloudSyncPanelProps {
  className?: string;
  showAuthForm?: boolean;
}

/**
 * CloudSyncPanel displays sync status and provides authentication UI
 *
 * Features:
 * - Sign in/sign up forms
 * - Sync status indicator
 * - Force sync button
 * - Pending operations counter
 * - Error display
 *
 * data-testid attributes per issue #84 spec:
 * - cloud-sync-panel: Main container
 * - cloud-sync-status: Status indicator
 * - cloud-sync-sign-in: Sign in button
 * - cloud-sync-sign-out: Sign out button
 * - cloud-sync-force-sync: Force sync button
 * - cloud-sync-email-input: Email input
 * - cloud-sync-password-input: Password input
 * - cloud-sync-error: Error message
 * - cloud-sync-pending-count: Pending operations count
 */
export function CloudSyncPanel({ className = '', showAuthForm = true }: CloudSyncPanelProps) {
  const {
    user,
    isAuthenticated,
    syncStatus,
    lastSync,
    error,
    pendingOperations,
    signIn,
    signUp,
    signOut,
    forceSyncAll,
  } = useCloudSync();

  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [isSignUp, setIsSignUp] = useState(false);
  const [authError, setAuthError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const handleAuth = async (e: React.FormEvent) => {
    e.preventDefault();
    setAuthError(null);
    setIsLoading(true);

    try {
      if (isSignUp) {
        await signUp(email, password);
      } else {
        await signIn(email, password);
      }
      setEmail('');
      setPassword('');
    } catch (err) {
      setAuthError(err instanceof Error ? err.message : 'Authentication failed');
    } finally {
      setIsLoading(false);
    }
  };

  const handleSignOut = async () => {
    setIsLoading(true);
    try {
      await signOut();
    } catch (err) {
      console.error('Sign out failed:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleForceSync = async () => {
    setIsLoading(true);
    try {
      await forceSyncAll();
    } catch (err) {
      console.error('Force sync failed:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const getSyncStatusIcon = (status: SyncStatus): string => {
    switch (status) {
      case 'idle': return '⚪';
      case 'syncing': return '🔄';
      case 'synced': return '✅';
      case 'error': return '❌';
      case 'offline': return '📴';
      default: return '⚪';
    }
  };

  const getSyncStatusText = (status: SyncStatus): string => {
    switch (status) {
      case 'idle': return 'Ready to sync';
      case 'syncing': return 'Syncing...';
      case 'synced': return 'All synced';
      case 'error': return 'Sync error';
      case 'offline': return 'Offline';
      default: return 'Unknown';
    }
  };

  const formatLastSync = (timestamp: number | null): string => {
    if (!timestamp) return 'Never';

    const now = Date.now();
    const diff = now - timestamp;
    const seconds = Math.floor(diff / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);

    if (hours > 0) return `${hours}h ago`;
    if (minutes > 0) return `${minutes}m ago`;
    if (seconds > 0) return `${seconds}s ago`;
    return 'Just now';
  };

  return (
    <div
      data-testid="cloud-sync-panel"
      className={`cloud-sync-panel ${className}`}
      style={{
        padding: '1rem',
        border: '1px solid #e0e0e0',
        borderRadius: '8px',
        backgroundColor: '#fafafa',
      }}
    >
      <h3 style={{ marginTop: 0, marginBottom: '1rem' }}>
        Cloud Sync
      </h3>

      {/* Sync Status */}
      <div
        data-testid="cloud-sync-status"
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '0.5rem',
          marginBottom: '1rem',
          padding: '0.5rem',
          backgroundColor: '#fff',
          borderRadius: '4px',
        }}
      >
        <span style={{ fontSize: '1.5rem' }}>
          {getSyncStatusIcon(syncStatus)}
        </span>
        <div style={{ flex: 1 }}>
          <div style={{ fontWeight: 'bold' }}>
            {getSyncStatusText(syncStatus)}
          </div>
          <div style={{ fontSize: '0.875rem', color: '#666' }}>
            Last sync: {formatLastSync(lastSync)}
          </div>
        </div>
        {pendingOperations > 0 && (
          <div
            data-testid="cloud-sync-pending-count"
            style={{
              padding: '0.25rem 0.5rem',
              backgroundColor: '#ff9800',
              color: '#fff',
              borderRadius: '12px',
              fontSize: '0.875rem',
              fontWeight: 'bold',
            }}
          >
            {pendingOperations} pending
          </div>
        )}
      </div>

      {/* Error Display */}
      {(error || authError) && (
        <div
          data-testid="cloud-sync-error"
          style={{
            padding: '0.75rem',
            backgroundColor: '#ffebee',
            color: '#c62828',
            borderRadius: '4px',
            marginBottom: '1rem',
            fontSize: '0.875rem',
          }}
        >
          {error || authError}
        </div>
      )}

      {/* Authenticated View */}
      {isAuthenticated ? (
        <div>
          <div
            style={{
              padding: '0.75rem',
              backgroundColor: '#e8f5e9',
              borderRadius: '4px',
              marginBottom: '1rem',
            }}
          >
            <div style={{ fontWeight: 'bold', color: '#2e7d32', marginBottom: '0.25rem' }}>
              Signed in
            </div>
            <div style={{ fontSize: '0.875rem', color: '#666' }}>
              {user?.email}
            </div>
          </div>

          <div style={{ display: 'flex', gap: '0.5rem' }}>
            <button
              data-testid="cloud-sync-force-sync"
              onClick={handleForceSync}
              disabled={isLoading || syncStatus === 'syncing'}
              style={{
                flex: 1,
                padding: '0.5rem 1rem',
                backgroundColor: '#2196f3',
                color: '#fff',
                border: 'none',
                borderRadius: '4px',
                cursor: isLoading ? 'not-allowed' : 'pointer',
                opacity: isLoading ? 0.6 : 1,
              }}
            >
              {syncStatus === 'syncing' ? 'Syncing...' : 'Force Sync'}
            </button>

            <button
              data-testid="cloud-sync-sign-out"
              onClick={handleSignOut}
              disabled={isLoading}
              style={{
                padding: '0.5rem 1rem',
                backgroundColor: '#f44336',
                color: '#fff',
                border: 'none',
                borderRadius: '4px',
                cursor: isLoading ? 'not-allowed' : 'pointer',
                opacity: isLoading ? 0.6 : 1,
              }}
            >
              Sign Out
            </button>
          </div>
        </div>
      ) : (
        // Authentication Form
        showAuthForm && (
          <form onSubmit={handleAuth}>
            <div style={{ marginBottom: '1rem' }}>
              <label
                htmlFor="cloud-sync-email"
                style={{
                  display: 'block',
                  marginBottom: '0.25rem',
                  fontSize: '0.875rem',
                  fontWeight: 'bold',
                }}
              >
                Email
              </label>
              <input
                id="cloud-sync-email"
                data-testid="cloud-sync-email-input"
                type="email"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                required
                disabled={isLoading}
                style={{
                  width: '100%',
                  padding: '0.5rem',
                  border: '1px solid #ccc',
                  borderRadius: '4px',
                  fontSize: '1rem',
                }}
              />
            </div>

            <div style={{ marginBottom: '1rem' }}>
              <label
                htmlFor="cloud-sync-password"
                style={{
                  display: 'block',
                  marginBottom: '0.25rem',
                  fontSize: '0.875rem',
                  fontWeight: 'bold',
                }}
              >
                Password
              </label>
              <input
                id="cloud-sync-password"
                data-testid="cloud-sync-password-input"
                type="password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                disabled={isLoading}
                minLength={6}
                style={{
                  width: '100%',
                  padding: '0.5rem',
                  border: '1px solid #ccc',
                  borderRadius: '4px',
                  fontSize: '1rem',
                }}
              />
            </div>

            <button
              data-testid="cloud-sync-sign-in"
              type="submit"
              disabled={isLoading}
              style={{
                width: '100%',
                padding: '0.75rem',
                backgroundColor: '#4caf50',
                color: '#fff',
                border: 'none',
                borderRadius: '4px',
                fontSize: '1rem',
                fontWeight: 'bold',
                cursor: isLoading ? 'not-allowed' : 'pointer',
                opacity: isLoading ? 0.6 : 1,
              }}
            >
              {isLoading ? 'Loading...' : isSignUp ? 'Sign Up' : 'Sign In'}
            </button>

            <button
              type="button"
              onClick={() => setIsSignUp(!isSignUp)}
              disabled={isLoading}
              style={{
                width: '100%',
                marginTop: '0.5rem',
                padding: '0.5rem',
                backgroundColor: 'transparent',
                color: '#2196f3',
                border: 'none',
                borderRadius: '4px',
                fontSize: '0.875rem',
                cursor: isLoading ? 'not-allowed' : 'pointer',
                textDecoration: 'underline',
              }}
            >
              {isSignUp ? 'Already have an account? Sign In' : 'Need an account? Sign Up'}
            </button>
          </form>
        )
      )}

      {/* Info Text */}
      <div
        style={{
          marginTop: '1rem',
          padding: '0.75rem',
          backgroundColor: '#e3f2fd',
          borderRadius: '4px',
          fontSize: '0.75rem',
          color: '#1565c0',
        }}
      >
        <strong>Cloud Sync Info:</strong>
        <ul style={{ margin: '0.5rem 0 0 0', paddingLeft: '1.5rem' }}>
          <li>Automatically syncs personalities, drawings, and game stats</li>
          <li>Works offline - syncs when connection is restored</li>
          <li>Data is encrypted with AES-256</li>
        </ul>
      </div>
    </div>
  );
}
