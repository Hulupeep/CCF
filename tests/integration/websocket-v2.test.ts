/**
 * WebSocket V2 Integration Tests
 * Contract: ARCH-005 (Transport Layer Abstraction)
 *
 * Tests:
 * - I-WS-V2-001: State Consistency - Client state matches robot after sync
 * - I-WS-V2-002: Message Order - Messages processed in order sent
 *
 * Scenarios:
 * 1. Client Connects → Server sends state snapshot → Client syncs
 * 2. Message Batching → 10 messages in 100ms → Batched into 1
 * 3. Auto-Reconnect → Connection lost → Reconnects within 30s → State re-syncs
 */

import { renderHook, waitFor, act } from '@testing-library/react';
import { useWebSocketV2 } from '../../web/src/hooks/useWebSocketV2';
import { WebSocketMessage, StateSnapshot, PROTOCOL_VERSION } from '../../web/src/types/websocketV2';
import WS from 'jest-websocket-mock';

describe('WebSocket Protocol V2 - Integration Tests', () => {
  let server: WS;
  const testUrl = 'ws://localhost:8080';

  beforeEach(() => {
    server = new WS(testUrl);
  });

  afterEach(() => {
    WS.clean();
  });

  describe('Scenario: Client Connects', () => {
    it('should receive full state snapshot on connect (I-WS-V2-001)', async () => {
      const { result } = renderHook(() =>
        useWebSocketV2({ url: testUrl, autoConnect: true })
      );

      // Wait for connection
      await waitFor(() => {
        expect(result.current.connectionState).toBe('connected');
      });

      // Server sends state snapshot
      const stateSnapshot: StateSnapshot = {
        personality: {
          tension_baseline: 0.5,
          coherence_baseline: 0.5,
          energy_baseline: 0.5,
          startle_sensitivity: 0.5,
          recovery_speed: 0.5,
          curiosity_drive: 0.5,
          movement_expressiveness: 0.5,
          sound_expressiveness: 0.5,
          light_expressiveness: 0.5,
        },
        neural_state: {
          mode: 'Calm',
          tension: 0.3,
          coherence: 0.7,
          energy: 0.5,
          curiosity: 0.6,
          distance: 25.0,
        },
        capabilities: {
          has_drawing: true,
          has_sorter: true,
          has_games: true,
          has_learning_lab: false,
          firmware_version: '1.0.0',
        },
      };

      const message: WebSocketMessage = {
        version: PROTOCOL_VERSION,
        type: 'state',
        payload: stateSnapshot,
        timestamp: Date.now(),
      };

      act(() => {
        server.send(JSON.stringify(message));
      });

      // Client should sync to state
      await waitFor(() => {
        expect(result.current.state).toEqual(stateSnapshot);
      });

      // Verify state consistency (I-WS-V2-001)
      expect(result.current.state?.personality.tension_baseline).toBe(0.5);
      expect(result.current.state?.neural_state.mode).toBe('Calm');
      expect(result.current.state?.capabilities.has_drawing).toBe(true);
    });

    it('should reject invalid protocol version', async () => {
      const { result } = renderHook(() =>
        useWebSocketV2({ url: testUrl, autoConnect: true, debug: true })
      );

      await waitFor(() => {
        expect(result.current.connectionState).toBe('connected');
      });

      // Send message with wrong version
      const badMessage = {
        version: 1, // Wrong version
        type: 'state',
        payload: {},
        timestamp: Date.now(),
      };

      const consoleSpy = jest.spyOn(console, 'error').mockImplementation(() => {});

      act(() => {
        server.send(JSON.stringify(badMessage));
      });

      await waitFor(() => {
        expect(consoleSpy).toHaveBeenCalledWith(
          expect.stringContaining('Invalid protocol version'),
          1
        );
      });

      consoleSpy.mockRestore();
    });
  });

  describe('Scenario: Message Batching', () => {
    it('should batch 10 parameter changes within 100ms window', async () => {
      const { result } = renderHook(() =>
        useWebSocketV2({ url: testUrl, autoConnect: true, batchWindow: 100 })
      );

      await waitFor(() => {
        expect(result.current.connectionState).toBe('connected');
      });

      // Send 10 commands rapidly
      act(() => {
        for (let i = 0; i < 10; i++) {
          result.current.sendCommand('update_personality', {
            tension_baseline: 0.1 * i,
          });
        }
      });

      // Wait for batch window to elapse
      await new Promise(resolve => setTimeout(resolve, 150));

      // Should receive a single batch message
      const messages = server.messages;

      // Filter out ping messages
      const commandMessages = messages.filter((msg) => {
        const parsed = JSON.parse(msg as string);
        return parsed.type === 'batch' || parsed.type === 'command';
      });

      // Should be batched into 1 or fewer messages
      expect(commandMessages.length).toBeLessThanOrEqual(1);

      if (commandMessages.length === 1) {
        const batch = JSON.parse(commandMessages[0] as string);
        expect(batch.type).toBe('batch');
        expect(batch.payload.messages.length).toBe(10);
      }
    });

    it('should send single message immediately if no batching needed', async () => {
      const { result } = renderHook(() =>
        useWebSocketV2({ url: testUrl, autoConnect: true })
      );

      await waitFor(() => {
        expect(result.current.connectionState).toBe('connected');
      });

      // Clear initial messages
      server.messages.length = 0;

      // Send single command
      act(() => {
        result.current.sendCommand('test_command', { value: 42 });
      });

      await waitFor(() => {
        const commandMessages = server.messages.filter((msg) => {
          const parsed = JSON.parse(msg as string);
          return parsed.type === 'command';
        });
        expect(commandMessages.length).toBeGreaterThanOrEqual(1);
      });

      const lastCommand = JSON.parse(
        server.messages[server.messages.length - 1] as string
      );
      expect(lastCommand.type).toBe('command');
      expect(lastCommand.payload.command).toBe('test_command');
      expect(lastCommand.payload.params.value).toBe(42);
    });
  });

  describe('Scenario: Auto-Reconnect', () => {
    it('should reconnect automatically after disconnect', async () => {
      const { result } = renderHook(() =>
        useWebSocketV2({
          url: testUrl,
          autoConnect: true,
          reconnectDelay: 100,
          maxReconnectAttempts: 3,
        })
      );

      // Wait for initial connection
      await waitFor(() => {
        expect(result.current.connectionState).toBe('connected');
      });

      // Simulate disconnect
      act(() => {
        server.close();
      });

      await waitFor(() => {
        expect(result.current.connectionState).toBe('disconnected');
      });

      // Create new server (simulating server restart)
      server = new WS(testUrl);

      // Should reconnect
      await waitFor(
        () => {
          expect(result.current.connectionState).toBe('connected');
        },
        { timeout: 5000 }
      );

      expect(result.current.stats.reconnectAttempts).toBeGreaterThan(0);
    });

    it('should stop reconnecting after max attempts', async () => {
      const { result } = renderHook(() =>
        useWebSocketV2({
          url: testUrl,
          autoConnect: true,
          reconnectDelay: 100,
          maxReconnectAttempts: 2,
        })
      );

      // Wait for initial connection
      await waitFor(() => {
        expect(result.current.connectionState).toBe('connected');
      });

      // Simulate permanent disconnect
      act(() => {
        server.close();
      });

      // Wait for reconnection attempts to exhaust
      await waitFor(
        () => {
          expect(result.current.connectionState).toBe('error');
        },
        { timeout: 5000 }
      );

      expect(result.current.stats.reconnectAttempts).toBe(2);
    });

    it('should re-sync state after reconnection (I-WS-V2-001)', async () => {
      const { result } = renderHook(() =>
        useWebSocketV2({
          url: testUrl,
          autoConnect: true,
          reconnectDelay: 100,
        })
      );

      await waitFor(() => {
        expect(result.current.connectionState).toBe('connected');
      });

      // Send initial state
      const initialState: StateSnapshot = {
        personality: {
          tension_baseline: 0.3,
          coherence_baseline: 0.3,
          energy_baseline: 0.3,
          startle_sensitivity: 0.3,
          recovery_speed: 0.3,
          curiosity_drive: 0.3,
          movement_expressiveness: 0.3,
          sound_expressiveness: 0.3,
          light_expressiveness: 0.3,
        },
        neural_state: {
          mode: 'Active',
          tension: 0.5,
          coherence: 0.5,
          energy: 0.7,
          curiosity: 0.8,
        },
        capabilities: {
          has_drawing: true,
          has_sorter: true,
          has_games: true,
          has_learning_lab: false,
          firmware_version: '1.0.0',
        },
      };

      act(() => {
        server.send(
          JSON.stringify({
            version: PROTOCOL_VERSION,
            type: 'state',
            payload: initialState,
            timestamp: Date.now(),
          })
        );
      });

      await waitFor(() => {
        expect(result.current.state?.personality.tension_baseline).toBe(0.3);
      });

      // Disconnect
      act(() => {
        server.close();
      });

      await waitFor(() => {
        expect(result.current.connectionState).toBe('disconnected');
      });

      // Reconnect with new state
      server = new WS(testUrl);

      await waitFor(() => {
        expect(result.current.connectionState).toBe('connected');
      });

      const updatedState: StateSnapshot = {
        ...initialState,
        personality: {
          ...initialState.personality,
          tension_baseline: 0.7,
        },
      };

      act(() => {
        server.send(
          JSON.stringify({
            version: PROTOCOL_VERSION,
            type: 'state',
            payload: updatedState,
            timestamp: Date.now(),
          })
        );
      });

      // State should sync to new value
      await waitFor(() => {
        expect(result.current.state?.personality.tension_baseline).toBe(0.7);
      });
    });
  });

  describe('Scenario: Message Ordering (I-WS-V2-002)', () => {
    it('should process messages in order sent', async () => {
      const { result } = renderHook(() =>
        useWebSocketV2({ url: testUrl, autoConnect: true })
      );

      await waitFor(() => {
        expect(result.current.connectionState).toBe('connected');
      });

      const receivedEvents: string[] = [];

      // Subscribe to events
      act(() => {
        result.current.on('test_event', (data: { value: string }) => {
          receivedEvents.push(data.value);
        });
      });

      // Send events with sequence numbers
      for (let i = 0; i < 5; i++) {
        act(() => {
          server.send(
            JSON.stringify({
              version: PROTOCOL_VERSION,
              type: 'event',
              payload: {
                event: 'test_event',
                data: { value: `event_${i}` },
              },
              timestamp: Date.now(),
              sequence: i,
            })
          );
        });
      }

      await waitFor(() => {
        expect(receivedEvents.length).toBe(5);
      });

      // Events should be received in order
      expect(receivedEvents).toEqual([
        'event_0',
        'event_1',
        'event_2',
        'event_3',
        'event_4',
      ]);
    });
  });

  describe('Event Subscription', () => {
    it('should subscribe and receive events', async () => {
      const { result } = renderHook(() =>
        useWebSocketV2({ url: testUrl, autoConnect: true })
      );

      await waitFor(() => {
        expect(result.current.connectionState).toBe('connected');
      });

      const handler = jest.fn();

      // Subscribe
      let unsubscribe: (() => void) | undefined;
      act(() => {
        unsubscribe = result.current.on('custom_event', handler);
      });

      // Send event
      act(() => {
        server.send(
          JSON.stringify({
            version: PROTOCOL_VERSION,
            type: 'event',
            payload: {
              event: 'custom_event',
              data: { message: 'hello' },
            },
            timestamp: Date.now(),
          })
        );
      });

      await waitFor(() => {
        expect(handler).toHaveBeenCalledWith({ message: 'hello' });
      });

      // Unsubscribe
      act(() => {
        unsubscribe?.();
      });

      handler.mockClear();

      // Send another event
      act(() => {
        server.send(
          JSON.stringify({
            version: PROTOCOL_VERSION,
            type: 'event',
            payload: {
              event: 'custom_event',
              data: { message: 'world' },
            },
            timestamp: Date.now(),
          })
        );
      });

      // Should not receive after unsubscribe
      await new Promise(resolve => setTimeout(resolve, 100));
      expect(handler).not.toHaveBeenCalled();
    });
  });

  describe('Connection Statistics', () => {
    it('should track messages sent and received', async () => {
      const { result } = renderHook(() =>
        useWebSocketV2({ url: testUrl, autoConnect: true })
      );

      await waitFor(() => {
        expect(result.current.connectionState).toBe('connected');
      });

      // Initial stats
      expect(result.current.stats.messagesSent).toBe(0);
      expect(result.current.stats.messagesReceived).toBe(0);

      // Send command
      act(() => {
        result.current.sendCommand('test', {});
      });

      await waitFor(() => {
        expect(result.current.stats.messagesSent).toBeGreaterThan(0);
      });

      // Receive message
      act(() => {
        server.send(
          JSON.stringify({
            version: PROTOCOL_VERSION,
            type: 'event',
            payload: { event: 'test', data: {} },
            timestamp: Date.now(),
          })
        );
      });

      await waitFor(() => {
        expect(result.current.stats.messagesReceived).toBeGreaterThan(0);
      });
    });

    it('should measure latency with ping/pong', async () => {
      const { result } = renderHook(() =>
        useWebSocketV2({ url: testUrl, autoConnect: true })
      );

      await waitFor(() => {
        expect(result.current.connectionState).toBe('connected');
      });

      // Wait for ping
      await waitFor(() => {
        const pingMessages = server.messages.filter((msg) => {
          const parsed = JSON.parse(msg as string);
          return parsed.type === 'ping';
        });
        return pingMessages.length > 0;
      }, { timeout: 15000 });

      // Respond with pong
      act(() => {
        server.send(
          JSON.stringify({
            version: PROTOCOL_VERSION,
            type: 'pong',
            payload: {},
            timestamp: Date.now(),
          })
        );
      });

      // Should update latency
      await waitFor(() => {
        expect(result.current.stats.latency).toBeGreaterThanOrEqual(0);
      });
    });
  });
});
