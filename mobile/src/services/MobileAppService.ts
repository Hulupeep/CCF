/**
 * Mobile App Service - Main service orchestrator
 * Issue: #88 (STORY-MOBILE-001)
 * Coordinates WebSocket, Discovery, and Cache services
 */

import {
  MobileAppService as IMobileAppService,
  Robot,
  NeuralState,
  PersonalityConfig,
  Drawing,
  AppSettings,
  Subscription,
} from '../types';
import { WebSocketClient } from './WebSocketClient';
import { RobotDiscoveryService } from './RobotDiscoveryService';
import { CacheService } from './CacheService';

const DEFAULT_SETTINGS: AppSettings = {
  autoReconnect: true,
  reconnectDelay: 2000,        // 2 seconds (I-MOBILE-001)
  cacheExpiry: 24,             // 24 hours (I-MOBILE-003)
  notificationsEnabled: true,
  theme: 'light',
};

export class MobileAppService implements IMobileAppService {
  private wsClient: WebSocketClient;
  private discoveryService: RobotDiscoveryService;
  private cacheService: CacheService;
  private settings: AppSettings;
  private currentRobot?: Robot;

  constructor() {
    this.settings = DEFAULT_SETTINGS;
    this.wsClient = new WebSocketClient(this.settings);
    this.discoveryService = new RobotDiscoveryService();
    this.cacheService = new CacheService();

    this.cacheService.setCacheExpiry(this.settings.cacheExpiry);
  }

  /**
   * Initialize service and restore cached state
   */
  async initialize(): Promise<void> {
    console.log('[AppService] Initializing...');

    // Try to restore from cache (I-MOBILE-003)
    const cachedState = await this.cacheService.loadAppState();
    if (cachedState) {
      console.log('[AppService] Restored from cache');
      this.currentRobot = cachedState.currentRobot;
    }

    // Start robot scanning
    this.discoveryService.startScanning();
  }

  /**
   * Discover robots on network
   */
  async discoverRobots(): Promise<Robot[]> {
    return this.discoveryService.discoverRobots();
  }

  /**
   * Connect to robot
   * I-MOBILE-001: Must connect within 3 seconds
   */
  async connectToRobot(robotId: string): Promise<void> {
    const robot = this.discoveryService.getRobot(robotId);
    if (!robot) {
      throw new Error(`Robot ${robotId} not found`);
    }

    console.log(`[AppService] Connecting to ${robot.name}...`);

    try {
      await this.wsClient.connect(robot.ipAddress, robot.port);
      this.currentRobot = robot;

      // Cache connection
      await this.cacheService.saveAppState({
        connected: true,
        currentRobot: robot,
        cachedDrawings: [],
        lastSync: Date.now(),
      });

      console.log(`[AppService] Connected to ${robot.name}`);
    } catch (error) {
      console.error('[AppService] Connection failed:', error);
      throw error;
    }
  }

  /**
   * Disconnect from robot
   */
  async disconnectFromRobot(): Promise<void> {
    console.log('[AppService] Disconnecting...');
    this.wsClient.disconnect();
    this.currentRobot = undefined;

    await this.cacheService.saveAppState({
      connected: false,
      cachedDrawings: [],
      lastSync: Date.now(),
    });
  }

  /**
   * Subscribe to neural state updates
   * Returns subscription with unsubscribe method
   */
  subscribeToNeuralState(callback: (state: NeuralState) => void): Subscription {
    const unsubscribe = this.wsClient.on('neural_state', (state: NeuralState) => {
      callback(state);
      // Cache neural state (I-MOBILE-003)
      this.cacheService.saveNeuralState(state).catch(console.error);
    });

    return { unsubscribe };
  }

  /**
   * Update personality configuration
   * I-MOBILE-001: Robot must respond within 200ms
   */
  async updatePersonality(config: PersonalityConfig): Promise<void> {
    if (!this.wsClient.isConnected()) {
      throw new Error('Not connected to robot');
    }

    console.log('[AppService] Updating personality...');

    this.wsClient.send({
      type: 'update_personality',
      payload: config,
    });

    // Cache personality (I-MOBILE-003)
    await this.cacheService.savePersonality(config);
  }

  /**
   * Fetch drawings from robot
   */
  async fetchDrawings(): Promise<Drawing[]> {
    if (!this.wsClient.isConnected()) {
      // Return cached drawings in offline mode (I-MOBILE-003)
      console.log('[AppService] Offline - loading cached drawings');
      return this.cacheService.loadDrawings();
    }

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Fetch timeout'));
      }, 5000);

      const unsubscribe = this.wsClient.on('drawings_list', async (drawings: Drawing[]) => {
        clearTimeout(timeout);
        unsubscribe();

        // Cache drawings (I-MOBILE-003)
        await this.cacheService.saveDrawings(drawings);
        resolve(drawings);
      });

      this.wsClient.send({ type: 'get_drawings' });
    });
  }

  /**
   * Download specific drawing
   */
  async downloadDrawing(id: string): Promise<Drawing> {
    if (!this.wsClient.isConnected()) {
      // Try to load from cache
      const drawings = await this.cacheService.loadDrawings();
      const drawing = drawings.find((d) => d.id === id);
      if (!drawing) {
        throw new Error('Drawing not found in cache');
      }
      return drawing;
    }

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error('Download timeout'));
      }, 10000);

      const unsubscribe = this.wsClient.on('drawing_data', (drawing: Drawing) => {
        if (drawing.id === id) {
          clearTimeout(timeout);
          unsubscribe();
          resolve(drawing);
        }
      });

      this.wsClient.send({ type: 'get_drawing', payload: { id } });
    });
  }

  /**
   * Get current settings
   */
  getSettings(): AppSettings {
    return { ...this.settings };
  }

  /**
   * Update settings
   * I-MOBILE-001: reconnectDelay must be ≤ 5000ms
   * I-MOBILE-003: cacheExpiry must be ≥ 24 hours
   */
  updateSettings(settings: Partial<AppSettings>): void {
    // Enforce I-MOBILE-001
    if (settings.reconnectDelay !== undefined && settings.reconnectDelay > 5000) {
      console.warn('[AppService] reconnectDelay capped at 5000ms (I-MOBILE-001)');
      settings.reconnectDelay = 5000;
    }

    // Enforce I-MOBILE-003
    if (settings.cacheExpiry !== undefined && settings.cacheExpiry < 24) {
      console.warn('[AppService] cacheExpiry must be at least 24 hours (I-MOBILE-003)');
      settings.cacheExpiry = 24;
    }

    this.settings = { ...this.settings, ...settings };
    this.wsClient.updateSettings(this.settings);
    this.cacheService.setCacheExpiry(this.settings.cacheExpiry);

    console.log('[AppService] Settings updated:', this.settings);
  }

  /**
   * Get current robot
   */
  getCurrentRobot(): Robot | undefined {
    return this.currentRobot;
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.wsClient.isConnected();
  }

  /**
   * Cleanup
   */
  async cleanup(): Promise<void> {
    console.log('[AppService] Cleaning up...');
    this.discoveryService.stopScanning();
    this.wsClient.disconnect();
  }
}
