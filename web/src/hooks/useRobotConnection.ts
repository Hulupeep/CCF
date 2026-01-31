/**
 * React hook for managing robot connections
 * Issue #77 - STORY-ARCH-008
 *
 * Integrates with WebSocket V2 (Issue #76) for robot connections
 */

import { useState, useCallback, useRef, useEffect } from 'react';
import { RobotStatus } from '../types/discovery';

export interface RobotConnection {
  robotId: string;
  ws: WebSocket | null;
  status: RobotStatus;
}

export interface UseRobotConnectionOptions {
  onStatusChange?: (robotId: string, status: RobotStatus) => void;
}

export function useRobotConnection(options: UseRobotConnectionOptions = {}) {
  const [connections, setConnections] = useState<Map<string, RobotConnection>>(new Map());
  const connectionsRef = useRef<Map<string, RobotConnection>>(new Map());

  // Update ref when state changes
  useEffect(() => {
    connectionsRef.current = connections;
  }, [connections]);

  /**
   * Connect to a robot
   */
  const connect = useCallback(
    async (robotId: string, ipAddress: string, port: number): Promise<void> => {
      // Check if already connected
      const existing = connectionsRef.current.get(robotId);
      if (existing?.ws?.readyState === WebSocket.OPEN) {
        console.log(`[Connection] Already connected to ${robotId}`);
        return;
      }

      // Close existing connection if any
      if (existing?.ws) {
        existing.ws.close();
      }

      const url = `ws://${ipAddress}:${port}`;
      console.log(`[Connection] Connecting to ${robotId} at ${url}`);

      return new Promise((resolve, reject) => {
        try {
          const ws = new WebSocket(url);

          // Update status to connecting
          const connection: RobotConnection = {
            robotId,
            ws,
            status: 'discovering',
          };

          setConnections(prev => new Map(prev).set(robotId, connection));
          options.onStatusChange?.(robotId, 'discovering');

          ws.onopen = () => {
            console.log(`[Connection] Connected to ${robotId}`);
            connection.status = 'connected';
            setConnections(prev => new Map(prev).set(robotId, connection));
            options.onStatusChange?.(robotId, 'connected');
            resolve();
          };

          ws.onerror = (error) => {
            console.error(`[Connection] Error connecting to ${robotId}:`, error);
            connection.status = 'error';
            setConnections(prev => new Map(prev).set(robotId, connection));
            options.onStatusChange?.(robotId, 'error');
            reject(new Error(`Failed to connect to ${robotId}`));
          };

          ws.onclose = () => {
            console.log(`[Connection] Disconnected from ${robotId}`);
            connection.status = 'disconnected';
            connection.ws = null;
            setConnections(prev => new Map(prev).set(robotId, connection));
            options.onStatusChange?.(robotId, 'disconnected');
          };

          ws.onmessage = (event) => {
            // Handle incoming messages
            try {
              const data = JSON.parse(event.data);
              console.log(`[Connection] Message from ${robotId}:`, data);
            } catch (err) {
              console.error(`[Connection] Failed to parse message from ${robotId}:`, err);
            }
          };
        } catch (error) {
          console.error(`[Connection] Failed to create WebSocket for ${robotId}:`, error);
          reject(error);
        }
      });
    },
    [options]
  );

  /**
   * Disconnect from a robot
   */
  const disconnect = useCallback((robotId: string) => {
    const connection = connectionsRef.current.get(robotId);
    if (!connection) {
      console.warn(`[Connection] No connection found for ${robotId}`);
      return;
    }

    console.log(`[Connection] Disconnecting from ${robotId}`);

    if (connection.ws) {
      connection.ws.close();
      connection.ws = null;
    }

    connection.status = 'disconnected';
    setConnections(prev => new Map(prev).set(robotId, connection));
    options.onStatusChange?.(robotId, 'disconnected');
  }, [options]);

  /**
   * Send message to a robot
   */
  const sendMessage = useCallback((robotId: string, message: any): boolean => {
    const connection = connectionsRef.current.get(robotId);
    if (!connection?.ws || connection.ws.readyState !== WebSocket.OPEN) {
      console.warn(`[Connection] Cannot send message to ${robotId}: not connected`);
      return false;
    }

    try {
      connection.ws.send(JSON.stringify(message));
      return true;
    } catch (error) {
      console.error(`[Connection] Failed to send message to ${robotId}:`, error);
      return false;
    }
  }, []);

  /**
   * Get connection status for a robot
   */
  const getStatus = useCallback((robotId: string): RobotStatus => {
    const connection = connectionsRef.current.get(robotId);
    return connection?.status || 'disconnected';
  }, []);

  /**
   * Check if connected to a robot
   */
  const isConnected = useCallback((robotId: string): boolean => {
    return getStatus(robotId) === 'connected';
  }, [getStatus]);

  /**
   * Cleanup on unmount
   */
  useEffect(() => {
    return () => {
      connectionsRef.current.forEach(connection => {
        if (connection.ws) {
          connection.ws.close();
        }
      });
    };
  }, []);

  return {
    connect,
    disconnect,
    sendMessage,
    getStatus,
    isConnected,
    connections: Array.from(connections.values()),
  };
}
