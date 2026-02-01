/**
 * Unit tests for VoiceControl component
 * Tests UI interactions and data-testid coverage
 *
 * Contracts: All VOICE contracts
 * Invariants: I-VOICE-003 (non-blocking UI feedback)
 */

import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import '@testing-library/jest-dom';
import { VoiceControl } from '../VoiceControl';
import { VoiceCommandService } from '../../services/voiceCommands';

// Mock SpeechRecognition
class MockSpeechRecognition {
  continuous = false;
  interimResults = false;
  lang = 'en-US';

  onstart: (() => void) | null = null;
  onresult: ((event: any) => void) | null = null;
  onerror: ((event: any) => void) | null = null;
  onend: (() => void) | null = null;

  start() {
    if (this.onstart) this.onstart();
  }

  stop() {
    if (this.onend) this.onend();
  }
}

describe('VoiceControl Component', () => {
  beforeEach(() => {
    VoiceCommandService.resetInstance();
    localStorage.clear();

    // Mock browser APIs directly on window object
    (window as any).SpeechRecognition = MockSpeechRecognition;
    (window as any).webkitSpeechRecognition = MockSpeechRecognition;
    (window as any).AudioContext = class {
      currentTime = 0;
      createOscillator() {
        return {
          frequency: { value: 0 },
          connect: jest.fn(),
          start: jest.fn(),
          stop: jest.fn(),
        };
      }
      createGain() {
        return {
          gain: {
            setValueAtTime: jest.fn(),
            exponentialRampToValueAtTime: jest.fn(),
          },
          connect: jest.fn(),
        };
      }
      get destination() {
        return {};
      }
    };

    // Mock console.warn
    jest.spyOn(console, 'warn').mockImplementation(() => {});
  });

  afterEach(() => {
    VoiceCommandService.resetInstance();
    jest.restoreAllMocks();
  });

  describe('Rendering and data-testid Coverage', () => {
    it('should render with voice-toggle button', () => {
      render(<VoiceControl />);
      expect(screen.getByTestId('voice-toggle')).toBeInTheDocument();
    });

    it('should render listening indicator', () => {
      render(<VoiceControl />);
      expect(screen.getByTestId('listening-indicator')).toBeInTheDocument();
    });

    it('should render microphone icon', () => {
      render(<VoiceControl />);
      expect(screen.getByTestId('microphone-icon')).toBeInTheDocument();
    });

    it('should render settings and history buttons', () => {
      render(<VoiceControl />);

      // Settings button (aria-label)
      const settingsBtn = screen.getByLabelText('Settings');
      expect(settingsBtn).toBeInTheDocument();

      // History button
      const historyBtn = screen.getByLabelText('History');
      expect(historyBtn).toBeInTheDocument();
    });
  });

  describe('Voice Toggle Functionality', () => {
    it('should toggle voice recognition on button click', async () => {
      render(<VoiceControl />);

      const toggleBtn = screen.getByTestId('voice-toggle');

      // Initially not listening
      expect(toggleBtn).toHaveClass('bg-gray-300');

      // Click to start listening
      fireEvent.click(toggleBtn);

      await waitFor(() => {
        expect(toggleBtn).toHaveClass('bg-green-500');
      });
    });

    it('should stop listening on second click', async () => {
      render(<VoiceControl />);

      const toggleBtn = screen.getByTestId('voice-toggle');

      // Start listening
      fireEvent.click(toggleBtn);
      await waitFor(() => {
        expect(toggleBtn).toHaveClass('bg-green-500');
      });

      // Stop listening
      fireEvent.click(toggleBtn);
      await waitFor(() => {
        expect(toggleBtn).toHaveClass('bg-gray-300');
      });
    });

    it('should show listening state text', async () => {
      render(<VoiceControl />);

      const toggleBtn = screen.getByTestId('voice-toggle');

      // Initially ready
      expect(screen.getByText('Ready')).toBeInTheDocument();

      // Start listening
      fireEvent.click(toggleBtn);

      await waitFor(() => {
        expect(screen.getByText('Listening...')).toBeInTheDocument();
      });
    });
  });

  describe('Settings Panel', () => {
    it('should show settings panel on button click', () => {
      render(<VoiceControl />);

      const settingsBtn = screen.getByLabelText('Settings');
      fireEvent.click(settingsBtn);

      expect(screen.getByTestId('settings-panel')).toBeInTheDocument();
    });

    it('should render wake word toggle', () => {
      render(<VoiceControl />);

      const settingsBtn = screen.getByLabelText('Settings');
      fireEvent.click(settingsBtn);

      expect(screen.getByTestId('wake-word-toggle')).toBeInTheDocument();
    });

    it('should show wake word input when enabled', () => {
      render(<VoiceControl />);

      const settingsBtn = screen.getByLabelText('Settings');
      fireEvent.click(settingsBtn);

      const wakeWordToggle = screen.getByTestId('wake-word-toggle');
      fireEvent.click(wakeWordToggle);

      expect(screen.getByTestId('wake-word-input')).toBeInTheDocument();
    });

    it('should render audio feedback toggle', () => {
      render(<VoiceControl />);

      const settingsBtn = screen.getByLabelText('Settings');
      fireEvent.click(settingsBtn);

      expect(screen.getByTestId('audio-feedback-toggle')).toBeInTheDocument();
    });

    it('should render language selector', () => {
      render(<VoiceControl />);

      const settingsBtn = screen.getByLabelText('Settings');
      fireEvent.click(settingsBtn);

      expect(screen.getByTestId('language-selector')).toBeInTheDocument();
    });

    it('should update wake word setting', () => {
      render(<VoiceControl />);

      const settingsBtn = screen.getByLabelText('Settings');
      fireEvent.click(settingsBtn);

      const wakeWordToggle = screen.getByTestId('wake-word-toggle');
      fireEvent.click(wakeWordToggle);

      const wakeWordInput = screen.getByTestId('wake-word-input');
      fireEvent.change(wakeWordInput, { target: { value: 'hello robot' } });

      expect(wakeWordInput).toHaveValue('hello robot');
    });
  });

  describe('Command History', () => {
    it('should show history panel on button click', () => {
      render(<VoiceControl />);

      const historyBtn = screen.getByLabelText('History');
      fireEvent.click(historyBtn);

      expect(screen.getByTestId('command-history')).toBeInTheDocument();
    });

    it('should show empty state when no history', () => {
      render(<VoiceControl />);

      const historyBtn = screen.getByLabelText('History');
      fireEvent.click(historyBtn);

      expect(screen.getByText('No commands yet')).toBeInTheDocument();
    });

    it('should render clear history button', () => {
      render(<VoiceControl />);

      const historyBtn = screen.getByLabelText('History');
      fireEvent.click(historyBtn);

      expect(screen.getByTestId('clear-history-btn')).toBeInTheDocument();
    });

    it('should clear history on button click with confirmation', async () => {
      const service = VoiceCommandService.getInstance();

      // Add some history
      await service.processCommand('stop', 1.0);

      render(<VoiceControl />);

      const historyBtn = screen.getByLabelText('History');
      fireEvent.click(historyBtn);

      // Mock window.confirm
      window.confirm = jest.fn(() => true);

      const clearBtn = screen.getByTestId('clear-history-btn');
      fireEvent.click(clearBtn);

      await waitFor(() => {
        expect(screen.getByText('No commands yet')).toBeInTheDocument();
      });
    });
  });

  describe('I-VOICE-003: Non-blocking UI Feedback', () => {
    it('should show recognized text without blocking', async () => {
      const service = VoiceCommandService.getInstance();

      render(<VoiceControl />);

      // Process command
      await service.processCommand('stop', 1.0);

      await waitFor(() => {
        expect(screen.getByTestId('recognized-text')).toBeInTheDocument();
      });
    });

    it('should show confidence meter', async () => {
      const service = VoiceCommandService.getInstance();

      render(<VoiceControl />);

      // Process command with specific confidence
      await service.processCommand('stop', 0.85);

      await waitFor(() => {
        expect(screen.getByTestId('voice-confidence')).toBeInTheDocument();
      });
    });

    it('should show processing feedback', async () => {
      render(<VoiceControl />);

      // Simulate processing state
      const service = VoiceCommandService.getInstance();

      // Register slow command
      service.registerCommand({
        pattern: /slow/i,
        action: 'SLOW',
        description: 'Slow command',
        requiresConfirmation: false,
        handler: async () => {
          await new Promise((resolve) => setTimeout(resolve, 100));
        },
      });

      const promise = service.processCommand('slow', 1.0);

      // Should show processing feedback
      await waitFor(() => {
        expect(screen.queryByTestId('voice-feedback')).toBeInTheDocument();
      });

      await promise;
    });
  });

  describe('Confirmation Prompt', () => {
    it('should show confirmation prompt for destructive commands', async () => {
      const service = VoiceCommandService.getInstance();

      render(<VoiceControl />);

      // Trigger destructive command
      await service.processCommand('delete all drawings', 1.0);

      await waitFor(() => {
        expect(screen.getByTestId('confirmation-prompt')).toBeInTheDocument();
      });
    });

    it('should show confirmation message', async () => {
      const service = VoiceCommandService.getInstance();

      render(<VoiceControl />);

      await service.processCommand('delete all drawings', 1.0);

      await waitFor(() => {
        expect(screen.getByText(/Are you sure/i)).toBeInTheDocument();
      });
    });
  });

  describe('Confidence Visualization', () => {
    it('should show green bar for high confidence (>80%)', async () => {
      const service = VoiceCommandService.getInstance();

      render(<VoiceControl />);

      await service.processCommand('stop', 0.9);

      await waitFor(() => {
        const confidenceMeter = screen.getByTestId('voice-confidence');
        const bar = confidenceMeter.querySelector('.bg-green-500');
        expect(bar).toBeInTheDocument();
      });
    });

    it('should show yellow bar for medium confidence (50-80%)', async () => {
      const service = VoiceCommandService.getInstance();

      render(<VoiceControl />);

      await service.processCommand('stop', 0.7);

      await waitFor(() => {
        const confidenceMeter = screen.getByTestId('voice-confidence');
        const bar = confidenceMeter.querySelector('.bg-yellow-500');
        expect(bar).toBeInTheDocument();
      });
    });

    it('should show red bar for low confidence (<50%)', async () => {
      const service = VoiceCommandService.getInstance();

      render(<VoiceControl />);

      // Force processing even with low confidence
      await service.processCommand('stop', 0.3);

      await waitFor(() => {
        const confidenceMeter = screen.getByTestId('voice-confidence');
        const bar = confidenceMeter.querySelector('.bg-red-500');
        expect(bar).toBeInTheDocument();
      });
    });
  });

  describe('Browser Compatibility Message', () => {
    it('should show unsupported message when browser lacks API', () => {
      // Remove SpeechRecognition support
      delete (window as any).SpeechRecognition;
      delete (window as any).webkitSpeechRecognition;

      render(<VoiceControl />);

      expect(
        screen.getByText(/Voice commands are not supported in this browser/i)
      ).toBeInTheDocument();
    });
  });

  describe('Accessibility', () => {
    it('should have aria-label for voice toggle', () => {
      render(<VoiceControl />);

      const toggleBtn = screen.getByTestId('voice-toggle');
      expect(toggleBtn).toHaveAttribute('aria-label');
    });

    it('should have aria-label for settings button', () => {
      render(<VoiceControl />);

      const settingsBtn = screen.getByLabelText('Settings');
      expect(settingsBtn).toHaveAttribute('aria-label', 'Settings');
    });

    it('should have aria-label for history button', () => {
      render(<VoiceControl />);

      const historyBtn = screen.getByLabelText('History');
      expect(historyBtn).toHaveAttribute('aria-label', 'History');
    });
  });

  describe('History Item Display', () => {
    it('should show history items with execution time', async () => {
      const service = VoiceCommandService.getInstance();

      // Add command to history
      await service.processCommand('stop', 1.0);

      render(<VoiceControl />);

      const historyBtn = screen.getByLabelText('History');
      fireEvent.click(historyBtn);

      const history = service.getCommandHistory();
      const historyItem = screen.getByTestId(`history-item-${history[0].id}`);

      expect(historyItem).toBeInTheDocument();
      expect(historyItem).toHaveTextContent('ms');
    });

    it('should show recognized status in history', async () => {
      const service = VoiceCommandService.getInstance();

      await service.processCommand('stop', 1.0);

      render(<VoiceControl />);

      const historyBtn = screen.getByLabelText('History');
      fireEvent.click(historyBtn);

      expect(screen.getByText(/✓/)).toBeInTheDocument();
    });

    it('should show not recognized status in history', async () => {
      const service = VoiceCommandService.getInstance();

      await service.processCommand('invalid xyz', 1.0);

      render(<VoiceControl />);

      const historyBtn = screen.getByLabelText('History');
      fireEvent.click(historyBtn);

      expect(screen.getByText(/✗ Not recognized/)).toBeInTheDocument();
    });
  });
});
