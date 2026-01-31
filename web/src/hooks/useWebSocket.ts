/**
 * WebSocket Hook for Neural State
 * Handles real-time connection to mBot server
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { NeuralState } from '../types/neural';

interface UseWebSocketOptions {
  url: string;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
}

interface UseWebSocketReturn {
  state: NeuralState | null;
  isConnected: boolean;
  error: Error | null;
  reconnect: () => void;
}

export function useWebSocket({
  url,
  reconnectInterval = 3000,
  maxReconnectAttempts = 10,
}: UseWebSocketOptions): UseWebSocketReturn {
  const [state, setState] = useState<NeuralState | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const wsRef = useRef<WebSocket | null>(null);
  const reconnectAttemptsRef = useRef(0);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  const connect = useCallback(() => {
    try {
      const ws = new WebSocket(url);
      wsRef.current = ws;

      ws.onopen = () => {
        console.log('WebSocket connected');
        setIsConnected(true);
        setError(null);
        reconnectAttemptsRef.current = 0;
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          setState({
            timestamp: Date.now(),
            mode: data.mode,
            tension: data.tension,
            coherence: data.coherence,
            energy: data.energy,
            curiosity: data.curiosity,
            distance: data.distance,
            gyro: data.gyro,
            sound: data.sound,
            light: data.light,
          });
        } catch (err) {
          console.error('Failed to parse WebSocket message:', err);
        }
      };

      ws.onerror = (event) => {
        console.error('WebSocket error:', event);
        setError(new Error('WebSocket connection error'));
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected');
        setIsConnected(false);
        wsRef.current = null;

        // Attempt reconnection
        if (reconnectAttemptsRef.current < maxReconnectAttempts) {
          reconnectAttemptsRef.current++;
          console.log(
            `Reconnecting... (attempt ${reconnectAttemptsRef.current}/${maxReconnectAttempts})`
          );
          reconnectTimeoutRef.current = setTimeout(connect, reconnectInterval);
        } else {
          setError(new Error('Max reconnection attempts reached'));
        }
      };
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to connect'));
    }
  }, [url, reconnectInterval, maxReconnectAttempts]);

  const reconnect = useCallback(() => {
    if (wsRef.current) {
      wsRef.current.close();
    }
    reconnectAttemptsRef.current = 0;
    connect();
  }, [connect]);

  useEffect(() => {
    connect();

    return () => {
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close();
      }
    };
  }, [connect]);

  return { state, isConnected, error, reconnect };
}
