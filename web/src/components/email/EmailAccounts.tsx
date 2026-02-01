/**
 * Email Accounts Component - Display and manage connected email accounts
 *
 * Contract: I-EMAIL-001
 * Issue: #95 Component 3/5
 */

import React, { useState, useEffect } from 'react';
import { EmailAccount } from '../../types/voice';

interface EmailAccountsProps {
  userId: string;
  onDisconnect?: (account: EmailAccount) => void;
}

interface EmailAccountCardProps {
  account: EmailAccount;
  onDisconnect: (account: EmailAccount) => void;
}

const EmailAccountCard: React.FC<EmailAccountCardProps> = ({ account, onDisconnect }) => {
  const [disconnecting, setDisconnecting] = useState(false);

  const handleDisconnect = async () => {
    if (!confirm(`Are you sure you want to disconnect ${account.email}?`)) {
      return;
    }

    setDisconnecting(true);

    try {
      await fetch('/api/email/disconnect', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          userId: account.userId,
          provider: account.provider,
        }),
      });

      onDisconnect(account);
    } catch (err) {
      console.error('Failed to disconnect account:', err);
      alert('Failed to disconnect account. Please try again.');
    } finally {
      setDisconnecting(false);
    }
  };

  const getProviderIcon = () => {
    if (account.provider === 'gmail') {
      return (
        <svg viewBox="0 0 24 24" width="32" height="32">
          <path fill="#EA4335" d="M5,3l7,5v12l-7,-5v-12z" />
          <path fill="#34A853" d="M19,3l-7,5v12l7,-5v-12z" />
          <path fill="#FBBC04" d="M19,3l-7,5l-7,-5h14z" />
          <path fill="#4285F4" d="M5,20l7,-5l7,5h-14z" />
        </svg>
      );
    } else {
      return (
        <svg viewBox="0 0 24 24" width="32" height="32">
          <path
            fill="#0078D4"
            d="M7,4v16h10v-16h-10zm5,14c-2.209,0 -4,-1.791 -4,-4s1.791,-4 4,-4 4,1.791 4,4 -1.791,4 -4,4z"
          />
          <circle fill="#FFF" cx="12" cy="14" r="3" />
        </svg>
      );
    }
  };

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp);
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  const isTokenExpiring = () => {
    const hourFromNow = Date.now() + 60 * 60 * 1000;
    return account.expiresAt < hourFromNow;
  };

  return (
    <div className="email-account-card" data-testid={`email-account-${account.provider}`}>
      <div className="account-info">
        <div className="provider-icon">{getProviderIcon()}</div>
        <div className="account-details">
          <div className="account-email">{account.email}</div>
          <div className="account-meta">
            <span className="provider-badge">{account.provider}</span>
            <span className="last-synced">
              Last synced: {formatDate(account.lastSynced)}
            </span>
          </div>
          {isTokenExpiring() && (
            <div className="token-warning">
              ⚠️ Token expiring soon - will auto-refresh
            </div>
          )}
        </div>
      </div>
      <button
        className="disconnect-btn"
        onClick={handleDisconnect}
        disabled={disconnecting}
        data-testid={`disconnect-${account.provider}-btn`}
      >
        {disconnecting ? 'Disconnecting...' : 'Disconnect'}
      </button>

      <style>{`
        .email-account-card {
          display: flex;
          align-items: center;
          justify-content: space-between;
          padding: 16px;
          background: white;
          border: 1px solid #e0e0e0;
          border-radius: 8px;
          margin-bottom: 12px;
        }

        .account-info {
          display: flex;
          align-items: center;
          gap: 16px;
          flex: 1;
        }

        .provider-icon {
          flex-shrink: 0;
        }

        .account-details {
          flex: 1;
        }

        .account-email {
          font-size: 16px;
          font-weight: 600;
          margin-bottom: 4px;
        }

        .account-meta {
          display: flex;
          align-items: center;
          gap: 12px;
          font-size: 14px;
          color: #666;
        }

        .provider-badge {
          display: inline-block;
          padding: 2px 8px;
          background: #e3f2fd;
          color: #1976d2;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 600;
          text-transform: uppercase;
        }

        .token-warning {
          margin-top: 8px;
          padding: 4px 8px;
          background: #fff3e0;
          color: #e65100;
          border-radius: 4px;
          font-size: 12px;
        }

        .disconnect-btn {
          padding: 8px 16px;
          background: white;
          border: 1px solid #d32f2f;
          color: #d32f2f;
          border-radius: 6px;
          font-size: 14px;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s;
        }

        .disconnect-btn:hover:not(:disabled) {
          background: #d32f2f;
          color: white;
        }

        .disconnect-btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }
      `}</style>
    </div>
  );
};

export const EmailAccounts: React.FC<EmailAccountsProps> = ({ userId, onDisconnect }) => {
  const [accounts, setAccounts] = useState<EmailAccount[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    fetchAccounts();
  }, [userId]);

  const fetchAccounts = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await fetch(`/api/email/accounts?userId=${userId}`);

      if (!response.ok) {
        throw new Error('Failed to fetch accounts');
      }

      const data = await response.json();
      setAccounts(data.accounts || []);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const handleDisconnect = (account: EmailAccount) => {
    setAccounts(prev => prev.filter(a => a.email !== account.email));
    onDisconnect?.(account);
  };

  if (loading) {
    return (
      <div className="email-accounts-loading" data-testid="email-accounts-loading">
        <div className="spinner" />
        <p>Loading email accounts...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="email-accounts-error" data-testid="email-accounts-error">
        <p>Error: {error}</p>
        <button onClick={fetchAccounts}>Retry</button>
      </div>
    );
  }

  return (
    <div className="email-accounts" data-testid="email-accounts">
      <div className="email-accounts-header">
        <h3>Connected Email Accounts</h3>
        {accounts.length > 0 && (
          <p className="account-count">{accounts.length} account(s) connected</p>
        )}
      </div>

      {accounts.length === 0 ? (
        <div className="no-accounts" data-testid="no-email-accounts">
          <p>No email accounts connected yet.</p>
          <p>Connect an account to receive email summaries in your briefings.</p>
        </div>
      ) : (
        <div className="accounts-list">
          {accounts.map(account => (
            <EmailAccountCard
              key={account.email}
              account={account}
              onDisconnect={handleDisconnect}
            />
          ))}
        </div>
      )}

      <style>{`
        .email-accounts {
          padding: 24px;
        }

        .email-accounts-header {
          margin-bottom: 20px;
        }

        .email-accounts-header h3 {
          font-size: 22px;
          margin-bottom: 4px;
        }

        .account-count {
          color: #666;
          font-size: 14px;
        }

        .no-accounts {
          text-align: center;
          padding: 40px 20px;
          background: #f8f9fa;
          border-radius: 8px;
          color: #666;
        }

        .no-accounts p {
          margin: 8px 0;
        }

        .accounts-list {
          /* Cards are styled in EmailAccountCard */
        }

        .email-accounts-loading {
          text-align: center;
          padding: 40px;
        }

        .email-accounts-loading .spinner {
          display: inline-block;
          width: 40px;
          height: 40px;
          border: 4px solid #f3f3f3;
          border-top: 4px solid #4285F4;
          border-radius: 50%;
          animation: spin 1s linear infinite;
          margin-bottom: 16px;
        }

        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }

        .email-accounts-error {
          text-align: center;
          padding: 40px;
          color: #d32f2f;
        }

        .email-accounts-error button {
          margin-top: 16px;
          padding: 8px 16px;
          background: #4285F4;
          color: white;
          border: none;
          border-radius: 6px;
          cursor: pointer;
        }
      `}</style>
    </div>
  );
};
