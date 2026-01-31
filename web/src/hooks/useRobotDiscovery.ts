/**
 * React hook for robot discovery
 * Issue #77 - STORY-ARCH-008
 */

import { useState, useEffect, useCallback } from 'react';
import { RobotWithState, IDiscoveryService } from '../types/discovery';

export interface UseRobotDiscoveryOptions {
  discoveryService: IDiscoveryService;
  autoStart?: boolean;
}

export function useRobotDiscovery({
  discoveryService,
  autoStart = true,
}: UseRobotDiscoveryOptions) {
  const [robots, setRobots] = useState<RobotWithState[]>([]);
  const [isDiscovering, setIsDiscovering] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  // Start discovery
  const startDiscovery = useCallback(async () => {
    try {
      setIsDiscovering(true);
      setError(null);
      await discoveryService.start();
      setRobots(discoveryService.getRobots());
    } catch (err) {
      setError(err as Error);
      setIsDiscovering(false);
    }
  }, [discoveryService]);

  // Stop discovery
  const stopDiscovery = useCallback(async () => {
    try {
      await discoveryService.stop();
      setIsDiscovering(false);
      setRobots([]);
    } catch (err) {
      setError(err as Error);
    }
  }, [discoveryService]);

  // Refresh robots list
  const refreshRobots = useCallback(() => {
    setRobots(discoveryService.getRobots());
  }, [discoveryService]);

  // Subscribe to discovery events
  useEffect(() => {
    const unsubscribe = discoveryService.subscribe(event => {
      switch (event.type) {
        case 'robot_discovered':
        case 'robot_lost':
        case 'robot_updated':
          refreshRobots();
          break;

        case 'error':
          setError(event.error);
          break;
      }
    });

    return unsubscribe;
  }, [discoveryService, refreshRobots]);

  // Auto-start discovery
  useEffect(() => {
    if (autoStart) {
      startDiscovery();
    }

    return () => {
      if (autoStart) {
        stopDiscovery();
      }
    };
  }, [autoStart, startDiscovery, stopDiscovery]);

  return {
    robots,
    isDiscovering,
    error,
    startDiscovery,
    stopDiscovery,
    refreshRobots,
  };
}
