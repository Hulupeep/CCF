/**
 * Email Accounts Component
 *
 * Manage connected email accounts
 * data-testid: email-accounts
 */

import React, { useState, useEffect } from 'react';
import { EmailAccount } from '../../types/voice';
import { EmailService } from '../../services/email/EmailService';

const emailService = new EmailService();

interface EmailAccountsProps {
  userId: string;
}

export function EmailAccounts({ userId }: EmailAccountsProps) {
  const [accounts, setAccounts] = useState<EmailAccount[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadAccounts();
  }, [userId]);

  const loadAccounts = async () => {
    setLoading(true);
    try {
      const accts = await emailService.getAccounts(userId);
      setAccounts(accts);
    } finally {
      setLoading(false);
    }
  };

  const handleConnect = (provider: 'gmail' | 'outlook') => {
    // Would initiate OAuth2 flow
    console.log(`Connecting ${provider}...`);
  };

  const handleDisconnect = async (accountId: string) => {
    await emailService.disconnectAccount(accountId);
    await loadAccounts();
  };

  if (loading) {
    return <div>Loading email accounts...</div>;
  }

  return (
    <div data-testid="email-accounts" className="email-accounts">
      <h2>Email Accounts</h2>

      {accounts.length === 0 && (
        <div className="empty-state">
          <p>No email accounts connected.</p>
        </div>
      )}

      <div className="accounts-list">
        {accounts.map(account => (
          <div key={account.email} className="account-card">
            <div className="account-provider">
              {account.provider.toUpperCase()}
            </div>
            <div className="account-email">{account.email}</div>
            <div className="account-status">
              {account.expiresAt > Date.now() ? '✅ Connected' : '❌ Expired'}
            </div>
            <button
              onClick={() => handleDisconnect(account.email)}
              className="disconnect-btn"
            >
              Disconnect
            </button>
          </div>
        ))}
      </div>

      <div className="connect-section">
        <h3>Connect New Account</h3>
        <button
          onClick={() => handleConnect('gmail')}
          data-testid="connect-email-btn"
          className="connect-btn gmail"
        >
          Connect Gmail
        </button>
        <button
          onClick={() => handleConnect('outlook')}
          className="connect-btn outlook"
        >
          Connect Outlook
        </button>
      </div>
    </div>
  );
}
