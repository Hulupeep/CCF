/**
 * Robot Discovery Panel Component
 * Issue #77 - STORY-ARCH-008
 *
 * Implements:
 * - Discovery panel UI listing all robots
 * - Shows robot name, IP, port, version
 * - Connect/disconnect buttons with state management
 * - Health indicators (connected, disconnected, error)
 * - Integration with WebSocket V2 for connections
 */

import React, { useMemo } from 'react';
import { IDiscoveryService, RobotWithState } from '../types/discovery';
import { useRobotDiscovery } from '../hooks/useRobotDiscovery';
import { useRobotConnection } from '../hooks/useRobotConnection';
import './RobotDiscovery.css';

export interface RobotDiscoveryProps {
  discoveryService: IDiscoveryService;
  onRobotConnect?: (robotId: string) => void;
  onRobotDisconnect?: (robotId: string) => void;
  className?: string;
}

export const RobotDiscovery: React.FC<RobotDiscoveryProps> = ({
  discoveryService,
  onRobotConnect,
  onRobotDisconnect,
  className = '',
}) => {
  const {
    robots,
    isDiscovering,
    error: discoveryError,
    startDiscovery,
    stopDiscovery,
    refreshRobots,
  } = useRobotDiscovery({
    discoveryService,
    autoStart: true,
  });

  const {
    connect,
    disconnect,
    getStatus,
    isConnected,
  } = useRobotConnection({
    onStatusChange: (robotId, status) => {
      // Update discovery service with connection status
      discoveryService.updateRobotStatus?.(robotId, status);

      // Notify parent
      if (status === 'connected') {
        onRobotConnect?.(robotId);
      } else if (status === 'disconnected') {
        onRobotDisconnect?.(robotId);
      }
    },
  });

  // Get robots with current connection status
  const robotsWithStatus = useMemo(() => {
    return robots.map(robot => ({
      ...robot,
      status: getStatus(robot.id),
    }));
  }, [robots, getStatus]);

  const handleConnect = async (robot: RobotWithState) => {
    try {
      await connect(robot.id, robot.ipAddress, robot.port);
    } catch (error) {
      console.error(`Failed to connect to ${robot.name}:`, error);
    }
  };

  const handleDisconnect = (robot: RobotWithState) => {
    disconnect(robot.id);
  };

  const handleToggleDiscovery = () => {
    if (isDiscovering) {
      stopDiscovery();
    } else {
      startDiscovery();
    }
  };

  return (
    <div className={`robot-discovery ${className}`} data-testid="robot-discovery-panel">
      {/* Header */}
      <div className="discovery-header">
        <h2>🤖 Robot Discovery</h2>
        <div className="discovery-controls">
          <button
            onClick={handleToggleDiscovery}
            className={`discovery-toggle ${isDiscovering ? 'active' : ''}`}
            data-testid="discovery-toggle"
          >
            {isDiscovering ? '⏸ Stop Discovery' : '▶ Start Discovery'}
          </button>
          <button
            onClick={refreshRobots}
            disabled={!isDiscovering}
            className="refresh-button"
            data-testid="refresh-button"
          >
            🔄 Refresh
          </button>
        </div>
      </div>

      {/* Error Display */}
      {discoveryError && (
        <div className="discovery-error" data-testid="discovery-error">
          ⚠️ Discovery Error: {discoveryError.message}
        </div>
      )}

      {/* Robots List */}
      <div className="robots-list" data-testid="robots-list">
        {robotsWithStatus.length === 0 ? (
          <div className="no-robots" data-testid="no-robots-message">
            {isDiscovering
              ? '🔍 Searching for robots...'
              : '📭 No robots found. Start discovery to search.'}
          </div>
        ) : (
          robotsWithStatus.map(robot => (
            <RobotCard
              key={robot.id}
              robot={robot}
              onConnect={() => handleConnect(robot)}
              onDisconnect={() => handleDisconnect(robot)}
              isConnected={isConnected(robot.id)}
            />
          ))
        )}
      </div>

      {/* Footer Stats */}
      {robotsWithStatus.length > 0 && (
        <div className="discovery-footer" data-testid="discovery-stats">
          <span>
            Found: <strong>{robotsWithStatus.length}</strong> robot
            {robotsWithStatus.length !== 1 ? 's' : ''}
          </span>
          <span>
            Connected:{' '}
            <strong>{robotsWithStatus.filter(r => isConnected(r.id)).length}</strong>
          </span>
        </div>
      )}
    </div>
  );
};

interface RobotCardProps {
  robot: RobotWithState;
  onConnect: () => void;
  onDisconnect: () => void;
  isConnected: boolean;
}

const RobotCard: React.FC<RobotCardProps> = ({
  robot,
  onConnect,
  onDisconnect,
  isConnected,
}) => {
  const statusIcon = {
    connected: '🟢',
    disconnected: '⚪',
    error: '🔴',
    discovering: '🟡',
  }[robot.status];

  const statusLabel = {
    connected: 'Connected',
    disconnected: 'Disconnected',
    error: 'Error',
    discovering: 'Connecting...',
  }[robot.status];

  return (
    <div
      className={`robot-card status-${robot.status}`}
      data-testid={`robot-card-${robot.id}`}
    >
      {/* Status Indicator */}
      <div className="robot-status" data-testid={`robot-status-${robot.id}`}>
        <span className="status-icon">{statusIcon}</span>
        <span className="status-label">{statusLabel}</span>
      </div>

      {/* Robot Info */}
      <div className="robot-info">
        <h3 className="robot-name" data-testid={`robot-name-${robot.id}`}>
          {robot.name}
        </h3>
        <div className="robot-details">
          <div className="detail-row">
            <span className="detail-label">IP:</span>
            <span className="detail-value" data-testid={`robot-ip-${robot.id}`}>
              {robot.ipAddress}
            </span>
          </div>
          <div className="detail-row">
            <span className="detail-label">Port:</span>
            <span className="detail-value" data-testid={`robot-port-${robot.id}`}>
              {robot.port}
            </span>
          </div>
          <div className="detail-row">
            <span className="detail-label">Version:</span>
            <span className="detail-value" data-testid={`robot-version-${robot.id}`}>
              {robot.version}
            </span>
          </div>
          {robot.metadata?.firmware && (
            <div className="detail-row">
              <span className="detail-label">Firmware:</span>
              <span className="detail-value" data-testid={`robot-firmware-${robot.id}`}>
                {robot.metadata.firmware}
              </span>
            </div>
          )}
        </div>

        {/* Capabilities */}
        {robot.metadata?.capabilities && robot.metadata.capabilities.length > 0 && (
          <div className="robot-capabilities">
            {robot.metadata.capabilities.map(cap => (
              <span key={cap} className="capability-badge">
                {cap}
              </span>
            ))}
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="robot-actions">
        {isConnected ? (
          <button
            onClick={onDisconnect}
            className="disconnect-button"
            data-testid={`disconnect-button-${robot.id}`}
          >
            🔌 Disconnect
          </button>
        ) : (
          <button
            onClick={onConnect}
            className="connect-button"
            disabled={robot.status === 'discovering'}
            data-testid={`connect-button-${robot.id}`}
          >
            🔗 Connect
          </button>
        )}
      </div>

      {/* Last Seen */}
      <div className="robot-last-seen">
        Last seen: {new Date(robot.lastSeen).toLocaleTimeString()}
      </div>
    </div>
  );
};

export default RobotDiscovery;
