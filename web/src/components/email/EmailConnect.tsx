/**
 * Email Connect Component - OAuth2 email account connection UI
 *
 * Contract: I-EMAIL-001
 * Issue: #95 Component 3/5
 */

import React, { useState, useEffect } from 'react';
import { GmailClient } from '../../services/email/GmailClient';
import { OutlookClient } from '../../services/email/OutlookClient';

interface EmailConnectProps {
  userId: string;
  onConnected: (provider: 'gmail' | 'outlook', account: any) => void;
  onError: (error: string) => void;
}

export const EmailConnect: React.FC<EmailConnectProps> = ({
  userId,
  onConnected,
  onError,
}) => {
  const [connecting, setConnecting] = useState(false);
  const [provider, setProvider] = useState<'gmail' | 'outlook' | null>(null);

  useEffect(() => {
    // Check for OAuth callback
    const params = new URLSearchParams(window.location.search);
    const code = params.get('code');
    const state = params.get('state');
    const error = params.get('error');

    if (error) {
      onError(`OAuth error: ${error}`);
      return;
    }

    if (code && state) {
      const providerFromState = state as 'gmail' | 'outlook';
      handleOAuthCallback(code, providerFromState);
    }
  }, []);

  const handleOAuthCallback = async (code: string, provider: 'gmail' | 'outlook') => {
    setConnecting(true);
    setProvider(provider);

    try {
      // Exchange code for tokens (handled by backend)
      const response = await fetch('/api/email/connect', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          provider,
          code,
          userId,
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Failed to connect account');
      }

      const account = await response.json();
      onConnected(provider, account);

      // Clean up URL
      window.history.replaceState({}, document.title, window.location.pathname);
    } catch (err) {
      onError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setConnecting(false);
      setProvider(null);
    }
  };

  const handleConnectGmail = async () => {
    setConnecting(true);
    setProvider('gmail');

    try {
      const client = new GmailClient();
      const authUrl = await client.getAuthUrl();

      // Add state parameter for OAuth flow
      const urlWithState = `${authUrl}&state=gmail`;

      // Redirect to Google OAuth
      window.location.href = urlWithState;
    } catch (err) {
      onError(err instanceof Error ? err.message : 'Failed to connect Gmail');
      setConnecting(false);
      setProvider(null);
    }
  };

  const handleConnectOutlook = async () => {
    setConnecting(true);
    setProvider('outlook');

    try {
      const client = new OutlookClient();
      const authUrl = await client.getAuthUrl();

      // Add state parameter for OAuth flow
      const urlWithState = `${authUrl}&state=outlook`;

      // Redirect to Microsoft OAuth
      window.location.href = urlWithState;
    } catch (err) {
      onError(err instanceof Error ? err.message : 'Failed to connect Outlook');
      setConnecting(false);
      setProvider(null);
    }
  };

  return (
    <div className="email-connect" data-testid="email-connect">
      <div className="email-connect-header">
        <h2>Connect Email Account</h2>
        <p>Connect your Gmail or Outlook account to receive personalized email summaries.</p>
      </div>

      <div className="email-connect-providers">
        <button
          className="email-provider-btn gmail-btn"
          onClick={handleConnectGmail}
          disabled={connecting}
          data-testid="connect-gmail-btn"
        >
          {connecting && provider === 'gmail' ? (
            <>
              <span className="spinner" />
              Connecting to Gmail...
            </>
          ) : (
            <>
              <svg className="provider-icon" viewBox="0 0 24 24" width="24" height="24">
                <path
                  fill="#EA4335"
                  d="M5,3l7,5v12l-7,-5v-12z"
                />
                <path
                  fill="#34A853"
                  d="M19,3l-7,5v12l7,-5v-12z"
                />
                <path
                  fill="#FBBC04"
                  d="M19,3l-7,5l-7,-5h14z"
                />
                <path
                  fill="#4285F4"
                  d="M5,20l7,-5l7,5h-14z"
                />
              </svg>
              Connect Gmail
            </>
          )}
        </button>

        <button
          className="email-provider-btn outlook-btn"
          onClick={handleConnectOutlook}
          disabled={connecting}
          data-testid="connect-outlook-btn"
        >
          {connecting && provider === 'outlook' ? (
            <>
              <span className="spinner" />
              Connecting to Outlook...
            </>
          ) : (
            <>
              <svg className="provider-icon" viewBox="0 0 24 24" width="24" height="24">
                <path
                  fill="#0078D4"
                  d="M7,4v16h10v-16h-10zm5,14c-2.209,0 -4,-1.791 -4,-4s1.791,-4 4,-4 4,1.791 4,4 -1.791,4 -4,4z"
                />
                <circle fill="#FFF" cx="12" cy="14" r="3" />
              </svg>
              Connect Outlook
            </>
          )}
        </button>
      </div>

      <div className="email-connect-info">
        <h3>Why do we need this?</h3>
        <ul>
          <li>📧 Get personalized email summaries in your morning briefing</li>
          <li>🔔 Receive notifications for important emails</li>
          <li>🤖 Let mBot prioritize emails for you</li>
          <li>🔒 Your credentials are never stored - we use secure OAuth2</li>
        </ul>
      </div>

      <style>{`
        .email-connect {
          max-width: 600px;
          margin: 0 auto;
          padding: 24px;
        }

        .email-connect-header {
          text-align: center;
          margin-bottom: 32px;
        }

        .email-connect-header h2 {
          font-size: 28px;
          margin-bottom: 8px;
        }

        .email-connect-header p {
          color: #666;
          font-size: 16px;
        }

        .email-connect-providers {
          display: flex;
          flex-direction: column;
          gap: 16px;
          margin-bottom: 32px;
        }

        .email-provider-btn {
          display: flex;
          align-items: center;
          justify-content: center;
          gap: 12px;
          padding: 16px 24px;
          border: 2px solid #ddd;
          border-radius: 8px;
          background: white;
          font-size: 16px;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s;
        }

        .email-provider-btn:hover:not(:disabled) {
          border-color: #4285F4;
          box-shadow: 0 2px 8px rgba(66, 133, 244, 0.2);
        }

        .email-provider-btn:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .provider-icon {
          flex-shrink: 0;
        }

        .spinner {
          display: inline-block;
          width: 20px;
          height: 20px;
          border: 3px solid #f3f3f3;
          border-top: 3px solid #4285F4;
          border-radius: 50%;
          animation: spin 1s linear infinite;
        }

        @keyframes spin {
          0% { transform: rotate(0deg); }
          100% { transform: rotate(360deg); }
        }

        .email-connect-info {
          background: #f8f9fa;
          border-radius: 8px;
          padding: 20px;
        }

        .email-connect-info h3 {
          font-size: 18px;
          margin-bottom: 12px;
        }

        .email-connect-info ul {
          list-style: none;
          padding: 0;
          margin: 0;
        }

        .email-connect-info li {
          padding: 8px 0;
          font-size: 14px;
          line-height: 1.5;
        }
      `}</style>
    </div>
  );
};
