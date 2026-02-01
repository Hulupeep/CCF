/**
 * WebSocket Client Service
 * Issue: #88 (STORY-MOBILE-001)
 * Invariant: I-MOBILE-001 - Auto-reconnect within 5 seconds
 */

import { WebSocketMessage, AppSettings } from '../types';

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string = '';
  private reconnectTimer: NodeJS.Timeout | null = null;
  private reconnectAttempts: number = 0;
  private maxReconnectAttempts: number = 5;
  private listeners: Map<string, Set<(data: any) => void>> = new Map();
  private settings: AppSettings;
  private isIntentionalClose: boolean = false;

  constructor(settings: AppSettings) {
    this.settings = settings;
  }

  /**
   * Connect to robot WebSocket server
   * I-MOBILE-001: Connection must establish within 3 seconds
   */
  async connect(ipAddress: string, port: number): Promise<void> {
    this.url = `ws://${ipAddress}:${port}/ws`;
    this.isIntentionalClose = false;

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Connection timeout (3s)'));
        this.ws?.close();
      }, 3000);

      try {
        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
          clearTimeout(timeout);
          this.reconnectAttempts = 0;
          console.log('[WebSocket] Connected to', this.url);
          this.emit('connection', { status: 'connected' });
          resolve();
        };

        this.ws.onmessage = (event) => {
          try {
            const message: WebSocketMessage = JSON.parse(event.data);
            this.handleMessage(message);
          } catch (error) {
            console.error('[WebSocket] Message parse error:', error);
          }
        };

        this.ws.onerror = (error) => {
          clearTimeout(timeout);
          console.error('[WebSocket] Error:', error);
          this.emit('error', { message: 'WebSocket error' });
          reject(error);
        };

        this.ws.onclose = () => {
          clearTimeout(timeout);
          console.log('[WebSocket] Connection closed');
          this.emit('connection', { status: 'disconnected' });

          // I-MOBILE-001: Auto-reconnect within 5 seconds
          if (this.settings.autoReconnect && !this.isIntentionalClose) {
            this.scheduleReconnect();
          }
        };
      } catch (error) {
        clearTimeout(timeout);
        reject(error);
      }
    });
  }

  /**
   * I-MOBILE-001: Reconnect automatically within 5 seconds
   */
  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('[WebSocket] Max reconnect attempts reached');
      this.emit('connection', { status: 'failed' });
      return;
    }

    const delay = Math.min(
      this.settings.reconnectDelay * Math.pow(2, this.reconnectAttempts),
      5000 // I-MOBILE-001: Maximum 5 seconds
    );

    console.log(`[WebSocket] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts + 1})`);

    this.reconnectTimer = setTimeout(() => {
      this.reconnectAttempts++;
      const [, hostPort] = this.url.split('//');
      const [ipAddress, port] = hostPort.split(':');
      this.connect(ipAddress, parseInt(port.split('/')[0])).catch((error) => {
        console.error('[WebSocket] Reconnect failed:', error);
      });
    }, delay);
  }

  /**
   * Disconnect from WebSocket
   */
  disconnect(): void {
    this.isIntentionalClose = true;
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }

  /**
   * Send message to robot
   */
  send(message: any): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(message));
    } else {
      console.warn('[WebSocket] Cannot send - not connected');
    }
  }

  /**
   * Subscribe to message type
   */
  on(event: string, callback: (data: any) => void): () => void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(callback);

    // Return unsubscribe function
    return () => {
      this.listeners.get(event)?.delete(callback);
    };
  }

  /**
   * Emit event to listeners
   */
  private emit(event: string, data: any): void {
    this.listeners.get(event)?.forEach((callback) => {
      try {
        callback(data);
      } catch (error) {
        console.error('[WebSocket] Listener error:', error);
      }
    });
  }

  /**
   * Handle incoming WebSocket message
   */
  private handleMessage(message: WebSocketMessage): void {
    this.emit(message.type, message.payload);
    this.emit('message', message);
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  /**
   * Update settings
   */
  updateSettings(settings: Partial<AppSettings>): void {
    this.settings = { ...this.settings, ...settings };
  }
}
