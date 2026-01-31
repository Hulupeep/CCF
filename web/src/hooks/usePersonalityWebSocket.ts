/**
 * WebSocket hook for personality updates
 * Implements I-PERS-UI-002: Debounced updates (max 2 updates/second)
 */

import { useEffect, useRef, useState, useCallback } from 'react';
import { ConnectionStatus, PersonalityConfig } from '../types/personality';

const UPDATE_INTERVAL_MS = 500; // 2 updates per second (20Hz display, 2Hz send)

export interface UsePersonalityWebSocketOptions {
  url?: string;
  autoConnect?: boolean;
  updateInterval?: number;
}

export function usePersonalityWebSocket(options: UsePersonalityWebSocketOptions = {}) {
  const {
    url = 'ws://localhost:8081',
    autoConnect = true,
    updateInterval = UPDATE_INTERVAL_MS,
  } = options;

  const [connectionStatus, setConnectionStatus] = useState<ConnectionStatus>('connecting');
  const wsRef = useRef<WebSocket | null>(null);
  const updateQueueRef = useRef<Partial<PersonalityConfig> | null>(null);
  const lastSendTimeRef = useRef<number>(0);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);

  // Send updates from queue with debouncing
  const processSendQueue = useCallback(() => {
    const now = Date.now();
    const timeSinceLastSend = now - lastSendTimeRef.current;

    if (
      updateQueueRef.current &&
      wsRef.current?.readyState === WebSocket.OPEN &&
      timeSinceLastSend >= updateInterval
    ) {
      const message = {
        type: 'personality_update',
        params: updateQueueRef.current,
      };

      wsRef.current.send(JSON.stringify(message));
      lastSendTimeRef.current = now;
      updateQueueRef.current = null;
    }
  }, [updateInterval]);

  // Queue a parameter update (debounced)
  const sendUpdate = useCallback((params: Partial<PersonalityConfig>) => {
    // Merge with existing queue
    updateQueueRef.current = {
      ...updateQueueRef.current,
      ...params,
    };
  }, []);

  // Send full personality config immediately (for preset loading)
  const sendImmediate = useCallback((config: PersonalityConfig) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      const message = {
        type: 'personality_update',
        params: config,
      };
      wsRef.current.send(JSON.stringify(message));
      lastSendTimeRef.current = Date.now();
      updateQueueRef.current = null;
    }
  }, []);

  // Connect to WebSocket
  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setConnectionStatus('connecting');

    const ws = new WebSocket(url);

    ws.onopen = () => {
      setConnectionStatus('connected');
    };

    ws.onclose = () => {
      setConnectionStatus('disconnected');
      // Auto-reconnect after 2 seconds
      setTimeout(() => {
        if (autoConnect) {
          connect();
        }
      }, 2000);
    };

    ws.onerror = () => {
      ws.close();
    };

    wsRef.current = ws;
  }, [url, autoConnect]);

  // Disconnect from WebSocket
  const disconnect = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close();
      wsRef.current = null;
    }
  }, []);

  // Start processing queue on mount
  useEffect(() => {
    intervalRef.current = setInterval(processSendQueue, 50); // Check queue every 50ms

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, [processSendQueue]);

  // Auto-connect on mount
  useEffect(() => {
    if (autoConnect) {
      connect();
    }

    return () => {
      disconnect();
    };
  }, [autoConnect, connect, disconnect]);

  return {
    connectionStatus,
    sendUpdate,
    sendImmediate,
    connect,
    disconnect,
  };
}
