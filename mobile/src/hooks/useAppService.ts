/**
 * React Hook for Mobile App Service
 * Issue: #88 (STORY-MOBILE-001)
 */

import { useEffect, useState } from 'react';
import { MobileAppService } from '../services/MobileAppService';
import { Robot, NeuralState, PersonalityConfig, Drawing, AppSettings } from '../types';

let serviceInstance: MobileAppService | null = null;

export function useAppService() {
  const [service] = useState(() => {
    if (!serviceInstance) {
      serviceInstance = new MobileAppService();
      serviceInstance.initialize().catch(console.error);
    }
    return serviceInstance;
  });

  const [connected, setConnected] = useState(false);
  const [currentRobot, setCurrentRobot] = useState<Robot | undefined>();

  useEffect(() => {
    const interval = setInterval(() => {
      setConnected(service.isConnected());
      setCurrentRobot(service.getCurrentRobot());
    }, 1000);

    return () => clearInterval(interval);
  }, [service]);

  return {
    service,
    connected,
    currentRobot,
  };
}

export function useRobotDiscovery() {
  const { service } = useAppService();
  const [robots, setRobots] = useState<Robot[]>([]);
  const [scanning, setScanning] = useState(false);
  const [error, setError] = useState<string>();

  const startScan = async () => {
    setScanning(true);
    setError(undefined);

    try {
      const discovered = await service.discoverRobots();
      setRobots(discovered);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Scan failed');
    } finally {
      setScanning(false);
    }
  };

  const connect = async (robotId: string) => {
    setError(undefined);
    try {
      await service.connectToRobot(robotId);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Connection failed');
      throw err;
    }
  };

  return {
    robots,
    scanning,
    error,
    startScan,
    connect,
  };
}

export function useNeuralState() {
  const { service } = useAppService();
  const [neuralState, setNeuralState] = useState<NeuralState | null>(null);

  useEffect(() => {
    const subscription = service.subscribeToNeuralState((state) => {
      setNeuralState(state);
    });

    return () => subscription.unsubscribe();
  }, [service]);

  return neuralState;
}

export function usePersonality() {
  const { service } = useAppService();
  const [config, setConfig] = useState<PersonalityConfig>({
    energy: 0.5,
    tension: 0.5,
    curiosity: 0.5,
    playfulness: 0.5,
    confidence: 0.5,
    focus: 0.5,
    empathy: 0.5,
    creativity: 0.5,
    persistence: 0.5,
  });

  const updateSlider = async (param: keyof PersonalityConfig, value: number) => {
    const newConfig = { ...config, [param]: value };
    setConfig(newConfig);
    await service.updatePersonality(newConfig);
  };

  const reset = async () => {
    const defaultConfig: PersonalityConfig = {
      energy: 0.5,
      tension: 0.5,
      curiosity: 0.5,
      playfulness: 0.5,
      confidence: 0.5,
      focus: 0.5,
      empathy: 0.5,
      creativity: 0.5,
      persistence: 0.5,
    };
    setConfig(defaultConfig);
    await service.updatePersonality(defaultConfig);
  };

  return {
    config,
    updateSlider,
    reset,
  };
}

export function useGallery() {
  const { service } = useAppService();
  const [drawings, setDrawings] = useState<Drawing[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>();

  const fetchDrawings = async () => {
    setLoading(true);
    setError(undefined);

    try {
      const result = await service.fetchDrawings();
      setDrawings(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Fetch failed');
    } finally {
      setLoading(false);
    }
  };

  return {
    drawings,
    loading,
    error,
    fetchDrawings,
  };
}

export function useSettings() {
  const { service } = useAppService();
  const [settings, setSettings] = useState<AppSettings>(service.getSettings());

  const updateSettings = (newSettings: Partial<AppSettings>) => {
    service.updateSettings(newSettings);
    setSettings(service.getSettings());
  };

  return {
    settings,
    updateSettings,
  };
}
