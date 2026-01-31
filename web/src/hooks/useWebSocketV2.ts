/**
 * WebSocket V2 Hook - State Sync & Auto-Reconnect
 * Contract: ARCH-005 (Transport Layer Abstraction)
 *
 * Features:
 * - Full state snapshot on connect (I-WS-V2-001)
 * - Message batching (100ms window)
 * - Auto-reconnect with exponential backoff
 * - Message ordering guarantees (I-WS-V2-002)
 * - Event subscription system
 */

import { useEffect, useRef, useState, useCallback } from 'react';
import {
  WebSocketMessage,
  StateSnapshot,
  ConnectionState,
  ConnectionStats,
  UseWebSocketV2Options,
  UseWebSocketV2Return,
  validateVersion,
  createCommand,
  createPing,
  isStateMessage,
  isEventMessage,
  isPongMessage,
  PROTOCOL_VERSION,
  BATCH_WINDOW_MS,
  RECONNECT_TIMEOUT_MS,
} from '../types/websocketV2';

/**
 * WebSocket V2 Hook with State Sync
 */
export function useWebSocketV2(options: UseWebSocketV2Options = {}): UseWebSocketV2Return {
  const {
    url = 'ws://localhost:8080',
    autoConnect = true,
    batchWindow = BATCH_WINDOW_MS,
    maxReconnectAttempts = Infinity,
    reconnectDelay = 1000,
    debug = false,
  } = options;

  // Connection state
  const [connectionState, setConnectionState] = useState<ConnectionState>('disconnected');
  const [state, setState] = useState<StateSnapshot | null>(null);
  const [stats, setStats] = useState<ConnectionStats>({
    messagesSent: 0,
    messagesReceived: 0,
    latency: 0,
    reconnectAttempts: 0,
    lastConnected: null,
    lastDisconnected: null,
  });

  // Refs
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const messageQueueRef = useRef<WebSocketMessage[]>([]);
  const batchTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const eventHandlersRef = useRef<Map<string, Set<(data: any) => void>>>(new Map());
  const pingIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const lastPingRef = useRef<number>(0);

  /**
   * Log debug messages
   */
  const log = useCallback((...args: any[]) => {
    if (debug) {
      console.log('[WebSocketV2]', ...args);
    }
  }, [debug]);

  /**
   * Update connection stats
   */
  const updateStats = useCallback((update: Partial<ConnectionStats>) => {
    setStats(prev => ({ ...prev, ...update }));
  }, []);

  /**
   * Flush message queue (send batched messages)
   */
  const flushMessageQueue = useCallback(() => {
    if (messageQueueRef.current.length === 0) {
      return;
    }

    if (wsRef.current?.readyState === WebSocket.OPEN) {
      const messages = messageQueueRef.current;
      messageQueueRef.current = [];

      // If single message, send directly
      if (messages.length === 1) {
        const json = JSON.stringify(messages[0]);
        wsRef.current.send(json);
        updateStats({ messagesSent: stats.messagesSent + 1 });
        log('Sent message:', messages[0].type);
      } else {
        // Batch multiple messages
        const batch: WebSocketMessage = {
          version: PROTOCOL_VERSION,
          type: 'batch',
          payload: { messages },
          timestamp: Date.now(),
        };
        const json = JSON.stringify(batch);
        wsRef.current.send(json);
        updateStats({ messagesSent: stats.messagesSent + messages.length });
        log('Sent batch:', messages.length, 'messages');
      }
    }

    if (batchTimeoutRef.current) {
      clearTimeout(batchTimeoutRef.current);
      batchTimeoutRef.current = null;
    }
  }, [stats.messagesSent, updateStats, log]);

  /**
   * Queue a message for sending (with batching)
   */
  const queueMessage = useCallback((message: WebSocketMessage) => {
    messageQueueRef.current.push(message);

    // Set timeout to flush queue after batch window
    if (!batchTimeoutRef.current) {
      batchTimeoutRef.current = setTimeout(flushMessageQueue, batchWindow);
    }
  }, [batchWindow, flushMessageQueue]);

  /**
   * Send a command to the robot
   */
  const sendCommand = useCallback((command: string, params: Record<string, any>) => {
    if (connectionState !== 'connected') {
      log('Cannot send command: not connected');
      return;
    }

    const message = createCommand(command, params);
    queueMessage(message);
    log('Queued command:', command);
  }, [connectionState, queueMessage, log]);

  /**
   * Handle incoming message
   */
  const handleMessage = useCallback((event: MessageEvent) => {
    try {
      const message: WebSocketMessage = JSON.parse(event.data);

      // Validate protocol version
      if (!validateVersion(message)) {
        console.error('Invalid protocol version:', message.version);
        return;
      }

      updateStats({ messagesReceived: stats.messagesReceived + 1 });
      log('Received message:', message.type);

      // Handle state snapshot (I-WS-V2-001)
      if (isStateMessage(message)) {
        setState(message.payload);
        log('State synced:', message.payload);
        return;
      }

      // Handle events
      if (isEventMessage(message)) {
        const { event, data } = message.payload;
        const handlers = eventHandlersRef.current.get(event);
        if (handlers) {
          handlers.forEach(handler => handler(data));
        }
        log('Event received:', event);
        return;
      }

      // Handle pong (latency measurement)
      if (isPongMessage(message)) {
        const latency = Date.now() - lastPingRef.current;
        updateStats({ latency });
        log('Pong received, latency:', latency, 'ms');
        return;
      }

    } catch (error) {
      console.error('Failed to parse message:', error);
    }
  }, [stats.messagesReceived, updateStats, log]);

  /**
   * Start ping interval (keep-alive)
   */
  const startPingInterval = useCallback(() => {
    if (pingIntervalRef.current) {
      clearInterval(pingIntervalRef.current);
    }

    pingIntervalRef.current = setInterval(() => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        lastPingRef.current = Date.now();
        const ping = createPing();
        wsRef.current.send(JSON.stringify(ping));
        log('Ping sent');
      }
    }, 10000); // Ping every 10 seconds
  }, [log]);

  /**
   * Stop ping interval
   */
  const stopPingInterval = useCallback(() => {
    if (pingIntervalRef.current) {
      clearInterval(pingIntervalRef.current);
      pingIntervalRef.current = null;
    }
  }, []);

  /**
   * Connect to WebSocket
   */
  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      log('Already connected');
      return;
    }

    log('Connecting to', url);
    setConnectionState('connecting');

    try {
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        log('Connected');
        setConnectionState('connected');
        reconnectAttemptsRef.current = 0;
        updateStats({
          reconnectAttempts: 0,
          lastConnected: Date.now(),
        });
        startPingInterval();

        // Server will send state snapshot automatically
      };

      ws.onmessage = handleMessage;

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        setConnectionState('error');
      };

      ws.onclose = () => {
        log('Disconnected');
        setConnectionState('disconnected');
        stopPingInterval();
        updateStats({
          lastDisconnected: Date.now(),
        });

        // Auto-reconnect with exponential backoff
        if (autoConnect && reconnectAttemptsRef.current < maxReconnectAttempts) {
          reconnectAttemptsRef.current++;
          const delay = Math.min(
            reconnectDelay * Math.pow(2, reconnectAttemptsRef.current - 1),
            RECONNECT_TIMEOUT_MS
          );

          log('Reconnecting in', delay, 'ms (attempt', reconnectAttemptsRef.current, ')');
          setConnectionState('reconnecting');
          updateStats({
            reconnectAttempts: reconnectAttemptsRef.current,
          });

          reconnectTimeoutRef.current = setTimeout(() => {
            connect();
          }, delay);
        } else if (reconnectAttemptsRef.current >= maxReconnectAttempts) {
          log('Max reconnect attempts reached');
          setConnectionState('error');
        }
      };

    } catch (error) {
      console.error('Failed to create WebSocket:', error);
      setConnectionState('error');
    }
  }, [
    url,
    autoConnect,
    maxReconnectAttempts,
    reconnectDelay,
    handleMessage,
    startPingInterval,
    stopPingInterval,
    updateStats,
    log,
  ]);

  /**
   * Disconnect from WebSocket
   */
  const disconnect = useCallback(() => {
    log('Disconnecting');

    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (batchTimeoutRef.current) {
      clearTimeout(batchTimeoutRef.current);
      batchTimeoutRef.current = null;
    }

    stopPingInterval();

    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }

    setConnectionState('disconnected');
  }, [stopPingInterval, log]);

  /**
   * Manually trigger reconnection
   */
  const reconnect = useCallback(() => {
    log('Manual reconnect');
    disconnect();
    reconnectAttemptsRef.current = 0;
    setTimeout(connect, 100);
  }, [connect, disconnect, log]);

  /**
   * Subscribe to events
   */
  const on = useCallback(<T = any>(event: string, handler: (data: T) => void) => {
    log('Subscribing to event:', event);

    if (!eventHandlersRef.current.has(event)) {
      eventHandlersRef.current.set(event, new Set());
    }

    eventHandlersRef.current.get(event)!.add(handler);

    // Return unsubscribe function
    return () => {
      log('Unsubscribing from event:', event);
      const handlers = eventHandlersRef.current.get(event);
      if (handlers) {
        handlers.delete(handler);
        if (handlers.size === 0) {
          eventHandlersRef.current.delete(event);
        }
      }
    };
  }, [log]);

  /**
   * Auto-connect on mount
   */
  useEffect(() => {
    if (autoConnect) {
      connect();
    }

    return () => {
      disconnect();
    };
  }, [autoConnect]); // Intentionally minimal deps to avoid reconnecting on every change

  /**
   * Flush queue on unmount
   */
  useEffect(() => {
    return () => {
      flushMessageQueue();
    };
  }, [flushMessageQueue]);

  return {
    connectionState,
    state,
    stats,
    sendCommand,
    reconnect,
    disconnect,
    on,
  };
}
